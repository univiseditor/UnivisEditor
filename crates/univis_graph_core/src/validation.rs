use std::collections::{HashMap, HashSet};

use crate::document::GraphDocument;
use crate::ports::{PortDefinition, PortSchema};
use crate::registry::GraphNodeRegistry;
use crate::schema::GraphSchema;
use crate::topology::{GraphTopologyAnalysis, analyze_graph_topology, connected_input_mask};

/// Stable categories for graph-document validation findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphValidationIssueKind {
    DuplicateNodeId,
    DuplicateEdge,
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

/// One validation issue found while inspecting a graph document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphValidationIssue {
    pub kind: GraphValidationIssueKind,
    pub edge_index: Option<usize>,
    pub node_ids: Vec<u64>,
    pub message: String,
}

impl GraphValidationIssue {
    pub fn new(
        kind: GraphValidationIssueKind,
        edge_index: Option<usize>,
        node_ids: Vec<u64>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            edge_index,
            node_ids,
            message: message.into(),
        }
    }
}

/// Aggregate validation output for a graph document, including topology state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GraphValidationReport {
    pub issues: Vec<GraphValidationIssue>,
    pub topology: GraphTopologyAnalysis<u64>,
}

impl GraphValidationReport {
    pub fn new(topology: GraphTopologyAnalysis<u64>) -> Self {
        Self {
            issues: Vec::new(),
            topology,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.is_valid()
    }

    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    pub fn push_issue(&mut self, issue: GraphValidationIssue) {
        self.issues.push(issue);
    }

    pub fn extend_issues<I>(&mut self, issues: I)
    where
        I: IntoIterator<Item = GraphValidationIssue>,
    {
        self.issues.extend(issues);
    }
}

/// Validates graph-document structure without applying schema-specific type or
/// requirement rules.
pub fn validate_graph_document_structure<S, Value, Prefab>(
    document: &GraphDocument<Value, Prefab>,
    registry: &GraphNodeRegistry<Value, PortDefinition<S>>,
) -> GraphValidationReport
where
    S: PortSchema,
{
    let mut report = GraphValidationReport::default();
    let mut nodes_by_id = HashMap::new();
    let mut seen_node_ids = HashSet::new();

    for node in &document.nodes {
        if !seen_node_ids.insert(node.id) {
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::DuplicateNodeId,
                None,
                vec![node.id],
                format!(
                    "Node id {} appears more than once in the document.",
                    node.id
                ),
            ));
        }

        nodes_by_id.insert(node.id, node);

