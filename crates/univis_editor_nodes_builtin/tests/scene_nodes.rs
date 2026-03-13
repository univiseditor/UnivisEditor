mod support;

use bevy::prelude::{Color, Vec2, Vec3};
use support::run_node;
use univis_editor_nodes_builtin::scene::{
    AddChildNode, Camera2DNode, MergeEntityNode, NameNode, SceneNode, SpriteNode, TextNode,
    TransformNode,
};
use univis_editor_nodes_builtin::scene_support::pure_entity_requirement_token;
use univis_node_graph::node_definition::{NodeDefinition, ProcessResult};
use univis_node_graph::value::NodeValue;
use univis_scene::{
    CAMERA2D_COMPONENT_KEY, EntityComponentValue, EntityValue, SPRITE_COMPONENT_KEY,
    TEXT2D_COMPONENT_KEY, TRANSFORM_COMPONENT_KEY, TransformComponentValue,
};

fn pure_transform_entity(translation: Vec3) -> EntityValue {
    EntityValue::default().with_component(EntityComponentValue::Transform(
        TransformComponentValue {
            translation,
            rotation_deg: 0.0,
            scale: Vec3::ONE,
        },
    ))
}

#[test]
fn transform_node_builds_a_pure_transform_entity() {
    let node = TransformNode;
    let (result, outputs) = run_node(&node, vec![]);

    assert!(matches!(result, ProcessResult::Success));
    assert_eq!(
        node.output_requirement_token(0, &[false]),
        Some(pure_entity_requirement_token(TRANSFORM_COMPONENT_KEY))
    );

    match &outputs[0] {
        NodeValue::Entity(entity) => {
            let transform = entity
                .component(TRANSFORM_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_transform)
                .expect("expected transform component");
            assert_eq!(transform.translation, Vec3::ZERO);
            assert_eq!(transform.scale, Vec3::ONE);
        }
        other => panic!("expected entity output, got {other:?}"),
    }
}

#[test]
fn sprite_node_uses_specific_transform_override_and_trims_sprite_id() {
    let override_transform = pure_transform_entity(Vec3::new(4.0, 5.0, 6.0));
    let (result, outputs) = run_node(
        &SpriteNode,
        vec![
            NodeValue::None,
            NodeValue::entity(override_transform),
            NodeValue::float(3.0),
            NodeValue::float(4.0),
            NodeValue::color(0.1, 0.2, 0.3, 0.9),
            NodeValue::string(" hero "),
        ],
    );

    assert!(matches!(result, ProcessResult::Success));

    match &outputs[0] {
        NodeValue::Entity(entity) => {
            let transform = entity
                .component(TRANSFORM_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_transform)
                .expect("expected transform");
            assert_eq!(transform.translation, Vec3::new(4.0, 5.0, 6.0));

            let sprite = entity
                .component(SPRITE_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_sprite)
                .expect("expected sprite");
            assert_eq!(sprite.size, Vec2::new(3.0, 4.0));
            assert_eq!(sprite.sprite_id.as_deref(), Some("hero"));
        }
        other => panic!("expected entity output, got {other:?}"),
    }
}

#[test]
fn name_and_text_nodes_preserve_existing_name_and_fallback_to_base_transform() {
    let base = EntityValue::named("Existing").with_component(EntityComponentValue::Transform(
        TransformComponentValue {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation_deg: 0.0,
            scale: Vec3::ONE,
        },
    ));
    let invalid_override = EntityValue::named("Invalid").with_component(
        EntityComponentValue::Transform(TransformComponentValue {
            translation: Vec3::new(9.0, 9.0, 9.0),
            rotation_deg: 0.0,
            scale: Vec3::ONE,
        }),
    );

    let (_, outputs) = run_node(
        &NameNode,
        vec![
            NodeValue::entity(base.clone()),
            NodeValue::string("Replacement"),
        ],
    );
    match &outputs[0] {
        NodeValue::Entity(entity) => assert_eq!(entity.name.as_deref(), Some("Existing")),
        other => panic!("expected entity output, got {other:?}"),
    }

    let (result, outputs) = run_node(
        &TextNode,
        vec![
            NodeValue::entity(base),
            NodeValue::entity(invalid_override),
            NodeValue::string("Hello"),
            NodeValue::float(32.0),
            NodeValue::Color(Color::WHITE),
        ],
    );
    assert!(matches!(result, ProcessResult::Error(_)));

    match &outputs[0] {
        NodeValue::Entity(entity) => {
            let transform = entity
                .component(TRANSFORM_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_transform)
                .expect("expected transform");
            assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));

            let text = entity
                .component(TEXT2D_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_text_2d)
                .expect("expected text");
            assert_eq!(text.content, "Hello");
            assert_eq!(text.font_size, 32.0);
        }
        other => panic!("expected entity output, got {other:?}"),
    }
}

#[test]
fn merge_entity_add_child_and_camera_nodes_compose_scene_entities() {
    let left = EntityValue::named("Parent").with_component(EntityComponentValue::Transform(
        TransformComponentValue {
            translation: Vec3::ZERO,
            rotation_deg: 0.0,
            scale: Vec3::ONE,
        },
    ));
    let right = EntityValue::default().with_component(EntityComponentValue::Camera2D(
        univis_scene::Camera2DComponentValue { zoom: 2.0 },
    ));

    let (_, outputs) = run_node(
        &MergeEntityNode,
        vec![NodeValue::entity(left.clone()), NodeValue::entity(right)],
    );
    let merged = match &outputs[0] {
        NodeValue::Entity(entity) => entity.clone(),
        other => panic!("expected entity output, got {other:?}"),
    };
    assert_eq!(merged.name.as_deref(), Some("Parent"));
    assert!(merged.component(CAMERA2D_COMPONENT_KEY).is_some());

    let child = EntityValue::named("Child");
    let (_, outputs) = run_node(
        &AddChildNode,
        vec![NodeValue::entity(merged.clone()), NodeValue::entity(child)],
    );
    match &outputs[0] {
        NodeValue::Entity(entity) => {
            assert_eq!(entity.children.len(), 1);
            assert_eq!(entity.children[0].name.as_deref(), Some("Child"));
        }
        other => panic!("expected entity output, got {other:?}"),
    }

    let (_, outputs) = run_node(
        &Camera2DNode,
        vec![
            NodeValue::entity(merged),
            NodeValue::None,
            NodeValue::float(1.5),
        ],
    );
    match &outputs[0] {
        NodeValue::Entity(entity) => {
            let camera = entity
                .component(CAMERA2D_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_camera_2d)
                .expect("expected camera");
            assert_eq!(camera.zoom, 2.0);
        }
        other => panic!("expected entity output, got {other:?}"),
    }
}

#[test]
fn scene_node_requires_an_input_entity() {
    let (result, _) = run_node(&SceneNode, vec![]);
    assert!(matches!(result, ProcessResult::MissingInput(0)));

    let (result, _) = run_node(&SceneNode, vec![NodeValue::entity(EntityValue::default())]);
    assert!(matches!(result, ProcessResult::Success));
}
