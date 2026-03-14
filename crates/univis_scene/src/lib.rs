use bevy::prelude::*;
use bevy::sprite::Anchor;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub const TRANSFORM_COMPONENT_KEY: &str = "scene/transform";
pub const SPRITE_COMPONENT_KEY: &str = "scene/sprite";
pub const CAMERA2D_COMPONENT_KEY: &str = "scene/camera_2d";
pub const TEXT2D_COMPONENT_KEY: &str = "scene/text_2d";
pub const VISIBILITY_COMPONENT_KEY: &str = "scene/visibility";
pub const ANCHOR_COMPONENT_KEY: &str = "scene/anchor";

pub fn component_display_name(key: &str) -> String {
    match key {
        TRANSFORM_COMPONENT_KEY => "Transform".to_string(),
        SPRITE_COMPONENT_KEY => "Sprite".to_string(),
        CAMERA2D_COMPONENT_KEY => "Camera2D".to_string(),
        TEXT2D_COMPONENT_KEY => "Text2D".to_string(),
        VISIBILITY_COMPONENT_KEY => "Visibility".to_string(),
        ANCHOR_COMPONENT_KEY => "Anchor".to_string(),
        _ => key
            .rsplit('/')
            .next()
            .unwrap_or(key)
            .replace(['_', '-'], " "),
    }
}

