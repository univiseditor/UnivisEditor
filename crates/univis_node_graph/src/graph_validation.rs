use std::collections::{HashMap, HashSet};
pub use univis_graph_core::prelude::{
    GraphTopologyAnalysis, analyze_graph_topology, connected_input_mask, would_create_cycle,
};

use crate::{
    document::GraphDocument,
    node_definition::NodeGraphSchema,
    node_registry::NodeRegistry,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphValidationIssueKind {
    DuplicateNodeId,
    MissingNodeDefinition,
    MissingSourceNode,
    MissingTargetNode,
    SelfConnection,
    InvalidOutputPort,
    InvalidInputPort,
    InputAlreadyConnected,
    IncompatiblePortTypes,
    UnsatisfiedPortRequirement,
    CycleDetected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphValidationIssue {
    pub kind: GraphValidationIssueKind,
    pub edge_index: Option<usize>,
    pub node_ids: Vec<u64>,
    pub message: String,
}

pub fn validate_graph_document(
    document: &GraphDocument,
    registry: &NodeRegistry,
) -> Vec<GraphValidationIssue> {
    let mut issues = Vec::new();
    let mut nodes_by_id = HashMap::new();
    let mut seen_node_ids = HashSet::new();

    for node in &document.nodes {
        if !seen_node_ids.insert(node.id) {
            issues.push(GraphValidationIssue {
                kind: GraphValidationIssueKind::DuplicateNodeId,
                edge_index: None,
                node_ids: vec![node.id],
                message: format!(
                    "Node id {} appears more than once in the document.",
                    node.id
                ),
            });
        }

        nodes_by_id.insert(node.id, node);

        if registry.get(&node.definition_id).is_none() {
            issues.push(GraphValidationIssue {
                kind: GraphValidationIssueKind::MissingNodeDefinition,
                edge_index: None,
                node_ids: vec![node.id],
                message: format!(
                    "Node {} references missing definition '{}'.",
                    node.id, node.definition_id
                ),
            });
        }
    }

    let mut claimed_inputs = HashSet::new();
    let mut topology_edges = Vec::new();
    let mut incoming_inputs: HashMap<u64, Vec<usize>> = HashMap::new();

    for edge in &document.edges {
        incoming_inputs
            .entry(edge.to_node_id)
            .or_default()
            .push(edge.to_index);
    }

    for (edge_index, edge) in document.edges.iter().enumerate() {
        let Some(from_node) = nodes_by_id.get(&edge.from_node_id) else {
            issues.push(GraphValidationIssue {
                kind: GraphValidationIssueKind::MissingSourceNode,
                edge_index: Some(edge_index),
                node_ids: vec![edge.from_node_id, edge.to_node_id],
                message: format!(
                    "Edge {} references missing source node {}.",
                    edge_index, edge.from_node_id
                ),
            });
            continue;
        };

        let Some(to_node) = nodes_by_id.get(&edge.to_node_id) else {
            issues.push(GraphValidationIssue {
                kind: GraphValidationIssueKind::MissingTargetNode,
                edge_index: Some(edge_index),
                node_ids: vec![edge.from_node_id, edge.to_node_id],
                message: format!(
                    "Edge {} references missing target node {}.",
                    edge_index, edge.to_node_id
                ),
            });
            continue;
        };

        if edge.from_node_id == edge.to_node_id {
            issues.push(GraphValidationIssue {
                kind: GraphValidationIssueKind::SelfConnection,
                edge_index: Some(edge_index),
                node_ids: vec![edge.from_node_id],
                message: format!(
                    "Edge {} creates a self-connection on node {}.",
                    edge_index, edge.from_node_id
                ),
            });
            continue;
        }

        if edge.from_index >= from_node.output_count {
            issues.push(GraphValidationIssue {
                kind: GraphValidationIssueKind::InvalidOutputPort,
                edge_index: Some(edge_index),
                node_ids: vec![edge.from_node_id],
                message: format!(
                    "Edge {} references output port {} on node {}, but the node has only {} outputs.",
                    edge_index, edge.from_index, edge.from_node_id, from_node.output_count
                ),
            });
            continue;
        }

        if edge.to_index >= to_node.input_count {
            issues.push(GraphValidationIssue {
                kind: GraphValidationIssueKind::InvalidInputPort,
                edge_index: Some(edge_index),
                node_ids: vec![edge.to_node_id],
                message: format!(
                    "Edge {} references input port {} on node {}, but the node has only {} inputs.",
                    edge_index, edge.to_index, edge.to_node_id, to_node.input_count
                ),
            });
            continue;
        }

        if !claimed_inputs.insert((edge.to_node_id, edge.to_index)) {
            issues.push(GraphValidationIssue {
                kind: GraphValidationIssueKind::InputAlreadyConnected,
                edge_index: Some(edge_index),
                node_ids: vec![edge.to_node_id],
                message: format!(
                    "Input {} on node {} is connected more than once.",
                    edge.to_index, edge.to_node_id
                ),
            });
        }

        if let (Some(from_definition), Some(to_definition)) = (
            registry.get(&from_node.definition_id),
            registry.get(&to_node.definition_id),
        ) {
            let from_outputs = from_definition.outputs();
            let to_inputs = to_definition.inputs();
            let from_port = from_outputs.get(edge.from_index);
            let to_port = to_inputs.get(edge.to_index);
            let from_type = from_port.map(|port| &port.value_type);
            let to_type = to_port.map(|port| &port.value_type);

            match (from_type, to_type) {
                (Some(from_type), Some(to_type)) => {
                    if !NodeGraphSchema::ports_compatible(from_type, to_type) {
                        issues.push(GraphValidationIssue {
                            kind: GraphValidationIssueKind::IncompatiblePortTypes,
                            edge_index: Some(edge_index),
                            node_ids: vec![edge.from_node_id, edge.to_node_id],
                            message: format!(
                                "Edge {} connects incompatible port types {} -> {}.",
                                edge_index,
                                from_type.display_name(),
                                to_type.display_name()
                            ),
                        });
                    } else if let Some(to_port) = to_port {
                        let source_connected_inputs = connected_input_mask(
                            from_node.input_count,
                            incoming_inputs
                                .get(&edge.from_node_id)
                                .into_iter()
                                .flatten()
                                .copied(),
                        );
                        let output_requirement_token = from_definition
                            .output_requirement_token(edge.from_index, &source_connected_inputs);

                        if !NodeGraphSchema::requirement_satisfied(
                            to_port.requirement.as_ref(),
                            output_requirement_token.as_deref(),
                        ) {
                            let requirement = to_port
                                .requirement
                                .as_ref()
                                .map(NodeGraphSchema::requirement_label)
                                .unwrap_or_else(|| "value".to_string());
                            issues.push(GraphValidationIssue {
                                kind: GraphValidationIssueKind::UnsatisfiedPortRequirement,
                                edge_index: Some(edge_index),
                                node_ids: vec![edge.from_node_id, edge.to_node_id],
                                message: format!(
                                    "Edge {} connects output {} on node {} to input {} on node {}, but the source does not satisfy requirement '{}'.",
                                    edge_index,
                                    edge.from_index,
                                    edge.from_node_id,
                                    edge.to_index,
                                    edge.to_node_id,
                                    requirement
                                ),
                            });
                        }
                    }
                }
                (None, _) => issues.push(GraphValidationIssue {
                    kind: GraphValidationIssueKind::InvalidOutputPort,
                    edge_index: Some(edge_index),
                    node_ids: vec![edge.from_node_id],
                    message: format!(
                        "Edge {} references missing output definition {} on node {}.",
                        edge_index, edge.from_index, edge.from_node_id
                    ),
                }),
                (_, None) => issues.push(GraphValidationIssue {
                    kind: GraphValidationIssueKind::InvalidInputPort,
                    edge_index: Some(edge_index),
                    node_ids: vec![edge.to_node_id],
                    message: format!(
                        "Edge {} references missing input definition {} on node {}.",
                        edge_index, edge.to_index, edge.to_node_id
                    ),
                }),
            }
        }

        topology_edges.push((edge.from_node_id, edge.to_node_id));
    }

    let topology =
        analyze_graph_topology(document.nodes.iter().map(|node| node.id), topology_edges);

    if topology.has_cycle_or_blocked_nodes() {
        let blocked = topology
            .blocked_nodes
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        issues.push(GraphValidationIssue {
            kind: GraphValidationIssueKind::CycleDetected,
            edge_index: None,
            node_ids: topology.blocked_nodes.clone(),
            message: format!(
                "Graph contains a cycle or blocked evaluation path involving node ids: {}.",
                blocked
            ),
        });
    }

    issues
}
