//! Core node-definition types and runtime traits.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::any::Any;
use std::sync::Arc;
use univis_scene::EntityValue;

use super::value::{NodeValue, NodeValues, ValueType};

/// Stable identifier for a node definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Display category used for grouping nodes in menus.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeCategory(pub String);

impl NodeCategory {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const MATH: &'static str = "Math";
    pub const LOGIC: &'static str = "Logic";
    pub const INPUT: &'static str = "Input";
    pub const OUTPUT: &'static str = "Output";
    pub const SCENE: &'static str = "Scene";
    pub const ADVANCED: &'static str = "Advanced";
}

/// Optional semantic requirement for a specialized port.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortRequirement {
    pub id: String,
    pub label: String,
    pub color: Option<Color>,
}

impl PortRequirement {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            color: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// Definition of a node input or output port.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDefinition {
    pub name: String,
    pub value_type: ValueType,
    pub description: Option<String>,
    pub default_value: Option<NodeValue>,
    pub color: Option<Color>,
    #[serde(default)]
    pub requirement: Option<PortRequirement>,
    #[serde(default)]
    pub editable_in_popup: bool,
    #[serde(default)]
    pub ui_step: Option<f64>,
    #[serde(default)]
    pub ui_min: Option<f64>,
    #[serde(default)]
    pub ui_max: Option<f64>,
}

impl PortDefinition {
    pub fn new(name: impl Into<String>, value_type: ValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
            description: None,
            default_value: None,
            color: None,
            requirement: None,
            editable_in_popup: false,
            ui_step: None,
            ui_min: None,
            ui_max: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_default(mut self, value: NodeValue) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_requirement(mut self, requirement: PortRequirement) -> Self {
        self.requirement = Some(requirement);
        self
    }

    pub fn editable_in_popup(mut self) -> Self {
        self.editable_in_popup = true;
        self
    }

    pub fn with_ui_step(mut self, step: f64) -> Self {
        self.ui_step = Some(step);
        self
    }

    pub fn with_ui_min(mut self, min: f64) -> Self {
        self.ui_min = Some(min);
        self
    }

    pub fn with_ui_max(mut self, max: f64) -> Self {
        self.ui_max = Some(max);
        self
    }

    pub fn with_ui_range(mut self, min: f64, max: f64) -> Self {
        self.ui_min = Some(min);
        self.ui_max = Some(max);
        self
    }

    pub fn resolve_color(&self) -> Color {
        self.color.unwrap_or_else(|| {
            self.requirement
                .as_ref()
                .and_then(|requirement| requirement.color)
                .unwrap_or_else(|| self.value_type.port_color())
        })
    }

    pub fn display_label(&self) -> String {
        if let Some(requirement) = &self.requirement {
            if self.name == requirement.label {
                format!("{} *", self.name)
            } else {
                format!("{} <{}>", self.name, requirement.label)
            }
        } else {
            self.name.clone()
        }
    }

    pub fn input_float(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Float)
    }

    pub fn input_int(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Int)
    }

    pub fn input_bool(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Bool)
    }

    pub fn input_string(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::String)
    }

    pub fn input_vec3(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Vec3)
    }

    pub fn input_tag(name: impl Into<String>, tag: impl Into<String>) -> Self {
        Self::new(name, ValueType::CustomTag(tag.into()))
    }

    pub fn input_entity(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Entity)
    }

    pub fn output_float(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Float)
    }

    pub fn output_any(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Any)
    }

    pub fn output_tag(name: impl Into<String>, tag: impl Into<String>) -> Self {
        Self::new(name, ValueType::CustomTag(tag.into()))
    }

    pub fn output_entity(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Entity)
    }
}

/// Runtime context passed to a node during processing.
pub struct ProcessContext<'a> {
    pub inputs: &'a [NodeValue],
    pub outputs: &'a mut [NodeValue],
    pub delta_time: f32,
    pub custom_data: &'a mut Option<Box<dyn Any + Send + Sync>>,
}