        if registry.get(&node.definition_id).is_none() {
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::MissingNodeDefinition,
                None,
                vec![node.id],
                format!(
                    "Node {} references missing definition '{}'.",
                    node.id, node.definition_id
                ),
            ));
        }
    }

    let mut claimed_single_inputs = HashSet::new();
    let mut seen_edges = HashSet::new();
    let mut topology_edges = Vec::new();

    for (edge_index, edge) in document.edges.iter().enumerate() {
        let edge_key = (
            edge.from_node_id,
            edge.from_index,
            edge.to_node_id,
            edge.to_index,
        );
        if !seen_edges.insert(edge_key) {
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::DuplicateEdge,
                Some(edge_index),
                vec![edge.from_node_id, edge.to_node_id],
                format!(
                    "Edge {} duplicates an existing connection {}:{} -> {}:{}.",
                    edge_index, edge.from_node_id, edge.from_index, edge.to_node_id, edge.to_index
                ),
            ));
        }

        let Some(from_node) = nodes_by_id.get(&edge.from_node_id) else {
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::MissingSourceNode,
                Some(edge_index),
                vec![edge.from_node_id, edge.to_node_id],
                format!(
                    "Edge {} references missing source node {}.",
                    edge_index, edge.from_node_id
                ),
            ));
            continue;
        };

        let Some(to_node) = nodes_by_id.get(&edge.to_node_id) else {
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::MissingTargetNode,
                Some(edge_index),
                vec![edge.from_node_id, edge.to_node_id],
                format!(
                    "Edge {} references missing target node {}.",
                    edge_index, edge.to_node_id
                ),
            ));
            continue;
        };

        if edge.from_node_id == edge.to_node_id {
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::SelfConnection,
                Some(edge_index),
                vec![edge.from_node_id],
                format!(
                    "Edge {} creates a self-connection on node {}.",
                    edge_index, edge.from_node_id
                ),
            ));
            continue;
        }

        if edge.from_index >= from_node.output_count {
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::InvalidOutputPort,
                Some(edge_index),
                vec![edge.from_node_id],
                format!(
                    "Edge {} references output port {} on node {}, but the node has only {} outputs.",
                    edge_index, edge.from_index, edge.from_node_id, from_node.output_count
                ),
            ));
            continue;
        }

        if edge.to_index >= to_node.input_count {
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::InvalidInputPort,
                Some(edge_index),
                vec![edge.to_node_id],
                format!(
                    "Edge {} references input port {} on node {}, but the node has only {} inputs.",
                    edge_index, edge.to_index, edge.to_node_id, to_node.input_count
                ),
            ));
            continue;
        }

        if let Some(from_definition) = registry.get(&from_node.definition_id) {
            let outputs = from_definition.outputs();
            if outputs.get(edge.from_index).is_none() {
                report.push_issue(GraphValidationIssue::new(
                    GraphValidationIssueKind::InvalidOutputPort,
                    Some(edge_index),
                    vec![edge.from_node_id],
                    format!(
                        "Edge {} references missing output definition {} on node {}.",
                        edge_index, edge.from_index, edge.from_node_id
                    ),
                ));
                continue;
            }
        }

        let target_allows_multiple = registry
            .get(&to_node.definition_id)
            .and_then(|definition| {
                let inputs = definition.inputs();
                inputs
                    .get(edge.to_index)
                    .map(|port| port.accepts_multiple_connections())
            })
            .unwrap_or(false);

        if let Some(to_definition) = registry.get(&to_node.definition_id) {
            let inputs = to_definition.inputs();
            if inputs.get(edge.to_index).is_none() {
                report.push_issue(GraphValidationIssue::new(
                    GraphValidationIssueKind::InvalidInputPort,
                    Some(edge_index),
                    vec![edge.to_node_id],
                    format!(
                        "Edge {} references missing input definition {} on node {}.",
                        edge_index, edge.to_index, edge.to_node_id
                    ),
                ));
                continue;
            }
        }

        if !target_allows_multiple
            && !claimed_single_inputs.insert((edge.to_node_id, edge.to_index))
        {
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::InputAlreadyConnected,
                Some(edge_index),
                vec![edge.to_node_id],
                format!(
                    "Input {} on node {} is connected more than once.",
                    edge.to_index, edge.to_node_id
                ),
            ));
        }

        topology_edges.push((edge.from_node_id, edge.to_node_id));
    }

    report.topology =
        analyze_graph_topology(document.nodes.iter().map(|node| node.id), topology_edges);

    if report.topology.has_cycle_or_blocked_nodes() {
        let blocked = report
            .topology
            .blocked_nodes
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        report.push_issue(GraphValidationIssue::new(
            GraphValidationIssueKind::CycleDetected,
            None,
            report.topology.blocked_nodes.clone(),
            format!(
                "Graph contains a cycle or blocked evaluation path involving node ids: {}.",
                blocked
            ),
        ));
    }

    report
}

