use crate::components::{
    COMPONENT_KIND_CAMERA2D, COMPONENT_KIND_ENTITY_ROOT, COMPONENT_KIND_SPRITE,
    COMPONENT_KIND_TRANSFORM, ECamera2D, EEntityRoot, ESprite, ETransform, component_display_name,
};
use crate::scene::{EditorComponentInstance, EditorComponentPayload, EditorScene};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ComponentFieldKind {
    Float,
    String,
    Bool,
}

#[derive(Debug, Clone)]
pub struct ComponentFieldSchema {
    pub key: String,
    pub label: String,
    pub kind: ComponentFieldKind,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: f32,
}

#[derive(Debug, Clone)]
pub struct ComponentSchema {
    pub kind: String,
    pub display_name: String,
    pub fields: Vec<ComponentFieldSchema>,
    pub can_have_children: bool,
    pub allowed_parent_kinds: Vec<String>,
    pub deletable: bool,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ComponentSchemaRegistry {
    schemas: HashMap<String, ComponentSchema>,
}

impl ComponentSchemaRegistry {
    pub fn register_defaults(&mut self) {
        self.schemas.clear();

        self.register(ComponentSchema {
            kind: COMPONENT_KIND_ENTITY_ROOT.to_string(),
            display_name: component_display_name(COMPONENT_KIND_ENTITY_ROOT),
            fields: vec![],
            can_have_children: true,
            allowed_parent_kinds: vec![],
            deletable: false,
        });

        self.register(ComponentSchema {
            kind: COMPONENT_KIND_TRANSFORM.to_string(),
            display_name: component_display_name(COMPONENT_KIND_TRANSFORM),
            fields: vec![
                ComponentFieldSchema {
                    key: "x".to_string(),
                    label: "X".to_string(),
                    kind: ComponentFieldKind::Float,
                    min: Some(-10000.0),
                    max: Some(10000.0),
                    step: 10.0,
                },
                ComponentFieldSchema {
                    key: "y".to_string(),
                    label: "Y".to_string(),
                    kind: ComponentFieldKind::Float,
                    min: Some(-10000.0),
                    max: Some(10000.0),
                    step: 10.0,
                },
                ComponentFieldSchema {
                    key: "rotation_deg".to_string(),
                    label: "Rotation".to_string(),
                    kind: ComponentFieldKind::Float,
                    min: Some(-3600.0),
                    max: Some(3600.0),
                    step: 5.0,
                },
                ComponentFieldSchema {
                    key: "scale_x".to_string(),
                    label: "Scale X".to_string(),
                    kind: ComponentFieldKind::Float,
                    min: Some(0.01),
                    max: Some(100.0),
                    step: 0.1,
                },
                ComponentFieldSchema {
                    key: "scale_y".to_string(),
                    label: "Scale Y".to_string(),
                    kind: ComponentFieldKind::Float,
                    min: Some(0.01),
                    max: Some(100.0),
                    step: 0.1,
                },
            ],
            can_have_children: true,
            allowed_parent_kinds: vec![
                COMPONENT_KIND_ENTITY_ROOT.to_string(),
                COMPONENT_KIND_TRANSFORM.to_string(),
            ],
            deletable: true,
        });

        self.register(ComponentSchema {
            kind: COMPONENT_KIND_SPRITE.to_string(),
            display_name: component_display_name(COMPONENT_KIND_SPRITE),
            fields: vec![
                ComponentFieldSchema {
                    key: "color_r".to_string(),
                    label: "Color R".to_string(),
                    kind: ComponentFieldKind::Float,
                    min: Some(0.0),
                    max: Some(1.0),
                    step: 0.05,
                },
                ComponentFieldSchema {
                    key: "color_g".to_string(),
                    label: "Color G".to_string(),
                    kind: ComponentFieldKind::Float,
                    min: Some(0.0),
                    max: Some(1.0),
                    step: 0.05,
                },
                ComponentFieldSchema {
                    key: "color_b".to_string(),
                    label: "Color B".to_string(),
                    kind: ComponentFieldKind::Float,
                    min: Some(0.0),
                    max: Some(1.0),
                    step: 0.05,
                },
                ComponentFieldSchema {
                    key: "color_a".to_string(),
                    label: "Color A".to_string(),
                    kind: ComponentFieldKind::Float,
                    min: Some(0.0),
                    max: Some(1.0),
                    step: 0.05,
                },
            ],
            can_have_children: false,
            allowed_parent_kinds: vec![
                COMPONENT_KIND_ENTITY_ROOT.to_string(),
                COMPONENT_KIND_TRANSFORM.to_string(),
            ],
            deletable: true,
        });

        self.register(ComponentSchema {
            kind: COMPONENT_KIND_CAMERA2D.to_string(),
            display_name: component_display_name(COMPONENT_KIND_CAMERA2D),
            fields: vec![ComponentFieldSchema {
                key: "zoom".to_string(),
                label: "Zoom".to_string(),
                kind: ComponentFieldKind::Float,
                min: Some(0.1),
                max: Some(10.0),
                step: 0.1,
            }],
            can_have_children: false,
            allowed_parent_kinds: vec![
                COMPONENT_KIND_ENTITY_ROOT.to_string(),
                COMPONENT_KIND_TRANSFORM.to_string(),
            ],
            deletable: true,
        });
    }

