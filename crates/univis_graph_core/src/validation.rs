use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

use crate::document::GraphDocument;
use crate::ports::{PortDefinition, PortSchema};
use crate::registry::GraphNodeRegistry;
use crate::schema::GraphSchema;
use crate::topology::{
    analyze_graph_topology, connected_input_mask, would_create_cycle, GraphTopologyAnalysis,
};

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
    UnsupportedConnectionPolicy,
    IncompatiblePortTypes,
    UnsatisfiedPortRequirement,
    CycleDetected,
}

/// Describes a single prospective connection between two graph nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphConnectionCandidate<NodeId> {
    pub from_node_id: NodeId,
    pub from_index: usize,
    pub to_node_id: NodeId,
    pub to_index: usize,
}

impl From<&crate::document::GraphDocumentEdge> for GraphConnectionCandidate<u64> {
    fn from(edge: &crate::document::GraphDocumentEdge) -> Self {
        Self {
            from_node_id: edge.from_node_id,
            from_index: edge.from_index,
            to_node_id: edge.to_node_id,
            to_index: edge.to_index,
        }
    }
}

/// Runtime options for validating a connection candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphConnectionValidationOptions {
    allow_multiple_inputs: bool,
}

impl GraphConnectionValidationOptions {
    pub const fn live_connection_rules() -> Self {
        Self {
            allow_multiple_inputs: false,
        }
    }
}

/// Structural data needed to validate a connection without schema-specific rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphStructuralConnectionValidationContext<NodeId> {
    pub candidate: GraphConnectionCandidate<NodeId>,
    pub source_output_count: usize,
    pub target_input_count: usize,
    pub source_output_available: bool,
    pub target_input_available: bool,
    pub target_accepts_multiple_connections: bool,
}

/// Schema-aware data needed to validate port compatibility and requirements.
#[derive(Clone, Copy)]
pub struct GraphSchemaConnectionValidationContext<'a, S: GraphSchema> {
    pub source_output: &'a PortDefinition<S>,
    pub target_input: &'a PortDefinition<S>,
    pub source_connected_inputs: &'a [bool],
    pub output_requirement_token: Option<&'a str>,
}

/// Standardized reasons a connection can be rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphConnectionValidationError {
    SelfConnection,
    InvalidOutputPort { index: usize, output_count: usize },
    MissingOutputDefinition { index: usize },
    InvalidInputPort { index: usize, input_count: usize },
    MissingInputDefinition { index: usize },
    DuplicateEdge,
    InputAlreadyConnected,
    MultipleInputsUnsupported,
    IncompatiblePortTypes,
    UnsatisfiedPortRequirement { requirement_label: String },
    CycleDetected,
}

impl GraphConnectionValidationError {
    pub fn validation_issue_kind(&self) -> Option<GraphValidationIssueKind> {
        match self {
            Self::SelfConnection => Some(GraphValidationIssueKind::SelfConnection),
            Self::InvalidOutputPort { .. } | Self::MissingOutputDefinition { .. } => {
                Some(GraphValidationIssueKind::InvalidOutputPort)
            }
            Self::InvalidInputPort { .. } | Self::MissingInputDefinition { .. } => {
                Some(GraphValidationIssueKind::InvalidInputPort)
            }
            Self::DuplicateEdge => Some(GraphValidationIssueKind::DuplicateEdge),
            Self::InputAlreadyConnected => Some(GraphValidationIssueKind::InputAlreadyConnected),
            Self::MultipleInputsUnsupported => {
                Some(GraphValidationIssueKind::UnsupportedConnectionPolicy)
            }
            Self::IncompatiblePortTypes => Some(GraphValidationIssueKind::IncompatiblePortTypes),
            Self::UnsatisfiedPortRequirement { .. } => {
                Some(GraphValidationIssueKind::UnsatisfiedPortRequirement)
            }
            Self::CycleDetected => Some(GraphValidationIssueKind::CycleDetected),
        }
    }

    pub fn retains_connection_for_topology(&self) -> bool {
        matches!(
            self,
            Self::DuplicateEdge
                | Self::InputAlreadyConnected
                | Self::IncompatiblePortTypes
                | Self::UnsatisfiedPortRequirement { .. }
                | Self::CycleDetected
        )
    }

