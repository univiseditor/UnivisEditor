//! Core node-definition types and runtime traits.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
pub use univis_graph_core::prelude::ProcessResult;
pub use univis_graph_core::prelude::{ConnectionPolicy, NodeCategory, NodeId};
use univis_graph_core::prelude::{
    GraphNodeDefinition as CoreGraphNodeDefinition, GraphSchema,
    PortDefinition as CorePortDefinition, PortSchema, ProcessContext as CoreProcessContext,
    ProcessValueAccess,
};
use univis_scene::EntityValue;

use super::value::{NodeValue, ValueType};

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

#[derive(Debug, Clone, Copy, Default)]
pub struct NodeGraphSchema;

impl PortSchema for NodeGraphSchema {
    type TypeTag = ValueType;
    type Requirement = PortRequirement;
    type DefaultValue = NodeValue;

    fn ports_compatible(from: &Self::TypeTag, to: &Self::TypeTag) -> bool {
        from.is_compatible_with(to)
    }
}

impl GraphSchema for NodeGraphSchema {
    fn requirement_satisfied(
        requirement: Option<&Self::Requirement>,
        output_requirement_token: Option<&str>,
    ) -> bool {
        match requirement {
            Some(requirement) => output_requirement_token == Some(requirement.id.as_str()),
            None => true,
        }
    }

    fn requirement_label(requirement: &Self::Requirement) -> String {
        requirement.label.clone()
    }
}

impl NodeGraphSchema {
    pub fn ports_compatible(from: &ValueType, to: &ValueType) -> bool {
        <Self as PortSchema>::ports_compatible(from, to)
    }

    pub fn requirement_satisfied(
        requirement: Option<&PortRequirement>,
        output_requirement_token: Option<&str>,
    ) -> bool {
        <Self as GraphSchema>::requirement_satisfied(requirement, output_requirement_token)
    }

    pub fn requirement_label(requirement: &PortRequirement) -> String {
        <Self as GraphSchema>::requirement_label(requirement)
    }
}

pub type NodeGraphPortSchema = NodeGraphSchema;

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
    pub connection_policy: ConnectionPolicy,
    #[serde(default)]
    pub editable_inline: bool,
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
            connection_policy: ConnectionPolicy::Single,
            editable_inline: false,
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

    pub fn with_connection_policy(mut self, policy: ConnectionPolicy) -> Self {
        self.connection_policy = policy;
        self
    }

    pub fn allow_multiple_connections(mut self) -> Self {
        self.connection_policy = ConnectionPolicy::Multiple;
        self
    }

    pub fn editable_inline(mut self) -> Self {
        self.editable_inline = true;
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

    pub fn accepts_multiple_connections(&self) -> bool {
        self.connection_policy == ConnectionPolicy::Multiple
    }

    pub fn as_core(&self) -> CorePortDefinition<NodeGraphSchema> {
        let mut core = CorePortDefinition::new(self.name.clone(), self.value_type.clone())
            .with_connection_policy(self.connection_policy);

        if let Some(description) = &self.description {
            core = core.with_description(description.clone());
        }

        if let Some(default_value) = &self.default_value {
            core = core.with_default(default_value.clone());
        }

        if let Some(requirement) = &self.requirement {
            core = core.with_requirement(requirement.clone());
        }

        core
    }
}

pub type ProcessContext<'a> = CoreProcessContext<'a, NodeValue>;

impl ProcessValueAccess for NodeValue {
    type Vec2 = Vec2;
    type Vec3 = Vec3;
    type TaggedPayload = JsonValue;
    type Entity = EntityValue;

    fn is_none(&self) -> bool {
        self.is_none()
    }

    fn as_float(&self) -> Option<f64> {
        self.as_float()
    }

    fn as_int(&self) -> Option<i64> {
        self.as_int()
    }

    fn as_bool(&self) -> Option<bool> {
        self.as_bool()
    }

    fn as_string(&self) -> Option<&str> {
        self.as_string()
    }

    fn as_vec2(&self) -> Option<<Self as ProcessValueAccess>::Vec2> {
        self.as_vec2()
    }

    fn as_vec3(&self) -> Option<<Self as ProcessValueAccess>::Vec3> {
        self.as_vec3()
    }

    fn as_tagged(&self) -> Option<(&str, &Self::TaggedPayload)> {
        self.as_tagged()
    }

    fn as_entity(&self) -> Option<&<Self as ProcessValueAccess>::Entity> {
        self.as_entity()
    }

    fn from_float(value: f64) -> Self {
        Self::float(value)
    }

    fn from_int(value: i64) -> Self {
        Self::int(value)
    }

    fn from_bool(value: bool) -> Self {
        Self::bool(value)
    }

