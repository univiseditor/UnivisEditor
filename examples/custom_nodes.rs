use bevy::prelude::*;
use serde_json::json;
use univis_editor_app::{NodeGraphPlugin, prelude::*};
use univis_editor_core::register_node;
use univis_editor_core::{
    node_definition::{
        NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
    },
    value::{NodeValue, ValueType},
};

const EXAMPLE_COLOR: Color = Color::srgb(0.2, 0.6, 0.8);

pub struct FloatBiasNode;

impl NodeDefinition for FloatBiasNode {
    fn id(&self) -> NodeId {
        NodeId::new("example/float_bias")
    }

    fn display_name(&self) -> &str {
        "Float Bias"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Examples/Custom")
    }

    fn color(&self) -> Color {
        EXAMPLE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("Value")
                .with_default(NodeValue::float(1.0))
                .editable_in_popup()
                .with_ui_step(0.1),
            PortDefinition::input_float("Bias")
                .with_default(NodeValue::float(0.5))
                .editable_in_popup()
                .with_ui_step(0.1),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let value = ctx.get_float_or(0, 0.0);
        let bias = ctx.get_float_or(1, 0.0);
        ctx.set_float(0, value + bias);
        ProcessResult::Success
    }
}

pub struct ToggleTextNode;

impl NodeDefinition for ToggleTextNode {
    fn id(&self) -> NodeId {
        NodeId::new("example/toggle_text")
    }

    fn display_name(&self) -> &str {
        "Toggle Text"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Examples/Custom")
    }

    fn color(&self) -> Color {
        Color::srgb(0.8, 0.5, 0.25)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_bool("Use A")
                .with_default(NodeValue::bool(true))
                .editable_in_popup(),
            PortDefinition::input_string("Text A")
                .with_default(NodeValue::string("Hello"))
                .editable_in_popup(),
            PortDefinition::input_string("Text B")
                .with_default(NodeValue::string("World"))
                .editable_in_popup(),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Selected", ValueType::String)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let use_a = ctx.get_bool_or(0, true);
        let selected = if use_a {
            ctx.get_string(1).unwrap_or("A").to_string()
        } else {
            ctx.get_string(2).unwrap_or("B").to_string()
        };
        ctx.set_string(0, selected);
        ProcessResult::Success
    }
}

pub struct Vec3BuildNode;

impl NodeDefinition for Vec3BuildNode {
    fn id(&self) -> NodeId {
        NodeId::new("example/vec3_build")
    }

    fn display_name(&self) -> &str {
        "Vec3 Build"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Examples/Custom")
    }

    fn color(&self) -> Color {
        Color::srgb(0.35, 0.65, 0.35)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("X")
                .with_default(NodeValue::float(0.0))
                .editable_in_popup()
                .with_ui_step(1.0),
            PortDefinition::input_float("Y")
                .with_default(NodeValue::float(0.0))
                .editable_in_popup()
                .with_ui_step(1.0),
            PortDefinition::input_float("Z")
                .with_default(NodeValue::float(0.0))
                .editable_in_popup()
                .with_ui_step(1.0),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Vector", ValueType::Vec3)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let x = ctx.get_float_or(0, 0.0) as f32;
        let y = ctx.get_float_or(1, 0.0) as f32;
        let z = ctx.get_float_or(2, 0.0) as f32;
        ctx.set_vec3(0, Vec3::new(x, y, z));
        ProcessResult::Success
    }
}

pub struct ColorBoostNode;

impl NodeDefinition for ColorBoostNode {
    fn id(&self) -> NodeId {
        NodeId::new("example/color_boost")
    }

    fn display_name(&self) -> &str {
        "Color Boost"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Examples/Custom")
    }

