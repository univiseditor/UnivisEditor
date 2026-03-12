mod support;

use support::run_node;
use univis_editor_nodes_builtin::logic::{AndNode, BranchNode, CompareNode, NotNode, OrNode};
use univis_node_graph::value::NodeValue;

#[test]
fn compare_node_reports_expected_comparisons() {
    let (_, outputs) = run_node(
        &CompareNode,
        vec![NodeValue::float(10.0), NodeValue::float(5.0)],
    );
    assert_eq!(
        outputs,
        vec![
            NodeValue::bool(false),
            NodeValue::bool(true),
            NodeValue::bool(true),
            NodeValue::bool(false),
        ]
    );
}

#[test]
fn branch_node_returns_the_selected_input_value() {
    let (_, outputs) = run_node(
        &BranchNode,
        vec![
            NodeValue::bool(true),
            NodeValue::float(100.0),
            NodeValue::float(200.0),
        ],
    );
    assert_eq!(outputs, vec![NodeValue::float(100.0)]);
}

#[test]
fn boolean_logic_nodes_emit_expected_truth_tables() {
    let (_, outputs) = run_node(&AndNode, vec![NodeValue::bool(true), NodeValue::bool(false)]);
    assert_eq!(outputs, vec![NodeValue::bool(false)]);

    let (_, outputs) = run_node(&OrNode, vec![NodeValue::bool(true), NodeValue::bool(false)]);
    assert_eq!(outputs, vec![NodeValue::bool(true)]);

    let (_, outputs) = run_node(&NotNode, vec![NodeValue::bool(true)]);
    assert_eq!(outputs, vec![NodeValue::bool(false)]);
}
