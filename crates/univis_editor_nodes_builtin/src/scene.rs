//! Graph-native scene nodes built on top of `EntityValue`.
use std::any::Any;

use crate::scene_support::{
    apply_resolved_transform, base_entity_or_empty, entity_extension_input, fallback_transform,
    finish_entity_process, missing_entity_input, popup_bool, popup_color, popup_float,
    popup_string, pure_entity_requirement_token, resolve_transform, scene_entity_input,
    scene_entity_output, transform_fallback_inputs, transform_override_input,
};
use bevy::prelude::*;
use univis_node_graph::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_node_graph::register_node;
use univis_node_graph::value::NodeValue;
use univis_scene::{
    ANCHOR_COMPONENT_KEY, AnchorComponentValue, CAMERA2D_COMPONENT_KEY, Camera2DComponentValue,
    EntityComponentValue, EntityValue, SPRITE_COMPONENT_KEY, SpriteComponentValue,
    TEXT2D_COMPONENT_KEY, TRANSFORM_COMPONENT_KEY, Text2DComponentValue, VISIBILITY_COMPONENT_KEY,
    VisibilityComponentValue,
};

const SCENE_NODE_COLOR: Color = Color::srgb(0.72, 0.55, 0.24);

fn scene_category() -> NodeCategory {
    NodeCategory::new(NodeCategory::SCENE)
}

fn require_scene_sink_input(ctx: &mut ProcessContext) -> ProcessResult {
    if ctx.get_entity(0).is_some() {
        ProcessResult::Success
    } else {
        ProcessResult::MissingInput(0)
    }
}

fn read_prefab_instance_root(
    custom_data: &mut Option<Box<dyn Any + Send + Sync>>,
) -> Option<EntityValue> {
    custom_data
        .as_deref_mut()
        .and_then(|data| data.downcast_mut::<EntityValue>())
        .cloned()
}

pub struct TransformNode;

impl NodeDefinition for TransformNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/transform")
    }

    fn display_name(&self) -> &str {
        "Transform"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Builds an EntityValue containing a Transform component")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        10
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "position", "scale", "rotation"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        let mut inputs = vec![entity_extension_input()];
        inputs.extend(transform_fallback_inputs());
        inputs
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        connected_inputs: &[bool],
    ) -> Option<String> {
        if output_index == 0 && !connected_inputs.first().copied().unwrap_or(false) {
            Some(pure_entity_requirement_token(TRANSFORM_COMPONENT_KEY))
        } else {
            None
        }
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        if entity.component(TRANSFORM_COMPONENT_KEY).is_none() {
            entity.set_component(EntityComponentValue::Transform(fallback_transform(
                ctx.get_float_or(1, 0.0),
                ctx.get_float_or(2, 0.0),
                ctx.get_float_or(3, 0.0),
                ctx.get_float_or(4, 0.0),
                ctx.get_float_or(5, 1.0),
                ctx.get_float_or(6, 1.0),
                ctx.get_float_or(7, 1.0),
            )));
        }

        finish_entity_process(ctx, entity, None)
    }
}

pub struct VisibilityNode;

impl NodeDefinition for VisibilityNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/visibility")
    }

    fn display_name(&self) -> &str {
        "Visibility"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Builds or overrides an EntityValue visibility component")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        16
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "visible", "hidden", "show", "hide"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![entity_extension_input(), popup_bool("Visible", true)]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        connected_inputs: &[bool],
    ) -> Option<String> {
        if output_index == 0 && !connected_inputs.first().copied().unwrap_or(false) {
            Some(pure_entity_requirement_token(VISIBILITY_COMPONENT_KEY))
        } else {
            None
        }
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        entity.set_component(EntityComponentValue::Visibility(VisibilityComponentValue {
            visible: ctx.get_bool_or(1, true),
        }));

        finish_entity_process(ctx, entity, None)
    }
}

pub struct AnchorNode;

