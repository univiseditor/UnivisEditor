//! Built-in logic nodes.
use bevy::prelude::*;
use univis_node_graph::live_graph::AuthoredNodeInputs;
use univis_node_graph::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_node_graph::register_node;
use univis_node_graph::value::{NodeValue, ValueType};
use univis_ui::prelude::*;

// ═════════════════════════════════════════════════════
// ═════════════════════════════════════════════════════

const LOGIC_NODE_COLOR: Color = Color::srgb(0.8, 0.5, 0.3);
const WORKFLOW_NODE_COLOR: Color = Color::srgb(0.55, 0.58, 0.72);

#[derive(Component)]
struct NoteBodyLabel {
    node_entity: Entity,
}

pub struct CompareNode;

impl NodeDefinition for CompareNode {
    fn id(&self) -> NodeId {
        NodeId::new("logic/compare")
    }

    fn display_name(&self) -> &str {
        "Compare"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::LOGIC)
    }

    fn description(&self) -> Option<&str> {
        Some("Compares two values")
    }

    fn color(&self) -> Color {
        LOGIC_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("A").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("B").with_default(NodeValue::float(0.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::new("Equal", ValueType::Bool),
            PortDefinition::new("Not Equal", ValueType::Bool),
            PortDefinition::new("Greater", ValueType::Bool),
            PortDefinition::new("Less", ValueType::Bool),
        ]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_float_or(0, 0.0);
        let b = ctx.get_float_or(1, 0.0);

        ctx.set_bool(0, (a - b).abs() < f64::EPSILON);
        ctx.set_bool(1, (a - b).abs() >= f64::EPSILON);
        ctx.set_bool(2, a > b);
        ctx.set_bool(3, a < b);

        ProcessResult::Success
    }
}

pub struct BranchNode;

impl NodeDefinition for BranchNode {
    fn id(&self) -> NodeId {
        NodeId::new("logic/branch")
    }

    fn display_name(&self) -> &str {
        "Branch"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::LOGIC)
    }

    fn description(&self) -> Option<&str> {
        Some("Returns True value if condition is true, else False value")
    }

    fn color(&self) -> Color {
        LOGIC_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_bool("Condition").with_default(NodeValue::bool(false)),
            PortDefinition::new("True", ValueType::Any).with_description("Value if true"),
            PortDefinition::new("False", ValueType::Any).with_description("Value if false"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_any("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let condition = ctx.get_bool_or(0, false);

        let result = if condition {
            ctx.inputs.get(1).cloned().unwrap_or_default()
        } else {
            ctx.inputs.get(2).cloned().unwrap_or_default()
        };

        ctx.set(0, result);
        ProcessResult::Success
    }
}

pub struct AndNode;

impl NodeDefinition for AndNode {
    fn id(&self) -> NodeId {
        NodeId::new("logic/and")
    }

    fn display_name(&self) -> &str {
        "And"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::LOGIC)
    }

    fn description(&self) -> Option<&str> {
        Some("Logical AND operation")
    }

    fn color(&self) -> Color {
        LOGIC_NODE_COLOR
    }

    fn icon(&self) -> Option<&str> {
        Some("&&")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_bool("A").with_default(NodeValue::bool(false)),
            PortDefinition::input_bool("B").with_default(NodeValue::bool(false)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Result", ValueType::Bool)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_bool_or(0, false);
        let b = ctx.get_bool_or(1, false);
        ctx.set_bool(0, a && b);
        ProcessResult::Success
    }
}

pub struct OrNode;

impl NodeDefinition for OrNode {
    fn id(&self) -> NodeId {
        NodeId::new("logic/or")
    }

    fn display_name(&self) -> &str {
        "Or"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::LOGIC)
    }

    fn description(&self) -> Option<&str> {
        Some("Logical OR operation")
    }

    fn color(&self) -> Color {
        LOGIC_NODE_COLOR
    }

    fn icon(&self) -> Option<&str> {
        Some("||")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_bool("A").with_default(NodeValue::bool(false)),
            PortDefinition::input_bool("B").with_default(NodeValue::bool(false)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Result", ValueType::Bool)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_bool_or(0, false);
        let b = ctx.get_bool_or(1, false);
        ctx.set_bool(0, a || b);
        ProcessResult::Success
    }
}

pub struct NotNode;

impl NodeDefinition for NotNode {
    fn id(&self) -> NodeId {
        NodeId::new("logic/not")
    }

    fn display_name(&self) -> &str {
        "Not"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::LOGIC)
    }

    fn description(&self) -> Option<&str> {
        Some("Logical NOT operation")
    }

    fn color(&self) -> Color {
        LOGIC_NODE_COLOR
    }

    fn icon(&self) -> Option<&str> {
        Some("!")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::input_bool("Value").with_default(NodeValue::bool(false))]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Result", ValueType::Bool)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let value = ctx.get_bool_or(0, false);
        ctx.set_bool(0, !value);
        ProcessResult::Success
    }
}

