//! عُقد الإخراج - لعرض ومراقبة القيم

use bevy::prelude::*;
use univis_editor_core::register_node;
use univis_editor_core::node_definition::{GraphNode, ValueDisplayLabel};
use univis_editor_core::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_editor_core::value::{NodeValue, ValueType};
use univis_ui::prelude::*;

// ═════════════════════════════════════════════════════
// لون موحد لعُقد الإخراج
// ═════════════════════════════════════════════════════

const OUTPUT_NODE_COLOR: Color = Color::srgb(0.3, 0.7, 0.5);

// ========== عقدة العرض المرئي ==========

/// مكون لتخزين القيمة المعروضة (للتحديث في واجهة المستخدم)
#[derive(Component, Default)]
pub struct ViewNodeDisplay {
    pub current_value: String,
    pub needs_update: bool,
}

/// عقدة View - تعرض القيمة بشكل مرئي في العقدة
pub struct ViewNode;

impl NodeDefinition for ViewNode {
    fn id(&self) -> NodeId {
        NodeId::new("output/view")
    }

    fn display_name(&self) -> &str {
        "View"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::OUTPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("Displays the input value visually in the node")
    }

    fn color(&self) -> Color {
        OUTPUT_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Value", ValueType::Any).with_description("The value to display")]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        // مخرج واحد من نفس النوع للسماح بتمرير القيمة
        vec![
            PortDefinition::new("Pass Through", ValueType::Any)
                .with_description("Passes the value through"),
        ]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        // نسخ المدخل إلى المخرج (Pass Through)
        if let Some(input) = ctx.inputs.get(0) {
            ctx.set(0, input.clone());
        }
        ProcessResult::Success
    }

    fn has_custom_body(&self) -> bool {
        true
    }

    fn build_body(&self, body: &mut ChildSpawnerCommands, node_entity: Entity) {
        // إنشاء منطقة عرض القيمة
        body.spawn((
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(28.0),
                padding: USides::axes(10.0, 5.0),
                background_color: Color::srgb(0.1, 0.1, 0.12),
                margin: USides::all(5.0),
                border_radius: UCornerRadius::all(4.0),
                ..default()
            },
            ULayout {
                justify_content: UJustifyContent::Center,
                align_items: UAlignItems::Center,
                ..default()
            },
        ))
        .with_children(|display_area| {
            display_area.spawn((
                ValueDisplayLabel { node_entity },
                UTextLabel {
                    text: "---".to_string(),
                    font_size: 12.0,
                    color: Color::srgb(0.9, 0.9, 0.4),
                    ..default()
                },
            ));
        });
    }
}

pub fn view_node_visual_hook(world: &mut World, node_entity: Entity) {
    let display_text = world
        .get::<GraphNode>(node_entity)
        .and_then(|graph| graph.values.get_input(0))
        .map(NodeValue::to_display_string)
        .unwrap_or_else(|| "---".to_string());

    let mut query = world.query::<(&ValueDisplayLabel, &mut UTextLabel)>();
    for (label, mut text) in query.iter_mut(world) {
        if label.node_entity == node_entity {
            text.text = display_text.clone();
        }
    }
}

// ========== عقدة المراقبة ==========

// عقدة Watch - تعرض القيمة مع اسمها
pub struct WatchNode;

impl NodeDefinition for WatchNode {
    fn id(&self) -> NodeId {
        NodeId::new("output/watch")
    }

    fn display_name(&self) -> &str {
        "Watch"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::OUTPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("Watches and displays a named value")
    }

    fn color(&self) -> Color {
        Color::srgb(0.4, 0.6, 0.7)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::new("Name", ValueType::String)
                .with_default(NodeValue::string("Value"))
                .with_description("Name label for the value"),
            PortDefinition::new("Value", ValueType::Any).with_description("The value to watch"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Pass Through", ValueType::Any)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        // تمرير القيمة
        if let Some(value) = ctx.inputs.get(1) {
            ctx.set(0, value.clone());
        }
        ProcessResult::Success
    }
}

// ========== عقدة التصحيح ==========

/// عقدة Debug - تطبع القيمة في الكونسول
pub struct DebugNode;

impl NodeDefinition for DebugNode {
    fn id(&self) -> NodeId {
        NodeId::new("output/debug")
    }

    fn display_name(&self) -> &str {
        "Debug"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::OUTPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("Prints value to console for debugging")
    }

    fn color(&self) -> Color {
        Color::srgb(0.5, 0.5, 0.5)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Value", ValueType::Any).with_description("Value to debug print")]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Pass Through", ValueType::Any)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        if let Some(value) = ctx.inputs.get(0) {
            println!("[Debug] {}", value.to_display_string());
            ctx.set(0, value.clone());
        }
        ProcessResult::Success
    }
}

register_node!(ViewNode, visual = view_node_visual_hook);
register_node!(WatchNode);
register_node!(DebugNode);
