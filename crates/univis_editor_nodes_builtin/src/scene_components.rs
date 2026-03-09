//! Craft scene components and composition nodes.

use bevy::prelude::*;
use univis_editor_core::register_node;
use serde_json::{Value as JsonValue, json};
use univis_editor_core::node_definition::{
    GraphNode, NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_editor_core::value::{NodeValue, ValueType};
use univis_ui::prelude::*;

pub const TAG_COMPONENT_SPRITE2D: &str = "component/sprite2d";
pub const TAG_COMPONENT_CAMERA2D: &str = "component/camera2d";
pub const TAG_SCENE_MAIN: &str = "scene/main";

const SCENE_COMPONENT_COLOR: Color = Color::srgb(0.3, 0.5, 0.85);
const SCENE_COMPOSITION_COLOR: Color = Color::srgb(0.22, 0.62, 0.46);
const SCENE_PREVIEW_READY: Color = Color::srgb(0.08, 0.23, 0.15);
const SCENE_PREVIEW_WAITING: Color = Color::srgb(0.22, 0.1, 0.08);

#[derive(Component)]
pub struct MainScenePreviewStatusText {
    node_entity: Entity,
}

#[derive(Component)]
pub struct MainScenePreviewCameraText {
    node_entity: Entity,
}

#[derive(Component)]
pub struct MainScenePreviewSpriteText {
    node_entity: Entity,
}

#[derive(Component)]
pub struct MainScenePreviewCanvas {
    node_entity: Entity,
}

#[derive(Component)]
pub struct MainScenePreviewSwatch {
    node_entity: Entity,
}

#[derive(Debug, Clone)]
struct ScenePreviewVisual {
    ready: bool,
    camera_x: f32,
    camera_y: f32,
    camera_zoom: f32,
    sprite_x: f32,
    sprite_y: f32,
    sprite_w: f32,
    sprite_h: f32,
    sprite_color: Color,
}

impl Default for ScenePreviewVisual {
    fn default() -> Self {
        Self {
            ready: false,
            camera_x: 0.0,
            camera_y: 0.0,
            camera_zoom: 1.0,
            sprite_x: 0.0,
            sprite_y: 0.0,
            sprite_w: 40.0,
            sprite_h: 40.0,
            sprite_color: Color::srgb(0.35, 0.35, 0.35),
        }
    }
}

pub fn main_scene_visual_hook(world: &mut World, node_entity: Entity) {
    let visual = world
        .get::<GraphNode>(node_entity)
        .map(build_scene_preview_visual)
        .unwrap_or_default();

    {
        let mut query = world.query::<(&MainScenePreviewStatusText, &mut UTextLabel)>();
        for (marker, mut text) in query.iter_mut(world) {
            if marker.node_entity != node_entity {
                continue;
            }

            if visual.ready {
                text.text = "Scene Ready".to_string();
                text.color = Color::srgb(0.55, 0.95, 0.72);
            } else {
                text.text = "Waiting: connect Sprite2D + Camera2D".to_string();
                text.color = Color::srgb(0.95, 0.68, 0.52);
            }
        }
    }

    {
        let mut query = world.query::<(&MainScenePreviewCameraText, &mut UTextLabel)>();
        for (marker, mut text) in query.iter_mut(world) {
            if marker.node_entity != node_entity {
                continue;
            }

            text.text = format!(
                "Camera  x:{:.0}  y:{:.0}  zoom:{:.2}",
                visual.camera_x, visual.camera_y, visual.camera_zoom
            );
            text.color = if visual.ready {
                Color::srgb(0.82, 0.92, 1.0)
            } else {
                Color::srgb(0.62, 0.7, 0.76)
            };
        }
    }

    {
        let mut query = world.query::<(&MainScenePreviewSpriteText, &mut UTextLabel)>();
        for (marker, mut text) in query.iter_mut(world) {
            if marker.node_entity != node_entity {
                continue;
            }

            text.text = format!(
                "Sprite  x:{:.0}  y:{:.0}  size:{:.0}x{:.0}",
                visual.sprite_x, visual.sprite_y, visual.sprite_w, visual.sprite_h
            );
            text.color = if visual.ready {
                Color::srgb(0.86, 1.0, 0.86)
            } else {
                Color::srgb(0.62, 0.76, 0.62)
            };
        }
    }

    {
        let mut query = world.query::<(&MainScenePreviewCanvas, &mut UNode)>();
        for (marker, mut canvas) in query.iter_mut(world) {
            if marker.node_entity != node_entity {
                continue;
            }

            canvas.background_color = if visual.ready {
                SCENE_PREVIEW_READY
            } else {
                SCENE_PREVIEW_WAITING
            };
        }
    }

    {
        let mut query = world.query::<(&MainScenePreviewSwatch, &mut UNode)>();
        for (marker, mut swatch) in query.iter_mut(world) {
            if marker.node_entity != node_entity {
                continue;
            }

            let sw = (visual.sprite_w * 0.28).clamp(14.0, 80.0);
            let sh = (visual.sprite_h * 0.28).clamp(14.0, 80.0);
            swatch.width = UVal::Px(sw);
            swatch.height = UVal::Px(sh);
            swatch.background_color = visual.sprite_color;
        }
    }
}

pub struct Sprite2DComponentNode;

impl NodeDefinition for Sprite2DComponentNode {
    fn id(&self) -> NodeId {
        NodeId::new("component/sprite2d")
    }

    fn display_name(&self) -> &str {
        "Sprite2D"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Craft/Scene Components")
    }

    fn description(&self) -> Option<&str> {
        Some("Independent sprite component payload")
    }

    fn color(&self) -> Color {
        SCENE_COMPONENT_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("X")
                .with_default(NodeValue::float(0.0))
                .editable_in_popup()
                .with_ui_step(10.0)
                .with_ui_range(-5000.0, 5000.0),
            PortDefinition::input_float("Y")
                .with_default(NodeValue::float(0.0))
                .editable_in_popup()
                .with_ui_step(10.0)
                .with_ui_range(-5000.0, 5000.0),
            PortDefinition::input_float("Z")
                .with_default(NodeValue::float(-20.0))
                .editable_in_popup()
                .with_ui_step(1.0)
                .with_ui_range(-1000.0, 1000.0),
            PortDefinition::input_float("Width")
                .with_default(NodeValue::float(120.0))
                .editable_in_popup()
                .with_ui_step(5.0)
                .with_ui_range(1.0, 2000.0),
            PortDefinition::input_float("Height")
                .with_default(NodeValue::float(120.0))
                .editable_in_popup()
                .with_ui_step(5.0)
                .with_ui_range(1.0, 2000.0),
            PortDefinition::new("Color", ValueType::Color)
                .with_default(NodeValue::color(0.18, 0.8, 0.45, 1.0))
                .editable_in_popup()
                .with_ui_step(0.05)
                .with_ui_range(0.0, 1.0),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_tag(
            "Sprite2D",
            TAG_COMPONENT_SPRITE2D,
        )]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let x = ctx.get_float_or(0, 0.0);
        let y = ctx.get_float_or(1, 0.0);
        let z = ctx.get_float_or(2, -20.0);
        let width = ctx.get_float_or(3, 120.0).max(1.0);
        let height = ctx.get_float_or(4, 120.0).max(1.0);
        let color = ctx
            .inputs
            .get(5)
            .and_then(NodeValue::as_color)
            .unwrap_or_else(|| Color::srgba(0.18, 0.8, 0.45, 1.0))
            .to_srgba();

        ctx.set_tagged(
            0,
            TAG_COMPONENT_SPRITE2D,
            json!({
                "kind": TAG_COMPONENT_SPRITE2D,
                "transform": {
                    "x": x,
                    "y": y,
                    "z": z
                },
                "size": {
                    "x": width,
                    "y": height
                },
                "color": [color.red, color.green, color.blue, color.alpha]
            }),
        );
        ProcessResult::Success
    }
}

