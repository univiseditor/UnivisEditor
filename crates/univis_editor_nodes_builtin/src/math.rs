//! عُقد الرياضيات الأساسية

use bevy::prelude::*;
use univis_editor_core::register_node;
use univis_editor_core::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_editor_core::value::NodeValue;

// ═════════════════════════════════════════════════════
// لون موحد لعُقد الرياضيات
// ═════════════════════════════════════════════════════

const MATH_NODE_COLOR: Color = Color::srgb(0.2, 0.5, 0.8);

// ========== عقدة الجمع ==========

pub struct AddNode;

impl NodeDefinition for AddNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/add")
    }

    fn display_name(&self) -> &str {
        "Add"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Adds two numbers together")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn icon(&self) -> Option<&str> {
        Some("+")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("A").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("B").with_default(NodeValue::float(0.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_float_or(0, 0.0);
        let b = ctx.get_float_or(1, 0.0);
        let result = a + b;
        ctx.set_float(0, result);
        ProcessResult::Success
    }
}

// ========== عقدة الطرح ==========

pub struct SubtractNode;

impl NodeDefinition for SubtractNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/subtract")
    }

    fn display_name(&self) -> &str {
        "Subtract"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Subtracts B from A")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn icon(&self) -> Option<&str> {
        Some("−")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("A").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("B").with_default(NodeValue::float(0.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_float_or(0, 0.0);
        let b = ctx.get_float_or(1, 0.0);
        ctx.set_float(0, a - b);
        ProcessResult::Success
    }
}

// ========== عقدة الضرب ==========

pub struct MultiplyNode;

impl NodeDefinition for MultiplyNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/multiply")
    }

    fn display_name(&self) -> &str {
        "Multiply"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Multiplies two numbers together")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn icon(&self) -> Option<&str> {
        Some("×")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("A").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("B").with_default(NodeValue::float(0.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_float_or(0, 0.0);
        let b = ctx.get_float_or(1, 0.0);
        ctx.set_float(0, a * b);
        ProcessResult::Success
    }
}

// ========== عقدة القسمة ==========

pub struct DivideNode;

impl NodeDefinition for DivideNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/divide")
    }

    fn display_name(&self) -> &str {
        "Divide"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Divides A by B")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn icon(&self) -> Option<&str> {
        Some("÷")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("A").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("B").with_default(NodeValue::float(1.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_float_or(0, 0.0);
        let b = ctx.get_float_or(1, 1.0);

        if b == 0.0 {
            return ProcessResult::Error("Division by zero".to_string());
        }

        ctx.set_float(0, a / b);
        ProcessResult::Success
    }
}

// ========== عقدة التقييد ==========

pub struct ClampNode;

impl NodeDefinition for ClampNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/clamp")
    }

    fn display_name(&self) -> &str {
        "Clamp"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Clamps a value between min and max")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("Value").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("Min").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("Max").with_default(NodeValue::float(1.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let value = ctx.get_float_or(0, 0.0);
        let min = ctx.get_float_or(1, 0.0);
        let max = ctx.get_float_or(2, 1.0);

        ctx.set_float(0, value.clamp(min, max));
        ProcessResult::Success
    }
}

// ========== عقدة الاستيفاء ==========

pub struct LerpNode;

impl NodeDefinition for LerpNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/lerp")
    }

    fn display_name(&self) -> &str {
        "Lerp"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Linear interpolation between A and B")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("A").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("B").with_default(NodeValue::float(1.0)),
            PortDefinition::input_float("T").with_default(NodeValue::float(0.5)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_float_or(0, 0.0);
        let b = ctx.get_float_or(1, 1.0);
        let t = ctx.get_float_or(2, 0.5);

        ctx.set_float(0, a + (b - a) * t);
        ProcessResult::Success
    }
}

// ========== عقدة الحد الأدنى ==========

pub struct MinNode;

impl NodeDefinition for MinNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/min")
    }

    fn display_name(&self) -> &str {
        "Min"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Returns the smaller of two values")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("A").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("B").with_default(NodeValue::float(0.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_float_or(0, 0.0);
        let b = ctx.get_float_or(1, 0.0);
        ctx.set_float(0, a.min(b));
        ProcessResult::Success
    }
}

// ========== عقدة الحد الأقصى ==========

pub struct MaxNode;

impl NodeDefinition for MaxNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/max")
    }

    fn display_name(&self) -> &str {
        "Max"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Returns the larger of two values")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("A").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("B").with_default(NodeValue::float(0.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_float_or(0, 0.0);
        let b = ctx.get_float_or(1, 0.0);
        ctx.set_float(0, a.max(b));
        ProcessResult::Success
    }
}

// ========== عقدة القيمة المطلقة ==========

pub struct AbsNode;

impl NodeDefinition for AbsNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/abs")
    }

    fn display_name(&self) -> &str {
        "Abs"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Returns the absolute value")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::input_float("Value").with_default(NodeValue::float(0.0))]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let value = ctx.get_float_or(0, 0.0);
        ctx.set_float(0, value.abs());
        ProcessResult::Success
    }
}

// ========== عقدة الجيب ==========

pub struct SinNode;

impl NodeDefinition for SinNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/sin")
    }

    fn display_name(&self) -> &str {
        "Sin"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Returns the sine of an angle (in radians)")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::input_float("Angle").with_default(NodeValue::float(0.0))]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let angle = ctx.get_float_or(0, 0.0);
        ctx.set_float(0, angle.sin());
        ProcessResult::Success
    }
}

// ========== عقدة جيب التمام ==========

pub struct CosNode;

impl NodeDefinition for CosNode {
    fn id(&self) -> NodeId {
        NodeId::new("math/cos")
    }

    fn display_name(&self) -> &str {
        "Cos"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Returns the cosine of an angle (in radians)")
    }

    fn color(&self) -> Color {
        MATH_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::input_float("Angle").with_default(NodeValue::float(0.0))]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let angle = ctx.get_float_or(0, 0.0);
        ctx.set_float(0, angle.cos());
        ProcessResult::Success
    }
}

register_node!(AddNode);
register_node!(SubtractNode);
register_node!(MultiplyNode);
register_node!(DivideNode);
register_node!(ClampNode);
register_node!(LerpNode);
register_node!(MinNode);
register_node!(MaxNode);
register_node!(AbsNode);
register_node!(SinNode);
register_node!(CosNode);