pub struct RerouteNode;

impl NodeDefinition for RerouteNode {
    fn id(&self) -> NodeId {
        NodeId::new("logic/reroute")
    }

    fn display_name(&self) -> &str {
        "Reroute"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::ADVANCED)
    }

    fn description(&self) -> Option<&str> {
        Some("Passes a value through so you can cleanly reroute wires")
    }

    fn color(&self) -> Color {
        WORKFLOW_NODE_COLOR
    }

    fn body_width(&self) -> f32 {
        120.0
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("In", ValueType::Any)]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_any("Out")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        ctx.set(0, ctx.inputs.first().cloned().unwrap_or_default());
        ProcessResult::Success
    }
}

pub struct NoteNode;

impl NodeDefinition for NoteNode {
    fn id(&self) -> NodeId {
        NodeId::new("logic/note")
    }

    fn display_name(&self) -> &str {
        "Note"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::ADVANCED)
    }

    fn description(&self) -> Option<&str> {
        Some("Stores a short comment directly in the graph canvas")
    }

    fn color(&self) -> Color {
        Color::srgb(0.78, 0.66, 0.28)
    }

    fn body_width(&self) -> f32 {
        220.0
    }

    fn body_height(&self) -> f32 {
        110.0
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["comment", "note", "annotation"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_string("Text")
                .with_default(NodeValue::string("Write a note..."))
                .with_description("Visible note text")
                .editable_inline(),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn process(&self, _ctx: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }

    fn has_custom_body(&self) -> bool {
        true
    }

    fn build_body(&self, body: &mut ChildSpawnerCommands, node_entity: Entity) {
        body.spawn((
            UNode {
                width: UVal::Px(180.0),
                height: UVal::Px(74.0),
                padding: USides::all(10.0),
                background_color: Color::srgba(0.08, 0.08, 0.1, 0.8),
                border_radius: UCornerRadius::all(8.0),
                ..default()
            },
            ULayout {
                justify_content: UJustifyContent::Start,
                align_items: UAlignItems::Start,
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((
                UTextLabel {
                    text: "Write a note...".to_string(),
                    font_size: 12.0,
                    color: Color::WHITE,
                    linebreak: LineBreak::WordBoundary,
                    autosize: false,
                    ..default()
                },
                NoteBodyLabel { node_entity },
            ));
        });
    }

    fn sync_visual(&self, world: &mut World, node_entity: Entity) {
        let note_text = world
            .get::<AuthoredNodeInputs>(node_entity)
            .and_then(|inputs| inputs.values.first())
            .and_then(NodeValue::as_string)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .unwrap_or("Write a note...")
            .to_string();

        let mut labels = world.query::<(&mut UTextLabel, &NoteBodyLabel)>();
        for (mut label, marker) in labels.iter_mut(world) {
            if marker.node_entity == node_entity && label.text != note_text {
                label.text = note_text.clone();
            }
        }
    }
}

register_node!(CompareNode);
register_node!(BranchNode);
register_node!(AndNode);
register_node!(OrNode);
register_node!(NotNode);
register_node!(RerouteNode);
register_node!(NoteNode);