pub struct Camera2DComponentNode;

impl NodeDefinition for Camera2DComponentNode {
    fn id(&self) -> NodeId {
        NodeId::new("component/camera2d")
    }

    fn display_name(&self) -> &str {
        "Camera2D"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Craft/Scene Components")
    }

    fn description(&self) -> Option<&str> {
        Some("Independent camera component payload")
    }

    fn color(&self) -> Color {
        SCENE_COMPONENT_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("X")
                .with_default(NodeValue::float(0.0))
                .editable_in_popup()
                .with_ui_step(10.0)
                .with_ui_range(-5000.0, 5000.0),
            PortDefinition::input_float("Y")
                .with_default(NodeValue::float(0.0))
                .editable_in_popup()
                .with_ui_step(10.0)
                .with_ui_range(-5000.0, 5000.0),
            PortDefinition::input_float("Zoom")
                .with_default(NodeValue::float(1.0))
                .editable_in_popup()
                .with_ui_step(0.1)
                .with_ui_range(0.2, 5.0),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_tag(
            "Camera2D",
            TAG_COMPONENT_CAMERA2D,
        )]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let x = ctx.get_float_or(0, 0.0);
        let y = ctx.get_float_or(1, 0.0);
        let zoom = ctx.get_float_or(2, 1.0).clamp(0.2, 5.0);