/// Validates a graph document, including schema-specific type and requirement
/// checks.
pub fn validate_graph_document<S, Value, Prefab>(
    document: &GraphDocument<Value, Prefab>,
    registry: &GraphNodeRegistry<Value, PortDefinition<S>>,
) -> GraphValidationReport
where
    S: GraphSchema,
{
    let mut report = validate_graph_document_structure(document, registry);
    let mut nodes_by_id = HashMap::new();
    let mut incoming_inputs: HashMap<u64, Vec<usize>> = HashMap::new();

    for node in &document.nodes {
        nodes_by_id.insert(node.id, node);
    }

    for edge in &document.edges {
        incoming_inputs
            .entry(edge.to_node_id)
            .or_default()
            .push(edge.to_index);
    }

    for (edge_index, edge) in document.edges.iter().enumerate() {
        let Some(from_node) = nodes_by_id.get(&edge.from_node_id) else {
            continue;
        };
        let Some(to_node) = nodes_by_id.get(&edge.to_node_id) else {
            continue;
        };

        if edge.from_node_id == edge.to_node_id {
            continue;
        }

        if edge.from_index >= from_node.output_count || edge.to_index >= to_node.input_count {
            continue;
        }

        let (Some(from_definition), Some(to_definition)) = (
            registry.get(&from_node.definition_id),
            registry.get(&to_node.definition_id),
        ) else {
            continue;
        };

        let from_outputs = from_definition.outputs();
        let to_inputs = to_definition.inputs();
        let Some(from_port) = from_outputs.get(edge.from_index) else {
            continue;
        };
        let Some(to_port) = to_inputs.get(edge.to_index) else {
            continue;
        };

        if !S::ports_compatible(&from_port.type_tag, &to_port.type_tag) {
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::IncompatiblePortTypes,
                Some(edge_index),
                vec![edge.from_node_id, edge.to_node_id],
                format!(
                    "Edge {} connects incompatible port types between node {} output {} and node {} input {}.",
                    edge_index, edge.from_node_id, edge.from_index, edge.to_node_id, edge.to_index
                ),
            ));
            continue;
        }

        let source_connected_inputs = connected_input_mask(
            from_node.input_count,
            incoming_inputs
                .get(&edge.from_node_id)
                .into_iter()
                .flatten()
                .copied(),
        );
        let output_requirement_token =
            from_definition.output_requirement_token(edge.from_index, &source_connected_inputs);

        if !S::requirement_satisfied(
            to_port.requirement.as_ref(),
            output_requirement_token.as_deref(),
        ) {
            let requirement = to_port
                .requirement
                .as_ref()
                .map(S::requirement_label)
                .unwrap_or_else(|| "value".to_string());
            report.push_issue(GraphValidationIssue::new(
                GraphValidationIssueKind::UnsatisfiedPortRequirement,
                Some(edge_index),
                vec![edge.from_node_id, edge.to_node_id],
                format!(
                    "Edge {} connects output {} on node {} to input {} on node {}, but the source does not satisfy requirement '{}'.",
                    edge_index,
                    edge.from_index,
                    edge.from_node_id,
                    edge.to_index,
                    edge.to_node_id,
                    requirement
                ),
            ));
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::{
        GraphValidationIssue, GraphValidationIssueKind, GraphValidationReport,
        validate_graph_document, validate_graph_document_structure,
    };
    use crate::document::{GraphDocument, GraphDocumentEdge, GraphDocumentNode};
    use crate::identity::{NodeCategory, NodeId};
    use crate::ports::{PortDefinition, PortSchema};
    use crate::processing::{GraphNodeDefinition, ProcessContext, ProcessResult};
    use crate::registry::GraphNodeRegistry;
    use crate::schema::GraphSchema;
    use crate::topology::GraphTopologyAnalysis;

    #[derive(Clone)]
    struct TestSchema;

    impl PortSchema for TestSchema {
        type TypeTag = &'static str;
        type Requirement = &'static str;
        type DefaultValue = ();

        fn ports_compatible(from: &Self::TypeTag, to: &Self::TypeTag) -> bool {
            from == to || *from == "any" || *to == "any"
        }
    }

    impl GraphSchema for TestSchema {
        fn requirement_satisfied(
            requirement: Option<&Self::Requirement>,
            output_requirement_token: Option<&str>,
        ) -> bool {
            requirement.is_none_or(|requirement| output_requirement_token == Some(*requirement))
        }

        fn requirement_label(requirement: &Self::Requirement) -> String {
            requirement.to_string()
        }
    }

    struct TestNode {
        id: NodeId,
        inputs: Vec<PortDefinition<TestSchema>>,
        outputs: Vec<PortDefinition<TestSchema>>,
        output_requirement_tokens: Vec<Option<&'static str>>,
    }

    impl GraphNodeDefinition<(), PortDefinition<TestSchema>> for TestNode {
        fn id(&self) -> NodeId {
            self.id.clone()
        }

        fn display_name(&self) -> &str {
            "Test Node"
        }

        fn category(&self) -> NodeCategory {
            NodeCategory::new("Tests")
        }

        fn inputs(&self) -> Vec<PortDefinition<TestSchema>> {
            self.inputs.clone()
        }

        fn outputs(&self) -> Vec<PortDefinition<TestSchema>> {
            self.outputs.clone()
        }

        fn output_requirement_token(
            &self,
            output_index: usize,
            _connected_inputs: &[bool],
        ) -> Option<String> {
            self.output_requirement_tokens
                .get(output_index)
                .and_then(|token| token.map(str::to_string))
        }

        fn process(&self, _context: &mut ProcessContext<'_, ()>) -> ProcessResult {
            ProcessResult::Success
        }
    }

    fn node(
        id: u64,
        definition_id: &str,
        input_count: usize,
        output_count: usize,
    ) -> GraphDocumentNode<()> {
        GraphDocumentNode {
            id,
            definition_id: NodeId::new(definition_id),
            position: [0.0, 0.0],
            inputs: vec![],
            input_count,
            output_count,
        }
    }

    #[test]
    fn default_report_starts_valid() {
        let report = GraphValidationReport::default();

        assert!(report.is_valid());
        assert!(!report.has_errors());
        assert_eq!(report.issue_count(), 0);
    }

    #[test]
    fn report_collects_issues() {
        let topology = GraphTopologyAnalysis {
            ordered_nodes: vec![1],
            blocked_nodes: vec![2],
        };
        let mut report = GraphValidationReport::new(topology.clone());

        report.push_issue(GraphValidationIssue::new(
            GraphValidationIssueKind::DuplicateNodeId,
            None,
            vec![1],
            "Node id 1 appears more than once.",
        ));

        assert!(!report.is_valid());
        assert!(report.has_errors());
        assert_eq!(report.issue_count(), 1);
        assert_eq!(report.topology, topology);
    }

    #[test]
    fn structural_validation_reports_duplicate_nodes_missing_defs_and_cycles() {
        let mut registry = GraphNodeRegistry::<(), PortDefinition<TestSchema>>::new();
        registry.register(TestNode {
            id: NodeId::new("tests/source"),
            inputs: vec![],
            outputs: vec![PortDefinition::<TestSchema>::new("Out 0", "any")],
            output_requirement_tokens: vec![None],
        });
        registry.register(TestNode {
            id: NodeId::new("tests/target"),
            inputs: vec![PortDefinition::<TestSchema>::new("In 0", "any")],
            outputs: vec![PortDefinition::<TestSchema>::new("Out 0", "any")],
            output_requirement_tokens: vec![None],
        });

        let document: GraphDocument<(), ()> = GraphDocument {
            nodes: vec![
                node(1, "tests/source", 0, 1),
                node(2, "tests/target", 1, 1),
                node(2, "tests/missing", 0, 0),
            ],
            edges: vec![
                GraphDocumentEdge {
                    from_node_id: 1,
                    from_index: 0,
                    to_node_id: 2,
                    to_index: 0,
                },
                GraphDocumentEdge {
                    from_node_id: 2,
                    from_index: 0,
                    to_node_id: 1,
                    to_index: 0,
                },
            ],
            ..GraphDocument::default()
        };

        let report = validate_graph_document_structure(&document, &registry);

        assert!(report.issues.iter().any(|issue| {
            issue.kind == GraphValidationIssueKind::DuplicateNodeId && issue.node_ids == vec![2]
        }));
        assert!(report.issues.iter().any(|issue| {
            issue.kind == GraphValidationIssueKind::MissingNodeDefinition
                && issue.node_ids == vec![2]
        }));
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.kind == GraphValidationIssueKind::CycleDetected)
        );
    }

    #[test]
    fn structural_validation_respects_input_connection_policy() {
        let mut registry = GraphNodeRegistry::<(), PortDefinition<TestSchema>>::new();
        registry.register(TestNode {
            id: NodeId::new("tests/source_a"),
            inputs: vec![],
            outputs: vec![PortDefinition::<TestSchema>::new("Out 0", "any")],
            output_requirement_tokens: vec![None],
        });
        registry.register(TestNode {
            id: NodeId::new("tests/source_b"),
            inputs: vec![],
            outputs: vec![PortDefinition::<TestSchema>::new("Out 0", "any")],
            output_requirement_tokens: vec![None],
        });
        registry.register(TestNode {
            id: NodeId::new("tests/single_sink"),
            inputs: vec![PortDefinition::<TestSchema>::new("In 0", "any")],
            outputs: vec![],
            output_requirement_tokens: vec![],
        });
        registry.register(TestNode {
            id: NodeId::new("tests/multi_sink"),
            inputs: vec![
                PortDefinition::<TestSchema>::new("In 0", "any").allow_multiple_connections(),
            ],
            outputs: vec![],
            output_requirement_tokens: vec![],
        });

        let single_sink_document: GraphDocument<(), ()> = GraphDocument {
            nodes: vec![
                node(1, "tests/source_a", 0, 1),
                node(2, "tests/source_b", 0, 1),
                node(3, "tests/single_sink", 1, 0),
            ],
            edges: vec![
                GraphDocumentEdge {
                    from_node_id: 1,
                    from_index: 0,
                    to_node_id: 3,
                    to_index: 0,
                },
                GraphDocumentEdge {
                    from_node_id: 2,
                    from_index: 0,
                    to_node_id: 3,
                    to_index: 0,
                },
            ],
            ..GraphDocument::default()
        };
        let multi_sink_document: GraphDocument<(), ()> = GraphDocument {
            nodes: vec![
                node(1, "tests/source_a", 0, 1),
                node(2, "tests/source_b", 0, 1),
                node(4, "tests/multi_sink", 1, 0),
            ],
            edges: vec![
                GraphDocumentEdge {
                    from_node_id: 1,
                    from_index: 0,
                    to_node_id: 4,
                    to_index: 0,
                },
                GraphDocumentEdge {
                    from_node_id: 2,
                    from_index: 0,
                    to_node_id: 4,
                    to_index: 0,
                },
            ],
            ..GraphDocument::default()
        };

        let single_sink_report =
            validate_graph_document_structure(&single_sink_document, &registry);
        let multi_sink_report = validate_graph_document_structure(&multi_sink_document, &registry);

        assert!(single_sink_report.issues.iter().any(|issue| {
            issue.kind == GraphValidationIssueKind::InputAlreadyConnected
                && issue.node_ids == vec![3]
        }));
        assert!(!multi_sink_report.issues.iter().any(|issue| {
            issue.kind == GraphValidationIssueKind::InputAlreadyConnected
                && issue.node_ids == vec![4]
        }));
    }

    #[test]
    fn schema_validation_reports_incompatible_types_and_unsatisfied_requirements() {
        let mut registry = GraphNodeRegistry::<(), PortDefinition<TestSchema>>::new();
        registry.register(TestNode {
            id: NodeId::new("tests/number_source"),
            inputs: vec![],
            outputs: vec![PortDefinition::<TestSchema>::new("Value", "number")],
            output_requirement_tokens: vec![None],
        });
        registry.register(TestNode {
            id: NodeId::new("tests/bool_sink"),
            inputs: vec![PortDefinition::<TestSchema>::new("Value", "bool")],
            outputs: vec![],
            output_requirement_tokens: vec![],
        });
        registry.register(TestNode {
            id: NodeId::new("tests/required_sink"),
            inputs: vec![
                PortDefinition::<TestSchema>::new("Value", "number")
                    .with_requirement("numeric-output"),
            ],
            outputs: vec![],
            output_requirement_tokens: vec![],
        });

        let document: GraphDocument<(), ()> = GraphDocument {
            nodes: vec![
                node(1, "tests/number_source", 0, 1),
                node(2, "tests/bool_sink", 1, 0),
                node(3, "tests/required_sink", 1, 0),
            ],
            edges: vec![
                GraphDocumentEdge {
                    from_node_id: 1,
                    from_index: 0,
                    to_node_id: 2,
                    to_index: 0,
                },
                GraphDocumentEdge {
                    from_node_id: 1,
                    from_index: 0,
                    to_node_id: 3,
                    to_index: 0,
                },
            ],
            ..GraphDocument::default()
        };

        let report = validate_graph_document(&document, &registry);

        assert!(report.issues.iter().any(|issue| {
            issue.kind == GraphValidationIssueKind::IncompatiblePortTypes
                && issue.node_ids == vec![1, 2]
        }));
        assert!(report.issues.iter().any(|issue| {
            issue.kind == GraphValidationIssueKind::UnsatisfiedPortRequirement
                && issue.node_ids == vec![1, 3]
        }));
    }

    #[test]
    fn schema_validation_accepts_satisfied_requirement_tokens() {
        let mut registry = GraphNodeRegistry::<(), PortDefinition<TestSchema>>::new();
        registry.register(TestNode {
            id: NodeId::new("tests/tagged_source"),
            inputs: vec![PortDefinition::<TestSchema>::new("Trigger", "any")],
            outputs: vec![PortDefinition::<TestSchema>::new("Entity", "number")],
            output_requirement_tokens: vec![Some("numeric-output")],
        });
        registry.register(TestNode {
            id: NodeId::new("tests/required_sink"),
            inputs: vec![
                PortDefinition::<TestSchema>::new("Value", "number")
                    .with_requirement("numeric-output"),
            ],
            outputs: vec![],
            output_requirement_tokens: vec![],
        });

        let document: GraphDocument<(), ()> = GraphDocument {
            nodes: vec![
                node(1, "tests/tagged_source", 1, 1),
                node(2, "tests/required_sink", 1, 0),
            ],
            edges: vec![GraphDocumentEdge {
                from_node_id: 1,
                from_index: 0,
                to_node_id: 2,
                to_index: 0,
            }],
            ..GraphDocument::default()
        };

        let report = validate_graph_document(&document, &registry);

        assert!(
            report
                .issues
                .iter()
                .all(|issue| issue.kind != GraphValidationIssueKind::UnsatisfiedPortRequirement)
        );
    }
}
