use crate::scene::{EditorComponentInstance, EditorComponentPayload, EditorEntity, EditorScene};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use univis_editor_core::value::{
    Camera2DComponentValue, EntityComponentValue, EntityValue, SpriteComponentValue,
    TransformComponentValue,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveEntityGraphProjection {
    pub source_entity_id: Option<u64>,
    pub entity_value: Option<EntityValue>,
    pub disconnected_component_ids: HashSet<u64>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EntityGraphProjection {
    pub entity_value: EntityValue,
    pub disconnected_component_ids: HashSet<u64>,
    pub warnings: Vec<String>,
}

pub fn project_active_entity(scene: &EditorScene) -> ActiveEntityGraphProjection {
    let Some(entity) = scene.get_active_entity() else {
        return ActiveEntityGraphProjection::default();
    };

    let projection = project_editor_entity(entity);
    ActiveEntityGraphProjection {
        source_entity_id: Some(entity.id),
        entity_value: Some(projection.entity_value),
        disconnected_component_ids: projection.disconnected_component_ids,
        warnings: projection.warnings,
    }
}

pub fn project_editor_entity(entity: &EditorEntity) -> EntityGraphProjection {
    let mut warnings = Vec::new();
    let mut composed_entity = EntityValue::named(entity.name.clone());

    let Some(root_component_id) = entity.root_component_id() else {
        warnings.push("Entity projection skipped because the root component is missing.".to_string());
        return EntityGraphProjection {
            entity_value: composed_entity,
            disconnected_component_ids: HashSet::new(),
            warnings,
        };
    };

    let components_by_id: HashMap<u64, &EditorComponentInstance> = entity
        .components
        .iter()
        .map(|component| (component.id, component))
        .collect();

    let mut children_by_parent: HashMap<u64, Vec<u64>> = HashMap::new();
    let mut parent_by_child: HashMap<u64, u64> = HashMap::new();

    for link in &entity.composition_links {
        if !components_by_id.contains_key(&link.child_component_id) {
            warnings.push(format!(
                "Projection skipped a link because child component {} is missing.",
                link.child_component_id
            ));
            continue;
        }

        if !components_by_id.contains_key(&link.parent_component_id) {
            warnings.push(format!(
                "Projection skipped a link because parent component {} is missing.",
                link.parent_component_id
            ));
            continue;
        }

        if let Some(previous_parent) =
            parent_by_child.insert(link.child_component_id, link.parent_component_id)
        {
            warnings.push(format!(
                "Component {} had multiple parents ({} and {}); projection kept the latest link.",
                link.child_component_id, previous_parent, link.parent_component_id
            ));
        }

        children_by_parent
            .entry(link.parent_component_id)
            .or_default()
            .push(link.child_component_id);
    }

    let mut visited_component_ids = HashSet::new();
    compose_component_subtree(
        root_component_id,
        &components_by_id,
        &children_by_parent,
        &mut visited_component_ids,
        &mut composed_entity,
        &mut warnings,
    );

    let disconnected_component_ids = entity
        .components
        .iter()
        .filter(|component| component.id != root_component_id && !visited_component_ids.contains(&component.id))
        .map(|component| component.id)
        .collect::<HashSet<_>>();

    if !disconnected_component_ids.is_empty() {
        warnings.push(format!(
            "{} component(s) are disconnected from the root and were excluded from graph projection.",
            disconnected_component_ids.len()
        ));
    }

    EntityGraphProjection {
        entity_value: composed_entity,
        disconnected_component_ids,
        warnings,
    }
}

fn compose_component_subtree(
    component_id: u64,
    components_by_id: &HashMap<u64, &EditorComponentInstance>,
    children_by_parent: &HashMap<u64, Vec<u64>>,
    visited_component_ids: &mut HashSet<u64>,
    composed_entity: &mut EntityValue,
    warnings: &mut Vec<String>,
) {
    if !visited_component_ids.insert(component_id) {
        warnings.push(format!(
            "Projection detected a cycle around component {} and skipped the repeated branch.",
            component_id
        ));
        return;
    }

    let Some(component) = components_by_id.get(&component_id).copied() else {
        warnings.push(format!(
            "Projection skipped missing component {} while composing the entity value.",
            component_id
        ));
        return;
    };

    if component.enabled {
        if let Some(entity_component) = project_component_payload(component) {
            let kind = entity_component.kind();
            if composed_entity.component(kind.clone()).is_some() {
                warnings.push(format!(
                    "Duplicate '{}' component was ignored during graph projection.",
                    component.display_name()
                ));
            } else {
                composed_entity.set_component(entity_component);
            }
        }
    }

    if let Some(children) = children_by_parent.get(&component_id) {
        for child_component_id in children {
            compose_component_subtree(
                *child_component_id,
                components_by_id,
                children_by_parent,
                visited_component_ids,
                composed_entity,
                warnings,
            );
        }
    }
}

fn project_component_payload(component: &EditorComponentInstance) -> Option<EntityComponentValue> {
    match &component.payload {
        EditorComponentPayload::EntityRoot(_) => None,
        EditorComponentPayload::Transform(value) => {
            Some(EntityComponentValue::Transform(TransformComponentValue {
                translation: Vec3::new(value.x, value.y, 0.0),
                rotation_deg: value.rotation_deg,
                scale: Vec3::new(value.scale_x, value.scale_y, 1.0),
            }))
        }
        EditorComponentPayload::Sprite(value) => {
            let sprite_id = (!value.sprite_id.is_empty()).then_some(value.sprite_id.clone());
            Some(EntityComponentValue::Sprite(SpriteComponentValue {
                size: Vec2::ONE,
                color: Color::srgba(value.color_r, value.color_g, value.color_b, value.color_a),
                sprite_id,
            }))
        }
        EditorComponentPayload::Camera2D(value) => {
            Some(EntityComponentValue::Camera2D(Camera2DComponentValue {
                zoom: value.zoom,
            }))
        }
        EditorComponentPayload::Unknown(value) => Some(EntityComponentValue::custom(
            component.kind.clone(),
            value.clone(),
        )),
    }
}
