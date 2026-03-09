use crate::components::{
    COMPONENT_KIND_CAMERA2D, COMPONENT_KIND_ENTITY_ROOT, COMPONENT_KIND_SPRITE,
    COMPONENT_KIND_TRANSFORM, ECamera2D, EEntityRoot, ESprite, ETransform, component_display_name,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SCENE_SAVE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum EditorComponentPayload {
    EntityRoot(EEntityRoot),
    Transform(ETransform),
    Sprite(ESprite),
    Camera2D(ECamera2D),
    Unknown(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorComponentInstance {
    pub id: u64,
    pub kind: String,
    pub enabled: bool,
    pub payload: EditorComponentPayload,
}

impl EditorComponentInstance {
    pub fn display_name(&self) -> String {
        component_display_name(&self.kind)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self.payload, EditorComponentPayload::Unknown(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedGraphLayoutNode {
    pub component_id: u64,
    pub position: [f32; 2],
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SavedComponentLink {
    pub child_component_id: u64,
    pub parent_component_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedEntityUiState {
    pub inspector_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorEntity {
    pub id: u64,
    pub name: String,
    pub components: Vec<EditorComponentInstance>,
    pub graph_layout: Vec<SavedGraphLayoutNode>,
    #[serde(default)]
    pub composition_links: Vec<SavedComponentLink>,
    #[serde(default)]
    pub ui: SavedEntityUiState,
}

impl EditorEntity {
    pub fn get_component_mut(&mut self, component_id: u64) -> Option<&mut EditorComponentInstance> {
        self.components
            .iter_mut()
            .find(|component| component.id == component_id)
    }

    pub fn get_component(&self, component_id: u64) -> Option<&EditorComponentInstance> {
        self.components
            .iter()
            .find(|component| component.id == component_id)
    }

    pub fn layout_for_component(&self, component_id: u64) -> Option<&SavedGraphLayoutNode> {
        self.graph_layout
            .iter()
            .find(|layout| layout.component_id == component_id)
    }

    pub fn root_component_id(&self) -> Option<u64> {
        self.components
            .iter()
            .find(|component| component.kind == COMPONENT_KIND_ENTITY_ROOT)
            .map(|component| component.id)
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct EditorScene {
    pub version: u32,
    pub entities: Vec<EditorEntity>,
    pub active_entity_id: Option<u64>,
    pub next_entity_id: u64,
    pub next_component_id: u64,
}

impl Default for EditorScene {
    fn default() -> Self {
        let mut scene = Self {
            version: SCENE_SAVE_VERSION,
            entities: Vec::new(),
            active_entity_id: None,
            next_entity_id: 1,
            next_component_id: 1,
        };

        let root_id = scene.alloc_entity_id();
        let root_component_id = scene.alloc_component_id();
        let transform_id = scene.alloc_component_id();

        scene.entities.push(EditorEntity {
            id: root_id,
            name: "Root Entity".to_string(),
            components: vec![
                EditorComponentInstance {
                    id: root_component_id,
                    kind: COMPONENT_KIND_ENTITY_ROOT.to_string(),
                    enabled: true,
                    payload: EditorComponentPayload::EntityRoot(EEntityRoot),
                },
                EditorComponentInstance {
                    id: transform_id,
                    kind: COMPONENT_KIND_TRANSFORM.to_string(),
                    enabled: true,
                    payload: EditorComponentPayload::Transform(ETransform::default()),
                },
            ],
            graph_layout: vec![
                SavedGraphLayoutNode {
                    component_id: root_component_id,
                    position: [160.0, 140.0],
                    selected: false,
                },
                SavedGraphLayoutNode {
                    component_id: transform_id,
                    position: [-220.0, 120.0],
                    selected: false,
                },
            ],
            composition_links: vec![SavedComponentLink {
                child_component_id: transform_id,
                parent_component_id: root_component_id,
            }],
            ui: SavedEntityUiState::default(),
        });

        scene.active_entity_id = Some(root_id);
        scene
    }
}

impl EditorScene {
    pub fn alloc_entity_id(&mut self) -> u64 {
        let id = self.next_entity_id;
        self.next_entity_id = self.next_entity_id.saturating_add(1);
        id
    }

    pub fn alloc_component_id(&mut self) -> u64 {
        let id = self.next_component_id;
        self.next_component_id = self.next_component_id.saturating_add(1);
        id
    }

    pub fn get_active_entity_mut(&mut self) -> Option<&mut EditorEntity> {
        let active_id = self.active_entity_id?;
        self.entities
            .iter_mut()
            .find(|entity| entity.id == active_id)
    }

    pub fn get_active_entity(&self) -> Option<&EditorEntity> {
        let active_id = self.active_entity_id?;
        self.entities.iter().find(|entity| entity.id == active_id)
    }

    pub fn ensure_active_entity(&mut self) {
        if self.entities.is_empty() {
            *self = Self::default();
            return;
        }

        if let Some(active_id) = self.active_entity_id {
            if self.entities.iter().any(|entity| entity.id == active_id) {
                return;
            }
        }

        self.active_entity_id = self.entities.first().map(|entity| entity.id);
    }

    pub fn set_active_entity(&mut self, entity_id: u64) {
        if self.entities.iter().any(|entity| entity.id == entity_id) {
            self.active_entity_id = Some(entity_id);
        }
    }

    pub fn active_entity_name(&self) -> String {
        self.get_active_entity()
            .map(|entity| entity.name.clone())
            .unwrap_or_else(|| "None".to_string())
    }

    pub fn sorted_entity_ids(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self.entities.iter().map(|entity| entity.id).collect();
        ids.sort_unstable();
        ids
    }

    pub fn default_component_payload(kind: &str) -> Option<EditorComponentPayload> {
        match kind {
            COMPONENT_KIND_ENTITY_ROOT => Some(EditorComponentPayload::EntityRoot(EEntityRoot)),
            COMPONENT_KIND_TRANSFORM => {
                Some(EditorComponentPayload::Transform(ETransform::default()))
            }
            COMPONENT_KIND_SPRITE => Some(EditorComponentPayload::Sprite(ESprite::default())),
            COMPONENT_KIND_CAMERA2D => Some(EditorComponentPayload::Camera2D(ECamera2D::default())),
            _ => None,
        }
    }

    pub fn ensure_entity_root_component(&mut self, entity_id: u64) -> Option<u64> {
        let entity_index = self
            .entities
            .iter()
            .position(|entity| entity.id == entity_id)?;

        if let Some(root_id) = self.entities[entity_index].root_component_id() {
            return Some(root_id);
        }

        let root_component_id = self.alloc_component_id();
        self.entities[entity_index]
            .components
            .push(EditorComponentInstance {
                id: root_component_id,
                kind: COMPONENT_KIND_ENTITY_ROOT.to_string(),
                enabled: true,
                payload: EditorComponentPayload::EntityRoot(EEntityRoot),
            });
        self.entities[entity_index]
            .graph_layout
            .push(SavedGraphLayoutNode {
                component_id: root_component_id,
                position: [160.0, 140.0],
                selected: false,
            });

        Some(root_component_id)
    }
}
