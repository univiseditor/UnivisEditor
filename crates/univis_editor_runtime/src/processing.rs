use bevy::prelude::*;
use univis_node_graph::node_registry::VisualSyncNode;
use univis_node_graph::prelude::{AuthoredNodeInputs, GraphNode, NodeRegistry, NodeValue};

use crate::connectivity::{NodeInputSignature, NodeOutputSignature};

pub(super) fn initialize_node_defaults_system(
    mut commands: Commands,
    registry: Res<NodeRegistry>,
    mut q_nodes: Query<(Entity, &mut GraphNode), Added<GraphNode>>,
) {
    for (entity, mut node) in q_nodes.iter_mut() {
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

        commands.entity(entity).insert((
            AuthoredNodeInputs {
                values: node.values.inputs.clone(),
            },
            NodeInputSignature {
                inputs: node.values.inputs.clone(),
            },
            NodeOutputSignature {
                outputs: node.values.outputs.clone(),
            },
        ));

        if definition.needs_visual_sync() {
            commands.entity(entity).insert(VisualSyncNode);
        }
    }
}
