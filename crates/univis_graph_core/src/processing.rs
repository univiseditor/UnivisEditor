use std::any::Any;

use crate::identity::{NodeCategory, NodeId};

pub trait ProcessValueAccess: Clone {
    type Vec2: Copy;
    type Vec3: Copy;
    type TaggedPayload;
    type Entity: Clone;

    fn is_none(&self) -> bool;
    fn as_float(&self) -> Option<f64>;
    fn as_int(&self) -> Option<i64>;
    fn as_bool(&self) -> Option<bool>;
    fn as_string(&self) -> Option<&str>;
    fn as_vec2(&self) -> Option<Self::Vec2>;
    fn as_vec3(&self) -> Option<Self::Vec3>;
    fn as_tagged(&self) -> Option<(&str, &Self::TaggedPayload)>;
    fn as_entity(&self) -> Option<&Self::Entity>;

    fn from_float(value: f64) -> Self;
    fn from_int(value: i64) -> Self;
    fn from_bool(value: bool) -> Self;
    fn from_string(value: String) -> Self;
    fn from_vec2(value: Self::Vec2) -> Self;
    fn from_vec3(value: Self::Vec3) -> Self;
    fn from_tagged(tag: String, payload: Self::TaggedPayload) -> Self;
    fn from_entity(value: Self::Entity) -> Self;
}

/// Runtime context passed to a node during processing.
pub struct ProcessContext<'a, Value> {
    pub inputs: &'a [Value],
    pub outputs: &'a mut [Value],
    pub delta_time: f32,
    pub custom_data: &'a mut Option<Box<dyn Any + Send + Sync>>,
}

impl<'a, Value> ProcessContext<'a, Value> {
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.inputs.get(index)
    }

    pub fn set(&mut self, index: usize, value: Value) {
        if let Some(out) = self.outputs.get_mut(index) {
            *out = value;
        }
    }
}

impl<'a, Value> ProcessContext<'a, Value>
where
    Value: ProcessValueAccess,
{
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

    pub fn get_vec2(&self, index: usize) -> Option<Value::Vec2> {
        self.inputs.get(index)?.as_vec2()
    }

    pub fn get_vec3(&self, index: usize) -> Option<Value::Vec3> {
        self.inputs.get(index)?.as_vec3()
    }

    pub fn get_string(&self, index: usize) -> Option<&str> {
        self.inputs.get(index)?.as_string()
    }

    pub fn get_tagged(&self, index: usize) -> Option<(&str, &Value::TaggedPayload)> {
        self.inputs.get(index)?.as_tagged()
    }

    pub fn get_entity(&self, index: usize) -> Option<Value::Entity> {
        self.inputs.get(index)?.as_entity().cloned()
    }

    pub fn set_float(&mut self, index: usize, value: f64) {
        self.set(index, Value::from_float(value));
    }

    pub fn set_int(&mut self, index: usize, value: i64) {
        self.set(index, Value::from_int(value));
    }

    pub fn set_bool(&mut self, index: usize, value: bool) {
        self.set(index, Value::from_bool(value));
    }

    pub fn set_vec2(&mut self, index: usize, value: Value::Vec2) {
        self.set(index, Value::from_vec2(value));
    }

    pub fn set_vec3(&mut self, index: usize, value: Value::Vec3) {
        self.set(index, Value::from_vec3(value));
    }

    pub fn set_string(&mut self, index: usize, value: impl Into<String>) {
        self.set(index, Value::from_string(value.into()));
    }

    pub fn set_tagged(
        &mut self,
        index: usize,
        tag: impl Into<String>,
        payload: Value::TaggedPayload,
    ) {
        self.set(index, Value::from_tagged(tag.into(), payload));
    }

    pub fn set_entity(&mut self, index: usize, value: Value::Entity) {
        self.set(index, Value::from_entity(value));
    }
}

/// Result of a node processing pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessResult {
    Success,
    Error(String),
    MissingInput(usize),
}

/// Engine-independent processing contract for a node definition.
pub trait GraphNodeDefinition<Value, Port>: Send + Sync {
    fn id(&self) -> NodeId;

    fn display_name(&self) -> &str;

    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::ADVANCED)
    }

    fn description(&self) -> Option<&str> {
        None
    }

    fn inputs(&self) -> Vec<Port>;

    fn outputs(&self) -> Vec<Port>;

    fn process(&self, context: &mut ProcessContext<'_, Value>) -> ProcessResult;

    fn show_in_menu(&self) -> bool {
        true
    }

    fn menu_order(&self) -> i32 {
        0
    }

    fn keywords(&self) -> Vec<&str> {
        vec![]
    }

    fn can_have_children(&self) -> bool {
        false
    }
}

pub use GraphNodeDefinition as NodeDefinition;

#[cfg(test)]
mod tests {
    use super::{GraphNodeDefinition, ProcessContext, ProcessResult, ProcessValueAccess};
    use crate::identity::{NodeCategory, NodeId};

    #[derive(Debug, Clone, PartialEq)]
    enum DummyValue {
        Float(f64),
        Bool(bool),
        String(String),
        None,
    }

    impl ProcessValueAccess for DummyValue {
        type Vec2 = ();
        type Vec3 = ();
        type TaggedPayload = ();
        type Entity = String;

        fn is_none(&self) -> bool {
            matches!(self, Self::None)
        }

        fn as_float(&self) -> Option<f64> {
            match self {
                Self::Float(value) => Some(*value),
                _ => None,
            }
        }

        fn as_int(&self) -> Option<i64> {
            None
        }

        fn as_bool(&self) -> Option<bool> {
            match self {
                Self::Bool(value) => Some(*value),
                _ => None,
            }
        }

        fn as_string(&self) -> Option<&str> {
            match self {
                Self::String(value) => Some(value),
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
            Self::Float(value)
        }

        fn from_int(_value: i64) -> Self {
            Self::None
        }

        fn from_bool(value: bool) -> Self {
            Self::Bool(value)
        }

        fn from_string(value: String) -> Self {
            Self::String(value)
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

    struct DummyNode;

    impl GraphNodeDefinition<DummyValue, &'static str> for DummyNode {
        fn id(&self) -> NodeId {
            NodeId::new("tests/dummy")
        }

        fn display_name(&self) -> &str {
            "Dummy"
        }

        fn category(&self) -> NodeCategory {
            NodeCategory::new("Tests")
        }

        fn inputs(&self) -> Vec<&'static str> {
            vec!["Input"]
        }

        fn outputs(&self) -> Vec<&'static str> {
            vec!["Output"]
        }

        fn process(&self, context: &mut ProcessContext<'_, DummyValue>) -> ProcessResult {
            context.set_float(0, context.get_float_or(0, 0.0) + 2.0);
            ProcessResult::Success
        }
    }

    #[test]
    fn process_context_supports_generic_value_helpers() {
        let mut custom_data = None;
        let mut outputs = vec![DummyValue::None];
        let mut context = ProcessContext {
            inputs: &[DummyValue::Float(3.0)],
            outputs: &mut outputs,
            delta_time: 0.0,
            custom_data: &mut custom_data,
        };

        DummyNode.process(&mut context);

        assert_eq!(context.outputs[0], DummyValue::Float(5.0));
    }
}
