use bevy::prelude::*;
use univis_editor_nodes_builtin::scene::PrefabInstanceData;
use univis_node_graph::prelude::{
    analyze_graph_topology, Connecting, GraphNode, LiveGraphDocumentState, NodeRegistry, NodeValue,
    ProcessContext, ProcessResult,
};
use univis_scene::{
    scene_document_signature, spawn_scene_document_recursive, EntitySpawnOptions, SceneDocument,
    SceneStats,
};

pub struct NodeRuntimePlugin;

impl Plugin for NodeRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphRuntimeDiagnostics>()
            .init_resource::<GraphSceneOutputs>()
            .add_systems(
                Update,
                (
                    initialize_node_defaults,
                    sync_prefab_instance_nodes,
                    propagate_and_process_nodes,
                    collect_scene_outputs,
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
    pub node_issues: Vec<GraphRuntimeNodeIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphRuntimeIssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRuntimeNodeIssue {
    pub node: Entity,
    pub definition_id: String,
    pub severity: GraphRuntimeIssueSeverity,
    pub message: String,
}

#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct GraphSceneOutputs {
    pub sinks: Vec<GraphSceneSinkOutput>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphSceneSinkOutput {
    pub node_entity: Entity,
    pub definition_id: String,
    pub mode: GraphSceneOutputMode,
    pub scene: Option<SceneDocument>,
    pub stats: Option<SceneStats>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphSceneOutputMode {
    World,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SceneSinkMode {
    None,
    World,
}

fn scene_sink_mode(definition_id: &str) -> SceneSinkMode {
    match definition_id {
        "scene/scene" => SceneSinkMode::World,
        _ => SceneSinkMode::None,
    }
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

fn sync_prefab_instance_nodes(
    live_document: Res<LiveGraphDocumentState>,
    mut q_nodes: Query<&mut GraphNode>,
) {
    for mut node in q_nodes.iter_mut() {
        if node.definition_id.as_str() != "scene/prefab_instance" {
            continue;
        }

        let prefab_id = node
            .values
            .inputs
            .first()
            .and_then(NodeValue::as_string)
            .map(str::trim)
            .unwrap_or("")
            .to_string();
        let root = live_document
            .document
            .prefabs
            .iter()
            .find(|prefab| prefab.id == prefab_id)
            .map(|prefab| prefab.root.clone());

        node.custom_data = Some(Box::new(PrefabInstanceData { prefab_id, root }));
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
        graph
            .connections
            .iter()
            .map(|conn| (conn.from_node, conn.to_node)),
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
    diagnostics.node_issues.clear();

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
                diagnostics.node_issues.push(GraphRuntimeNodeIssue {
                    node: entity,
                    definition_id: definition_id.to_string(),
                    severity: GraphRuntimeIssueSeverity::Error,
                    message: msg,
                });
            }
            ProcessResult::MissingInput(index) => {
                debug!("Node {} missing input at index {}", definition_id, index);
                diagnostics.node_issues.push(GraphRuntimeNodeIssue {
                    node: entity,
                    definition_id: definition_id.to_string(),
                    severity: GraphRuntimeIssueSeverity::Warning,
                    message: format!("Missing input at index {}", index),
                });
            }
        }

        node.values.outputs = outputs;
        processed_outputs.insert(entity, node.values.outputs.clone());
    }
}

fn collect_scene_outputs(
    q_nodes: Query<(Entity, &GraphNode)>,
    mut scene_outputs: ResMut<GraphSceneOutputs>,
) {
    let mut next_outputs = Vec::new();

    for (node_entity, node) in q_nodes.iter() {
        let mode = match scene_sink_mode(node.definition_id.as_str()) {
            SceneSinkMode::World => GraphSceneOutputMode::World,
            SceneSinkMode::None => continue,
        };

        let scene = node
            .values
            .inputs
            .first()
            .and_then(NodeValue::as_entity)
            .cloned()
            .map(SceneDocument::from_entity_value);
        let stats = scene.as_ref().map(SceneDocument::stats);
        let signature = scene_document_signature(scene.as_ref());

        next_outputs.push(GraphSceneSinkOutput {
            node_entity,
            definition_id: node.definition_id.as_str().to_string(),
            mode,
            scene,
            stats,
            signature,
        });
    }

    next_outputs.sort_by_key(|sink| sink.node_entity.index());
    if scene_outputs.sinks != next_outputs {
        scene_outputs.sinks = next_outputs;
    }
}

fn sync_scene_nodes_to_world(
    mut commands: Commands,
    scene_outputs: Res<GraphSceneOutputs>,
    q_nodes: Query<(Entity, Option<&SceneWorldDisplayState>), With<GraphNode>>,
) {
    for (node_entity, state) in q_nodes.iter() {
        let next_output = scene_outputs.sinks.iter().find(|sink| {
            sink.node_entity == node_entity && sink.mode == GraphSceneOutputMode::World
        });

        if next_output.is_none() {
            if let Some(state) = state.cloned() {
                if let Some(root) = state.root {
                    commands.entity(root).try_despawn();
                }
                commands
                    .entity(node_entity)
                    .try_remove::<SceneWorldDisplayState>();
            }
            continue;
        }

        let next_output = next_output.expect("checked above");
        let next_scene = next_output.scene.as_ref();
        let next_signature = next_output.signature.clone();
        let mut next_state = state.cloned().unwrap_or_default();
        let unchanged = next_state.last_signature == next_signature
            && if next_scene.is_some() {
                next_state.root.is_some()
            } else {
                next_state.root.is_none()
            };

        if unchanged {
            continue;
        }

        if let Some(root) = next_state.root.take() {
            commands.entity(root).try_despawn();
        }

        if let Some(scene) = next_scene.as_ref() {
            let spawn_options = EntitySpawnOptions::default();
            let root = commands
                .spawn((
                    SceneWorldRoot { node_entity },
                    Name::new("Scene Display Root"),
                    Transform::default(),
                    Visibility::Visible,
                ))
                .id();
            commands.entity(root).with_children(|parent| {
                spawn_scene_document_recursive(parent, scene, &spawn_options);
            });
            next_state.root = Some(root);
        }

        next_state.last_signature = next_signature;
        commands.entity(node_entity).try_insert(next_state);
    }
}

fn cleanup_orphaned_scene_roots(
    mut commands: Commands,
    q_roots: Query<(Entity, &SceneWorldRoot)>,
    q_nodes: Query<(), With<GraphNode>>,
) {
    for (root, marker) in q_roots.iter() {
        if q_nodes.get(marker.node_entity).is_err() {
            commands.entity(root).try_despawn();
        }
    }
}

pub mod prelude {
    pub use crate::{
        GraphRuntimeDiagnostics, GraphSceneOutputMode, GraphSceneOutputs, GraphSceneSinkOutput,
        NodeRuntimePlugin,
    };
}

#[cfg(test)]
mod tests {
    use super::{scene_sink_mode, SceneSinkMode};

    #[test]
    fn scene_sinks_are_treated_as_world_sinks() {
        assert_eq!(scene_sink_mode("scene/scene"), SceneSinkMode::World);
        assert_eq!(scene_sink_mode("scene/transform"), SceneSinkMode::None);
    }
}
