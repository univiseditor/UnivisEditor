//! Adapter-owned runtime values for the Bevy-facing node-graph layer.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use univis_scene::EntityValue;

/// Supported value kinds for the Bevy-facing node-graph adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    Float,
    Int,
    Bool,
    String,
    Vec2,
    Vec3,
    Vec4,
    Color,
    Entity,
    CustomTag(String),
    Any,
}

impl ValueType {
    pub fn display_name(&self) -> String {
        match self {
            ValueType::Float => "Float".to_string(),
            ValueType::Int => "Int".to_string(),
            ValueType::Bool => "Bool".to_string(),
            ValueType::String => "String".to_string(),
            ValueType::Vec2 => "Vec2".to_string(),
            ValueType::Vec3 => "Vec3".to_string(),
            ValueType::Vec4 => "Vec4".to_string(),
            ValueType::Color => "Color".to_string(),
            ValueType::Entity => "Entity".to_string(),
            ValueType::CustomTag(tag) => format!("Tag({})", tag),
            ValueType::Any => "Any".to_string(),
        }
    }

    pub fn port_color(&self) -> Color {
        match self {
            ValueType::Float => Color::srgb(0.4, 0.7, 1.0),
            ValueType::Int => Color::srgb(0.3, 1.0, 0.5),
            ValueType::Bool => Color::srgb(1.0, 0.3, 0.3),
            ValueType::String => Color::srgb(1.0, 0.8, 0.2),
            ValueType::Vec2 => Color::srgb(0.8, 0.4, 1.0),
            ValueType::Vec3 => Color::srgb(0.6, 0.8, 1.0),
            ValueType::Vec4 => Color::srgb(0.5, 0.9, 0.9),
            ValueType::Color => Color::srgb(1.0, 0.5, 0.8),
            ValueType::Entity => Color::srgb(0.68, 0.56, 0.24),
            ValueType::CustomTag(tag) => custom_tag_color(tag),
            ValueType::Any => Color::srgb(0.7, 0.7, 0.7),
        }
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        match (self, other) {
            (a, b) if a == b => true,
            (_, ValueType::Any) => true,
            (ValueType::Any, _) => true,
            (ValueType::CustomTag(from_tag), ValueType::CustomTag(to_tag)) => from_tag == to_tag,
            (ValueType::Int, ValueType::Float) => true,
            (ValueType::Float, ValueType::Int) => true,
            (ValueType::Vec2, ValueType::Vec3) => true,
            (ValueType::Vec2, ValueType::Vec4) => true,
            (ValueType::Vec3, ValueType::Vec2) => true,
            (ValueType::Vec3, ValueType::Vec4) => true,
            (ValueType::Vec4, ValueType::Vec2) => true,
            (ValueType::Vec4, ValueType::Vec3) => true,
            (ValueType::Color, ValueType::Vec4) => true,
            (ValueType::Vec4, ValueType::Color) => true,
            _ => false,
        }
    }
}

fn custom_tag_color(tag: &str) -> Color {
    let mut hasher = DefaultHasher::new();
    tag.hash(&mut hasher);
    let hash = hasher.finish();

    let r = ((hash & 0xFF) as f32 / 255.0) * 0.45 + 0.35;
    let g = (((hash >> 8) & 0xFF) as f32 / 255.0) * 0.45 + 0.35;
    let b = (((hash >> 16) & 0xFF) as f32 / 255.0) * 0.45 + 0.35;

    Color::srgb(r, g, b)
}

/// Dynamic value flowing through graph edges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    String(String),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Color(Color),
    Entity(EntityValue),
    TaggedData { tag: String, payload: JsonValue },
    None,
}

impl Default for NodeValue {
    fn default() -> Self {
        NodeValue::None
    }
}

