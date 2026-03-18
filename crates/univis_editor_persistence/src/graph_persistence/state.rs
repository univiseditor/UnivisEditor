use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashMap;
use univis_editor_ui::menu::ContextMenuState;
use univis_editor_ui::node_popup::NodePopupState;
use univis_editor_ui::overlay::{GraphOverlayState, GraphOverlaySurface};
use univis_node_graph::prelude::*;

const DEFAULT_SAVE_FILE_PATH: &str = "assets/graphs/current_graph.json";
const DEFAULT_BACKUP_DIRECTORY: &str = "assets/graphs/backups";

/// Persistence configuration for save/load and autosave flows.
#[derive(Resource, Debug, Clone)]
pub struct GraphPersistenceSettings {
    pub file_path: String,
    pub pretty_json: bool,
    pub autosave_enabled: bool,
    pub autosave_interval_secs: f32,
    pub backup_directory: String,
    pub max_backup_files: usize,
    pub status_duration_secs: f32,
    pub confirm_reload_window_secs: f32,
}

impl Default for GraphPersistenceSettings {
    fn default() -> Self {
        Self {
            file_path: DEFAULT_SAVE_FILE_PATH.to_string(),
            pretty_json: true,
            autosave_enabled: true,
            autosave_interval_secs: 120.0,
            backup_directory: DEFAULT_BACKUP_DIRECTORY.to_string(),
            max_backup_files: 10,
            status_duration_secs: 4.0,
            confirm_reload_window_secs: 3.0,
        }
    }
}

/// Runtime state tracked by the persistence plugin.
#[derive(Resource, Debug, Clone)]
pub struct GraphPersistenceRuntimeState {
    pub dirty: bool,
    pub initialized: bool,
    pub last_saved_signature: Option<String>,
    pub autosave_elapsed_secs: f32,
    pub open_confirm_until_secs: Option<f64>,
    pub needs_rebaseline: bool,
    pub requires_resave_after_migration: bool,
}