        ctx.set_tagged(
            0,
            TAG_COMPONENT_CAMERA2D,
            json!({
                "kind": TAG_COMPONENT_CAMERA2D,
                "transform": {
                    "x": x,
                    "y": y
                },
                "zoom": zoom
            }),
        );
        ProcessResult::Success
    }
}

pub struct MainSceneComposeNode;

impl NodeDefinition for MainSceneComposeNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/main")
    }

    fn display_name(&self) -> &str {
        "Main Scene"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Craft/Scene Composition")
    }

    fn description(&self) -> Option<&str> {
        Some("Compose Sprite2D + Camera2D into one scene payload")
    }

    fn color(&self) -> Color {
        SCENE_COMPOSITION_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_tag("Sprite", TAG_COMPONENT_SPRITE2D),
            PortDefinition::input_tag("Camera", TAG_COMPONENT_CAMERA2D),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_tag("Scene", TAG_SCENE_MAIN)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let Some((sprite_tag, sprite_payload)) = ctx.get_tagged(0) else {
            return ProcessResult::MissingInput(0);
        };
        let Some((camera_tag, camera_payload)) = ctx.get_tagged(1) else {
            return ProcessResult::MissingInput(1);
        };

        if sprite_tag != TAG_COMPONENT_SPRITE2D {
            return ProcessResult::Error(format!(
                "Expected {}, got {}",
                TAG_COMPONENT_SPRITE2D, sprite_tag
            ));
        }
        if camera_tag != TAG_COMPONENT_CAMERA2D {
            return ProcessResult::Error(format!(
                "Expected {}, got {}",
                TAG_COMPONENT_CAMERA2D, camera_tag
            ));
        }

        ctx.set_tagged(
            0,
            TAG_SCENE_MAIN,
            json!({
                "kind": TAG_SCENE_MAIN,
                "sprite": sprite_payload,
                "camera": camera_payload
            }),
        );
        ProcessResult::Success
    }

    fn has_custom_body(&self) -> bool {
        true
    }

    fn build_body(&self, body: &mut ChildSpawnerCommands, node_entity: Entity) {
        body.spawn((
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(150.0),
                padding: USides::all(8.0),
                background_color: SCENE_PREVIEW_WAITING,
                border_radius: UCornerRadius::all(6.0),
                ..default()
            },
            ULayout {
                flex_direction: UFlexDirection::Column,
                gap: 5.0,
                ..default()
            },
            MainScenePreviewCanvas { node_entity },
        ))
        .with_children(|panel| {
            panel.spawn((
                MainScenePreviewStatusText { node_entity },
                UNode {
                    width: UVal::Percent(1.0),
                    ..default()
                },
                UTextLabel {
                    text: "Waiting: connect Sprite2D + Camera2D".to_string(),
                    font_size: 10.0,
                    color: Color::srgb(0.95, 0.68, 0.52),
                    autosize: false,
                    ..default()
                },
            ));

            panel
                .spawn((
                    UNode {
                        width: UVal::Percent(1.0),
                        height: UVal::Px(68.0),
                        padding: USides::all(8.0),
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.22),
                        border_radius: UCornerRadius::all(6.0),
                        ..default()
                    },
                    ULayout {
                        justify_content: UJustifyContent::Center,
                        align_items: UAlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|preview| {
                    preview.spawn((
                        MainScenePreviewSwatch { node_entity },
                        UNode {
                            width: UVal::Px(26.0),
                            height: UVal::Px(26.0),
                            background_color: Color::srgb(0.35, 0.35, 0.35),
                            border_radius: UCornerRadius::all(5.0),
                            ..default()
                        },
                    ));
                });

            panel.spawn((
                MainScenePreviewCameraText { node_entity },
                UNode {
                    width: UVal::Percent(1.0),
                    ..default()
                },
                UTextLabel {
                    text: "Camera  x:0  y:0  zoom:1.00".to_string(),
                    font_size: 10.0,
                    color: Color::srgb(0.62, 0.7, 0.76),
                    autosize: false,
                    ..default()
                },
            ));

            panel.spawn((
                MainScenePreviewSpriteText { node_entity },
                UNode {
                    width: UVal::Percent(1.0),
                    ..default()
                },
                UTextLabel {
                    text: "Sprite  x:0  y:0  size:0x0".to_string(),
                    font_size: 10.0,
                    color: Color::srgb(0.62, 0.76, 0.62),
                    autosize: false,
                    ..default()
                },
            ));
        });
    }
}

