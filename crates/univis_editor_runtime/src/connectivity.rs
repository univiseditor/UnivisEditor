use std::collections::HashMap;

use bevy::prelude::*;
use univis_node_graph::prelude::{analyze_graph_topology, GraphConnection, GraphNode, NodeValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphInputSource {
    pub source_node: Entity,
    pub source_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphOutputTarget {
    pub target_node: Entity,
    pub target_index: usize,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphConnectivityIndex {
    pub incoming_by_node_input: HashMap<Entity, Vec<Option<GraphInputSource>>>,
    pub outgoing_by_node_output: HashMap<Entity, Vec<Vec<GraphOutputTarget>>>,
    pub ordered_nodes: Vec<Entity>,
    pub blocked_nodes: Vec<Entity>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphResolvedInputs {
    pub by_node: HashMap<Entity, Vec<NodeValue>>,
}

#[derive(Component, Debug, Clone, Default)]
pub struct NodeInputSignature {
    pub inputs: Vec<NodeValue>,
}

#[derive(Component, Debug, Clone, Default)]
pub struct NodeOutputSignature {
    pub outputs: Vec<NodeValue>,
}

pub fn rebuild_connectivity_index_system(
    mut index: ResMut<GraphConnectivityIndex>,
    mut resolved_inputs: ResMut<GraphResolvedInputs>,
    q_nodes: Query<(Entity, &GraphNode)>,
    q_added_nodes: Query<Entity, Added<GraphNode>>,
    q_connections: Query<&GraphConnection>,
    q_changed_connections: Query<Entity, Or<(Added<GraphConnection>, Changed<GraphConnection>)>>,
    mut removed_nodes: RemovedComponents<GraphNode>,
    mut removed_connections: RemovedComponents<GraphConnection>,
) {
    let should_refresh = !q_added_nodes.is_empty()
        || !q_changed_connections.is_empty()
        || removed_nodes.read().next().is_some()
        || removed_connections.read().next().is_some()
        || (index.ordered_nodes.is_empty() && (!q_nodes.is_empty() || !q_connections.is_empty()));
    if !should_refresh {
        return;
    }

    let nodes: Vec<(Entity, usize, usize)> = q_nodes
        .iter()
        .map(|(entity, node)| (entity, node.values.inputs.len(), node.values.outputs.len()))
        .collect();

    index.incoming_by_node_input.clear();
    index.outgoing_by_node_output.clear();
    index.incoming_by_node_input.reserve(nodes.len());
    index.outgoing_by_node_output.reserve(nodes.len());

    for (entity, input_count, output_count) in &nodes {
        index
            .incoming_by_node_input
            .insert(*entity, vec![None; *input_count]);
        index
            .outgoing_by_node_output
            .insert(*entity, vec![Vec::new(); *output_count]);
    }

    for connection in q_connections.iter() {
        if let Some(inputs) = index.incoming_by_node_input.get_mut(&connection.to_node) {
            if connection.to_index < inputs.len() {
                inputs[connection.to_index] = Some(GraphInputSource {
                    source_node: connection.from_node,
                    source_index: connection.from_index,
                });
            }
        }

        if let Some(outputs) = index.outgoing_by_node_output.get_mut(&connection.from_node) {
            if connection.from_index < outputs.len() {
                outputs[connection.from_index].push(GraphOutputTarget {
                    target_node: connection.to_node,
                    target_index: connection.to_index,
                });
            }
        }
    }

    let topology = analyze_graph_topology(
        nodes.iter().map(|(entity, _, _)| *entity),
        q_connections.iter().map(|connection| (connection.from_node, connection.to_node)),
    );

    resolved_inputs
        .by_node
        .retain(|entity, _| index.incoming_by_node_input.contains_key(entity));

    index.ordered_nodes.clear();
    index.ordered_nodes.extend(topology.ordered_nodes);
    index.blocked_nodes.clear();
    index.blocked_nodes.extend(topology.blocked_nodes);
}