impl NodeValue {
    pub fn value_type(&self) -> ValueType {
        match self {
            NodeValue::Float(_) => ValueType::Float,
            NodeValue::Int(_) => ValueType::Int,
            NodeValue::Bool(_) => ValueType::Bool,
            NodeValue::String(_) => ValueType::String,
            NodeValue::Vec2(_) => ValueType::Vec2,
            NodeValue::Vec3(_) => ValueType::Vec3,
            NodeValue::Vec4(_) => ValueType::Vec4,
            NodeValue::Color(_) => ValueType::Color,
            NodeValue::Entity(_) => ValueType::Entity,
            NodeValue::TaggedData { tag, .. } => ValueType::CustomTag(tag.clone()),
            NodeValue::None => ValueType::Any,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, NodeValue::None)
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            NodeValue::Float(f) => Some(*f),
            NodeValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            NodeValue::Int(i) => Some(*i),
            NodeValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            NodeValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            NodeValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_vec2(&self) -> Option<Vec2> {
        match self {
            NodeValue::Vec2(v) => Some(*v),
            NodeValue::Vec3(v) => Some(v.truncate()),
            NodeValue::Vec4(v) => Some(v.truncate().truncate()),
            _ => None,
        }
    }

    pub fn as_vec3(&self) -> Option<Vec3> {
        match self {
            NodeValue::Vec3(v) => Some(*v),
            NodeValue::Vec4(v) => Some(v.truncate()),
            NodeValue::Vec2(v) => Some(v.extend(0.0)),
            _ => None,
        }
    }

    pub fn as_vec4(&self) -> Option<Vec4> {
        match self {
            NodeValue::Vec4(v) => Some(*v),
            NodeValue::Vec3(v) => Some(v.extend(1.0)),
            NodeValue::Vec2(v) => Some(v.extend(0.0).extend(1.0)),
            _ => None,
        }
    }

    pub fn as_color(&self) -> Option<Color> {
        match self {
            NodeValue::Color(c) => Some(*c),
            NodeValue::Vec4(v) => Some(Color::srgba(v.x, v.y, v.z, v.w)),
            _ => None,
        }
    }

    pub fn as_entity(&self) -> Option<&EntityValue> {
        match self {
            NodeValue::Entity(entity) => Some(entity),
            _ => None,
        }
    }

    pub fn as_tagged(&self) -> Option<(&str, &JsonValue)> {
        match self {
            NodeValue::TaggedData { tag, payload } => Some((tag.as_str(), payload)),
            _ => None,
        }
    }

    pub fn float(value: f64) -> Self {
        NodeValue::Float(value)
    }

    pub fn int(value: i64) -> Self {
        NodeValue::Int(value)
    }

    pub fn bool(value: bool) -> Self {
        NodeValue::Bool(value)
    }

    pub fn string(value: impl Into<String>) -> Self {
        NodeValue::String(value.into())
    }

    pub fn vec2(x: f32, y: f32) -> Self {
        NodeValue::Vec2(Vec2::new(x, y))
    }

    pub fn vec3(x: f32, y: f32, z: f32) -> Self {
        NodeValue::Vec3(Vec3::new(x, y, z))
    }

    pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Self {
        NodeValue::Vec4(Vec4::new(x, y, z, w))
    }

    pub fn color(r: f32, g: f32, b: f32, a: f32) -> Self {
        NodeValue::Color(Color::srgba(r, g, b, a))
    }

    pub fn entity(value: EntityValue) -> Self {
        NodeValue::Entity(value)
    }

    pub fn tagged(tag: impl Into<String>, payload: JsonValue) -> Self {
        NodeValue::TaggedData {
            tag: tag.into(),
            payload,
        }
    }

    pub fn is_compatible(from: &ValueType, to: &ValueType) -> bool {
        from.is_compatible_with(to)
    }

    pub fn to_display_string(&self) -> String {
        match self {
            NodeValue::Float(f) => format!("{:.2}", f),
            NodeValue::Int(i) => i.to_string(),
            NodeValue::Bool(b) => b.to_string(),
            NodeValue::String(s) => {
                if s.len() > 20 {
                    format!("{}...", &s[..17])
                } else {
                    s.clone()
                }
            }
            NodeValue::Vec2(v) => format!("({:.1}, {:.1})", v.x, v.y),
            NodeValue::Vec3(v) => format!("({:.1}, {:.1}, {:.1})", v.x, v.y, v.z),
            NodeValue::Vec4(v) => format!("({:.1}, {:.1}, {:.1}, {:.1})", v.x, v.y, v.z, v.w),
            NodeValue::Color(c) => {
                let srgba = c.to_srgba();
                format!(
                    "({:.1}, {:.1}, {:.1}, {:.1})",
                    srgba.red, srgba.green, srgba.blue, srgba.alpha
                )
            }
            NodeValue::Entity(entity) => {
                let name = entity.name.as_deref().unwrap_or("Entity");
                format!(
                    "{} [{}c {}ch]",
                    name,
                    entity.components.len(),
                    entity.children.len()
                )
            }
            NodeValue::TaggedData { tag, payload } => {
                let payload_text = payload.to_string();
                let clipped = if payload_text.len() > 32 {
                    format!("{}...", &payload_text[..29])
                } else {
                    payload_text
                };
                format!("{} {}", tag, clipped)
            }
            NodeValue::None => "None".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NodeValues {
    pub inputs: Vec<NodeValue>,
    pub outputs: Vec<NodeValue>,
}

impl NodeValues {
    pub fn new(input_count: usize, output_count: usize) -> Self {
        Self {
            inputs: vec![NodeValue::None; input_count],
            outputs: vec![NodeValue::None; output_count],
        }
    }

    pub fn set_input(&mut self, index: usize, value: NodeValue) {
        if index < self.inputs.len() {
            self.inputs[index] = value;
        }
    }

    pub fn set_output(&mut self, index: usize, value: NodeValue) {
        if index < self.outputs.len() {
            self.outputs[index] = value;
        }
    }

    pub fn get_input(&self, index: usize) -> Option<&NodeValue> {
        self.inputs.get(index)
    }

    pub fn get_output(&self, index: usize) -> Option<&NodeValue> {
        self.outputs.get(index)
    }
}
