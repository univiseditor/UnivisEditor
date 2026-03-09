//! عقد المواد الخام - قابلة لإعادة الاستخدام عبر عدة تركيبات

use bevy::prelude::*;
use univis_editor_core::register_node;
use serde_json::json;
use univis_editor_core::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_editor_core::value::NodeValue;

const MATERIALS_COLOR: Color = Color::srgb(0.42, 0.55, 0.75);

pub struct MetalMaterialNode;

impl NodeDefinition for MetalMaterialNode {
    fn id(&self) -> NodeId {
        NodeId::new("material/metal")
    }

    fn display_name(&self) -> &str {
        "Metal Material"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Craft/Materials")
    }

    fn description(&self) -> Option<&str> {
        Some("Reusable metal material payload")
    }

    fn color(&self) -> Color {
        MATERIALS_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("Density")
                .with_default(NodeValue::float(7.8))
                .editable_in_popup()
                .with_ui_step(0.1)
                .with_ui_range(0.1, 40.0),
            PortDefinition::input_float("Thickness")
                .with_default(NodeValue::float(2.0))
                .editable_in_popup()
                .with_ui_step(0.1)
                .with_ui_range(0.1, 20.0),
            PortDefinition::input_float("Sound")
                .with_default(NodeValue::float(0.5))
                .editable_in_popup()
                .with_ui_step(0.05)
                .with_ui_range(0.0, 1.0),
            PortDefinition::new("Color", univis_editor_core::value::ValueType::Color)
                .with_default(NodeValue::color(0.72, 0.74, 0.78, 1.0))
                .editable_in_popup()
                .with_ui_step(0.05)
                .with_ui_range(0.0, 1.0),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_tag("Material", "material/metal")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let density = ctx.get_float_or(0, 7.8);
        let thickness = ctx.get_float_or(1, 2.0);
        let sound = ctx.get_float_or(2, 0.5);
        let color = ctx
            .inputs
            .get(3)
            .and_then(NodeValue::as_color)
            .unwrap_or_else(|| Color::srgba(0.72, 0.74, 0.78, 1.0))
            .to_srgba();

        ctx.set_tagged(
            0,
            "material/metal",
            json!({
                "density": density,
                "thickness": thickness,
                "sound": sound,
                "color": [color.red, color.green, color.blue, color.alpha]
            }),
        );
        ProcessResult::Success
    }
}

pub struct RubberMaterialNode;

impl NodeDefinition for RubberMaterialNode {
    fn id(&self) -> NodeId {
        NodeId::new("material/rubber")
    }

    fn display_name(&self) -> &str {
        "Rubber Material"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Craft/Materials")
    }

    fn description(&self) -> Option<&str> {
        Some("Reusable rubber material payload")
    }

    fn color(&self) -> Color {
        MATERIALS_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("Density")
                .with_default(NodeValue::float(1.2))
                .editable_in_popup()
                .with_ui_step(0.1)
                .with_ui_range(0.1, 10.0),
            PortDefinition::input_float("Elasticity")
                .with_default(NodeValue::float(0.8))
                .editable_in_popup()
                .with_ui_step(0.05)
                .with_ui_range(0.0, 1.0),
            PortDefinition::input_float("Damping")
                .with_default(NodeValue::float(0.25))
                .editable_in_popup()
                .with_ui_step(0.05)
                .with_ui_range(0.0, 1.0),
            PortDefinition::new("Color", univis_editor_core::value::ValueType::Color)
                .with_default(NodeValue::color(0.08, 0.08, 0.08, 1.0))
                .editable_in_popup()
                .with_ui_step(0.05)
                .with_ui_range(0.0, 1.0),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_tag("Material", "material/rubber")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let density = ctx.get_float_or(0, 1.2);
        let elasticity = ctx.get_float_or(1, 0.8);
        let damping = ctx.get_float_or(2, 0.25);
        let color = ctx
            .inputs
            .get(3)
            .and_then(NodeValue::as_color)
            .unwrap_or_else(|| Color::srgba(0.08, 0.08, 0.08, 1.0))
            .to_srgba();

        ctx.set_tagged(
            0,
            "material/rubber",
            json!({
                "density": density,
                "elasticity": elasticity,
                "damping": damping,
                "color": [color.red, color.green, color.blue, color.alpha]
            }),
        );
        ProcessResult::Success
    }
}

pub struct AirMaterialNode;

impl NodeDefinition for AirMaterialNode {
    fn id(&self) -> NodeId {
        NodeId::new("material/air")
    }

    fn display_name(&self) -> &str {
        "Air Material"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Craft/Materials")
    }

    fn description(&self) -> Option<&str> {
        Some("Reusable air material payload")
    }

    fn color(&self) -> Color {
        MATERIALS_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("Pressure")
                .with_default(NodeValue::float(32.0))
                .editable_in_popup()
                .with_ui_step(0.5)
                .with_ui_range(1.0, 120.0),
            PortDefinition::input_float("Temperature")
                .with_default(NodeValue::float(20.0))
                .editable_in_popup()
                .with_ui_step(0.5)
                .with_ui_range(-50.0, 150.0),
            PortDefinition::input_float("Humidity")
                .with_default(NodeValue::float(0.4))
                .editable_in_popup()
                .with_ui_step(0.05)
                .with_ui_range(0.0, 1.0),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_tag("Material", "material/air")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let pressure = ctx.get_float_or(0, 32.0);
        let temperature = ctx.get_float_or(1, 20.0);
        let humidity = ctx.get_float_or(2, 0.4);

        ctx.set_tagged(
            0,
            "material/air",
            json!({
                "pressure": pressure,
                "temperature": temperature,
                "humidity": humidity
            }),
        );
        ProcessResult::Success
    }
}

register_node!(MetalMaterialNode);
register_node!(RubberMaterialNode);
register_node!(AirMaterialNode);
