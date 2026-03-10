//! نظام القيم الديناميكية للعُقد
//! يدعم الأنواع الأساسية إضافةً إلى EntityValue التركيبي للمشاهد المعتمدة على الجراف.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// أنواع البيانات المدعومة
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    Any, // لأي نوع آخر
}

impl ValueType {
    /// الحصول على اسم النوع للعرض
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

    /// لون المنفذ حسب النوع
    pub fn port_color(&self) -> Color {
        match self {
            ValueType::Float => Color::srgb(0.4, 0.7, 1.0), // أزرق فاتح
            ValueType::Int => Color::srgb(0.3, 1.0, 0.5),   // أخضر
            ValueType::Bool => Color::srgb(1.0, 0.3, 0.3),  // أحمر
            ValueType::String => Color::srgb(1.0, 0.8, 0.2), // أصفر
            ValueType::Vec2 => Color::srgb(0.8, 0.4, 1.0),  // بنفسجي
            ValueType::Vec3 => Color::srgb(0.6, 0.8, 1.0),  // سماوي
            ValueType::Vec4 => Color::srgb(0.5, 0.9, 0.9),  // تركواز
            ValueType::Color => Color::srgb(1.0, 0.5, 0.8), // وردي
            ValueType::Entity => Color::srgb(0.88, 0.67, 0.28),
            ValueType::CustomTag(tag) => custom_tag_color(tag),
            ValueType::Any => Color::srgb(0.7, 0.7, 0.7), // رمادي
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

fn default_entity_scale() -> Vec3 {
    Vec3::ONE
}

fn default_entity_rotation_deg() -> f32 {
    0.0
}

fn default_entity_sprite_size() -> Vec2 {
    Vec2::ONE
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformComponentValue {
    pub translation: Vec3,
    #[serde(default = "default_entity_rotation_deg")]
    pub rotation_deg: f32,
    #[serde(default = "default_entity_scale")]
    pub scale: Vec3,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteComponentValue {
    #[serde(default = "default_entity_sprite_size")]
    pub size: Vec2,
    pub color: Color,
    #[serde(default)]
    pub sprite_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera2DComponentValue {
    pub zoom: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityComponentKind {
    Transform,
    Sprite,
    Camera2D,
    Custom(String),
}

impl EntityComponentKind {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Transform => "Transform",
            Self::Sprite => "Sprite",
            Self::Camera2D => "Camera2D",
            Self::Custom(kind) => kind.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EntityMergePolicy {
    #[default]
    KeepExisting,
    ReplaceExisting,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityComponentValue {
    Transform(TransformComponentValue),
    Sprite(SpriteComponentValue),
    Camera2D(Camera2DComponentValue),
    Custom { kind: String, payload: JsonValue },
}

impl EntityComponentValue {
    pub fn transform(translation: Vec3) -> Self {
        Self::Transform(TransformComponentValue {
            translation,
            rotation_deg: default_entity_rotation_deg(),
            scale: default_entity_scale(),
        })
    }

    pub fn sprite(size: Vec2, color: Color) -> Self {
        Self::Sprite(SpriteComponentValue {
            size,
            color,
            sprite_id: None,
        })
    }

    pub fn sprite_with_id(size: Vec2, color: Color, sprite_id: impl Into<String>) -> Self {
        Self::Sprite(SpriteComponentValue {
            size,
            color,
            sprite_id: Some(sprite_id.into()),
        })
    }

    pub fn camera_2d(zoom: f32) -> Self {
        Self::Camera2D(Camera2DComponentValue { zoom })
    }

    pub fn custom(kind: impl Into<String>, payload: JsonValue) -> Self {
        Self::Custom {
            kind: kind.into(),
            payload,
        }
    }

    pub fn kind(&self) -> EntityComponentKind {
        match self {
            Self::Transform(_) => EntityComponentKind::Transform,
            Self::Sprite(_) => EntityComponentKind::Sprite,
            Self::Camera2D(_) => EntityComponentKind::Camera2D,
            Self::Custom { kind, .. } => EntityComponentKind::Custom(kind.clone()),
        }
    }

    pub fn kind_name(&self) -> &str {
        match self {
            Self::Transform(_) => "Transform",
            Self::Sprite(_) => "Sprite",
            Self::Camera2D(_) => "Camera2D",
            Self::Custom { kind, .. } => kind.as_str(),
        }
    }

    pub fn as_transform(&self) -> Option<&TransformComponentValue> {
        match self {
            Self::Transform(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_sprite(&self) -> Option<&SpriteComponentValue> {
        match self {
            Self::Sprite(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_camera_2d(&self) -> Option<&Camera2DComponentValue> {
        match self {
            Self::Camera2D(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityValue {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub components: Vec<EntityComponentValue>,
    #[serde(default)]
    pub children: Vec<EntityValue>,
}

impl EntityValue {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    pub fn with_component(mut self, component: EntityComponentValue) -> Self {
        self.set_component(component);
        self
    }

    pub fn with_child(mut self, child: EntityValue) -> Self {
        self.children.push(child);
        self
    }

    pub fn merge_with(&self, other: &EntityValue) -> Self {
        self.merge_with_policy(other, EntityMergePolicy::KeepExisting)
    }

    pub fn merge_with_policy(&self, other: &EntityValue, policy: EntityMergePolicy) -> Self {
        let mut merged = self.clone();
        merged.merge_in_place_with_policy(other, policy);
        merged
    }

    pub fn merge_in_place(&mut self, other: &EntityValue) {
        self.merge_in_place_with_policy(other, EntityMergePolicy::KeepExisting);
    }

    pub fn merge_in_place_with_policy(
        &mut self,
        other: &EntityValue,
        policy: EntityMergePolicy,
    ) {
        if self.name.is_none() {
            self.name = other.name.clone();
        }

        for component in &other.components {
            self.merge_component(component.clone(), policy);
        }

        self.children.extend(other.children.iter().cloned());
    }

    pub fn component(&self, kind: EntityComponentKind) -> Option<&EntityComponentValue> {
        self.components
            .iter()
            .find(|component| component.kind() == kind)
    }

    pub fn component_by_name(&self, kind: &str) -> Option<&EntityComponentValue> {
        self.components
            .iter()
            .find(|component| component.kind_name() == kind)
    }

    pub fn find_first_component(&self, kind: EntityComponentKind) -> Option<&EntityComponentValue> {
        if let Some(component) = self.component(kind.clone()) {
            return Some(component);
        }

        for child in &self.children {
            if let Some(component) = child.find_first_component(kind.clone()) {
                return Some(component);
            }
        }

        None
    }

    pub fn transform(&self) -> Option<&TransformComponentValue> {
        self.components.iter().find_map(EntityComponentValue::as_transform)
    }

    pub fn set_component(&mut self, component: EntityComponentValue) {
        if let Some(existing) = self
            .components
            .iter_mut()
            .find(|existing| existing.kind() == component.kind())
        {
            *existing = component;
        } else {
            self.components.push(component);
        }
    }

    fn merge_component(&mut self, component: EntityComponentValue, policy: EntityMergePolicy) {
        if let Some(existing) = self
            .components
            .iter_mut()
            .find(|existing| existing.kind() == component.kind())
        {
            if matches!(policy, EntityMergePolicy::ReplaceExisting) {
                *existing = component;
            }
        } else {
            self.components.push(component);
        }
    }
}

/// قيمة ديناميكية يمكن تمريرها بين العُقد
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
    None, // قيمة فارغة
}

impl Default for NodeValue {
    fn default() -> Self {
        NodeValue::None
    }
}

impl NodeValue {
    /// الحصول على نوع القيمة
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

    /// التحقق إذا كانت القيمة فارغة
    pub fn is_none(&self) -> bool {
        matches!(self, NodeValue::None)
    }

    // === استخراج القيم ===

    pub fn as_float(&self) -> Option<f64> {
        match self {
            NodeValue::Float(f) => Some(*f),
            NodeValue::Int(i) => Some(*i as f64), // تحويل تلقائي
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            NodeValue::Int(i) => Some(*i),
            NodeValue::Float(f) => Some(*f as i64), // تحويل مع فقدان
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
            NodeValue::Vec3(v) => Some(v.truncate()), // تحويل مع فقدان z
            NodeValue::Vec4(v) => Some(v.truncate().truncate()), // تحويل مع فقدان
            _ => None,
        }
    }

    pub fn as_vec3(&self) -> Option<Vec3> {
        match self {
            NodeValue::Vec3(v) => Some(*v),
            NodeValue::Vec4(v) => Some(v.truncate()),
            NodeValue::Vec2(v) => Some(v.extend(0.0)), // تمديد
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
            NodeValue::Entity(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_tagged(&self) -> Option<(&str, &JsonValue)> {
        match self {
            NodeValue::TaggedData { tag, payload } => Some((tag.as_str(), payload)),
            _ => None,
        }
    }

    // === منشئات سهلة ===

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

    /// التحقق من التوافق بين نوعين
    /// يعيد true إذا كان يمكن تحويل from إلى to
    pub fn is_compatible(from: &ValueType, to: &ValueType) -> bool {
        match (from, to) {
            // نفس النوع دائماً متوافق
            (a, b) if a == b => true,

            // Any يقبل أي شيء
            (_, ValueType::Any) => true,
            (ValueType::Any, _) => true,

            // وسوم المواد/الأجزاء يجب أن تتطابق تماماً
            (ValueType::CustomTag(from_tag), ValueType::CustomTag(to_tag)) => from_tag == to_tag,

            // تحويلات رقمية
            (ValueType::Int, ValueType::Float) => true,
            (ValueType::Float, ValueType::Int) => true,

            // تحويلات Vec
            (ValueType::Vec2, ValueType::Vec3) => true,
            (ValueType::Vec2, ValueType::Vec4) => true,
            (ValueType::Vec3, ValueType::Vec2) => true,
            (ValueType::Vec3, ValueType::Vec4) => true,
            (ValueType::Vec4, ValueType::Vec2) => true,
            (ValueType::Vec4, ValueType::Vec3) => true,

            // Color و Vec4
            (ValueType::Color, ValueType::Vec4) => true,
            (ValueType::Vec4, ValueType::Color) => true,

            // Entity values compose only with entity ports.
            (ValueType::Entity, ValueType::Entity) => true,

            _ => false,
        }
    }

    /// تمثيل نصي للقيمة (للعرض)
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
                let label = entity
                    .name
                    .clone()
                    .unwrap_or_else(|| "Entity".to_string());
                format!(
                    "{} [{} component(s), {} child(ren)]",
                    label,
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

/// حاوية لقيم العقدة (مداخل ومخارج)
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