pub fn component_port_color(key: &str) -> Color {
    match key {
        TRANSFORM_COMPONENT_KEY => Color::srgb(0.34, 0.77, 0.95),
        SPRITE_COMPONENT_KEY => Color::srgb(0.94, 0.53, 0.34),
        CAMERA2D_COMPONENT_KEY => Color::srgb(0.96, 0.83, 0.32),
        TEXT2D_COMPONENT_KEY => Color::srgb(0.71, 0.60, 0.95),
        VISIBILITY_COMPONENT_KEY => Color::srgb(0.45, 0.92, 0.58),
        ANCHOR_COMPONENT_KEY => Color::srgb(0.84, 0.68, 0.28),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisibilityComponentValue {
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnchorComponentValue {
    pub position: Vec2,
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
    Visibility(VisibilityComponentValue),
    Anchor(AnchorComponentValue),
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
            Self::Visibility(_) => VISIBILITY_COMPONENT_KEY,
            Self::Anchor(_) => ANCHOR_COMPONENT_KEY,
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

    pub fn as_visibility(&self) -> Option<&VisibilityComponentValue> {
        match self {
            Self::Visibility(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_anchor(&self) -> Option<&AnchorComponentValue> {
        match self {
            Self::Anchor(value) => Some(value),
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

#[derive(Debug, Clone, Default)]
pub struct EntitySpawnOptions {
    pub name_prefix: Option<String>,
}

impl EntitySpawnOptions {
    pub fn with_name_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.name_prefix = Some(prefix.into());
        self
    }
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

    pub fn merge_in_place_with_policy(&mut self, other: &EntityValue, policy: EntityMergePolicy) {
        if self.name.is_none() {
            self.name = other.name.clone();
        }

        for component in &other.components {
            self.merge_component(component.clone(), policy);
        }

        self.children.extend(other.children.iter().cloned());
    }

    pub fn component(&self, key: &str) -> Option<&EntityComponentValue> {
        self.components
            .iter()
            .find(|component| component.key() == key)
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
        self.components
            .iter()
            .find_map(EntityComponentValue::as_transform)
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SceneDocument {
    #[serde(default)]
    pub root: EntityValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SceneStats {
    pub entity_count: usize,
    pub named_entity_count: usize,
    pub sprite_count: usize,
    pub text_count: usize,
    pub camera_count: usize,
    pub max_depth: usize,
}

impl SceneDocument {
    pub fn new(root: EntityValue) -> Self {
        Self { root }
    }

    pub fn from_entity_value(root: EntityValue) -> Self {
        Self::new(root)
    }

    pub fn stats(&self) -> SceneStats {
        fn visit(entity: &EntityValue, depth: usize, stats: &mut SceneStats) {
            stats.entity_count += 1;
            stats.max_depth = stats.max_depth.max(depth);

            if entity.name.as_deref().is_some_and(|name| !name.is_empty()) {
                stats.named_entity_count += 1;
            }
            if entity.component(SPRITE_COMPONENT_KEY).is_some() {
                stats.sprite_count += 1;
            }
            if entity.component(TEXT2D_COMPONENT_KEY).is_some() {
                stats.text_count += 1;
            }
            if entity.component(CAMERA2D_COMPONENT_KEY).is_some() {
                stats.camera_count += 1;
            }

            for child in &entity.children {
                visit(child, depth + 1, stats);
            }
        }

        let mut stats = SceneStats::default();
        visit(&self.root, 1, &mut stats);
        stats
    }

    pub fn root_name(&self) -> Option<&str> {
        self.root.name.as_deref().filter(|name| !name.is_empty())
    }

    pub fn child_names(&self, limit: usize) -> Vec<String> {
        self.root
            .children
            .iter()
            .take(limit)
            .map(|child| {
                child
                    .name
                    .clone()
                    .unwrap_or_else(|| "<unnamed>".to_string())
            })
            .collect()
    }

    pub fn signature(&self) -> String {
        format!("{:?}", self.root)
    }
}

pub fn entity_value_signature(entity: Option<&EntityValue>) -> Option<String> {
    entity.map(|entity| format!("{entity:?}"))
}

pub fn scene_document_signature(scene: Option<&SceneDocument>) -> Option<String> {
    scene.map(SceneDocument::signature)
}

pub fn scene_transform_to_bevy(transform: &TransformComponentValue) -> Transform {
    Transform {
        translation: transform.translation,
        rotation: Quat::from_rotation_z(transform.rotation_deg.to_radians()),
        scale: transform.scale,
    }
}

pub fn spawn_entity_value_recursive(
    parent: &mut ChildSpawnerCommands,
    entity_value: &EntityValue,
    options: &EntitySpawnOptions,
) {
    let transform = entity_value
        .component(TRANSFORM_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_transform)
        .map(scene_transform_to_bevy)
        .unwrap_or_default();
    let sprite = entity_value
        .component(SPRITE_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_sprite)
        .cloned();
    let text = entity_value
        .component(TEXT2D_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_text_2d)
        .cloned();
    let visibility = entity_value
        .component(VISIBILITY_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_visibility)
        .cloned();
    let anchor = entity_value
        .component(ANCHOR_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_anchor)
        .cloned();
    let _has_camera = entity_value.component(CAMERA2D_COMPONENT_KEY).is_some();

    let mut entity_commands = parent.spawn((
        transform,
        if visibility.as_ref().is_some_and(|value| !value.visible) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        },
    ));

    if let Some(name) = entity_value.name.as_deref().filter(|name| !name.is_empty()) {
        let name = if let Some(prefix) = options.name_prefix.as_deref() {
            format!("{prefix}{name}")
        } else {
            name.to_string()
        };
        entity_commands.insert(Name::new(name));
    }

    if let Some(sprite) = sprite {
        entity_commands.insert(Sprite::from_color(sprite.color, sprite.size));
    }

    if let Some(text) = text {
        entity_commands.insert((
            Text2d::new(text.content),
            TextFont::from_font_size(text.font_size),
            TextColor(text.color),
        ));
    }

    if let Some(anchor) = anchor {
        entity_commands.insert(Anchor(anchor.position));
    }

    // Cameras are currently kept inert during graph materialization because the editor
    // still assumes a single active Camera2d for interaction and overlays.

    let children = entity_value.children.clone();
    entity_commands.with_children(|next_parent| {
        for child in &children {
            spawn_entity_value_recursive(next_parent, child, options);
        }
    });
}

pub fn spawn_scene_document_recursive(
    parent: &mut ChildSpawnerCommands,
    scene: &SceneDocument,
    options: &EntitySpawnOptions,
) {
    spawn_entity_value_recursive(parent, &scene.root, options);
}

pub fn spawn_entity_value_recursive_world(
    parent: &mut ChildSpawner,
    entity_value: &EntityValue,
    options: &EntitySpawnOptions,
) {
    let transform = entity_value
        .component(TRANSFORM_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_transform)
        .map(scene_transform_to_bevy)
        .unwrap_or_default();
    let sprite = entity_value
        .component(SPRITE_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_sprite)
        .cloned();
    let text = entity_value
        .component(TEXT2D_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_text_2d)
        .cloned();
    let visibility = entity_value
        .component(VISIBILITY_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_visibility)
        .cloned();
    let anchor = entity_value
        .component(ANCHOR_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_anchor)
        .cloned();
    let _has_camera = entity_value.component(CAMERA2D_COMPONENT_KEY).is_some();

    let mut entity_commands = parent.spawn((
        transform,
        if visibility.as_ref().is_some_and(|value| !value.visible) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        },
    ));

    if let Some(name) = entity_value.name.as_deref().filter(|name| !name.is_empty()) {
        let name = if let Some(prefix) = options.name_prefix.as_deref() {
            format!("{prefix}{name}")
        } else {
            name.to_string()
        };
        entity_commands.insert(Name::new(name));
    }

    if let Some(sprite) = sprite {
        entity_commands.insert(Sprite::from_color(sprite.color, sprite.size));
    }

    if let Some(text) = text {
        entity_commands.insert((
            Text2d::new(text.content),
            TextFont::from_font_size(text.font_size),
            TextColor(text.color),
        ));
    }

    if let Some(anchor) = anchor {
        entity_commands.insert(Anchor(anchor.position));
    }

    let children = entity_value.children.clone();
    entity_commands.with_children(|next_parent| {
        for child in &children {
            spawn_entity_value_recursive_world(next_parent, child, options);
        }
    });
}

pub fn spawn_scene_document_recursive_world(
    parent: &mut ChildSpawner,
    scene: &SceneDocument,
    options: &EntitySpawnOptions,
) {
    spawn_entity_value_recursive_world(parent, &scene.root, options);
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
