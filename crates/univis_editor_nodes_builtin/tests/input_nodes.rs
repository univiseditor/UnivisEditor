mod support;

use bevy::prelude::{Vec2, Vec3};
use support::run_node;
use univis_editor_nodes_builtin::input::{
    BooleanNode, ColorNode, IntegerNode, NumberNode, TextNode, Vector2Node, Vector3Node,
};
use univis_node_graph::value::NodeValue;

#[test]
fn scalar_input_nodes_emit_expected_defaults() {
    let (_, outputs) = run_node(&NumberNode, vec![]);
    assert_eq!(outputs, vec![NodeValue::float(1.0)]);

    let (_, outputs) = run_node(&IntegerNode, vec![]);
    assert_eq!(outputs, vec![NodeValue::int(1)]);

    let (_, outputs) = run_node(&BooleanNode, vec![]);
    assert_eq!(outputs, vec![NodeValue::bool(false)]);

    let (_, outputs) = run_node(&TextNode, vec![]);
    assert_eq!(outputs, vec![NodeValue::string("")]);
}

#[test]
fn vector_input_nodes_compose_values_from_ports() {
    let (_, outputs) = run_node(
        &Vector2Node,
        vec![NodeValue::float(3.0), NodeValue::float(4.0)],
    );
    assert_eq!(outputs, vec![NodeValue::Vec2(Vec2::new(3.0, 4.0))]);

    let (_, outputs) = run_node(
        &Vector3Node,
        vec![
            NodeValue::float(1.0),
            NodeValue::float(2.0),
            NodeValue::float(3.0),
        ],
    );
    assert_eq!(outputs, vec![NodeValue::Vec3(Vec3::new(1.0, 2.0, 3.0))]);
}

#[test]
fn color_input_node_emits_rgba_color() {
    let (_, outputs) = run_node(
        &ColorNode,
        vec![
            NodeValue::float(0.2),
            NodeValue::float(0.4),
            NodeValue::float(0.6),
            NodeValue::float(0.8),
        ],
    );

    match &outputs[0] {
        NodeValue::Color(color) => {
            let srgba = color.to_srgba();
            assert!((srgba.red - 0.2).abs() < 0.01);
            assert!((srgba.green - 0.4).abs() < 0.01);
            assert!((srgba.blue - 0.6).abs() < 0.01);
            assert!((srgba.alpha - 0.8).abs() < 0.01);
        }
        other => panic!("expected color output, got {other:?}"),
    }
}