impl NodeDefinition for AnchorNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/anchor")
    }

    fn display_name(&self) -> &str {
        "Anchor"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Builds or overrides an EntityValue anchor component")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        18
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "anchor", "pivot", "origin"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            entity_extension_input(),
            popup_float("Ax", 0.0, 0.1, Some(-1.0), Some(1.0)),
            popup_float("Ay", 0.0, 0.1, Some(-1.0), Some(1.0)),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        connected_inputs: &[bool],
    ) -> Option<String> {
        if output_index == 0 && !connected_inputs.first().copied().unwrap_or(false) {
            Some(pure_entity_requirement_token(ANCHOR_COMPONENT_KEY))
        } else {
            None
        }
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        entity.set_component(EntityComponentValue::Anchor(AnchorComponentValue {
            position: Vec2::new(
                ctx.get_float_or(1, 0.0) as f32,
                ctx.get_float_or(2, 0.0) as f32,
            ),
        }));

        finish_entity_process(ctx, entity, None)
    }
}

pub struct PrefabInstanceNode;

impl NodeDefinition for PrefabInstanceNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/prefab_instance")
    }

    fn display_name(&self) -> &str {
        "Prefab Instance"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Resolves a saved prefab from the current graph document into an EntityValue")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        19
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "prefab", "instance", "asset"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![popup_string("Prefab Id", "prefab_1")]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let requested_id = ctx.get_string(0).map(str::trim).unwrap_or("").to_string();
        if requested_id.is_empty() {
            ctx.set(0, NodeValue::None);
            return ProcessResult::MissingInput(0);
        }

        let Some(root) = read_prefab_instance_root(ctx.custom_data) else {
            ctx.set(0, NodeValue::None);
            return ProcessResult::Error(format!(
                "Prefab '{}' is not available in the current graph document.",
                requested_id
            ));
        };

        ctx.set_entity(0, root);
        ProcessResult::Success
    }
}

pub struct SpriteNode;

impl NodeDefinition for SpriteNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/sprite")
    }

    fn display_name(&self) -> &str {
        "Sprite"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Builds an EntityValue containing a Sprite component")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        20
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "sprite", "2d", "texture"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        let mut inputs = vec![
            entity_extension_input(),
            transform_override_input(),
            popup_float("Width", 1.0, 0.1, Some(0.0), None),
            popup_float("Height", 1.0, 0.1, Some(0.0), None),
            popup_color("Color", Color::WHITE),
            popup_string("Sprite Id", ""),
        ];
        inputs.extend(transform_fallback_inputs());
        inputs
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        let (transform, warning) = resolve_transform(ctx, &entity, 1, 6, "Transform");
        apply_resolved_transform(&mut entity, &transform);

        if entity.component(SPRITE_COMPONENT_KEY).is_none() {
            let sprite_id = ctx
                .get_string(5)
                .map(str::trim)
                .filter(|value| !value.is_empty());
            entity.set_component(EntityComponentValue::Sprite(SpriteComponentValue {
                size: Vec2::new(
                    ctx.get_float_or(2, 1.0) as f32,
                    ctx.get_float_or(3, 1.0) as f32,
                ),
                color: ctx
                    .inputs
                    .get(4)
                    .and_then(NodeValue::as_color)
                    .unwrap_or(Color::WHITE),
                sprite_id: sprite_id.map(ToOwned::to_owned),
            }));
        }

        finish_entity_process(ctx, entity, warning)
    }
}

pub struct Camera2DNode;

impl NodeDefinition for Camera2DNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/camera_2d")
    }

    fn display_name(&self) -> &str {
        "Camera 2D"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Builds an EntityValue containing a Camera2D component")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        30
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "camera", "2d", "zoom"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        let mut inputs = vec![
            entity_extension_input(),
            transform_override_input(),
            popup_float("Zoom", 1.0, 0.1, Some(0.01), None),
        ];
        inputs.extend(transform_fallback_inputs());
        inputs
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        let (transform, warning) = resolve_transform(ctx, &entity, 1, 3, "Transform");
        apply_resolved_transform(&mut entity, &transform);

        if entity.component(CAMERA2D_COMPONENT_KEY).is_none() {
            entity.set_component(EntityComponentValue::Camera2D(Camera2DComponentValue {
                zoom: ctx.get_float_or(2, 1.0) as f32,
            }));
        }

        finish_entity_process(ctx, entity, warning)
    }
}

pub struct ScaleNode;