impl<'a> ProcessContext<'a> {
    // ═════════════════════════════════════════════════════
    // ═════════════════════════════════════════════════════

    pub fn get_float(&self, index: usize) -> Option<f64> {
        self.inputs.get(index)?.as_float()
    }

    pub fn get_float_or(&self, index: usize, default: f64) -> f64 {
        self.get_float(index).unwrap_or(default)
    }

    pub fn get_int(&self, index: usize) -> Option<i64> {
        self.inputs.get(index)?.as_int()
    }

    pub fn get_int_or(&self, index: usize, default: i64) -> i64 {
        self.get_int(index).unwrap_or(default)
    }

    pub fn get_bool(&self, index: usize) -> Option<bool> {
        self.inputs.get(index)?.as_bool()
    }

    pub fn get_bool_or(&self, index: usize, default: bool) -> bool {
        self.get_bool(index).unwrap_or(default)
    }

    pub fn get_vec2(&self, index: usize) -> Option<Vec2> {
        self.inputs.get(index)?.as_vec2()
    }

    pub fn get_vec3(&self, index: usize) -> Option<Vec3> {
        self.inputs.get(index)?.as_vec3()
    }

    pub fn get_string(&self, index: usize) -> Option<&str> {
        self.inputs.get(index)?.as_string()
    }

    pub fn get_tagged(&self, index: usize) -> Option<(&str, &JsonValue)> {
        self.inputs.get(index)?.as_tagged()
    }

    pub fn get_entity(&self, index: usize) -> Option<EntityValue> {
        self.inputs.get(index)?.as_entity().cloned()
    }

    pub fn set(&mut self, index: usize, value: NodeValue) {
        if let Some(out) = self.outputs.get_mut(index) {
            *out = value;
        }
    }

    pub fn set_float(&mut self, index: usize, value: f64) {
        self.set(index, NodeValue::float(value));
    }

    pub fn set_int(&mut self, index: usize, value: i64) {
        self.set(index, NodeValue::int(value));
    }

    pub fn set_bool(&mut self, index: usize, value: bool) {
        self.set(index, NodeValue::bool(value));
    }

    pub fn set_vec2(&mut self, index: usize, value: Vec2) {
        self.set(index, NodeValue::Vec2(value));
    }

    pub fn set_vec3(&mut self, index: usize, value: Vec3) {
        self.set(index, NodeValue::Vec3(value));
    }

    pub fn set_string(&mut self, index: usize, value: impl Into<String>) {
        self.set(index, NodeValue::string(value));
    }

    pub fn set_tagged(&mut self, index: usize, tag: impl Into<String>, payload: JsonValue) {
        self.set(index, NodeValue::tagged(tag, payload));
    }

    pub fn set_entity(&mut self, index: usize, value: EntityValue) {
        self.set(index, NodeValue::entity(value));
    }
}

/// Result of a node processing pass.
#[derive(Debug)]
pub enum ProcessResult {
    Success,
    Error(String),
    MissingInput(usize),
}

/// Trait implemented by every node definition.
pub trait NodeDefinition: Send + Sync {
    fn id(&self) -> NodeId;

