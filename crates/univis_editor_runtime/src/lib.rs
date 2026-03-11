use bevy::prelude::*;
use univis_node_graph::prelude::{
    analyze_graph_topology, Connecting, GraphNode, NodeRegistry, NodeValue, ProcessContext,
    ProcessResult,
};
use univis_scene::{
    EntityComponentValue, EntityValue, TransformComponentValue, CAMERA2D_COMPONENT_KEY,
    SPRITE_COMPONENT_KEY, TEXT2D_COMPONENT_KEY, TRANSFORM_COMPONENT_KEY,
};

pub struct NodeRuntimePlugin;

impl Plugin for NodeRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphRuntimeDiagnostics>()
            .add_systems(
                Update,
                (
                    initialize_node_defaults,
                    propagate_and_process_nodes,
                    sync_scene_nodes_to_world,
                    cleanup_orphaned_scene_roots,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphRuntimeDiagnostics {
    pub blocked_nodes: Vec<Entity>,
}

#[derive(Component, Clone, Default)]
struct SceneWorldDisplayState {
    root: Option<Entity>,
    last_signature: Option<String>,
}

#[derive(Component)]
struct SceneWorldRoot {
    node_entity: Entity,
}

fn is_scene_sink(node: &GraphNode) -> bool {
    node.definition_id.as_str() == "scene/scene"
}

fn entity_signature(entity: Option<&EntityValue>) -> Option<String> {
    entity.map(|entity| format!("{entity:?}"))
}

fn transform_to_bevy(transform: &TransformComponentValue) -> Transform {
    Transform {
        translation: transform.translation,
        rotation: Quat::from_rotation_z(transform.rotation_deg.to_radians()),
        scale: transform.scale,
    }
}

fn spawn_scene_entity_recursive(parent: &mut ChildSpawnerCommands, entity_value: &EntityValue) {
    let transform = entity_value
        .component(TRANSFORM_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_transform)
        .map(transform_to_bevy)
        .unwrap_or_default();
    let sprite = entity_value
        .component(SPRITE_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_sprite)
        .cloned();
    let text = entity_value
        .component(TEXT2D_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_text_2d)
        .cloned();
    let _has_camera = entity_value.component(CAMERA2D_COMPONENT_KEY).is_some();

    let mut entity_commands = parent.spawn((transform, Visibility::Visible));

    if let Some(name) = entity_value.name.as_deref().filter(|name| !name.is_empty()) {
        entity_commands.insert(Name::new(name.to_string()));
    }

    if let Some(sprite) = sprite {
        entity_commands.insert(Sprite::from_color(sprite.color, sprite.size));
    }

    if let Some(text) = text {
        entity_commands.insert((
            Text2d::new(text.content),
            TextFont::from_font_size(text.font_size),
            TextColor(text.color),
        ));
    }

    // `univis_ui` currently assumes there is exactly one `Camera2d` in the world
    // for picking and panel interaction. Spawning scene cameras here breaks editor input,
    // so the world-display sink keeps camera data inert for now.

    let children = entity_value.children.clone();
    entity_commands.with_children(|next_parent| {
        for child in &children {
            spawn_scene_entity_recursive(next_parent, child);
        }
    });
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

fn sync_scene_nodes_to_world(
    mut commands: Commands,
    q_nodes: Query<(Entity, &GraphNode, Option<&SceneWorldDisplayState>)>,
) {
    for (node_entity, node, state) in q_nodes.iter() {
        if !is_scene_sink(node) {
            continue;
        }

        let input_entity = node.values.inputs.first().and_then(NodeValue::as_entity).cloned();
        let next_signature = entity_signature(input_entity.as_ref());
        let mut next_state = state.cloned().unwrap_or_default();
        let unchanged = next_state.last_signature == next_signature
            && match input_entity {
                Some(_) => next_state.root.is_some(),
                None => next_state.root.is_none(),
            };

        if unchanged {
            continue;
        }

        if let Some(root) = next_state.root.take() {
            commands.entity(root).despawn();
        }

        if let Some(entity_value) = input_entity.as_ref() {
            let root = commands
                .spawn((
                    SceneWorldRoot { node_entity },
                    Name::new("Scene Display Root"),
                    Transform::default(),
                    Visibility::Visible,
                ))
                .id();
            commands.entity(root).with_children(|parent| {
                spawn_scene_entity_recursive(parent, entity_value);
            });
            next_state.root = Some(root);
        }

        next_state.last_signature = next_signature;
        commands.entity(node_entity).insert(next_state);
    }
}

fn cleanup_orphaned_scene_roots(
    mut commands: Commands,
    q_roots: Query<(Entity, &SceneWorldRoot)>,
    q_nodes: Query<(), With<GraphNode>>,
) {
    for (root, marker) in q_roots.iter() {
        if q_nodes.get(marker.node_entity).is_err() {
            commands.entity(root).despawn();
        }
    }
}

pub mod prelude {
    pub use crate::{GraphRuntimeDiagnostics, NodeRuntimePlugin};
}
