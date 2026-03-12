use bevy::prelude::Color;
use univis_node_graph::document::{GraphDocument, GraphDocumentEdge, GraphDocumentNode};
use univis_node_graph::graph_validation::{
    GraphValidationIssueKind, analyze_graph_topology, validate_graph_document, would_create_cycle,
};
use univis_node_graph::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, PortRequirement, ProcessContext,
    ProcessResult,
};
use univis_node_graph::node_registry::NodeRegistry;
use univis_node_graph::value::{NodeValue, ValueType};

fn node(id: u64, definition_id: &str, input_count: usize, output_count: usize) -> GraphDocumentNode {
    GraphDocumentNode {
        id,
        definition_id: NodeId::new(definition_id),
        position: [0.0, id as f32],
        inputs: vec![NodeValue::None; input_count],
        input_count,
        output_count,
    }
}

struct FloatSourceNode;

impl NodeDefinition for FloatSourceNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/float_source")
    }

    fn display_name(&self) -> &str {
        "Float Source"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Value")]
    }

    fn process(&self, _context: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

struct BoolSinkNode;

impl NodeDefinition for BoolSinkNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/bool_sink")
    }

    fn display_name(&self) -> &str {
        "Bool Sink"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::input_bool("Value")]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn process(&self, _context: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

struct PassThroughNode;

impl NodeDefinition for PassThroughNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/pass_through")
    }

    fn display_name(&self) -> &str {
        "Pass Through"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("In", ValueType::Any)]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_any("Out")]
    }

    fn process(&self, _context: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

struct RequirementSinkNode;

impl NodeDefinition for RequirementSinkNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/requirement_sink")
    }

    fn display_name(&self) -> &str {
        "Requirement Sink"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::new("Entity", ValueType::Any).with_requirement(PortRequirement {
                id: "tests/required-output".to_string(),
                label: "Required".to_string(),
                color: Some(Color::srgb(0.9, 0.4, 0.2)),
            }),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn process(&self, _context: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

#[test]
fn topology_helpers_report_blocked_nodes_and_future_cycles() {
    let analysis = analyze_graph_topology([1_u64, 2, 3], [(1, 2), (2, 3), (3, 2)]);
    assert_eq!(analysis.ordered_nodes, vec![1]);
    assert_eq!(analysis.blocked_nodes, vec![2, 3]);

    assert!(would_create_cycle([(1_u64, 2), (2, 3)], 3, 1));
    assert!(!would_create_cycle([(1_u64, 2), (2, 3)], 1, 3));
}

#[test]
fn validate_graph_document_flags_missing_definitions_and_incompatible_ports() {
    let mut registry = NodeRegistry::new();
    registry.register(FloatSourceNode);
    registry.register(BoolSinkNode);

    let document = GraphDocument {
        nodes: vec![
            node(1, "tests/float_source", 0, 1),
            node(2, "tests/bool_sink", 1, 0),
            node(3, "tests/missing_definition", 0, 0),
        ],
        edges: vec![GraphDocumentEdge {
            from_node_id: 1,
            from_index: 0,
            to_node_id: 2,
            to_index: 0,
        }],
        ..GraphDocument::default()
    };

    let issues = validate_graph_document(&document, &registry);
    assert!(issues.iter().any(|issue| {
        issue.kind == GraphValidationIssueKind::MissingNodeDefinition && issue.node_ids == vec![3]
    }));
    assert!(issues.iter().any(|issue| {
        issue.kind == GraphValidationIssueKind::IncompatiblePortTypes
            && issue.node_ids == vec![1, 2]
    }));
}

#[test]
fn validate_graph_document_flags_unsatisfied_requirements_and_cycles() {
    let mut registry = NodeRegistry::new();
    registry.register(PassThroughNode);
    registry.register(RequirementSinkNode);

    let document = GraphDocument {
        nodes: vec![
            node(1, "tests/pass_through", 1, 1),
            node(2, "tests/pass_through", 1, 1),
            node(3, "tests/requirement_sink", 1, 0),
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
            GraphDocumentEdge {
                from_node_id: 1,
                from_index: 0,
                to_node_id: 3,
                to_index: 0,
            },
        ],
        ..GraphDocument::default()
    };

    let issues = validate_graph_document(&document, &registry);
    assert!(issues.iter().any(|issue| {
        issue.kind == GraphValidationIssueKind::UnsatisfiedPortRequirement
            && issue.node_ids == vec![1, 3]
    }));
    assert!(issues.iter().any(|issue| issue.kind == GraphValidationIssueKind::CycleDetected));
}
