use std::collections::HashMap;

use bevy::prelude::*;
use univis_graph_core::prelude::{
    ExecutableGraph, ExecutableNodeBuildStatus, ExecutableNodeDiagnostic,
};
use univis_node_graph::prelude::{
    AuthoredNodeInputs, GraphConnection, GraphDocumentEdgeSnapshot, GraphDocumentNodeSnapshot,
    GraphNode, NodeRegistry, NodeValue, build_graph_document_from_snapshots,
};

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

#[derive(Resource, Default)]
pub struct GraphExecutableRuntimeState {
    pub graph: ExecutableGraph<NodeValue>,
    pub entity_to_node_id: HashMap<Entity, u64>,
    pub node_id_to_entity: HashMap<u64, Entity>,
    pub node_diagnostics: HashMap<u64, ExecutableNodeDiagnostic>,
}

impl GraphExecutableRuntimeState {
    pub fn node_id_for_entity(&self, entity: Entity) -> Option<u64> {
        self.entity_to_node_id.get(&entity).copied()
    }

    pub fn entity_for_node_id(&self, node_id: u64) -> Option<Entity> {
        self.node_id_to_entity.get(&node_id).copied()
    }
}

pub fn rebuild_connectivity_index_system(
    registry: Res<NodeRegistry>,
    mut state: ResMut<GraphExecutableRuntimeState>,
    mut index: ResMut<GraphConnectivityIndex>,
    mut resolved_inputs: ResMut<GraphResolvedInputs>,
    q_nodes: Query<(Entity, &GraphNode, Option<&AuthoredNodeInputs>)>,
    q_added_nodes: Query<Entity, Added<GraphNode>>,
    q_connections: Query<&GraphConnection>,
    q_changed_connections: Query<Entity, Or<(Added<GraphConnection>, Changed<GraphConnection>)>>,
    mut removed_nodes: RemovedComponents<GraphNode>,
    mut removed_connections: RemovedComponents<GraphConnection>,
) {
    let node_count = q_nodes.iter().len();
    let should_refresh = registry.is_changed()
        || !q_added_nodes.is_empty()
        || !q_changed_connections.is_empty()
        || removed_nodes.read().next().is_some()
        || removed_connections.read().next().is_some()
        || state.entity_to_node_id.len() != node_count
        || (state.graph.node_count() == 0 && (!q_nodes.is_empty() || !q_connections.is_empty()));
    if !should_refresh {
        return;
    }

    let node_snapshots =
        q_nodes.iter().map(
            |(entity, node, authored_inputs)| GraphDocumentNodeSnapshot {
                entity,
                definition_id: node.definition_id.clone(),
                inputs: authored_inputs
                    .map(|inputs| inputs.values.clone())
                    .unwrap_or_else(|| node.values.inputs.clone()),
                input_count: node.values.inputs.len(),
                output_count: node.values.outputs.len(),
                position: [0.0, 0.0],
                selected: false,
            },
        );
    let edge_snapshots = q_connections
        .iter()
        .map(|connection| GraphDocumentEdgeSnapshot {
            from_entity: connection.from_node,
            from_index: connection.from_index,
            to_entity: connection.to_node,
            to_index: connection.to_index,
        });
    let build = build_graph_document_from_snapshots(
        node_snapshots,
        edge_snapshots,
        None,
        Some(&state.entity_to_node_id),
    );
    let build_report = ExecutableGraph::build(&build.document, registry.core_registry());

    state.graph = build_report.graph;
    state.entity_to_node_id = build.entity_to_node_id;
    state.node_id_to_entity = build.node_id_to_entity;
    state.node_diagnostics = build_report.node_diagnostics;

    project_runtime_resources(&state, &mut index, &mut resolved_inputs);
}

pub fn project_runtime_resources(
    state: &GraphExecutableRuntimeState,
    index: &mut GraphConnectivityIndex,
    resolved_inputs: &mut GraphResolvedInputs,
) {
    index.incoming_by_node_input.clear();
    index.outgoing_by_node_output.clear();
    index.ordered_nodes.clear();
    index.blocked_nodes.clear();
    resolved_inputs.by_node.clear();

    let mut node_ids = state
        .graph
        .iter_nodes()
        .map(|node| node.node_id())
        .collect::<Vec<_>>();
    node_ids.sort_unstable();

    for node_id in node_ids {
        let Some(node) = state.graph.get_node(node_id) else {
            continue;
        };
        let Some(entity) = state.entity_for_node_id(node_id) else {
            continue;
        };

        let mut incoming = vec![None; node.input_count()];
        for (input_index, slot) in incoming.iter_mut().enumerate() {
            *slot = node
                .links()
                .incoming_links_for_input(input_index)
                .and_then(|links| links.first())
                .and_then(|link| {
                    state
                        .entity_for_node_id(link.node_id)
                        .map(|source_node| GraphInputSource {
                            source_node,
                            source_index: link.port_index,
                        })
                });
        }

        let mut outgoing = vec![Vec::new(); node.output_count()];
        for (output_index, targets) in outgoing.iter_mut().enumerate() {
            if let Some(links) = node.links().outgoing_links_for_output(output_index) {
                for link in links {
                    if let Some(target_node) = state.entity_for_node_id(link.node_id) {
                        targets.push(GraphOutputTarget {
                            target_node,
                            target_index: link.port_index,
                        });
                    }
                }
            }
        }

        index.incoming_by_node_input.insert(entity, incoming);
        index.outgoing_by_node_output.insert(entity, outgoing);
        index.ordered_nodes.push(entity);
        if node.is_blocked() {
            index.blocked_nodes.push(entity);
        }
        resolved_inputs
            .by_node
            .insert(entity, node.resolved_inputs().to_vec());
    }

    for (node_id, diagnostic) in &state.node_diagnostics {
        if diagnostic.status != ExecutableNodeBuildStatus::Blocked {
            continue;
        }
        if let Some(entity) = state.entity_for_node_id(*node_id) {
            if !index.blocked_nodes.contains(&entity) {
                index.blocked_nodes.push(entity);
            }
        }
    }

    index.blocked_nodes.sort_by_key(|entity| entity.index());
}
