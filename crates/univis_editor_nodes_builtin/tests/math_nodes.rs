mod support;

use support::run_node;
use univis_editor_nodes_builtin::math::{AddNode, ClampNode, DivideNode, LerpNode, MultiplyNode};
use univis_node_graph::node_definition::ProcessResult;
use univis_node_graph::value::NodeValue;

#[test]
fn add_and_multiply_nodes_compute_expected_results() {
    let (result, outputs) = run_node(&AddNode, vec![NodeValue::float(5.0), NodeValue::float(3.0)]);
    assert!(matches!(result, ProcessResult::Success));
    assert_eq!(outputs, vec![NodeValue::float(8.0)]);

    let (_, outputs) = run_node(
        &MultiplyNode,
        vec![NodeValue::float(4.0), NodeValue::float(5.0)],
    );
    assert_eq!(outputs, vec![NodeValue::float(20.0)]);
}

#[test]
fn divide_node_reports_division_by_zero() {
    let (result, outputs) = run_node(
        &DivideNode,
        vec![NodeValue::float(20.0), NodeValue::float(4.0)],
    );
    assert!(matches!(result, ProcessResult::Success));
    assert_eq!(outputs, vec![NodeValue::float(5.0)]);

    let (result, _) = run_node(
        &DivideNode,
        vec![NodeValue::float(10.0), NodeValue::float(0.0)],
    );
    assert!(matches!(result, ProcessResult::Error(_)));
}

#[test]
fn clamp_and_lerp_nodes_apply_expected_ranges() {
    let (_, outputs) = run_node(
        &ClampNode,
        vec![
            NodeValue::float(12.0),
            NodeValue::float(0.0),
            NodeValue::float(5.0),
        ],
    );
    assert_eq!(outputs, vec![NodeValue::float(5.0)]);

    let (_, outputs) = run_node(
        &LerpNode,
        vec![
            NodeValue::float(0.0),
            NodeValue::float(10.0),
            NodeValue::float(0.25),
        ],
    );
    assert_eq!(outputs, vec![NodeValue::float(2.5)]);
}
