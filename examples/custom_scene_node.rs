use bevy::prelude::*;
use serde_json::json;
use univis_editor_app::{NodeGraphPlugin, prelude::*};
use univis_editor_nodes_builtin::scene_support::{
    apply_resolved_transform, base_entity_or_empty, entity_extension_input,
    finish_entity_process, popup_color, popup_float, popup_string, resolve_transform,
    scene_entity_output, transform_fallback_inputs, transform_override_input,
};
use univis_node_graph::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_node_graph::register_node;
use univis_scene::{EntityComponentValue, SPRITE_COMPONENT_KEY};

const EXAMPLE_COLOR: Color = Color::srgb(0.18, 0.62, 0.54);
const MESH2D_COMPONENT_KEY: &str = "example/mesh2d";

fn color_payload(color: Color) -> serde_json::Value {
    let srgba = color.to_srgba();
    json!([srgba.red, srgba.green, srgba.blue, srgba.alpha])
}

pub struct Mesh2DNode;

impl NodeDefinition for Mesh2DNode {
    fn id(&self) -> NodeId {
        NodeId::new("example/mesh2d")
    }

    fn display_name(&self) -> &str {
        "Mesh2D"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Examples/Scene")
    }

    fn description(&self) -> Option<&str> {
        Some("Example external scene node that composes a custom Mesh2D component into EntityValue")
    }

    fn color(&self) -> Color {
        EXAMPLE_COLOR
    }

    fn menu_order(&self) -> i32 {
        10
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["example", "scene", "mesh", "2d", "custom"]
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        let mut inputs = vec![
            entity_extension_input(),
            transform_override_input(),
            popup_string("Shape", "quad"),
            popup_float("Width", 2.0, 0.1, Some(0.01), None),
            popup_float("Height", 2.0, 0.1, Some(0.01), None),
            popup_color("Color", Color::srgb(0.3, 0.8, 0.7)),
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

        if entity.component(MESH2D_COMPONENT_KEY).is_none() {
            let color = ctx
                .inputs
                .get(5)
                .and_then(univis_node_graph::value::NodeValue::as_color)
                .unwrap_or(Color::WHITE);

            entity.set_component(EntityComponentValue::Custom {
                key: MESH2D_COMPONENT_KEY.to_string(),
                payload: json!({
                    "shape": ctx.get_string(2).unwrap_or("quad"),
                    "width": ctx.get_float_or(3, 2.0),
                    "height": ctx.get_float_or(4, 2.0),
                    "color": color_payload(color),
                }),
            });
        }

        finish_entity_process(ctx, entity, warning)
    }
}

pub struct SpriteFrameNode;

impl NodeDefinition for SpriteFrameNode {
    fn id(&self) -> NodeId {
        NodeId::new("example/sprite_frame")
    }

    fn display_name(&self) -> &str {
        "Sprite Frame"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Examples/Scene")
    }

    fn description(&self) -> Option<&str> {
        Some("Small companion node that proves external nodes can compose over built-in scene entities")
    }

    fn color(&self) -> Color {
        Color::srgb(0.78, 0.46, 0.24)
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            entity_extension_input(),
            popup_string("Frame Tag", "idle"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![scene_entity_output("Entity")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let mut entity = base_entity_or_empty(ctx, 0);
        if entity.component(SPRITE_COMPONENT_KEY).is_none() {
            return ProcessResult::Error(
                "Sprite Frame expects an entity that already contains a Sprite component"
                    .to_string(),
            );
        }

        entity.set_component(EntityComponentValue::Custom {
            key: "example/sprite_frame".to_string(),
            payload: json!({
                "tag": ctx.get_string(1).unwrap_or("idle"),
            }),
        });

        finish_entity_process(ctx, entity, None)
    }
}

register_node!(Mesh2DNode);
register_node!(SpriteFrameNode);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((NodeGraphPlugin, InfiniteGridPlugin))
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((Camera2d, GraphCamera));
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            scale: 50.0,
            dot_fadeout_strength: 0.0,
            z_axis_color: Color::NONE,
            x_axis_color: Color::NONE,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::Y, Vec3::Z)),
    ));
}
