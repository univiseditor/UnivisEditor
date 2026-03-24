use bevy::prelude::*;
use univis_node_graph::prelude::NodeValue;
use univis_scene::{SceneDocument, SceneStats, scene_document_signature};

use crate::connectivity::GraphExecutableRuntimeState;

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

pub(super) fn collect_scene_outputs_system(
    executable_state: Res<GraphExecutableRuntimeState>,
    mut scene_outputs: ResMut<GraphSceneOutputs>,
) {
    let mut next_outputs = Vec::new();

    for node in executable_state.graph.iter_nodes() {
        let Some(node_entity) = executable_state.entity_for_node_id(node.node_id()) else {
            continue;
        };
        let mode = match scene_sink_mode(node.definition_id().as_str()) {
            SceneSinkMode::World => GraphSceneOutputMode::World,
            SceneSinkMode::None => continue,
        };

        let scene = node
            .resolved_inputs()
            .first()
            .and_then(NodeValue::as_entity)
            .cloned()
            .map(SceneDocument::from_entity_value);
        let stats = scene.as_ref().map(SceneDocument::stats);
        let signature = scene_document_signature(scene.as_ref());

        next_outputs.push(GraphSceneSinkOutput {
            node_entity,
            definition_id: node.definition_id().as_str().to_string(),
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

#[cfg(test)]
mod tests {
    use super::{SceneSinkMode, scene_sink_mode};

    #[test]
    fn scene_sinks_are_treated_as_world_sinks() {
        assert_eq!(scene_sink_mode("scene/scene"), SceneSinkMode::World);
        assert_eq!(scene_sink_mode("scene/transform"), SceneSinkMode::None);
    }
}