    fn from_string(value: String) -> Self {
        Self::string(value)
    }

    fn from_vec2(value: <Self as ProcessValueAccess>::Vec2) -> Self {
        Self::Vec2(value)
    }

    fn from_vec3(value: <Self as ProcessValueAccess>::Vec3) -> Self {
        Self::Vec3(value)
    }

    fn from_tagged(tag: String, payload: Self::TaggedPayload) -> Self {
        Self::tagged(tag, payload)
    }

    fn from_entity(value: <Self as ProcessValueAccess>::Entity) -> Self {
        Self::entity(value)
    }
}

/// Bevy-facing node definition with editor/runtime hooks.
pub trait BevyNodeDefinition: Send + Sync {
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

pub use crate::live_graph::{
    AuthoredNodeInputs, Dragging, GraphNode, GraphPort, InputConnection, InputPort,
    OutputConnections, OutputPort, OutputTarget, PortRef, PortType, Selected, ValueDisplayLabel,
};
pub use BevyNodeDefinition as NodeDefinition;

pub struct BevyNodeDefinitionAdapterRef<'a> {
    inner: &'a dyn BevyNodeDefinition,
}

impl<'a> BevyNodeDefinitionAdapterRef<'a> {
    pub fn new(inner: &'a dyn BevyNodeDefinition) -> Self {
        Self { inner }
    }
}

impl CoreGraphNodeDefinition<NodeValue, CorePortDefinition<NodeGraphSchema>>
    for BevyNodeDefinitionAdapterRef<'_>
{
    fn id(&self) -> NodeId {
        self.inner.id()
    }

    fn display_name(&self) -> &str {
        self.inner.display_name()
    }

    fn category(&self) -> NodeCategory {
        self.inner.category()
    }

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn inputs(&self) -> Vec<CorePortDefinition<NodeGraphSchema>> {
        self.inner
            .inputs()
            .into_iter()
            .map(|port| port.as_core())
            .collect()
    }

    fn outputs(&self) -> Vec<CorePortDefinition<NodeGraphSchema>> {
        self.inner
            .outputs()
            .into_iter()
            .map(|port| port.as_core())
            .collect()
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        connected_inputs: &[bool],
    ) -> Option<String> {
        self.inner
            .output_requirement_token(output_index, connected_inputs)
    }

    fn process(&self, context: &mut CoreProcessContext<'_, NodeValue>) -> ProcessResult {
        self.inner.process(context)
    }

    fn show_in_menu(&self) -> bool {
        self.inner.show_in_menu()
    }

    fn menu_order(&self) -> i32 {
        self.inner.menu_order()
    }

    fn keywords(&self) -> Vec<&str> {
        self.inner.keywords()
    }

    fn can_have_children(&self) -> bool {
        self.inner.can_have_children()
    }
}

pub struct OwnedBevyNodeDefinitionAdapter {
    inner: ArcNodeDefinition,
}

impl OwnedBevyNodeDefinitionAdapter {
    pub fn new(inner: ArcNodeDefinition) -> Self {
        Self { inner }
    }
}

impl CoreGraphNodeDefinition<NodeValue, CorePortDefinition<NodeGraphSchema>>
    for OwnedBevyNodeDefinitionAdapter
{
    fn id(&self) -> NodeId {
        self.inner.id()
    }

    fn display_name(&self) -> &str {
        self.inner.display_name()
    }

    fn category(&self) -> NodeCategory {
        self.inner.category()
    }

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn inputs(&self) -> Vec<CorePortDefinition<NodeGraphSchema>> {
        self.inner
            .inputs()
            .into_iter()
            .map(|port| port.as_core())
            .collect()
    }

    fn outputs(&self) -> Vec<CorePortDefinition<NodeGraphSchema>> {
        self.inner
            .outputs()
            .into_iter()
            .map(|port| port.as_core())
            .collect()
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        connected_inputs: &[bool],
    ) -> Option<String> {
        self.inner
            .output_requirement_token(output_index, connected_inputs)
    }

    fn process(&self, context: &mut CoreProcessContext<'_, NodeValue>) -> ProcessResult {
        self.inner.process(context)
    }

    fn show_in_menu(&self) -> bool {
        self.inner.show_in_menu()
    }

    fn menu_order(&self) -> i32 {
        self.inner.menu_order()
    }

    fn keywords(&self) -> Vec<&str> {
        self.inner.keywords()
    }

    fn can_have_children(&self) -> bool {
        self.inner.can_have_children()
    }
}

pub type CoreNodeDefinitionRef<'a> = BevyNodeDefinitionAdapterRef<'a>;
pub type ArcBevyNodeDefinition = Arc<dyn BevyNodeDefinition>;
pub type ArcNodeDefinition = ArcBevyNodeDefinition;