    fn display_name(&self) -> &str;

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::ADVANCED)
    }

    fn description(&self) -> Option<&str> {
        None
    }

    fn color(&self) -> Color {
        Color::srgb(0.2, 0.2, 0.25)
    }

    fn title_color(&self) -> Color {
        Color::WHITE
    }

    fn wire_color(&self) -> Color {
        self.color()
    }

    fn icon(&self) -> Option<&str> {
        None
    }

    fn inputs(&self) -> Vec<PortDefinition>;

    fn outputs(&self) -> Vec<PortDefinition>;

    fn output_requirement_token(
        &self,
        _output_index: usize,
        _connected_inputs: &[bool],
    ) -> Option<String> {
        None
    }

    fn process(&self, context: &mut ProcessContext) -> ProcessResult;

    /// ═══════════════════════════════════════════════════════════
    /// ═══════════════════════════════════════════════════════════

    fn show_in_menu(&self) -> bool {
        true
    }

    fn menu_order(&self) -> i32 {
        0
    }

    fn keywords(&self) -> Vec<&str> {
        vec![]
    }

    /// ═══════════════════════════════════════════════════════════

    fn can_have_children(&self) -> bool {
        false
    }

    fn default_input_values(&self) -> Vec<NodeValue> {
        self.inputs()
            .iter()
            .map(|p| p.default_value.clone().unwrap_or(NodeValue::None))
            .collect()
    }

    fn input_port_colors(&self) -> Vec<Color> {
        self.inputs().iter().map(|p| p.resolve_color()).collect()
    }

    fn output_port_colors(&self) -> Vec<Color> {
        self.outputs().iter().map(|p| p.resolve_color()).collect()
    }

    fn body_width(&self) -> f32 {
        150.0
    }

    fn body_height(&self) -> f32 {
        let ins = self.inputs().len();
        let outs = self.outputs().len();
        50.0 + (ins.max(outs) as f32 * 22.0)
    }

    ///
    ///
    /// ```ignore
    /// fn build_body(&self, body: &mut ChildSpawnerCommands, node_entity: Entity) {
    ///     body.spawn(UTextLabel {
    ///         text: "Custom Content".to_string(),
    ///         font_size: 12.0,
    ///         color: Color::WHITE,
    ///         ..default()
    ///     });
    /// }
    /// ```
    fn build_body(&self, _body: &mut ChildSpawnerCommands, _node_entity: Entity) {}

    fn has_custom_body(&self) -> bool {
        false
    }

    fn needs_visual_sync(&self) -> bool {
        self.has_custom_body()
    }

    fn sync_visual(&self, _world: &mut World, _node_entity: Entity) {}
}

pub type ArcNodeDefinition = Arc<dyn NodeDefinition>;

#[derive(Component)]
pub struct GraphNode {
    pub definition_id: NodeId,
    pub values: NodeValues,
    pub custom_data: Option<Box<dyn Any + Send + Sync>>,
}

impl GraphNode {
    pub fn new(definition_id: NodeId, input_count: usize, output_count: usize) -> Self {
        Self {
            definition_id,
            values: NodeValues::new(input_count, output_count),
            custom_data: None,
        }
    }
}

#[derive(Component)]
pub struct GraphPort {
    pub node_entity: Entity,
    pub port_type: PortType,
    pub index: usize,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    Input,
    Output,
}

#[derive(Component, Default)]
pub struct InputConnection {
    pub source_node: Option<Entity>,
    pub source_port_index: Option<usize>,
}

#[derive(Component, Default)]
pub struct OutputConnections {
    pub targets: Vec<OutputTarget>,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputTarget {
    pub target_node: Entity,
    pub target_port_index: usize,
}

/// ═══════════════════════════════════════════════════════════════
/// ═══════════════════════════════════════════════════════════════
///
///
///
///
/// ═══════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct InputPort {
    pub node_entity: Entity,
    pub index: usize,
    pub value_type: ValueType,

    pub source: Option<PortRef>,
}

#[derive(Component)]
pub struct OutputPort {
    pub node_entity: Entity,
    pub index: usize,
    pub value_type: ValueType,

    pub value: NodeValue,

    pub targets: Vec<PortRef>,
}

#[derive(Debug, Clone, Copy)]
pub struct PortRef {
    pub node: Entity,
    pub port_index: usize,
}

impl InputPort {
    pub fn get_value(
        &self,
        outputs: &std::collections::HashMap<Entity, Vec<NodeValue>>,
    ) -> NodeValue {
        if let Some(ref source) = self.source {
            if let Some(node_outputs) = outputs.get(&source.node) {
                if source.port_index < node_outputs.len() {
                    return node_outputs[source.port_index].clone();
                }
            }
        }
        NodeValue::None
    }
}

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct ValueDisplayLabel {
    pub node_entity: Entity,
}

#[derive(Component)]
pub struct Dragging;