    pub fn register(&mut self, schema: ComponentSchema) {
        self.schemas.insert(schema.kind.clone(), schema);
    }

    pub fn get(&self, kind: &str) -> Option<&ComponentSchema> {
        self.schemas.get(kind)
    }

    pub fn known_kinds(&self) -> Vec<String> {
        let mut kinds: Vec<String> = self.schemas.keys().cloned().collect();
        kinds.sort();
        kinds
    }

    pub fn can_have_children(&self, kind: &str) -> bool {
        self.schemas
            .get(kind)
            .map(|schema| schema.can_have_children)
            .unwrap_or(true)
    }

    pub fn is_deletable(&self, kind: &str) -> bool {
        self.schemas
            .get(kind)
            .map(|schema| schema.deletable)
            .unwrap_or(true)
    }

    pub fn can_be_child(&self, kind: &str) -> bool {
        if kind == COMPONENT_KIND_ENTITY_ROOT {
            return false;
        }

        self.schemas
            .get(kind)
            .map(|schema| !schema.allowed_parent_kinds.is_empty())
            .unwrap_or(true)
    }

    pub fn can_attach_to_parent(&self, child_kind: &str, parent_kind: &str) -> bool {
        if child_kind == COMPONENT_KIND_ENTITY_ROOT {
            return false;
        }

        let Some(child_schema) = self.schemas.get(child_kind) else {
            // Unknown placeholder components are allowed to preserve topology during load.
            return true;
        };

        if child_schema.allowed_parent_kinds.is_empty() {
            return false;
        }

        child_schema
            .allowed_parent_kinds
            .iter()
            .any(|kind| kind == parent_kind)
    }

    pub fn create_default_component(
        &self,
        scene: &mut EditorScene,
        kind: &str,
    ) -> Option<EditorComponentInstance> {
        let payload = EditorScene::default_component_payload(kind)?;
        Some(EditorComponentInstance {
            id: scene.alloc_component_id(),
            kind: kind.to_string(),
            enabled: true,
            payload,
        })
    }

    pub fn validate_component(&self, component: &mut EditorComponentInstance) {
        match &mut component.payload {
            EditorComponentPayload::EntityRoot(_) => {}
            EditorComponentPayload::Transform(value) => {
                value.scale_x = value.scale_x.clamp(0.01, 100.0);
                value.scale_y = value.scale_y.clamp(0.01, 100.0);
                value.rotation_deg = value.rotation_deg.clamp(-3600.0, 3600.0);
                value.x = value.x.clamp(-10000.0, 10000.0);
                value.y = value.y.clamp(-10000.0, 10000.0);
            }
            EditorComponentPayload::Sprite(value) => {
                value.color_r = value.color_r.clamp(0.0, 1.0);
                value.color_g = value.color_g.clamp(0.0, 1.0);
                value.color_b = value.color_b.clamp(0.0, 1.0);
                value.color_a = value.color_a.clamp(0.0, 1.0);
            }
            EditorComponentPayload::Camera2D(value) => {
                value.zoom = value.zoom.clamp(0.1, 10.0);
            }
            EditorComponentPayload::Unknown(_) => {}
        }
    }