fn build_scene_preview_visual(node: &GraphNode) -> ScenePreviewVisual {
    let Some(NodeValue::TaggedData { tag, payload }) = node.values.outputs.first() else {
        return ScenePreviewVisual::default();
    };
    if tag != TAG_SCENE_MAIN {
        return ScenePreviewVisual::default();
    }

    let Some(camera) = payload.get("camera") else {
        return ScenePreviewVisual::default();
    };
    let Some(sprite) = payload.get("sprite") else {
        return ScenePreviewVisual::default();
    };

    ScenePreviewVisual {
        ready: true,
        camera_x: json_path_f32(camera, &["transform", "x"], 0.0),
        camera_y: json_path_f32(camera, &["transform", "y"], 0.0),
        camera_zoom: json_path_f32(camera, &["zoom"], 1.0),
        sprite_x: json_path_f32(sprite, &["transform", "x"], 0.0),
        sprite_y: json_path_f32(sprite, &["transform", "y"], 0.0),
        sprite_w: json_path_f32(sprite, &["size", "x"], 40.0).max(1.0),
        sprite_h: json_path_f32(sprite, &["size", "y"], 40.0).max(1.0),
        sprite_color: json_color(sprite.get("color"))
            .unwrap_or_else(|| Color::srgb(0.35, 0.35, 0.35)),
    }
}

fn json_path_f32(value: &JsonValue, path: &[&str], default: f32) -> f32 {
    let mut current = value;
    for key in path {
        let Some(next) = current.get(*key) else {
            return default;
        };
        current = next;
    }

    current.as_f64().map(|v| v as f32).unwrap_or(default)
}

fn json_color(value: Option<&JsonValue>) -> Option<Color> {
    let arr = value?.as_array()?;
    if arr.len() < 4 {
        return None;
    }

    let r = arr[0].as_f64()? as f32;
    let g = arr[1].as_f64()? as f32;
    let b = arr[2].as_f64()? as f32;
    let a = arr[3].as_f64()? as f32;
    Some(Color::srgba(r, g, b, a))
}

register_node!(Sprite2DComponentNode);
register_node!(Camera2DComponentNode);
register_node!(MainSceneComposeNode, visual = main_scene_visual_hook);
