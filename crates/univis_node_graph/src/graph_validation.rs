use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

use crate::{
    document::GraphDocument,
    node_definition::{ArcNodeDefinition, PortRequirement},
    node_registry::NodeRegistry,
    value::NodeValue,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GraphTopologyAnalysis<T> {
    pub ordered_nodes: Vec<T>,
    pub blocked_nodes: Vec<T>,
}

pub fn connected_input_mask(
    input_count: usize,
    edges: impl IntoIterator<Item = usize>,
) -> Vec<bool> {
    let mut connected = vec![false; input_count];
    for input_index in edges {
        if let Some(slot) = connected.get_mut(input_index) {
            *slot = true;
        }
    }
    connected
}

pub fn output_satisfies_requirement(
    definition: &ArcNodeDefinition,
    output_index: usize,
    connected_inputs: &[bool],
    requirement: Option<&PortRequirement>,
) -> bool {
    let Some(requirement) = requirement else {
        return true;
    };

    definition.output_requirement_token(output_index, connected_inputs)
        == Some(requirement.id.clone())
}

impl<T> GraphTopologyAnalysis<T> {
    pub fn has_cycle_or_blocked_nodes(&self) -> bool {
        !self.blocked_nodes.is_empty()
    }
}

pub fn analyze_graph_topology<T, NI, EI>(nodes: NI, edges: EI) -> GraphTopologyAnalysis<T>
where
    T: Copy + Eq + Hash,
    NI: IntoIterator<Item = T>,
    EI: IntoIterator<Item = (T, T)>,
{
    let ordered_input_nodes: Vec<T> = nodes.into_iter().collect();
    let mut adjacency: HashMap<T, HashSet<T>> = HashMap::new();
    let mut in_degree: HashMap<T, usize> = HashMap::new();

    for node in &ordered_input_nodes {
        adjacency.entry(*node).or_default();
        in_degree.entry(*node).or_insert(0);
    }

    for (from, to) in edges {
        if !in_degree.contains_key(&from) || !in_degree.contains_key(&to) {
            continue;
        }

        if adjacency.entry(from).or_default().insert(to) {
            *in_degree.entry(to).or_insert(0) += 1;
        }
    }

    let mut queue = VecDeque::new();
    for node in &ordered_input_nodes {
        if in_degree.get(node).copied().unwrap_or_default() == 0 {
            queue.push_back(*node);
        }
    }

    let mut ordered_nodes = Vec::with_capacity(ordered_input_nodes.len());
    let mut processed = HashSet::with_capacity(ordered_input_nodes.len());

    while let Some(node) = queue.pop_front() {
        if !processed.insert(node) {
            continue;
        }

        ordered_nodes.push(node);

        if let Some(dependents) = adjacency.get(&node) {
            for dependent in dependents {
                if let Some(degree) = in_degree.get_mut(dependent) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(*dependent);
                    }
                }
            }
        }
    }

    let blocked_nodes = ordered_input_nodes
        .into_iter()
        .filter(|node| !processed.contains(node))
        .collect();

    GraphTopologyAnalysis {
        ordered_nodes,
        blocked_nodes,
    }
}

pub fn would_create_cycle<T, EI>(edges: EI, from: T, to: T) -> bool
where
    T: Copy + Eq + Hash,
    EI: IntoIterator<Item = (T, T)>,
{
    if from == to {
        return true;
    }

    let mut adjacency: HashMap<T, Vec<T>> = HashMap::new();
    for (edge_from, edge_to) in edges {
        adjacency.entry(edge_from).or_default().push(edge_to);
    }

    let mut queue = VecDeque::from([to]);
    let mut visited = HashSet::new();

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current) {
            continue;
        }

        if current == from {
            return true;
        }

        if let Some(next_nodes) = adjacency.get(&current) {
            queue.extend(next_nodes.iter().copied());
        }
    }

    false
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
                    if !NodeValue::is_compatible(from_type, to_type) {
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

                        if !output_satisfies_requirement(
                            &from_definition,
                            edge.from_index,
                            &source_connected_inputs,
                            to_port.requirement.as_ref(),
                        ) {
                            let requirement = to_port
                                .requirement
                                .as_ref()
                                .map(|requirement| requirement.label.as_str())
                                .unwrap_or("value");
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
