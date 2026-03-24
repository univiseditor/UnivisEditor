#![allow(dead_code)]

use univis_graph_core::prelude::{
    GraphNodeDefinition, GraphNodeRegistry, GraphSchema, NodeCategory, NodeId, PortDefinition,
    PortSchema, ProcessContext, ProcessResult, ProcessValueAccess,
};

#[derive(Clone, Debug, PartialEq)]
pub enum DemoValue {
    Number(f64),
    Bool(bool),
    Text(String),
    None,
}

impl DemoValue {
    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    pub fn boolean(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn label(&self) -> String {
        match self {
            Self::Number(value) => format!("{value:.2}"),
            Self::Bool(value) => value.to_string(),
            Self::Text(value) => value.clone(),
            Self::None => "None".to_string(),
        }
    }
}

impl Default for DemoValue {
    fn default() -> Self {
        Self::None
    }
}

impl ProcessValueAccess for DemoValue {
    type Vec2 = ();
    type Vec3 = ();
    type TaggedPayload = ();
    type Entity = ();

    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    fn as_float(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    fn as_int(&self) -> Option<i64> {
        self.as_float().map(|value| value as i64)
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    fn as_vec2(&self) -> Option<Self::Vec2> {
        None
    }

    fn as_vec3(&self) -> Option<Self::Vec3> {
        None
    }

    fn as_tagged(&self) -> Option<(&str, &Self::TaggedPayload)> {
        None
    }

    fn as_entity(&self) -> Option<&Self::Entity> {
        None
    }

    fn from_float(value: f64) -> Self {
        Self::number(value)
    }

    fn from_int(value: i64) -> Self {
        Self::number(value as f64)
    }

    fn from_bool(value: bool) -> Self {
        Self::boolean(value)
    }

    fn from_string(value: String) -> Self {
        Self::text(value)
    }

    fn from_vec2(_value: Self::Vec2) -> Self {
        Self::None
    }

    fn from_vec3(_value: Self::Vec3) -> Self {
        Self::None
    }

    fn from_tagged(_tag: String, _payload: Self::TaggedPayload) -> Self {
        Self::None
    }

    fn from_entity(_value: Self::Entity) -> Self {
        Self::None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DemoTypeTag {
    Number,
    Bool,
    Text,
    Any,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoRequirement {
    pub token: &'static str,
    pub label: &'static str,
}

pub struct DemoSchema;

impl PortSchema for DemoSchema {
    type TypeTag = DemoTypeTag;
    type Requirement = DemoRequirement;
    type DefaultValue = DemoValue;

    fn ports_compatible(from: &Self::TypeTag, to: &Self::TypeTag) -> bool {
        match (from, to) {
            (a, b) if a == b => true,
            (_, DemoTypeTag::Any) => true,
            (DemoTypeTag::Any, _) => true,
            _ => false,
        }
    }
}

impl GraphSchema for DemoSchema {
    fn requirement_satisfied(
        requirement: Option<&Self::Requirement>,
        output_requirement_token: Option<&str>,
    ) -> bool {
        match requirement {
            Some(requirement) => output_requirement_token == Some(requirement.token),
            None => true,
        }
    }

    fn requirement_label(requirement: &Self::Requirement) -> String {
        requirement.label.to_string()
    }
}

pub type DemoPort = PortDefinition<DemoSchema>;
pub type DemoRegistry = GraphNodeRegistry<DemoValue, DemoPort>;

pub fn build_demo_registry() -> DemoRegistry {
    let mut registry = DemoRegistry::new();
    registry.register(NumberInputNode);
    registry.register(TextInputNode);
    registry.register(AddNode);
    registry.register(PublishNumberNode);
    registry.register(FormatNode);
    registry.register(CounterNode);
    registry.register(GroupNode);
    registry
}

pub fn describe_values(values: &[DemoValue]) -> Vec<String> {
    values.iter().map(DemoValue::label).collect()
}

struct NumberInputNode;

impl GraphNodeDefinition<DemoValue, DemoPort> for NumberInputNode {
    fn id(&self) -> NodeId {
        NodeId::new("demo/input_number")
    }

    fn display_name(&self) -> &str {
        "Number Input"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::INPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("Copies an authored numeric value to its output.")
    }

    fn inputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Value", DemoTypeTag::Number).with_default(DemoValue::number(0.0))]
    }

    fn outputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Value", DemoTypeTag::Number)]
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        _connected_inputs: &[bool],
    ) -> Option<String> {
        (output_index == 0).then_some("numeric-output".to_string())
    }

    fn process(&self, context: &mut ProcessContext<'_, DemoValue>) -> ProcessResult {
        let value = context.get_float_or(0, 0.0);
        context.set_float(0, value);
        ProcessResult::Success
    }

    fn menu_order(&self) -> i32 {
        10
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["number", "input", "constant"]
    }
}

struct TextInputNode;

impl GraphNodeDefinition<DemoValue, DemoPort> for TextInputNode {
    fn id(&self) -> NodeId {
        NodeId::new("demo/input_text")
    }

    fn display_name(&self) -> &str {
        "Text Input"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::INPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("Copies an authored text value to its output.")
    }

    fn inputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Text", DemoTypeTag::Text).with_default(DemoValue::text("hello"))]
    }

    fn outputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Text", DemoTypeTag::Text)]
    }

    fn process(&self, context: &mut ProcessContext<'_, DemoValue>) -> ProcessResult {
        let value = context.get_string(0).unwrap_or_default().to_string();
        context.set_string(0, value);
        ProcessResult::Success
    }

    fn menu_order(&self) -> i32 {
        11
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["text", "input"]
    }
}

struct AddNode;

impl GraphNodeDefinition<DemoValue, DemoPort> for AddNode {
    fn id(&self) -> NodeId {
        NodeId::new("demo/add")
    }