    pub fn apply_field_delta(
        &self,
        component: &mut EditorComponentInstance,
        field_key: &str,
        delta: f32,
    ) {
        match &mut component.payload {
            EditorComponentPayload::EntityRoot(_) => {}
            EditorComponentPayload::Transform(value) => match field_key {
                "x" => value.x += delta,
                "y" => value.y += delta,
                "rotation_deg" => value.rotation_deg += delta,
                "scale_x" => value.scale_x += delta,
                "scale_y" => value.scale_y += delta,
                _ => {}
            },
            EditorComponentPayload::Sprite(value) => match field_key {
                "color_r" => value.color_r += delta,
                "color_g" => value.color_g += delta,
                "color_b" => value.color_b += delta,
                "color_a" => value.color_a += delta,
                _ => {}
            },
            EditorComponentPayload::Camera2D(value) => {
                if field_key == "zoom" {
                    value.zoom += delta;
                }
            }
            EditorComponentPayload::Unknown(_) => {}
        }

        self.validate_component(component);
    }

    pub fn fallback_component(
        &self,
        id: u64,
        kind: String,
        enabled: bool,
        raw: serde_json::Value,
    ) -> EditorComponentInstance {
        let _ = self;
        EditorComponentInstance {
            id,
            kind,
            enabled,
            payload: EditorComponentPayload::Unknown(raw),
        }
    }

    pub fn deserialize_known_payload(
        &self,
        kind: &str,
        value: serde_json::Value,
    ) -> Result<EditorComponentPayload, String> {
        let _ = self;
        match kind {
            COMPONENT_KIND_ENTITY_ROOT => serde_json::from_value::<EEntityRoot>(value)
                .map(EditorComponentPayload::EntityRoot)
                .map_err(|err| format!("invalid root payload: {}", err)),
            COMPONENT_KIND_TRANSFORM => serde_json::from_value::<ETransform>(value)
                .map(EditorComponentPayload::Transform)
                .map_err(|err| format!("invalid transform payload: {}", err)),
            COMPONENT_KIND_SPRITE => serde_json::from_value::<ESprite>(value)
                .map(EditorComponentPayload::Sprite)
                .map_err(|err| format!("invalid sprite payload: {}", err)),
            COMPONENT_KIND_CAMERA2D => serde_json::from_value::<ECamera2D>(value)
                .map(EditorComponentPayload::Camera2D)
                .map_err(|err| format!("invalid camera payload: {}", err)),
            _ => Err("unknown component kind".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{COMPONENT_KIND_ENTITY_ROOT, COMPONENT_KIND_TRANSFORM};
    use crate::scene::{EditorComponentPayload, EditorScene};

    #[test]
    fn transform_defaults_and_validation_work() {
        let mut registry = ComponentSchemaRegistry::default();
        registry.register_defaults();

        let mut scene = EditorScene::default();
        let mut component = registry
            .create_default_component(&mut scene, COMPONENT_KIND_TRANSFORM)
            .expect("transform component should exist");

        if let EditorComponentPayload::Transform(value) = &mut component.payload {
            value.scale_x = -50.0;
            value.scale_y = 1000.0;
            value.rotation_deg = 9999.0;
        }

        registry.validate_component(&mut component);

        match component.payload {
            EditorComponentPayload::Transform(value) => {
                assert_eq!(value.scale_x, 0.01);
                assert_eq!(value.scale_y, 100.0);
                assert_eq!(value.rotation_deg, 3600.0);
            }
            _ => panic!("unexpected payload type"),
        }
    }

    #[test]
    fn composition_rules_are_registered() {
        let mut registry = ComponentSchemaRegistry::default();
        registry.register_defaults();

        assert!(!registry.can_be_child(COMPONENT_KIND_ENTITY_ROOT));
        assert!(registry.can_have_children(COMPONENT_KIND_ENTITY_ROOT));
        assert!(!registry.is_deletable(COMPONENT_KIND_ENTITY_ROOT));
        assert!(
            registry.can_attach_to_parent(COMPONENT_KIND_TRANSFORM, COMPONENT_KIND_ENTITY_ROOT)
        );
    }
}