    pub fn related_node_ids<NodeId: Copy>(
        &self,
        candidate: GraphConnectionCandidate<NodeId>,
    ) -> Vec<NodeId> {
        match self {
            Self::SelfConnection => vec![candidate.from_node_id],
            Self::InvalidOutputPort { .. } | Self::MissingOutputDefinition { .. } => {
                vec![candidate.from_node_id]
            }
            Self::InvalidInputPort { .. }
            | Self::MissingInputDefinition { .. }
            | Self::InputAlreadyConnected
            | Self::MultipleInputsUnsupported => vec![candidate.to_node_id],
            Self::DuplicateEdge
            | Self::IncompatiblePortTypes
            | Self::UnsatisfiedPortRequirement { .. }
            | Self::CycleDetected => vec![candidate.from_node_id, candidate.to_node_id],
        }
    }
}

impl fmt::Display for GraphConnectionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelfConnection => write!(f, "A node cannot connect to itself."),
            Self::InvalidOutputPort {
                index,
                output_count,
            } => write!(
                f,
                "Output port {} is out of bounds ({} outputs).",
                index, output_count
            ),
            Self::MissingOutputDefinition { index } => {
                write!(f, "Output definition {} is missing.", index)
            }
            Self::InvalidInputPort { index, input_count } => write!(
                f,
                "Input port {} is out of bounds ({} inputs).",
                index, input_count
            ),
            Self::MissingInputDefinition { index } => {
                write!(f, "Input definition {} is missing.", index)
            }
            Self::DuplicateEdge => write!(f, "This connection already exists."),
            Self::InputAlreadyConnected => write!(f, "The target input is already connected."),
            Self::MultipleInputsUnsupported => write!(
                f,
                "Multiple-source inputs are currently unsupported in this graph stack."
            ),
            Self::IncompatiblePortTypes => {
                write!(f, "The source and target port types are incompatible.")
            }
            Self::UnsatisfiedPortRequirement { requirement_label } => write!(
                f,
                "The target input requirement '{}' is not satisfied.",
                requirement_label
            ),
            Self::CycleDetected => write!(f, "This connection would create a cycle."),
        }
    }
}

impl std::error::Error for GraphConnectionValidationError {}

/// Validates structural connection rules that should stay shared across core,
/// UI preview, and persistence/apply flows.
pub fn validate_structural_connection_candidate<NodeId, ExistingEdges>(
    context: GraphStructuralConnectionValidationContext<NodeId>,
    existing_edges: ExistingEdges,
    options: GraphConnectionValidationOptions,
) -> Result<(), GraphConnectionValidationError>
where
    NodeId: Copy + Eq + Hash,
    ExistingEdges: IntoIterator<Item = GraphConnectionCandidate<NodeId>>,
{
    let candidate = context.candidate;

    if candidate.from_node_id == candidate.to_node_id {
        return Err(GraphConnectionValidationError::SelfConnection);
    }

    if candidate.from_index >= context.source_output_count {
        return Err(GraphConnectionValidationError::InvalidOutputPort {
            index: candidate.from_index,
            output_count: context.source_output_count,
        });
    }

    if candidate.to_index >= context.target_input_count {
        return Err(GraphConnectionValidationError::InvalidInputPort {
            index: candidate.to_index,
            input_count: context.target_input_count,
        });
    }

    if !context.source_output_available {
        return Err(GraphConnectionValidationError::MissingOutputDefinition {
            index: candidate.from_index,
        });
    }

    if !context.target_input_available {
        return Err(GraphConnectionValidationError::MissingInputDefinition {
            index: candidate.to_index,
        });
    }

    let existing_edges: Vec<_> = existing_edges.into_iter().collect();

    if existing_edges.iter().any(|edge| *edge == candidate) {
        return Err(GraphConnectionValidationError::DuplicateEdge);
    }

    if context.target_accepts_multiple_connections {
        if !options.allow_multiple_inputs {
            return Err(GraphConnectionValidationError::MultipleInputsUnsupported);
        }
    } else if existing_edges
        .iter()
        .any(|edge| edge.to_node_id == candidate.to_node_id && edge.to_index == candidate.to_index)
    {
        return Err(GraphConnectionValidationError::InputAlreadyConnected);
    }

    if would_create_cycle(
        existing_edges
            .iter()
            .map(|edge| (edge.from_node_id, edge.to_node_id)),
        candidate.from_node_id,
        candidate.to_node_id,
    ) {
        return Err(GraphConnectionValidationError::CycleDetected);
    }

    Ok(())
}

