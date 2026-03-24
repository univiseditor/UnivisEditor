use std::collections::HashMap;

use bevy::prelude::*;
use univis_graph_core::executable::ExecutableNode;
use univis_graph_core::prelude::ExecutableGraph;
use univis_node_graph::prelude::{
    build_graph_document_from_snapshots, graph_node_authored_inputs_for_snapshot,
    AuthoredNodeInputs, GraphConnection, GraphDocumentEdgeSnapshot, GraphDocumentNodeSnapshot,
    GraphNode, NodeRegistry, NodeValue,
};

#[derive(Component, Debug, Clone, Default)]
pub(crate) struct NodeInputSignature {
    pub inputs: Vec<NodeValue>,
}

#[derive(Component, Debug, Clone, Default)]
pub(crate) struct NodeOutputSignature {
    pub outputs: Vec<NodeValue>,
}

#[derive(Resource, Default)]
pub struct GraphExecutableRuntimeState {
    pub graph: ExecutableGraph<NodeValue>,
    pub entity_to_node_id: HashMap<Entity, u64>,
    pub node_id_to_entity: HashMap<u64, Entity>,
}

impl GraphExecutableRuntimeState {
    pub fn node_id_for_entity(&self, entity: Entity) -> Option<u64> {
        self.entity_to_node_id.get(&entity).copied()
    }

    pub fn entity_for_node_id(&self, node_id: u64) -> Option<Entity> {
        self.node_id_to_entity.get(&node_id).copied()
    }

    pub fn node_for_entity(&self, entity: Entity) -> Option<&ExecutableNode<NodeValue>> {
        self.node_id_for_entity(entity)
            .and_then(|node_id| self.graph.get_node(node_id))
    }

    pub fn blocked_entities(&self) -> Vec<Entity> {
        let mut blocked = self
            .graph
            .blocked_node_ids()
            .into_iter()
            .filter_map(|node_id| self.entity_for_node_id(node_id))
            .collect::<Vec<_>>();
        blocked.sort_by_key(|entity| entity.index());
        blocked
    }
}

pub fn rebuild_executable_runtime_state_system(
    registry: Res<NodeRegistry>,
    mut state: ResMut<GraphExecutableRuntimeState>,
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
                inputs: graph_node_authored_inputs_for_snapshot(node, authored_inputs),
                input_count: node.input_projection_len(),
                output_count: node.output_projection_len(),
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

    state.graph = build_report.into_graph();
    state.entity_to_node_id = build.entity_to_node_id;
    state.node_id_to_entity = build.node_id_to_entity;
}
