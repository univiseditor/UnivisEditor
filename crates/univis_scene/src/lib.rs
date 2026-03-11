use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub const TRANSFORM_COMPONENT_KEY: &str = "scene/transform";
pub const SPRITE_COMPONENT_KEY: &str = "scene/sprite";
pub const CAMERA2D_COMPONENT_KEY: &str = "scene/camera_2d";
pub const TEXT2D_COMPONENT_KEY: &str = "scene/text_2d";

pub fn component_display_name(key: &str) -> String {
    match key {
        TRANSFORM_COMPONENT_KEY => "Transform".to_string(),
        SPRITE_COMPONENT_KEY => "Sprite".to_string(),
        CAMERA2D_COMPONENT_KEY => "Camera2D".to_string(),
        TEXT2D_COMPONENT_KEY => "Text2D".to_string(),
        _ => key.rsplit('/').next().unwrap_or(key).replace(['_', '-'], " "),
    }
}

pub fn component_port_color(key: &str) -> Color {
    match key {
        TRANSFORM_COMPONENT_KEY => Color::srgb(0.34, 0.77, 0.95),
        SPRITE_COMPONENT_KEY => Color::srgb(0.94, 0.53, 0.34),
        CAMERA2D_COMPONENT_KEY => Color::srgb(0.96, 0.83, 0.32),
        TEXT2D_COMPONENT_KEY => Color::srgb(0.71, 0.60, 0.95),
        _ => custom_component_color(key),
    }
}

pub fn pure_entity_requirement_token(component_key: &str) -> String {
    format!("scene/entity:pure:{component_key}")
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text2DComponentValue {
    pub content: String,
    pub font_size: f32,
    pub color: Color,
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
    Text2D(Text2DComponentValue),
    Custom {
        #[serde(alias = "kind")]
        key: String,
        payload: JsonValue,
    },
}

impl EntityComponentValue {
    pub fn key(&self) -> &str {
        match self {
            Self::Transform(_) => TRANSFORM_COMPONENT_KEY,
            Self::Sprite(_) => SPRITE_COMPONENT_KEY,
            Self::Camera2D(_) => CAMERA2D_COMPONENT_KEY,
            Self::Text2D(_) => TEXT2D_COMPONENT_KEY,
            Self::Custom { key, .. } => key.as_str(),
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

    pub fn as_text_2d(&self) -> Option<&Text2DComponentValue> {
        match self {
            Self::Text2D(value) => Some(value),
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

    pub fn component(&self, key: &str) -> Option<&EntityComponentValue> {
        self.components.iter().find(|component| component.key() == key)
    }

    pub fn find_first_component(&self, key: &str) -> Option<&EntityComponentValue> {
        if let Some(component) = self.component(key) {
            return Some(component);
        }

        for child in &self.children {
            if let Some(component) = child.find_first_component(key) {
                return Some(component);
            }
        }

        None
    }

    pub fn transform(&self) -> Option<&TransformComponentValue> {
        self.components.iter().find_map(EntityComponentValue::as_transform)
    }

    pub fn is_pure_component(&self, key: &str) -> bool {
        self.name.is_none()
            && self.children.is_empty()
            && self.components.len() == 1
            && self
                .components
                .first()
                .map(|component| component.key() == key)
                .unwrap_or(false)
    }

    pub fn set_component(&mut self, component: EntityComponentValue) {
        if let Some(existing) = self
            .components
            .iter_mut()
            .find(|existing| existing.key() == component.key())
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
            .find(|existing| existing.key() == component.key())
        {
            if matches!(policy, EntityMergePolicy::ReplaceExisting) {
                *existing = component;
            }
        } else {
            self.components.push(component);
        }
    }
}

fn custom_component_color(key: &str) -> Color {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();

    let r = ((hash & 0xFF) as f32 / 255.0) * 0.45 + 0.35;
    let g = (((hash >> 8) & 0xFF) as f32 / 255.0) * 0.45 + 0.35;
    let b = (((hash >> 16) & 0xFF) as f32 / 255.0) * 0.45 + 0.35;

    Color::srgb(r, g, b)
}
