use bevy::prelude::*;
use std::collections::HashSet;

pub use univis_editor_core::mode::{EditorMode, EditorModeState};

#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveEntityContext {
    pub active_entity_id: Option<u64>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedComponentContext {
    pub component_id: Option<u64>,
}

#[derive(Resource, Debug, Clone)]
pub struct GameEditorRuntimeState {
    pub dirty: bool,
    pub initialized: bool,
    pub last_saved_signature: Option<String>,
    pub autosave_elapsed_secs: f32,
    pub open_confirm_until_secs: Option<f64>,
    pub needs_rebaseline: bool,
    pub canvas_needs_rebuild: bool,
    pub links_need_rebuild: bool,
}

impl Default for GameEditorRuntimeState {
    fn default() -> Self {
        Self {
            dirty: false,
            initialized: false,
            last_saved_signature: None,
            autosave_elapsed_secs: 0.0,
            open_confirm_until_secs: None,
            needs_rebaseline: false,
            canvas_needs_rebuild: true,
            links_need_rebuild: true,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ComponentCompositionDiagnostics {
    pub orphan_component_ids: HashSet<u64>,
    pub invalid_link_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentModeNode {
    pub editor_entity_id: u64,
    pub component_id: u64,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct SelectActiveEntityRequest {
    pub entity_id: u64,
}

#[derive(Message, Debug, Clone)]
pub struct AddComponentNodeRequest {
    pub kind: String,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct RemoveComponentNodeRequest {
    pub component_id: u64,
}

#[derive(Message, Debug, Clone, Copy, Default)]
pub struct SaveSceneRequest;

#[derive(Message, Debug, Clone, Copy, Default)]
pub struct LoadSceneRequest;

#[derive(Message, Debug, Clone)]
pub struct SaveSceneToPathRequest {
    pub path: String,
}

#[derive(Message, Debug, Clone)]
pub struct LoadSceneFromPathRequest {
    pub path: String,
    pub force_if_dirty: bool,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct SetEditorModeRequest {
    pub mode: EditorMode,
}
