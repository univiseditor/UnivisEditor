mod support;

use bevy::prelude::{Color, Vec2, Vec3};
use support::run_node;
use univis_editor_nodes_builtin::scene::{
    AddChildNode, AnchorNode, Camera2DNode, GroupNode, MergeEntityNode, NameNode, RotationNode,
    ScaleNode, SceneNode, SpriteNode, TextNode, TransformNode, VisibilityNode, ZOrderNode,
};
use univis_editor_nodes_builtin::scene_support::pure_entity_requirement_token;
use univis_node_graph::node_definition::{NodeDefinition, ProcessResult};
use univis_node_graph::value::NodeValue;
use univis_scene::{
    EntityComponentValue, EntityValue, TransformComponentValue, ANCHOR_COMPONENT_KEY,
    CAMERA2D_COMPONENT_KEY, SPRITE_COMPONENT_KEY, TEXT2D_COMPONENT_KEY, TRANSFORM_COMPONENT_KEY,
    VISIBILITY_COMPONENT_KEY,
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
fn visibility_and_anchor_nodes_build_pure_scene_components() {
    let visibility = VisibilityNode;
    let (result, outputs) = run_node(&visibility, vec![NodeValue::None, NodeValue::bool(false)]);
    assert!(matches!(result, ProcessResult::Success));
    assert_eq!(
        visibility.output_requirement_token(0, &[false]),
        Some(pure_entity_requirement_token(VISIBILITY_COMPONENT_KEY))
    );

    match &outputs[0] {
        NodeValue::Entity(entity) => {
            let visibility = entity
                .component(VISIBILITY_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_visibility)
                .expect("expected visibility component");
            assert!(!visibility.visible);
        }
        other => panic!("expected entity output, got {other:?}"),
    }

    let anchor = AnchorNode;
    let (result, outputs) = run_node(
        &anchor,
        vec![
            NodeValue::None,
            NodeValue::float(-0.5),
            NodeValue::float(0.25),
        ],
    );
    assert!(matches!(result, ProcessResult::Success));
    assert_eq!(
        anchor.output_requirement_token(0, &[false]),
        Some(pure_entity_requirement_token(ANCHOR_COMPONENT_KEY))
    );

    match &outputs[0] {
        NodeValue::Entity(entity) => {
            let anchor = entity
                .component(ANCHOR_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_anchor)
                .expect("expected anchor component");
            assert_eq!(anchor.position, Vec2::new(-0.5, 0.25));
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
fn transform_helper_nodes_override_scale_rotation_and_z_order() {
    let base = EntityValue::default().with_component(EntityComponentValue::Transform(
        TransformComponentValue {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation_deg: 10.0,
            scale: Vec3::splat(2.0),
        },
    ));

    let (_, outputs) = run_node(
        &ScaleNode,
        vec![
            NodeValue::entity(base.clone()),
            NodeValue::None,
            NodeValue::float(3.0),
            NodeValue::float(4.0),
            NodeValue::float(5.0),
        ],
    );
    let scaled = match &outputs[0] {
        NodeValue::Entity(entity) => entity.clone(),
        other => panic!("expected entity output, got {other:?}"),
    };
    let transform = scaled
        .component(TRANSFORM_COMPONENT_KEY)
        .and_then(EntityComponentValue::as_transform)
        .expect("expected transform");
    assert_eq!(transform.scale, Vec3::new(3.0, 4.0, 5.0));
    assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));

    let (_, outputs) = run_node(
        &RotationNode,
        vec![
            NodeValue::entity(base.clone()),
            NodeValue::None,
            NodeValue::float(45.0),
        ],
    );
    match &outputs[0] {
        NodeValue::Entity(entity) => {
            let transform = entity
                .component(TRANSFORM_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_transform)
                .expect("expected transform");
            assert_eq!(transform.rotation_deg, 45.0);
        }
        other => panic!("expected entity output, got {other:?}"),
    }

    let (_, outputs) = run_node(
        &ZOrderNode,
        vec![
            NodeValue::entity(base),
            NodeValue::None,
            NodeValue::float(12.5),
        ],
    );
    match &outputs[0] {
        NodeValue::Entity(entity) => {
            let transform = entity
                .component(TRANSFORM_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_transform)
                .expect("expected transform");
            assert_eq!(transform.translation.z, 12.5);
            assert_eq!(transform.translation.x, 1.0);
            assert_eq!(transform.translation.y, 2.0);
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
fn group_node_creates_a_named_container_with_children() {
    let child_a = EntityValue::named("A");
    let child_b = EntityValue::named("B");

    let (result, outputs) = run_node(
        &GroupNode,
        vec![
            NodeValue::None,
            NodeValue::None,
            NodeValue::entity(child_a),
            NodeValue::entity(child_b),
            NodeValue::string("UI Group"),
        ],
    );
    assert!(matches!(result, ProcessResult::Success));

    match &outputs[0] {
        NodeValue::Entity(entity) => {
            assert_eq!(entity.name.as_deref(), Some("UI Group"));
            assert_eq!(entity.children.len(), 2);
            assert_eq!(entity.children[0].name.as_deref(), Some("A"));
            assert_eq!(entity.children[1].name.as_deref(), Some("B"));
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