    fn color(&self) -> Color {
        Color::srgb(0.7, 0.35, 0.75)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::new("Base", ValueType::Color)
                .with_default(NodeValue::color(0.6, 0.7, 0.9, 1.0))
                .editable_in_popup(),
            PortDefinition::input_float("Gain")
                .with_default(NodeValue::float(1.2))
                .editable_in_popup()
                .with_ui_step(0.1)
                .with_ui_min(0.0)
                .with_ui_max(2.0),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Boosted", ValueType::Color)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let base = ctx
            .inputs
            .first()
            .and_then(NodeValue::as_color)
            .unwrap_or(Color::WHITE);
        let gain = ctx.get_float_or(1, 1.0) as f32;
        let srgba = base.to_srgba();
        let boosted = Color::srgba(
            (srgba.red * gain).clamp(0.0, 1.0),
            (srgba.green * gain).clamp(0.0, 1.0),
            (srgba.blue * gain).clamp(0.0, 1.0),
            srgba.alpha,
        );
        ctx.set(0, NodeValue::Color(boosted));
        ProcessResult::Success
    }
}

pub struct MaterialMetalNode;

impl NodeDefinition for MaterialMetalNode {
    fn id(&self) -> NodeId {
        NodeId::new("example/material_metal")
    }

    fn display_name(&self) -> &str {
        "Material Metal"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Examples/Custom")
    }

    fn color(&self) -> Color {
        Color::srgb(0.45, 0.55, 0.72)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("Density")
                .with_default(NodeValue::float(7.8))
                .editable_in_popup()
                .with_ui_step(0.1),
            PortDefinition::input_float("Thickness")
                .with_default(NodeValue::float(0.03))
                .editable_in_popup()
                .with_ui_step(0.01),
            PortDefinition::new("Name", ValueType::String)
                .with_default(NodeValue::string("Steel"))
                .editable_in_popup(),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_tag("Material", "material/metal")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let density = ctx.get_float_or(0, 7.8);
        let thickness = ctx.get_float_or(1, 0.03);
        let name = ctx.get_string(2).unwrap_or("Steel");
        ctx.set_tagged(
            0,
            "material/metal",
            json!({
                "name": name,
                "density": density,
                "thickness": thickness
            }),
        );
        ProcessResult::Success
    }
}

pub struct WheelPartNode;

impl NodeDefinition for WheelPartNode {
    fn id(&self) -> NodeId {
        NodeId::new("example/wheel_part")
    }

    fn display_name(&self) -> &str {
        "Wheel Part"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Examples/Custom")
    }

    fn color(&self) -> Color {
        Color::srgb(0.62, 0.42, 0.2)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_tag("Metal", "material/metal"),
            PortDefinition::input_int("Spokes")
                .with_default(NodeValue::int(5))
                .editable_in_popup()
                .with_ui_step(1.0)
                .with_ui_min(1.0)
                .with_ui_max(24.0),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_tag("Wheel", "part/wheel")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let (_, metal_payload) = match ctx.get_tagged(0) {
            Some(tagged) => tagged,
            None => return ProcessResult::MissingInput(0),
        };
        let spokes = ctx.get_int_or(1, 5);
        ctx.set_tagged(
            0,
            "part/wheel",
            json!({
                "metal": metal_payload.clone(),
                "spokes": spokes
            }),
        );
        ProcessResult::Success
    }
}

pub struct PrefixTextNode {
    pub prefix: &'static str,
}

impl NodeDefinition for PrefixTextNode {
    fn id(&self) -> NodeId {
        NodeId::new("example/prefix_text")
    }

    fn display_name(&self) -> &str {
        "Prefix Text (Ctor)"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Examples/Custom")
    }

    fn color(&self) -> Color {
        Color::srgb(0.25, 0.75, 0.7)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_string("Text")
                .with_default(NodeValue::string("demo"))
                .editable_in_popup(),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Result", ValueType::String)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let text = ctx.get_string(0).unwrap_or("demo");
        ctx.set_string(0, format!("{} {}", self.prefix, text));
        ProcessResult::Success
    }
}

register_node!(FloatBiasNode);
register_node!(ToggleTextNode);
register_node!(Vec3BuildNode);
register_node!(ColorBoostNode);
register_node!(MaterialMetalNode);
register_node!(WheelPartNode);
register_node!(PrefixTextNode, ctor = || PrefixTextNode { prefix: "[Custom]" });

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((NodeGraphPlugin, InfiniteGridPlugin))
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((Camera2d, GraphCamera));
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            scale: 50.0,
            dot_fadeout_strength: 0.0,
            z_axis_color: Color::NONE,
            x_axis_color: Color::NONE,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::Y, Vec3::Z)),
    ));
}
