//! Built-in input nodes.
use bevy::prelude::*;
use univis_node_graph::live_graph::GraphNode;
use univis_node_graph::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_node_graph::register_node;
use univis_node_graph::value::{NodeValue, ValueType};
use univis_ui::prelude::*;

const INPUT_NODE_COLOR: Color = Color::srgb(0.4, 0.8, 0.4);

#[derive(Component)]
struct NumberNodeDragWidget {
    node_entity: Entity,
}

#[derive(Component)]
struct IntegerNodeDragWidget {
    node_entity: Entity,
}

#[derive(Component)]
struct BooleanNodeToggleWidget {
    node_entity: Entity,
}

#[derive(Component)]
struct TextNodeFieldWidget {
    node_entity: Entity,
}

fn write_single_output(world: &mut World, node_entity: Entity, value: NodeValue) {
    if let Some(mut graph_node) = world.get_mut::<GraphNode>(node_entity) {
        if graph_node.values.outputs.len() == 1 && graph_node.values.outputs[0] != value {
            graph_node.values.outputs[0] = value;
        }
    }
}

fn read_number_widget_value(world: &mut World, node_entity: Entity) -> Option<f32> {
    let mut query = world.query::<(&NumberNodeDragWidget, &UDragValue)>();
    query
        .iter(world)
        .find_map(|(marker, drag)| (marker.node_entity == node_entity).then_some(drag.value))
}

fn read_integer_widget_value(world: &mut World, node_entity: Entity) -> Option<f32> {
    let mut query = world.query::<(&IntegerNodeDragWidget, &UDragValue)>();
    query
        .iter(world)
        .find_map(|(marker, drag)| (marker.node_entity == node_entity).then_some(drag.value))
}

fn read_boolean_widget_value(world: &mut World, node_entity: Entity) -> Option<bool> {
    let mut query = world.query::<(&BooleanNodeToggleWidget, &UToggle)>();
    query
        .iter(world)
        .find_map(|(marker, toggle)| (marker.node_entity == node_entity).then_some(toggle.checked))
}

fn read_text_widget_value(world: &mut World, node_entity: Entity) -> Option<String> {
    let mut query = world.query::<(&TextNodeFieldWidget, &UTextField)>();
    query.iter(world).find_map(|(marker, field)| {
        (marker.node_entity == node_entity).then_some(field.text.clone())
    })
}

pub struct NumberNode;

impl NodeDefinition for NumberNode {
    fn id(&self) -> NodeId {
        NodeId::new("input/number")
    }

    fn display_name(&self) -> &str {
        "Number"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::INPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("A floating-point number value")
    }

    fn color(&self) -> Color {
        INPUT_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Value")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        if ctx.outputs.get(0).map(|v| v.is_none()).unwrap_or(true) {
            ctx.set_float(0, 1.0);
        }
        ProcessResult::Success
    }

    fn has_custom_body(&self) -> bool {
        true
    }

    fn build_body(&self, body: &mut ChildSpawnerCommands, node_entity: Entity) {
        body.spawn((
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(34.0),
                ..default()
            },
            ULayout {
                justify_content: UJustifyContent::Center,
                align_items: UAlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                UNode {
                    width: UVal::Px(140.0),
                    height: UVal::Px(30.0),
                    ..default()
                },
                UDragValue::new()
                    .with_range(-100.0, 100.0)
                    .with_step(0.1)
                    .with_decimals(2)
                    .with_value(1.0),
                NumberNodeDragWidget { node_entity },
            ));
        });
    }

    fn sync_visual(&self, world: &mut World, node_entity: Entity) {
        if let Some(value) = read_number_widget_value(world, node_entity) {
            write_single_output(world, node_entity, NodeValue::float(value as f64));
        }
    }
}

pub struct IntegerNode;

impl NodeDefinition for IntegerNode {
    fn id(&self) -> NodeId {
        NodeId::new("input/integer")
    }

    fn display_name(&self) -> &str {
        "Integer"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::INPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("An integer number value")
    }

    fn color(&self) -> Color {
        INPUT_NODE_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Value", ValueType::Int)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        if ctx.outputs.get(0).map(|v| v.is_none()).unwrap_or(true) {
            ctx.set_int(0, 1);
        }
        ProcessResult::Success
    }

    fn has_custom_body(&self) -> bool {
        true
    }

    fn build_body(&self, body: &mut ChildSpawnerCommands, node_entity: Entity) {
        body.spawn((
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(34.0),
                ..default()
            },
            ULayout {
                justify_content: UJustifyContent::Center,
                align_items: UAlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                UNode {
                    width: UVal::Px(140.0),
                    height: UVal::Px(30.0),
                    ..default()
                },
                UDragValue::new()
                    .with_range(-100000.0, 100000.0)
                    .with_step(1.0)
                    .with_decimals(0)
                    .with_value(1.0),
                IntegerNodeDragWidget { node_entity },
            ));
        });
    }

    fn sync_visual(&self, world: &mut World, node_entity: Entity) {
        if let Some(value) = read_integer_widget_value(world, node_entity) {
            write_single_output(world, node_entity, NodeValue::int(value.round() as i64));
        }
    }
}