/// Validates schema-driven connection rules such as port compatibility and
/// requirement matching.
pub fn validate_schema_connection_candidate<S>(
    context: GraphSchemaConnectionValidationContext<'_, S>,
) -> Result<(), GraphConnectionValidationError>
where
    S: GraphSchema,
{
    if !S::ports_compatible(
        &context.source_output.type_tag,
        &context.target_input.type_tag,
    ) {
        return Err(GraphConnectionValidationError::IncompatiblePortTypes);
    }

    if !S::requirement_satisfied(
        context.target_input.requirement.as_ref(),
        context.output_requirement_token,
    ) {
        let requirement_label = context
            .target_input
            .requirement
            .as_ref()
            .map(S::requirement_label)
            .unwrap_or_else(|| "value".to_string());
        return Err(GraphConnectionValidationError::UnsatisfiedPortRequirement {
            requirement_label,
        });
    }

    Ok(())
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

fn structural_connection_issue_from_error(
    edge_index: usize,
    candidate: GraphConnectionCandidate<u64>,
    error: &GraphConnectionValidationError,
) -> Option<GraphValidationIssue> {
    let kind = error.validation_issue_kind()?;
    let node_ids = error.related_node_ids(candidate);
    let message = match error {
        GraphConnectionValidationError::SelfConnection => format!(
            "Edge {} creates a self-connection on node {}.",
            edge_index, candidate.from_node_id
        ),
        GraphConnectionValidationError::InvalidOutputPort {
            index,
            output_count,
        } => format!(
            "Edge {} references output port {} on node {}, but the node has only {} outputs.",
            edge_index, index, candidate.from_node_id, output_count
        ),
        GraphConnectionValidationError::MissingOutputDefinition { index } => format!(
            "Edge {} references missing output definition {} on node {}.",
            edge_index, index, candidate.from_node_id
        ),
        GraphConnectionValidationError::InvalidInputPort { index, input_count } => format!(
            "Edge {} references input port {} on node {}, but the node has only {} inputs.",
            edge_index, index, candidate.to_node_id, input_count
        ),
        GraphConnectionValidationError::MissingInputDefinition { index } => format!(
            "Edge {} references missing input definition {} on node {}.",
            edge_index, index, candidate.to_node_id
        ),
        GraphConnectionValidationError::DuplicateEdge => format!(
            "Edge {} duplicates an existing connection {}:{} -> {}:{}.",
            edge_index,
            candidate.from_node_id,
            candidate.from_index,
            candidate.to_node_id,
            candidate.to_index
        ),
        GraphConnectionValidationError::InputAlreadyConnected => format!(
            "Input {} on node {} is connected more than once.",
            candidate.to_index, candidate.to_node_id
        ),
        GraphConnectionValidationError::MultipleInputsUnsupported => format!(
            "Edge {} targets input {} on node {}, but `ConnectionPolicy::Multiple` is currently unsupported in the workspace.",
            edge_index, candidate.to_index, candidate.to_node_id
        ),
        GraphConnectionValidationError::IncompatiblePortTypes => format!(
            "Edge {} connects incompatible port types between node {} output {} and node {} input {}.",
            edge_index,
            candidate.from_node_id,
            candidate.from_index,
            candidate.to_node_id,
            candidate.to_index
        ),
        GraphConnectionValidationError::UnsatisfiedPortRequirement { requirement_label } => {
            format!(
                "Edge {} connects output {} on node {} to input {} on node {}, but the source does not satisfy requirement '{}'.",
                edge_index,
                candidate.from_index,
                candidate.from_node_id,
                candidate.to_index,
                candidate.to_node_id,
                requirement_label
            )
        }
        GraphConnectionValidationError::CycleDetected => format!(
            "Edge {} would create a cycle between node {} and node {}.",
            edge_index, candidate.from_node_id, candidate.to_node_id
        ),
    };

    Some(GraphValidationIssue::new(
        kind,
        Some(edge_index),
        node_ids,
        message,
    ))
}

fn push_topology_cycle_issue_if_needed(report: &mut GraphValidationReport) {
    if !report.topology.has_cycle_or_blocked_nodes()
        || report
            .issues
            .iter()
            .any(|issue| issue.kind == GraphValidationIssueKind::CycleDetected)
    {
        return;
    }

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
    let validation_options = GraphConnectionValidationOptions::live_connection_rules();

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

    let mut retained_edges = Vec::new();

    for (edge_index, edge) in document.edges.iter().enumerate() {
        let candidate = GraphConnectionCandidate::from(edge);

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

        let source_output_available = registry
            .get(&from_node.definition_id)
            .map(|definition| definition.outputs().get(edge.from_index).is_some())
            .unwrap_or(true);
        let target_input_available = registry
            .get(&to_node.definition_id)
            .map(|definition| definition.inputs().get(edge.to_index).is_some())
            .unwrap_or(true);
        let target_accepts_multiple_connections = registry
            .get(&to_node.definition_id)
            .and_then(|definition| {
                let inputs = definition.inputs();
                inputs
                    .get(edge.to_index)
                    .map(|port| port.accepts_multiple_connections())
            })
            .unwrap_or(false);

        let structural_context = GraphStructuralConnectionValidationContext {
            candidate,
            source_output_count: from_node.output_count,
            target_input_count: to_node.input_count,
            source_output_available,
            target_input_available,
            target_accepts_multiple_connections,
        };

        match validate_structural_connection_candidate(
            structural_context,
            retained_edges.iter().copied(),
            validation_options,
        ) {
            Ok(()) => retained_edges.push(candidate),
            Err(error) => {
                if let Some(issue) =
                    structural_connection_issue_from_error(edge_index, candidate, &error)
                {
                    report.push_issue(issue);
                }

                if error.retains_connection_for_topology() {
                    retained_edges.push(candidate);
                }
            }
        }
    }

    report.topology = analyze_graph_topology(
        document.nodes.iter().map(|node| node.id),
        retained_edges
            .iter()
            .map(|edge| (edge.from_node_id, edge.to_node_id)),
    );
    push_topology_cycle_issue_if_needed(&mut report);

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

        if let Err(error) =
            validate_schema_connection_candidate(GraphSchemaConnectionValidationContext {
                source_output: from_port,
                target_input: to_port,
                source_connected_inputs: &source_connected_inputs,
                output_requirement_token: output_requirement_token.as_deref(),
            })
        {
            if let Some(issue) = structural_connection_issue_from_error(
                edge_index,
                GraphConnectionCandidate::from(edge),
                &error,
            ) {
                report.push_issue(issue);
            }
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::{
        validate_graph_document, validate_graph_document_structure, GraphValidationIssue,
        GraphValidationIssueKind, GraphValidationReport,
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
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.kind == GraphValidationIssueKind::CycleDetected));
    }

    #[test]
    fn structural_validation_rejects_multiple_input_policy_until_supported() {
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
                PortDefinition::<TestSchema>::new("In 0", "any").allow_multiple_connections()
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
        assert!(multi_sink_report.issues.iter().any(|issue| {
            issue.kind == GraphValidationIssueKind::UnsupportedConnectionPolicy
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
            inputs: vec![PortDefinition::<TestSchema>::new("Value", "number")
                .with_requirement("numeric-output")],
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
            inputs: vec![PortDefinition::<TestSchema>::new("Value", "number")
                .with_requirement("numeric-output")],
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

        assert!(report
            .issues
            .iter()
            .all(|issue| issue.kind != GraphValidationIssueKind::UnsatisfiedPortRequirement));
    }
}
