use bevy::prelude::*;
use univis_node_graph::prelude::{GraphNode, NodeRegistry, NodeValue};

pub(super) fn initialize_node_defaults_system(
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
