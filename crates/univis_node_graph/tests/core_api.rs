use bevy::prelude::{Color, Vec2, Vec3};
use serde_json::json;
use univis_node_graph::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_node_graph::node_registry::NodeRegistry;
use univis_node_graph::value::{NodeValue, ValueType};

struct SearchableTestNode;

impl NodeDefinition for SearchableTestNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/searchable")
    }

    fn display_name(&self) -> &str {
        "Searchable"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn description(&self) -> Option<&str> {
        Some("A searchable node used in tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::input_float("Value").with_default(NodeValue::float(1.0))]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        ctx.set_float(0, ctx.get_float_or(0, 0.0) + 1.0);
        ProcessResult::Success
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["search", "sample"]
    }
}

struct HiddenMenuNode;

impl NodeDefinition for HiddenMenuNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/hidden")
    }

    fn display_name(&self) -> &str {
        "Hidden"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_any("Result")]
    }

    fn process(&self, _ctx: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }

    fn show_in_menu(&self) -> bool {
        false
    }
}

#[test]
fn node_value_conversions_and_display_strings_work() {
    let tagged = NodeValue::tagged("mesh", json!({ "id": 1 }));

    assert_eq!(NodeValue::float(3.5).as_int(), Some(3));
    assert_eq!(NodeValue::int(7).as_float(), Some(7.0));
    assert_eq!(NodeValue::string("hello").as_string(), Some("hello"));
    assert_eq!(
        NodeValue::vec2(2.0, 4.0).as_vec3(),
        Some(Vec3::new(2.0, 4.0, 0.0))
    );
    assert_eq!(
        NodeValue::vec3(1.0, 2.0, 3.0).as_vec2(),
        Some(Vec2::new(1.0, 2.0))
    );
    let (tag, payload) = tagged.as_tagged().expect("expected tagged payload");
    assert_eq!(tag, "mesh");
    assert_eq!(payload, &json!({ "id": 1 }));
    assert!(NodeValue::string("hello world")
        .to_display_string()
        .contains("hello"));
}

#[test]
fn process_context_reads_and_writes_typed_values() {
    let inputs = vec![
        NodeValue::float(5.5),
        NodeValue::int(4),
        NodeValue::bool(true),
        NodeValue::string("text"),
        NodeValue::Vec3(Vec3::new(1.0, 2.0, 3.0)),
    ];
    let mut outputs = vec![NodeValue::None; 5];
    let mut custom_data = None;

    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };

    assert_eq!(ctx.get_float(0), Some(5.5));
    assert_eq!(ctx.get_int(1), Some(4));
    assert_eq!(ctx.get_bool(2), Some(true));
    assert_eq!(ctx.get_string(3), Some("text"));
    assert_eq!(ctx.get_vec3(4), Some(Vec3::new(1.0, 2.0, 3.0)));
    assert_eq!(ctx.get_float_or(99, 42.0), 42.0);

    ctx.set_float(0, 9.25);
    ctx.set_int(1, 11);
    ctx.set_bool(2, false);
    ctx.set_string(3, "updated");
    ctx.set_vec2(4, Vec2::new(8.0, 13.0));

    assert_eq!(ctx.outputs[0], NodeValue::float(9.25));
    assert_eq!(ctx.outputs[1], NodeValue::int(11));
    assert_eq!(ctx.outputs[2], NodeValue::bool(false));
    assert_eq!(ctx.outputs[3], NodeValue::string("updated"));
    assert_eq!(ctx.outputs[4], NodeValue::Vec2(Vec2::new(8.0, 13.0)));
}

#[test]
fn port_definition_builders_and_color_resolution_work() {
    let port = PortDefinition::input_float("Strength")
        .with_description("Controls output strength")
        .with_default(NodeValue::float(2.5))
        .with_color(Color::srgb(0.8, 0.2, 0.4));

    assert_eq!(port.name(), "Strength");
    assert_eq!(port.value_type(), &ValueType::Float);
    assert_eq!(port.description(), Some("Controls output strength"));
    assert_eq!(port.default_value(), Some(&NodeValue::float(2.5)));

    let resolved = port.resolve_color().to_srgba();
    assert!((resolved.red - 0.8).abs() < 0.01);
    assert!((resolved.green - 0.2).abs() < 0.01);
    assert!((resolved.blue - 0.4).abs() < 0.01);

    let any_port = PortDefinition::output_any("Anything");
    assert_eq!(any_port.value_type(), &ValueType::Any);
}

#[test]
fn node_registry_tracks_registration_search_and_menu_nodes() {
    let mut registry = NodeRegistry::new();
    registry.register(SearchableTestNode);
    registry.register(HiddenMenuNode);

    let searchable_id = NodeId::new("tests/searchable");
    let hidden_id = NodeId::new("tests/hidden");

    assert_eq!(registry.len(), 2);
    assert!(registry.contains(&searchable_id));
    assert!(registry.contains(&hidden_id));
    assert_eq!(registry.get_all_ids().len(), 2);
    assert_eq!(registry.get_menu_nodes().len(), 1);
    assert_eq!(registry.search("sample").len(), 1);
    assert_eq!(registry.search("hidden").len(), 1);

    let definition = registry.get(&searchable_id).expect("node should exist");
    assert_eq!(
        definition.default_input_values(),
        vec![NodeValue::float(1.0)]
    );
    assert_eq!(definition.input_port_colors().len(), 1);
}