impl NodeDefinition for ScaleNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/scale")
    }

    fn display_name(&self) -> &str {
        "Scale"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Applies scale values onto an EntityValue Transform")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        32
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "scale", "transform", "resize"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        let mut inputs = vec![
            entity_extension_input(),
            transform_override_input(),
            popup_float("Sx", 1.0, 0.1, Some(0.0), None),
            popup_float("Sy", 1.0, 0.1, Some(0.0), None),
            popup_float("Sz", 1.0, 0.1, Some(0.0), None),
        ];
        inputs.extend(transform_fallback_inputs());
        inputs
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        connected_inputs: &[bool],
    ) -> Option<String> {
        if output_index == 0 && !connected_inputs.first().copied().unwrap_or(false) {
            Some(pure_entity_requirement_token(TRANSFORM_COMPONENT_KEY))
        } else {
            None
        }
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        let (mut transform, warning) = resolve_transform(ctx, &entity, 1, 5, "Transform");
        transform.value.scale = Vec3::new(
            ctx.get_float_or(2, 1.0) as f32,
            ctx.get_float_or(3, 1.0) as f32,
            ctx.get_float_or(4, 1.0) as f32,
        );
        apply_resolved_transform(&mut entity, &transform);

        finish_entity_process(ctx, entity, warning)
    }
}

pub struct RotationNode;

impl NodeDefinition for RotationNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/rotation")
    }

    fn display_name(&self) -> &str {
        "Rotation"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Applies a rotation override onto an EntityValue Transform")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        34
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "rotation", "angle", "transform"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        let mut inputs = vec![
            entity_extension_input(),
            transform_override_input(),
            popup_float("Degrees", 0.0, 1.0, None, None),
        ];
        inputs.extend(transform_fallback_inputs());
        inputs
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        connected_inputs: &[bool],
    ) -> Option<String> {
        if output_index == 0 && !connected_inputs.first().copied().unwrap_or(false) {
            Some(pure_entity_requirement_token(TRANSFORM_COMPONENT_KEY))
        } else {
            None
        }
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        let (mut transform, warning) = resolve_transform(ctx, &entity, 1, 3, "Transform");
        transform.value.rotation_deg = ctx.get_float_or(2, 0.0) as f32;
        apply_resolved_transform(&mut entity, &transform);

        finish_entity_process(ctx, entity, warning)
    }
}

pub struct ZOrderNode;

impl NodeDefinition for ZOrderNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/z_order")
    }

    fn display_name(&self) -> &str {
        "Z Order"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Applies a Z translation override onto an EntityValue Transform")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        36
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "z", "layer", "order", "depth"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        let mut inputs = vec![
            entity_extension_input(),
            transform_override_input(),
            popup_float("Z", 0.0, 0.1, None, None),
        ];
        inputs.extend(transform_fallback_inputs());
        inputs
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn output_requirement_token(
        &self,
        output_index: usize,
        connected_inputs: &[bool],
    ) -> Option<String> {
        if output_index == 0 && !connected_inputs.first().copied().unwrap_or(false) {
            Some(pure_entity_requirement_token(TRANSFORM_COMPONENT_KEY))
        } else {
            None
        }
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        let (mut transform, warning) = resolve_transform(ctx, &entity, 1, 3, "Transform");
        transform.value.translation.z = ctx.get_float_or(2, 0.0) as f32;
        apply_resolved_transform(&mut entity, &transform);

        finish_entity_process(ctx, entity, warning)
    }
}

pub struct MergeEntityNode;

impl NodeDefinition for MergeEntityNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/merge_entity")
    }

    fn display_name(&self) -> &str {
        "Merge Entity"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Merges two EntityValue inputs while keeping existing components")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        40
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "merge", "compose"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            scene_entity_input("Left").with_description("Base entity"),
            scene_entity_input("Right").with_description("Entity to merge into Left"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let Some(left) = ctx.get_entity(0) else {
            return missing_entity_input(ctx, 0);
        };
        let Some(right) = ctx.get_entity(1) else {
            return missing_entity_input(ctx, 1);
        };

        finish_entity_process(ctx, left.merge_with(&right), None)
    }
}

pub struct AddChildNode;

impl NodeDefinition for AddChildNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/add_child")
    }

    fn display_name(&self) -> &str {
        "Add Child"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Attaches a child EntityValue to a parent EntityValue")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        50
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "child", "parent", "hierarchy"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            scene_entity_input("Parent").with_description("Parent entity"),
            scene_entity_input("Child").with_description("Child entity"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let Some(mut parent) = ctx.get_entity(0) else {
            return missing_entity_input(ctx, 0);
        };
        let Some(child) = ctx.get_entity(1) else {
            return missing_entity_input(ctx, 1);
        };

        parent.children.push(child);
        finish_entity_process(ctx, parent, None)
    }
}