pub struct BooleanNode;

impl NodeDefinition for BooleanNode {
    fn id(&self) -> NodeId {
        NodeId::new("input/boolean")
    }

    fn display_name(&self) -> &str {
        "Boolean"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::INPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("A true/false value")
    }

    fn color(&self) -> Color {
        Color::srgb(0.9, 0.4, 0.4)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Value", ValueType::Bool)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        if ctx.outputs.get(0).map(|v| v.is_none()).unwrap_or(true) {
            ctx.set_bool(0, false);
        }
        ProcessResult::Success
    }

    fn has_custom_body(&self) -> bool {
        true
    }

    fn build_body(&self, body: &mut ChildSpawnerCommands, node_entity: Entity) {
        body.spawn((
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(36.0),
                ..default()
            },
            ULayout {
                justify_content: UJustifyContent::Center,
                align_items: UAlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                UToggle::new().with_checked(false).with_size(64.0, 30.0),
                BooleanNodeToggleWidget { node_entity },
            ));
        });
    }

    fn sync_visual(&self, world: &mut World, node_entity: Entity) {
        if let Some(checked) = read_boolean_widget_value(world, node_entity) {
            write_single_output(world, node_entity, NodeValue::bool(checked));
        }
    }
}

pub struct TextNode;

impl NodeDefinition for TextNode {
    fn id(&self) -> NodeId {
        NodeId::new("input/text")
    }

    fn display_name(&self) -> &str {
        "Text"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::INPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("A text string value")
    }

    fn color(&self) -> Color {
        Color::srgb(0.9, 0.8, 0.3)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Value", ValueType::String)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        if ctx.outputs.get(0).map(|v| v.is_none()).unwrap_or(true) {
            ctx.set_string(0, "");
        }
        ProcessResult::Success
    }

    fn has_custom_body(&self) -> bool {
        true
    }

    fn build_body(&self, body: &mut ChildSpawnerCommands, node_entity: Entity) {
        body.spawn((
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(36.0),
                ..default()
            },
            ULayout {
                justify_content: UJustifyContent::Center,
                align_items: UAlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                UTextField::new()
                    .with_text("")
                    .with_placeholder("type text")
                    .with_size(180.0, 30.0),
                TextNodeFieldWidget { node_entity },
            ));
        });
    }

    fn sync_visual(&self, world: &mut World, node_entity: Entity) {
        if let Some(text) = read_text_widget_value(world, node_entity) {
            write_single_output(world, node_entity, NodeValue::string(text));
        }
    }
}

pub struct Vector2Node;

impl NodeDefinition for Vector2Node {
    fn id(&self) -> NodeId {
        NodeId::new("input/vector2")
    }

    fn display_name(&self) -> &str {
        "Vector2"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::INPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("A 2D vector value")
    }

    fn color(&self) -> Color {
        Color::srgb(0.7, 0.4, 0.9)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("X").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("Y").with_default(NodeValue::float(0.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Value", ValueType::Vec2)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let x = ctx.get_float_or(0, 0.0) as f32;
        let y = ctx.get_float_or(1, 0.0) as f32;

        ctx.set_vec2(0, Vec2::new(x, y));
        ProcessResult::Success
    }
}

pub struct Vector3Node;

impl NodeDefinition for Vector3Node {
    fn id(&self) -> NodeId {
        NodeId::new("input/vector3")
    }

    fn display_name(&self) -> &str {
        "Vector3"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::INPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("A 3D vector value")
    }

    fn color(&self) -> Color {
        Color::srgb(0.6, 0.7, 0.9)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("X").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("Y").with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("Z").with_default(NodeValue::float(0.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Value", ValueType::Vec3)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let x = ctx.get_float_or(0, 0.0) as f32;
        let y = ctx.get_float_or(1, 0.0) as f32;
        let z = ctx.get_float_or(2, 0.0) as f32;

        ctx.set_vec3(0, Vec3::new(x, y, z));
        ProcessResult::Success
    }
}

pub struct ColorNode;

impl NodeDefinition for ColorNode {
    fn id(&self) -> NodeId {
        NodeId::new("input/color")
    }

    fn display_name(&self) -> &str {
        "Color"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::INPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("An RGBA color value")
    }

    fn color(&self) -> Color {
        Color::srgb(1.0, 0.5, 0.8)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("R").with_default(NodeValue::float(1.0)),
            PortDefinition::input_float("G").with_default(NodeValue::float(1.0)),
            PortDefinition::input_float("B").with_default(NodeValue::float(1.0)),
            PortDefinition::input_float("A").with_default(NodeValue::float(1.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Value", ValueType::Color)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let r = ctx.get_float_or(0, 1.0) as f32;
        let g = ctx.get_float_or(1, 1.0) as f32;
        let b = ctx.get_float_or(2, 1.0) as f32;
        let a = ctx.get_float_or(3, 1.0) as f32;

        ctx.set(0, NodeValue::color(r, g, b, a));
        ProcessResult::Success
    }
}

register_node!(NumberNode);
register_node!(IntegerNode);
register_node!(BooleanNode);
register_node!(TextNode);
register_node!(Vector2Node);
register_node!(Vector3Node);
register_node!(ColorNode);