    fn display_name(&self) -> &str {
        "Add"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::MATH)
    }

    fn description(&self) -> Option<&str> {
        Some("Adds two numeric inputs.")
    }

    fn inputs(&self) -> Vec<DemoPort> {
        vec![
            DemoPort::new("A", DemoTypeTag::Number).with_default(DemoValue::number(0.0)),
            DemoPort::new("B", DemoTypeTag::Number).with_default(DemoValue::number(0.0)),
        ]
    }

    fn outputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Sum", DemoTypeTag::Number)]
    }

    fn process(&self, context: &mut ProcessContext<'_, DemoValue>) -> ProcessResult {
        let a = context.get_float_or(0, 0.0);
        let b = context.get_float_or(1, 0.0);
        context.set_float(0, a + b);
        ProcessResult::Success
    }

    fn menu_order(&self) -> i32 {
        20
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["math", "sum", "number"]
    }
}

struct PublishNumberNode;

impl GraphNodeDefinition<DemoValue, DemoPort> for PublishNumberNode {
    fn id(&self) -> NodeId {
        NodeId::new("demo/publish_number")
    }

    fn display_name(&self) -> &str {
        "Publish Number"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::OUTPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("Relays a number while advertising the numeric-output requirement token.")
    }

    fn inputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Value", DemoTypeTag::Number).with_default(DemoValue::number(0.0))]
    }

    fn outputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Published", DemoTypeTag::Number)]
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        _connected_inputs: &[bool],
    ) -> Option<String> {
        (output_index == 0).then_some("numeric-output".to_string())
    }

    fn process(&self, context: &mut ProcessContext<'_, DemoValue>) -> ProcessResult {
        let value = context.get_float_or(0, 0.0);
        context.set_float(0, value);
        ProcessResult::Success
    }

    fn menu_order(&self) -> i32 {
        30
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["publish", "number", "output"]
    }
}

struct FormatNode;

impl GraphNodeDefinition<DemoValue, DemoPort> for FormatNode {
    fn id(&self) -> NodeId {
        NodeId::new("demo/format")
    }

    fn display_name(&self) -> &str {
        "Format Number"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::OUTPUT)
    }

    fn description(&self) -> Option<&str> {
        Some("Formats a numeric input into text.")
    }

    fn inputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Value", DemoTypeTag::Number)
            .with_default(DemoValue::number(0.0))
            .with_requirement(DemoRequirement {
                token: "numeric-output",
                label: "Numeric Output",
            })]
    }

    fn outputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Text", DemoTypeTag::Text)]
    }

    fn process(&self, context: &mut ProcessContext<'_, DemoValue>) -> ProcessResult {
        let value = context.get_float_or(0, 0.0);
        context.set_string(0, format!("value={value:.1}"));
        ProcessResult::Success
    }

    fn menu_order(&self) -> i32 {
        31
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["format", "text", "number"]
    }
}

struct CounterNode;

impl GraphNodeDefinition<DemoValue, DemoPort> for CounterNode {
    fn id(&self) -> NodeId {
        NodeId::new("demo/counter")
    }

    fn display_name(&self) -> &str {
        "Counter"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::ADVANCED)
    }

    fn description(&self) -> Option<&str> {
        Some("Accumulates a numeric step inside node custom data.")
    }

    fn inputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Step", DemoTypeTag::Number).with_default(DemoValue::number(1.0))]
    }

    fn outputs(&self) -> Vec<DemoPort> {
        vec![DemoPort::new("Count", DemoTypeTag::Number)]
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        _connected_inputs: &[bool],
    ) -> Option<String> {
        (output_index == 0).then_some("numeric-output".to_string())
    }

    fn process(&self, context: &mut ProcessContext<'_, DemoValue>) -> ProcessResult {
        let step = context.get_float_or(0, 1.0);
        let slot = context.custom_data.get_or_insert_with(|| Box::new(0.0_f64));
        let Some(total) = slot.downcast_mut::<f64>() else {
            return ProcessResult::Error(
                "Counter node expected `custom_data` to store an f64.".to_string(),
            );
        };

        *total += step;
        let next_total = *total;
        context.set_float(0, next_total);
        ProcessResult::Success
    }

    fn show_in_menu(&self) -> bool {
        false
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["counter", "custom-data", "stateful"]
    }
}

struct GroupNode;

impl GraphNodeDefinition<DemoValue, DemoPort> for GroupNode {
    fn id(&self) -> NodeId {
        NodeId::new("demo/group")
    }

    fn display_name(&self) -> &str {
        "Group"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::ADVANCED)
    }

    fn description(&self) -> Option<&str> {
        Some("Zero-IO container node used to demonstrate can_have_children.")
    }

    fn inputs(&self) -> Vec<DemoPort> {
        vec![]
    }

    fn outputs(&self) -> Vec<DemoPort> {
        vec![]
    }

    fn process(&self, _context: &mut ProcessContext<'_, DemoValue>) -> ProcessResult {
        ProcessResult::Success
    }

    fn can_have_children(&self) -> bool {
        true
    }

    fn menu_order(&self) -> i32 {
        90
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["group", "container"]
    }
}
