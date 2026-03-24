use bevy::prelude::*;
use univis_node_graph::node_registry::VisualSyncNode;
use univis_node_graph::prelude::{AuthoredNodeInputs, GraphNode, NodeRegistry};

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

        let mut authored_inputs = AuthoredNodeInputs {
            values: definition.default_input_values(),
        };
        authored_inputs.ensure_len(node.input_projection_len());
        node.sync_input_projection_from_authored(&authored_inputs);
        node.clear_output_projection();

        commands.entity(entity).insert((
            authored_inputs.clone(),
            NodeInputSignature {
                inputs: authored_inputs.values,
            },
            NodeOutputSignature {
                outputs: node.output_projection_values().to_vec(),
            },
        ));

        if definition.needs_visual_sync() {
            commands.entity(entity).insert(VisualSyncNode);
        }
    }
}
