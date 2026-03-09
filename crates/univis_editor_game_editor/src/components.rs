use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const COMPONENT_KIND_TRANSFORM: &str = "editor/transform";
pub const COMPONENT_KIND_SPRITE: &str = "editor/sprite";
pub const COMPONENT_KIND_CAMERA2D: &str = "editor/camera2d";
pub const COMPONENT_KIND_ENTITY_ROOT: &str = "editor/entity_root";

#[derive(Debug, Clone, Reflect, Serialize, Deserialize, Default)]
pub struct EEntityRoot;

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ETransform {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub rotation_deg: f32,
    #[serde(default = "default_scale")]
    pub scale_x: f32,
    #[serde(default = "default_scale")]
    pub scale_y: f32,
}

impl Default for ETransform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation_deg: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ESprite {
    #[serde(default = "default_color_r")]
    pub color_r: f32,
    #[serde(default = "default_color_g")]
    pub color_g: f32,
    #[serde(default = "default_color_b")]
    pub color_b: f32,
    #[serde(default = "default_color_a")]
    pub color_a: f32,
    #[serde(default)]
    pub sprite_id: String,
}

impl Default for ESprite {
    fn default() -> Self {
        Self {
            color_r: 1.0,
            color_g: 1.0,
            color_b: 1.0,
            color_a: 1.0,
            sprite_id: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ECamera2D {
    #[serde(default = "default_scale")]
    pub zoom: f32,
}

impl Default for ECamera2D {
    fn default() -> Self {
        Self { zoom: 1.0 }
    }
}

fn default_scale() -> f32 {
    1.0
}

fn default_color_r() -> f32 {
    1.0
}

fn default_color_g() -> f32 {
    1.0
}

fn default_color_b() -> f32 {
    1.0
}

fn default_color_a() -> f32 {
    1.0
}

pub fn component_display_name(kind: &str) -> String {
    match kind {
        COMPONENT_KIND_ENTITY_ROOT => "Entity Root".to_string(),
        COMPONENT_KIND_TRANSFORM => "Transform".to_string(),
        COMPONENT_KIND_SPRITE => "Sprite".to_string(),
        COMPONENT_KIND_CAMERA2D => "Camera2D".to_string(),
        _ => format!("Unknown: {}", kind),
    }
}
