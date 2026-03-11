use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use univis_editor_core::prelude::{
    analyze_graph_topology, Connecting, EntityComponentValue, EntityValue, GraphNode, NodeId,
    NodeRegistry, NodeValue, ProcessContext, ProcessResult,
    SpriteComponentValue, TransformComponentValue,
};

pub const WORLD_OUTPUT_NODE_ID: &str = "output/world";
pub const WORLD_OUTPUT_RENDER_LAYER: usize = 31;

pub struct NodeRuntimePlugin;

impl Plugin for NodeRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphRuntimeDiagnostics>()
            .init_resource::<WorldOutputRuntimeState>()
            .add_systems(
            Update,
            (
                initialize_node_defaults,
                propagate_and_process_nodes,
                sync_world_outputs,
            )
                .chain(),
        );
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphRuntimeDiagnostics {
    pub blocked_nodes: Vec<Entity>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct WorldOutputRuntimeState {
    pub cached_outputs: HashMap<Entity, Option<EntityValue>>,
    pub spawned_roots: HashMap<Entity, Vec<Entity>>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct RuntimeWorldOutputEntity {
    pub output_node: Entity,
}

fn initialize_node_defaults(
    registry: Res<NodeRegistry>,
    mut q_nodes: Query<(Entity, &mut GraphNode), Added<GraphNode>>,
) {
    for (_entity, mut node) in q_nodes.iter_mut() {
        let Some(definition) = registry.get(&node.definition_id) else {
            continue;
        };

        let default_inputs = definition.default_input_values();
        for (i, default_val) in default_inputs.iter().enumerate() {
            if i < node.values.inputs.len() {
                node.values.inputs[i] = default_val.clone();
            }
        }

        for output in node.values.outputs.iter_mut() {
            *output = NodeValue::None;
        }
    }
}

fn propagate_and_process_nodes(
    registry: Res<NodeRegistry>,
    graph: Res<Connecting>,
    mut diagnostics: ResMut<GraphRuntimeDiagnostics>,
    mut q_nodes: Query<(Entity, &mut GraphNode)>,
    time: Res<Time>,
) {
    let all_entities: Vec<Entity> = q_nodes.iter().map(|(e, _)| e).collect();
    let topology = analyze_graph_topology(
        all_entities.iter().copied(),
        graph.connections.iter().map(|conn| (conn.from_node, conn.to_node)),
    );
    let sorted_nodes = topology.ordered_nodes;
    let blocked_nodes = topology.blocked_nodes;

    if diagnostics.blocked_nodes != blocked_nodes {
        if blocked_nodes.is_empty() {
            diagnostics.blocked_nodes.clear();
        } else {
            diagnostics.blocked_nodes = blocked_nodes.clone();
            warn!(
                "Graph runtime skipped {} node(s) because the graph contains a cycle or blocked dependency path.",
                diagnostics.blocked_nodes.len()
            );
        }
    }

    let mut processed_outputs = std::collections::HashMap::<Entity, Vec<NodeValue>>::new();
    for (entity, node) in q_nodes.iter() {
        processed_outputs.insert(entity, node.values.outputs.clone());
    }

    for entity in sorted_nodes {
        let Ok((_, mut node)) = q_nodes.get_mut(entity) else {
            continue;
        };

        for conn in &graph.connections {
            if conn.to_node == entity {
                if let Some(outputs) = processed_outputs.get(&conn.from_node) {
                    if conn.from_index < outputs.len() && conn.to_index < node.values.inputs.len() {
                        node.values.inputs[conn.to_index] = outputs[conn.from_index].clone();
                    }
                }
            }
        }

        let definition_id = node.definition_id.clone();
        let Some(definition) = registry.get(&definition_id) else {
            continue;
        };

        let inputs = node.values.inputs.clone();
        let mut outputs = node.values.outputs.clone();
        let custom_data = &mut node.custom_data;

        let mut context = ProcessContext {
            inputs: &inputs,
            outputs: &mut outputs,
            delta_time: time.delta_secs(),
            custom_data,
        };

        let result = definition.process(&mut context);
        match result {
            ProcessResult::Success => {}
            ProcessResult::Error(msg) => {
                warn!("Node {} error: {}", definition_id, msg);
            }
            ProcessResult::MissingInput(index) => {
                debug!("Node {} missing input at index {}", definition_id, index);
            }
        }

        node.values.outputs = outputs;
        processed_outputs.insert(entity, node.values.outputs.clone());
    }
}

fn sync_world_outputs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut world_output_state: ResMut<WorldOutputRuntimeState>,
    q_nodes: Query<(Entity, &GraphNode)>,
) {
    let output_definition = NodeId::new(WORLD_OUTPUT_NODE_ID);
    let mut active_output_nodes = HashSet::new();

    for (node_entity, node) in q_nodes.iter() {
        if node.definition_id != output_definition {
            continue;
        }

        active_output_nodes.insert(node_entity);
        let next_output = node.values.inputs.first().and_then(NodeValue::as_entity).cloned();
        let cached_output = world_output_state.cached_outputs.get(&node_entity);

        if cached_output == Some(&next_output) {
            continue;
        }

        if let Some(existing_roots) = world_output_state.spawned_roots.remove(&node_entity) {
            despawn_output_roots(&mut commands, existing_roots);
        }

        let mut spawned_roots = Vec::new();
        if let Some(entity_value) = &next_output {
            let root_entity =
                spawn_entity_value_recursive(&mut commands, &asset_server, entity_value, node_entity);
            spawned_roots.push(root_entity);
        }

        if spawned_roots.is_empty() {
            world_output_state.spawned_roots.remove(&node_entity);
        } else {
            world_output_state
                .spawned_roots
                .insert(node_entity, spawned_roots);
        }

        world_output_state.cached_outputs.insert(node_entity, next_output);
    }

    let stale_outputs: Vec<Entity> = world_output_state
        .cached_outputs
        .keys()
        .copied()
        .filter(|node_entity| !active_output_nodes.contains(node_entity))
        .collect();

    for node_entity in stale_outputs {
        world_output_state.cached_outputs.remove(&node_entity);
        if let Some(existing_roots) = world_output_state.spawned_roots.remove(&node_entity) {
            despawn_output_roots(&mut commands, existing_roots);
        }
    }
}

fn despawn_output_roots(commands: &mut Commands, roots: Vec<Entity>) {
    for root in roots {
        commands.entity(root).despawn();
    }
}

fn spawn_entity_value_recursive(
    commands: &mut Commands,
    asset_server: &AssetServer,
    entity_value: &EntityValue,
    output_node: Entity,
) -> Entity {
    let transform = transform_from_component(entity_value.transform());
    let mut entity_commands = commands.spawn((
        transform,
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        RenderLayers::layer(WORLD_OUTPUT_RENDER_LAYER),
        RuntimeWorldOutputEntity { output_node },
    ));

    if let Some(name) = &entity_value.name {
        entity_commands.insert(Name::new(name.clone()));
    }

    for component in &entity_value.components {
        match component {
            EntityComponentValue::Transform(_) => {}
            EntityComponentValue::Sprite(sprite) => {
                entity_commands.insert(sprite_from_component(sprite, asset_server));
            }
            EntityComponentValue::Camera2D(camera) => {
                entity_commands.insert((
                    Camera2d,
                    Camera {
                        is_active: false,
                        ..default()
                    },
                    Projection::Orthographic(OrthographicProjection {
                        scale: camera.zoom.max(0.01),
                        ..OrthographicProjection::default_2d()
                    }),
                ));
            }
            EntityComponentValue::Custom { .. } => {}
        }
    }

    let entity = entity_commands.id();

    for child in &entity_value.children {
        let child_entity = spawn_entity_value_recursive(commands, asset_server, child, output_node);
        commands.entity(entity).add_child(child_entity);
    }

    entity
}

fn transform_from_component(transform: Option<&TransformComponentValue>) -> Transform {
    let Some(transform) = transform else {
        return Transform::default();
    };

    Transform {
        translation: transform.translation,
        rotation: Quat::from_rotation_z(transform.rotation_deg.to_radians()),
        scale: transform.scale,
    }
}

fn sprite_from_component(
    sprite: &SpriteComponentValue,
    asset_server: &AssetServer,
) -> Sprite {
    if let Some(path) = sprite
        .sprite_id
        .as_ref()
        .map(|path| path.trim())
        .filter(|path| !path.is_empty())
    {
        let mut runtime_sprite = Sprite::from_image(asset_server.load(path.to_string()));
        runtime_sprite.color = sprite.color;
        runtime_sprite.custom_size = Some(sprite.size);
        runtime_sprite
    } else {
        Sprite::from_color(sprite.color, sprite.size)
    }
}

pub mod prelude {
    pub use crate::{
        GraphRuntimeDiagnostics, NodeRuntimePlugin, RuntimeWorldOutputEntity,
        WorldOutputRuntimeState, WORLD_OUTPUT_NODE_ID, WORLD_OUTPUT_RENDER_LAYER,
    };
}