pub struct GroupNode;

impl NodeDefinition for GroupNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/group")
    }

    fn display_name(&self) -> &str {
        "Group"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Builds a group entity with optional transform and up to two direct children")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        55
    }

    fn keywords(&self) -> Vec<&str> {
        vec![
            "scene",
            "entity",
            "group",
            "container",
            "parent",
            "children",
        ]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        let mut inputs = vec![
            entity_extension_input(),
            transform_override_input(),
            scene_entity_input("Child A").with_description("First child entity"),
            scene_entity_input("Child B").with_description("Optional second child entity"),
            popup_string("Name", "Group"),
        ];
        inputs.extend(transform_fallback_inputs());
        inputs
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        let (transform, warning) = resolve_transform(ctx, &entity, 1, 5, "Transform");
        apply_resolved_transform(&mut entity, &transform);

        if entity.name.is_none() {
            let name = ctx.get_string(4).map(str::trim).unwrap_or("");
            if !name.is_empty() {
                entity.name = Some(name.to_string());
            }
        }

        if let Some(child) = ctx.get_entity(2) {
            entity.children.push(child);
        }
        if let Some(child) = ctx.get_entity(3) {
            entity.children.push(child);
        }

        finish_entity_process(ctx, entity, warning)
    }
}

pub struct NameNode;

impl NodeDefinition for NameNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/name")
    }

    fn display_name(&self) -> &str {
        "Name"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Assigns a graph-native name to an entity if it does not already have one")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        15
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "name", "label"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![entity_extension_input(), popup_string("Name", "Entity")]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        if entity.name.is_none() {
            let name = ctx.get_string(1).map(str::trim).unwrap_or("");
            if !name.is_empty() {
                entity.name = Some(name.to_string());
            }
        }

        finish_entity_process(ctx, entity, None)
    }
}

pub struct TextNode;

impl NodeDefinition for TextNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/text")
    }

    fn display_name(&self) -> &str {
        "Text"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Builds an EntityValue containing a Text2D component")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        25
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "text", "label", "2d"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        let mut inputs = vec![
            entity_extension_input(),
            transform_override_input(),
            popup_string("Content", "Hello"),
            popup_float("Font Size", 24.0, 1.0, Some(1.0), None),
            popup_color("Color", Color::WHITE),
        ];
        inputs.extend(transform_fallback_inputs());
        inputs
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        let (transform, warning) = resolve_transform(ctx, &entity, 1, 5, "Transform");
        apply_resolved_transform(&mut entity, &transform);

        if entity.component(TEXT2D_COMPONENT_KEY).is_none() {
            entity.set_component(EntityComponentValue::Text2D(Text2DComponentValue {
                content: ctx.get_string(2).unwrap_or("Hello").to_string(),
                font_size: ctx.get_float_or(3, 24.0) as f32,
                color: ctx
                    .inputs
                    .get(4)
                    .and_then(NodeValue::as_color)
                    .unwrap_or(Color::WHITE),
            }));
        }

        finish_entity_process(ctx, entity, warning)
    }
}

pub struct SceneNode;

impl NodeDefinition for SceneNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/scene")
    }

    fn display_name(&self) -> &str {
        "Scene"
    }

    fn category(&self) -> NodeCategory {
        scene_category()
    }

    fn description(&self) -> Option<&str> {
        Some("Declares the graph scene output that runtime systems materialize into the world")
    }

    fn color(&self) -> Color {
        SCENE_NODE_COLOR
    }

    fn menu_order(&self) -> i32 {
        90
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["scene", "entity", "output", "world", "materialize"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            scene_entity_input("Entity")
                .with_description("EntityValue that becomes the graph scene output"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        require_scene_sink_input(ctx)
    }
}

register_node!(TransformNode);
register_node!(NameNode);
register_node!(VisibilityNode);
register_node!(AnchorNode);
register_node!(PrefabInstanceNode);
register_node!(SpriteNode);
register_node!(TextNode);
register_node!(Camera2DNode);
register_node!(ScaleNode);
register_node!(RotationNode);
register_node!(ZOrderNode);
register_node!(MergeEntityNode);
register_node!(AddChildNode);
register_node!(GroupNode);
register_node!(SceneNode);
