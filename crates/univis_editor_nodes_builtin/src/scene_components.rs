//! Craft scene components and composition nodes.

use bevy::prelude::*;
use univis_editor_core::register_node;
use univis_editor_core::node_definition::{
    GraphNode, NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_editor_core::value::{EntityValue, NodeValue, ValueType, EntityComponentValue};
use univis_ui::prelude::*;

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
        vec![PortDefinition::output_entity("Entity")]
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

        let entity = EntityValue::named("Sprite2D")
            .with_component(EntityComponentValue::transform(Vec3::new(
                x as f32, y as f32, z as f32,
            )))
            .with_component(EntityComponentValue::sprite(
                Vec2::new(width as f32, height as f32),
                Color::srgba(color.red, color.green, color.blue, color.alpha),
            ));

        ctx.set_entity(0, entity);
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
        vec![PortDefinition::output_entity("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let x = ctx.get_float_or(0, 0.0);
        let y = ctx.get_float_or(1, 0.0);
        let zoom = ctx.get_float_or(2, 1.0).clamp(0.2, 5.0);

        let entity = EntityValue::named("Camera2D")
            .with_component(EntityComponentValue::transform(Vec3::new(x as f32, y as f32, 0.0)))
            .with_component(EntityComponentValue::camera_2d(zoom as f32));

        ctx.set_entity(0, entity);
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
        Some("Compose Sprite2D + Camera2D entities into one scene root")
    }

    fn color(&self) -> Color {
        SCENE_COMPOSITION_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_entity("Sprite"),
            PortDefinition::input_entity("Camera"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_entity("Scene")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let Some(sprite_entity) = ctx.get_entity(0).cloned() else {
            return ProcessResult::MissingInput(0);
        };
        let Some(camera_entity) = ctx.get_entity(1).cloned() else {
            return ProcessResult::MissingInput(1);
        };

        let scene_root = EntityValue::named("Main Scene")
            .with_child(sprite_entity)
            .with_child(camera_entity);

        ctx.set_entity(0, scene_root);
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

pub struct MergeEntityNode;

impl NodeDefinition for MergeEntityNode {
    fn id(&self) -> NodeId {
        NodeId::new("entity/merge")
    }

    fn display_name(&self) -> &str {
        "Merge Entity"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Craft/Scene Composition")
    }

    fn description(&self) -> Option<&str> {
        Some("Keep base components, add missing components from the second entity, and append children")
    }

    fn color(&self) -> Color {
        SCENE_COMPOSITION_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_entity("Base")
                .with_description("Primary entity value that receives merged components"),
            PortDefinition::input_entity("Addition")
                .with_description("Entity value to merge into the base"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_entity("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let Some(base) = ctx.get_entity(0).cloned() else {
            return ProcessResult::MissingInput(0);
        };
        let Some(addition) = ctx.get_entity(1).cloned() else {
            return ProcessResult::MissingInput(1);
        };

        ctx.set_entity(0, base.merge_with(&addition));
        ProcessResult::Success
    }
}

pub struct AddChildNode;

impl NodeDefinition for AddChildNode {
    fn id(&self) -> NodeId {
        NodeId::new("entity/add_child")
    }

    fn display_name(&self) -> &str {
        "Add Child"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Craft/Scene Composition")
    }

    fn description(&self) -> Option<&str> {
        Some("Attach a child entity to a parent entity and return a new parent value")
    }

    fn color(&self) -> Color {
        SCENE_COMPOSITION_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_entity("Parent")
                .with_description("Entity value that will receive the child"),
            PortDefinition::input_entity("Child")
                .with_description("Entity value to append as a child"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_entity("Parent")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let Some(parent) = ctx.get_entity(0).cloned() else {
            return ProcessResult::MissingInput(0);
        };
        let Some(child) = ctx.get_entity(1).cloned() else {
            return ProcessResult::MissingInput(1);
        };

        ctx.set_entity(0, parent.with_child(child));
        ProcessResult::Success
    }
}

fn build_scene_preview_visual(node: &GraphNode) -> ScenePreviewVisual {
    let Some(NodeValue::Entity(scene_entity)) = node.values.outputs.first() else {
        return ScenePreviewVisual::default();
    };
    let Some((camera_transform, camera)) = find_first_camera(scene_entity) else {
        return ScenePreviewVisual::default();
    };
    let Some((sprite_transform, sprite)) = find_first_sprite(scene_entity) else {
        return ScenePreviewVisual::default();
    };

    ScenePreviewVisual {
        ready: true,
        camera_x: camera_transform
            .map(|transform| transform.translation.x)
            .unwrap_or(0.0),
        camera_y: camera_transform
            .map(|transform| transform.translation.y)
            .unwrap_or(0.0),
        camera_zoom: camera.zoom.max(0.2),
        sprite_x: sprite_transform
            .map(|transform| transform.translation.x)
            .unwrap_or(0.0),
        sprite_y: sprite_transform
            .map(|transform| transform.translation.y)
            .unwrap_or(0.0),
        sprite_w: sprite.size.x.max(1.0),
        sprite_h: sprite.size.y.max(1.0),
        sprite_color: sprite.color,
    }
}

fn find_first_camera(
    entity: &EntityValue,
) -> Option<(
    Option<&univis_editor_core::value::TransformComponentValue>,
    &univis_editor_core::value::Camera2DComponentValue,
)> {
    if let Some(camera) = entity
        .components
        .iter()
        .find_map(EntityComponentValue::as_camera_2d)
    {
        return Some((entity.transform(), camera));
    }

    for child in &entity.children {
        if let Some(found) = find_first_camera(child) {
            return Some(found);
        }
    }

    None
}

fn find_first_sprite(
    entity: &EntityValue,
) -> Option<(
    Option<&univis_editor_core::value::TransformComponentValue>,
    &univis_editor_core::value::SpriteComponentValue,
)> {
    if let Some(sprite) = entity
        .components
        .iter()
        .find_map(EntityComponentValue::as_sprite)
    {
        return Some((entity.transform(), sprite));
    }

    for child in &entity.children {
        if let Some(found) = find_first_sprite(child) {
            return Some(found);
        }
    }

    None
}

register_node!(Sprite2DComponentNode);
register_node!(Camera2DComponentNode);
register_node!(MergeEntityNode);
register_node!(AddChildNode);
register_node!(MainSceneComposeNode, visual = main_scene_visual_hook);