impl Default for GraphPersistenceRuntimeState {
    fn default() -> Self {
        Self {
            dirty: false,
            initialized: false,
            last_saved_signature: None,
            autosave_elapsed_secs: 0.0,
            open_confirm_until_secs: None,
            needs_rebaseline: false,
            requires_resave_after_migration: false,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct GraphHistorySettings {
    pub enabled: bool,
    pub max_entries: usize,
}

impl Default for GraphHistorySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 64,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphHistoryState {
    pub past: Vec<GraphDocument>,
    pub future: Vec<GraphDocument>,
    pub last_document: Option<GraphDocument>,
    pub last_signature: Option<String>,
    pub awaiting_rebaseline: bool,
}

impl GraphHistoryState {
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
        self.last_document = None;
        self.last_signature = None;
        self.awaiting_rebaseline = false;
    }

    pub fn rebaseline_to_document(&mut self, document: &GraphDocument) {
        self.last_signature = crate::format::graph_document_signature(document).ok();
        self.last_document = Some(document.clone());
        self.awaiting_rebaseline = false;
    }

    pub fn trim_to_limit(&mut self, max_entries: usize) {
        if self.past.len() > max_entries {
            let trim = self.past.len() - max_entries;
            self.past.drain(0..trim);
        }
    }
}

/// Feature flag that enables or disables persistence systems.
#[derive(Resource, Debug, Clone, Copy)]
pub struct GraphPersistenceActivation {
    pub enabled: bool,
}

impl Default for GraphPersistenceActivation {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphPersistenceStatusSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct GraphPersistenceStatusMessage {
    pub text: String,
    pub severity: GraphPersistenceStatusSeverity,
    pub expires_at_secs: f64,
}

/// Current status message shown by persistence UI.
#[derive(Resource, Debug, Clone, Default)]
pub struct GraphPersistenceStatus {
    pub active: Option<GraphPersistenceStatusMessage>,
}

#[derive(Message, Debug, Clone, Copy, Default)]
pub struct SaveGraphRequest;

#[derive(Message, Debug, Clone, Copy, Default)]
pub struct LoadGraphRequest;

#[derive(Message, Debug, Clone)]
pub struct SaveGraphToPathRequest {
    pub path: String,
}

#[derive(Message, Debug, Clone)]
pub struct LoadGraphFromPathRequest {
    pub path: String,
    pub force_if_dirty: bool,
}

#[derive(Message, Debug, Clone)]
pub struct ApplyGraphDocumentRequest {
    pub document: GraphDocument,
    pub source_label: String,
    pub track_for_undo: bool,
}

#[derive(Resource, Default)]
pub(super) struct PendingGraphLoad {
    pub is_pending: bool,
    pub source_label: Option<String>,
    pub migration_note: Option<String>,
    pub requires_resave_after_migration: bool,
    pub origin: PendingGraphApplyOrigin,
    pub node_map: HashMap<u64, Entity>,
    pub node_inputs: Vec<(Entity, Vec<NodeValue>)>,
    pub edges: Vec<GraphDocumentEdge>,
    pub selected_node_ids: Vec<u64>,
    pub camera: Option<GraphDocumentCameraState>,
    pub placeholder_count: usize,
    pub validation_issue_count: usize,
}

impl PendingGraphLoad {
    pub fn reset(&mut self) {
        self.is_pending = false;
        self.source_label = None;
        self.migration_note = None;
        self.requires_resave_after_migration = false;
        self.origin = PendingGraphApplyOrigin::Load;
        self.node_map.clear();
        self.node_inputs.clear();
        self.edges.clear();
        self.selected_node_ids.clear();
        self.camera = None;
        self.placeholder_count = 0;
        self.validation_issue_count = 0;
    }
}

#[derive(SystemParam)]
pub(super) struct MutationUiState<'w> {
    pub drag_state: ResMut<'w, DragState>,
    pub wire_state: ResMut<'w, WireConnectionState>,
    pub popup: ResMut<'w, NodePopupState>,
    pub menu_state: ResMut<'w, ContextMenuState>,
    pub overlay: ResMut<'w, GraphOverlayState>,
}

impl MutationUiState<'_> {
    pub fn reset(&mut self) {
        self.drag_state.clear();
        self.wire_state.clear();
        self.popup.open_for = None;
        self.menu_state.is_open = false;
        self.menu_state.mode = univis_editor_ui::menu::ContextMenuMode::Actions;
        self.menu_state.search_query.clear();
        self.overlay.active_surface = GraphOverlaySurface::None;
    }
}

#[derive(SystemParam)]
pub(super) struct LoadGraphRuntimeParams<'w> {
    pub settings: ResMut<'w, GraphPersistenceSettings>,
    pub live_document: ResMut<'w, LiveGraphDocumentState>,
    pub pending: ResMut<'w, PendingGraphLoad>,
    pub runtime: ResMut<'w, GraphPersistenceRuntimeState>,
    pub history: ResMut<'w, GraphHistoryState>,
    pub mutation_tracker: ResMut<'w, GraphMutationTracker>,
    pub status: ResMut<'w, GraphPersistenceStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum PendingGraphApplyOrigin {
    #[default]
    Load,
    Mutation,
    Undo,
    Redo,
}

pub(super) fn set_persistence_status(
    status: &mut GraphPersistenceStatus,
    severity: GraphPersistenceStatusSeverity,
    text: String,
    now_secs: f64,
    duration_secs: f32,
) {
    status.active = Some(GraphPersistenceStatusMessage {
        text,
        severity,
        expires_at_secs: now_secs + duration_secs as f64,
    });
}

pub(super) fn graph_persistence_enabled(activation: Option<&GraphPersistenceActivation>) -> bool {
    activation
        .map(|activation| activation.enabled)
        .unwrap_or(true)
}
