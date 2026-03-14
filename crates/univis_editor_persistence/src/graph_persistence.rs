//! Graph persistence resources and systems.
use crate::format::{
    graph_document_signature, parse_graph_document_payload, prepare_graph_document_write,
    serialize_graph_document, ParsedGraphDocument, PreparedGraphWrite,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use univis_editor_ui::menu::ContextMenuState;
use univis_editor_ui::node_popup::NodePopupState;
use univis_editor_ui::node_spawn::{
    spawn_node_from_definition_entity, spawn_placeholder_node_entity,
};
use univis_editor_ui::prelude::GraphCamera;
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
        self.last_signature = graph_document_signature(document).ok();
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
struct PendingGraphLoad {
    is_pending: bool,
    source_label: Option<String>,
    origin: PendingGraphApplyOrigin,
    node_map: HashMap<u64, Entity>,
    node_inputs: Vec<(Entity, Vec<NodeValue>)>,
    edges: Vec<GraphDocumentEdge>,
    selected_node_ids: Vec<u64>,
    camera: Option<GraphDocumentCameraState>,
    placeholder_count: usize,
    validation_issue_count: usize,
}

#[derive(SystemParam)]
struct MutationUiState<'w> {
    drag_state: ResMut<'w, DragState>,
    wire_state: ResMut<'w, WireConnectionState>,
    popup: ResMut<'w, NodePopupState>,
    menu_state: ResMut<'w, ContextMenuState>,
    overlay: ResMut<'w, GraphOverlayState>,
}

impl MutationUiState<'_> {
    fn reset(&mut self) {
        self.drag_state.clear();
        self.wire_state.clear();
        self.popup.open_for = None;
        self.menu_state.is_open = false;
        self.menu_state.search_query.clear();
        self.overlay.active_surface = GraphOverlaySurface::None;
    }
}

#[derive(SystemParam)]
struct LoadGraphRuntimeParams<'w> {
    settings: ResMut<'w, GraphPersistenceSettings>,
    graph: ResMut<'w, Connecting>,
    live_document: ResMut<'w, LiveGraphDocumentState>,
    pending: ResMut<'w, PendingGraphLoad>,
    runtime: ResMut<'w, GraphPersistenceRuntimeState>,
    history: ResMut<'w, GraphHistoryState>,
    mutation_tracker: ResMut<'w, GraphMutationTracker>,
    status: ResMut<'w, GraphPersistenceStatus>,
}

impl PendingGraphLoad {
    fn reset(&mut self) {
        self.is_pending = false;
        self.source_label = None;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PendingGraphApplyOrigin {
    #[default]
    Load,
    Mutation,
    Undo,
    Redo,
}

/// Plugin that wires save/load and autosave systems into the app.
pub struct GraphPersistencePlugin;

impl Plugin for GraphPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphPersistenceSettings>()
            .init_resource::<GraphPersistenceRuntimeState>()
            .init_resource::<GraphHistorySettings>()
            .init_resource::<GraphHistoryState>()
            .init_resource::<GraphPersistenceActivation>()
            .init_resource::<GraphPersistenceStatus>()
            .init_resource::<PendingGraphLoad>()
            .add_message::<SaveGraphRequest>()
            .add_message::<LoadGraphRequest>()
            .add_message::<SaveGraphToPathRequest>()
            .add_message::<LoadGraphFromPathRequest>()
            .add_message::<ApplyGraphDocumentRequest>()
            .add_systems(
                PreUpdate,
                (
                    graph_persistence_shortcuts,
                    handle_history_requests,
                    handle_save_graph_requests,
                    handle_load_graph_requests,
                )
                    .chain(),
            )
            .add_systems(PreUpdate, handle_apply_graph_document_requests)
            .add_systems(
                PostUpdate,
                (
                    finalize_pending_graph_load,
                    capture_graph_history_snapshot
                        .after(univis_editor_ui::interaction::sync_live_graph_document_state),
                    refresh_dirty_state,
                    autosave_dirty_graph,
                    expire_persistence_status,
                )
                    .chain(),
            );
    }
}

fn graph_persistence_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    activation: Option<Res<GraphPersistenceActivation>>,
    mut command_writer: MessageWriter<GraphCommandRequest>,
) {
    if !graph_persistence_enabled(activation.as_deref()) {
        return;
    }

    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift_pressed = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if ctrl_pressed && !shift_pressed && keys.just_pressed(KeyCode::KeyS) {
        command_writer.write(GraphCommandRequest::SaveGraph);
    }

    if ctrl_pressed && !shift_pressed && keys.just_pressed(KeyCode::KeyZ) {
        command_writer.write(GraphCommandRequest::UndoGraphChange);
    }

    if ctrl_pressed
        && (keys.just_pressed(KeyCode::KeyY) || (shift_pressed && keys.just_pressed(KeyCode::KeyZ)))
    {
        command_writer.write(GraphCommandRequest::RedoGraphChange);
    }

    if ctrl_pressed && shift_pressed && keys.just_pressed(KeyCode::KeyS) {
        let stamped_path = format!("assets/graphs/graph_{}.json", unix_timestamp_millis());
        command_writer.write(GraphCommandRequest::SaveGraphToPath { path: stamped_path });
    }

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyO) {
        command_writer.write(GraphCommandRequest::LoadGraph);
    }
}

fn handle_history_requests(
    mut commands: Commands,
    mut command_requests: MessageReader<GraphCommandRequest>,
    activation: Option<Res<GraphPersistenceActivation>>,
    history_settings: Res<GraphHistorySettings>,
    registry: Res<NodeRegistry>,
    mut graph: ResMut<Connecting>,
    q_existing_nodes: Query<Entity, With<GraphNode>>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut pending: ResMut<PendingGraphLoad>,
    mut history: ResMut<GraphHistoryState>,
    mut mutation_tracker: ResMut<GraphMutationTracker>,
    mut status: ResMut<GraphPersistenceStatus>,
    mut ui_state: MutationUiState,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    if !graph_persistence_enabled(activation.as_deref()) || !history_settings.enabled {
        return;
    }

    if pending.is_pending {
        return;
    }

    let mut origin = None;
    for command in command_requests.read() {
        match command {
            GraphCommandRequest::UndoGraphChange => origin = Some(PendingGraphApplyOrigin::Undo),
            GraphCommandRequest::RedoGraphChange => origin = Some(PendingGraphApplyOrigin::Redo),
            _ => {}
        }
    }

    let Some(origin) = origin else {
        return;
    };

    let current_document = live_document.document.clone();
    let target_document = match origin {
        PendingGraphApplyOrigin::Undo => {
            let Some(previous) = history.past.pop() else {
                set_persistence_status(
                    &mut status,
                    GraphPersistenceStatusSeverity::Warning,
                    "Nothing to undo.".to_string(),
                    time.elapsed_secs_f64(),
                    settings.status_duration_secs,
                );
                return;
            };
            history.future.push(current_document);
            previous
        }
        PendingGraphApplyOrigin::Redo => {
            let Some(next) = history.future.pop() else {
                set_persistence_status(
                    &mut status,
                    GraphPersistenceStatusSeverity::Warning,
                    "Nothing to redo.".to_string(),
                    time.elapsed_secs_f64(),
                    settings.status_duration_secs,
                );
                return;
            };
            history.past.push(current_document);
            history.trim_to_limit(history_settings.max_entries);
            next
        }
        PendingGraphApplyOrigin::Mutation => return,
        PendingGraphApplyOrigin::Load => return,
    };

    let validation_issue_count = validate_graph_document(&target_document, &registry).len();
    live_document.document.prefabs = target_document.prefabs.clone();
    live_document.document.subgraphs = target_document.subgraphs.clone();
    ui_state.reset();
    stage_graph_document_apply(
        &mut commands,
        &registry,
        &mut graph,
        &q_existing_nodes,
        &mut pending,
        target_document.clone(),
        origin,
        match origin {
            PendingGraphApplyOrigin::Mutation => "graph snapshot".to_string(),
            PendingGraphApplyOrigin::Undo => "undo snapshot".to_string(),
            PendingGraphApplyOrigin::Redo => "redo snapshot".to_string(),
            PendingGraphApplyOrigin::Load => settings.file_path.clone(),
        },
        validation_issue_count,
    );

    history.awaiting_rebaseline = true;
    history.last_document = Some(target_document.clone());
    history.last_signature = graph_document_signature(&target_document).ok();
    mutation_tracker.capture_requested = false;

    set_persistence_status(
        &mut status,
        GraphPersistenceStatusSeverity::Info,
        match origin {
            PendingGraphApplyOrigin::Mutation => "Applying graph snapshot...".to_string(),
            PendingGraphApplyOrigin::Undo => "Applying undo snapshot...".to_string(),
            PendingGraphApplyOrigin::Redo => "Applying redo snapshot...".to_string(),
            PendingGraphApplyOrigin::Load => "Applying graph snapshot...".to_string(),
        },
        time.elapsed_secs_f64(),
        settings.status_duration_secs,
    );
}

fn handle_save_graph_requests(
    mut save_requests: MessageReader<SaveGraphRequest>,
    mut save_to_path_requests: MessageReader<SaveGraphToPathRequest>,
    mut command_requests: MessageReader<GraphCommandRequest>,
    activation: Option<Res<GraphPersistenceActivation>>,
    mut settings: ResMut<GraphPersistenceSettings>,
    registry: Res<NodeRegistry>,
    graph: Res<Connecting>,
    q_nodes: Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: Query<(&Transform, &Projection), With<GraphCamera>>,
    live_document: Res<LiveGraphDocumentState>,
    mut runtime: ResMut<GraphPersistenceRuntimeState>,
    mut status: ResMut<GraphPersistenceStatus>,
    time: Res<Time>,
) {
    if !graph_persistence_enabled(activation.as_deref()) {
        return;
    }

    let mut target_paths = Vec::new();
    let mut save_current = false;

    for request in save_to_path_requests.read() {
        settings.file_path = request.path.clone();
        target_paths.push(request.path.clone());
    }

    for command in command_requests.read() {
        match command {
            GraphCommandRequest::SaveGraph => {
                save_current = true;
            }
            GraphCommandRequest::SaveGraphToPath { path } => {
                settings.file_path = path.clone();
                target_paths.push(path.clone());
            }
            _ => {}
        }
    }

    for _ in save_requests.read() {
        save_current = true;
    }
    if save_current {
        target_paths.push(settings.file_path.clone());
    }

    if target_paths.is_empty() {
        return;
    }

    for path in target_paths {
        match persist_graph_to_path(
            &path,
            settings.pretty_json,
            &registry,
            &graph,
            &q_nodes,
            &q_camera,
            &live_document.document,
        ) {
            Ok(prepared) => {
                runtime.last_saved_signature = Some(prepared.signature);
                runtime.dirty = false;
                runtime.initialized = true;
                runtime.autosave_elapsed_secs = 0.0;
                runtime.open_confirm_until_secs = None;

                if prepared.validation_issue_count > 0 {
                    set_persistence_status(
                        &mut status,
                        GraphPersistenceStatusSeverity::Warning,
                        format!(
                            "Graph saved to {} with {} validation issue(s).",
                            path, prepared.validation_issue_count
                        ),
                        time.elapsed_secs_f64(),
                        settings.status_duration_secs,
                    );
                } else {
                    set_persistence_status(
                        &mut status,
                        GraphPersistenceStatusSeverity::Info,
                        format!("Graph saved to {}", path),
                        time.elapsed_secs_f64(),
                        settings.status_duration_secs,
                    );
                }
            }
            Err(err) => {
                warn!("Failed to save graph to {}: {}", path, err);
                set_persistence_status(
                    &mut status,
                    GraphPersistenceStatusSeverity::Error,
                    format!("Save failed: {}", err),
                    time.elapsed_secs_f64(),
                    settings.status_duration_secs,
                );
            }
        }
    }
}

fn handle_load_graph_requests(
    mut commands: Commands,
    mut load_requests: MessageReader<LoadGraphRequest>,
    mut load_from_path_requests: MessageReader<LoadGraphFromPathRequest>,
    mut command_requests: MessageReader<GraphCommandRequest>,
    activation: Option<Res<GraphPersistenceActivation>>,
    registry: Res<NodeRegistry>,
    q_existing_nodes: Query<Entity, With<GraphNode>>,
    mut load_runtime: LoadGraphRuntimeParams,
    mut ui_state: MutationUiState,
    time: Res<Time>,
) {
    if !graph_persistence_enabled(activation.as_deref()) {
        return;
    }

    let mut requested: Option<(String, bool)> = None;
    let mut load_current = false;

    for request in load_from_path_requests.read() {
        requested = Some((request.path.clone(), request.force_if_dirty));
    }

    for command in command_requests.read() {
        match command {
            GraphCommandRequest::LoadGraph => {
                load_current = true;
            }
            GraphCommandRequest::LoadGraphFromPath {
                path,
                force_if_dirty,
            } => {
                requested = Some((path.clone(), *force_if_dirty));
            }
            _ => {}
        }
    }

    for _ in load_requests.read() {
        load_current = true;
    }
    if load_current {
        requested = Some((load_runtime.settings.file_path.clone(), false));
    }

    let Some((path, force_if_dirty)) = requested else {
        return;
    };

    let now = time.elapsed_secs_f64();
    if load_runtime.runtime.dirty && !force_if_dirty {
        let confirmed = load_runtime
            .runtime
            .open_confirm_until_secs
            .map(|deadline| now <= deadline)
            .unwrap_or(false);

        if !confirmed {
            load_runtime.runtime.open_confirm_until_secs =
                Some(now + load_runtime.settings.confirm_reload_window_secs as f64);
            set_persistence_status(
                &mut load_runtime.status,
                GraphPersistenceStatusSeverity::Warning,
                "Unsaved changes detected. Press Ctrl+O again to confirm reload.".to_string(),
                now,
                load_runtime.settings.status_duration_secs,
            );
            return;
        }
    }

    load_runtime.runtime.open_confirm_until_secs = None;
    load_runtime.settings.file_path = path.clone();

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            warn!("Failed to read graph file {}: {}", path, err);
            set_persistence_status(
                &mut load_runtime.status,
                GraphPersistenceStatusSeverity::Error,
                format!("Open failed: cannot read {}", path),
                now,
                load_runtime.settings.status_duration_secs,
            );
            return;
        }
    };

    let ParsedGraphDocument {
        document: save_file,
        migration_note,
    } = match parse_graph_document_payload(&content) {
        Ok(result) => result,
        Err(err) => {
            warn!("Failed to parse/migrate graph JSON {}: {}", path, err);
            set_persistence_status(
                &mut load_runtime.status,
                GraphPersistenceStatusSeverity::Error,
                format!("Open failed: {}", err),
                now,
                load_runtime.settings.status_duration_secs,
            );
            return;
        }
    };
    let validation_issues = validate_graph_document(&save_file, &registry);
    if !validation_issues.is_empty() {
        warn!(
            "Loaded graph document {} with {} validation issue(s)",
            path,
            validation_issues.len()
        );
        for issue in validation_issues.iter().take(5) {
            warn!("Graph validation: {}", issue.message);
        }
    }

    load_runtime.live_document.document.prefabs = save_file.prefabs.clone();
    load_runtime.live_document.document.subgraphs = save_file.subgraphs.clone();
    load_runtime.pending.reset();
    ui_state.reset();
    stage_graph_document_apply(
        &mut commands,
        &registry,
        &mut load_runtime.graph,
        &q_existing_nodes,
        &mut load_runtime.pending,
        save_file.clone(),
        PendingGraphApplyOrigin::Load,
        path.clone(),
        validation_issues.len(),
    );
    load_runtime.history.clear();
    load_runtime.history.awaiting_rebaseline = true;
    load_runtime.history.last_document = Some(save_file);
    load_runtime.mutation_tracker.capture_requested = false;

    if let Some(note) = migration_note {
        set_persistence_status(
            &mut load_runtime.status,
            GraphPersistenceStatusSeverity::Info,
            note,
            now,
            load_runtime.settings.status_duration_secs,
        );
    } else {
        set_persistence_status(
            &mut load_runtime.status,
            GraphPersistenceStatusSeverity::Info,
            format!("Loading graph from {}", path),
            now,
            load_runtime.settings.status_duration_secs,
        );
    }
}

fn handle_apply_graph_document_requests(
    mut commands: Commands,
    mut apply_requests: MessageReader<ApplyGraphDocumentRequest>,
    activation: Option<Res<GraphPersistenceActivation>>,
    registry: Res<NodeRegistry>,
    mut graph: ResMut<Connecting>,
    q_existing_nodes: Query<Entity, With<GraphNode>>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut pending: ResMut<PendingGraphLoad>,
    mut history: ResMut<GraphHistoryState>,
    mut mutation_tracker: ResMut<GraphMutationTracker>,
    mut status: ResMut<GraphPersistenceStatus>,
    mut ui_state: MutationUiState,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    if !graph_persistence_enabled(activation.as_deref()) {
        return;
    }

    let Some(request) = apply_requests.read().last().cloned() else {
        return;
    };

    let validation_issue_count = validate_graph_document(&request.document, &registry).len();
    live_document.document.prefabs = request.document.prefabs.clone();
    live_document.document.subgraphs = request.document.subgraphs.clone();
    ui_state.reset();
    stage_graph_document_apply(
        &mut commands,
        &registry,
        &mut graph,
        &q_existing_nodes,
        &mut pending,
        request.document,
        PendingGraphApplyOrigin::Mutation,
        request.source_label.clone(),
        validation_issue_count,
    );

    if request.track_for_undo {
        mutation_tracker.capture_requested = true;
    }
    history.awaiting_rebaseline = false;

    set_persistence_status(
        &mut status,
        if validation_issue_count > 0 {
            GraphPersistenceStatusSeverity::Warning
        } else {
            GraphPersistenceStatusSeverity::Info
        },
        format!("Applying {}...", request.source_label),
        time.elapsed_secs_f64(),
        settings.status_duration_secs,
    );
}

fn finalize_pending_graph_load(
    mut commands: Commands,
    mut pending: ResMut<PendingGraphLoad>,
    activation: Option<Res<GraphPersistenceActivation>>,
    mut graph: ResMut<Connecting>,
    q_ports: Query<(Entity, &GraphPort)>,
    mut q_nodes: Query<&mut GraphNode>,
    mut q_camera: Query<(&mut Transform, &mut Projection), With<GraphCamera>>,
    mut runtime: ResMut<GraphPersistenceRuntimeState>,
    mut status: ResMut<GraphPersistenceStatus>,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    if !graph_persistence_enabled(activation.as_deref()) {
        return;
    }

    if !pending.is_pending {
        return;
    }

    let mut skipped_link_count = 0usize;
    let mut input_ports: HashMap<(Entity, usize), Entity> = HashMap::new();
    let mut output_ports: HashMap<(Entity, usize), Entity> = HashMap::new();

    for (port_entity, graph_port) in q_ports.iter() {
        match graph_port.port_type {
            PortType::Input => {
                input_ports.insert((graph_port.node_entity, graph_port.index), port_entity);
            }
            PortType::Output => {
                output_ports.insert((graph_port.node_entity, graph_port.index), port_entity);
            }
        }
    }

    graph.connections.clear();
    for edge in &pending.edges {
        let Some(from_node) = pending.node_map.get(&edge.from_node_id).copied() else {
            skipped_link_count += 1;
            continue;
        };
        let Some(to_node) = pending.node_map.get(&edge.to_node_id).copied() else {
            skipped_link_count += 1;
            continue;
        };

        let Some(from_port) = output_ports.get(&(from_node, edge.from_index)).copied() else {
            skipped_link_count += 1;
            continue;
        };
        let Some(to_port) = input_ports.get(&(to_node, edge.to_index)).copied() else {
            skipped_link_count += 1;
            continue;
        };

        let Ok((_, from_port_data)) = q_ports.get(from_port) else {
            skipped_link_count += 1;
            continue;
        };
        let Ok((_, to_port_data)) = q_ports.get(to_port) else {
            skipped_link_count += 1;
            continue;
        };

        if from_port_data.port_type != PortType::Output || to_port_data.port_type != PortType::Input
        {
            skipped_link_count += 1;
            continue;
        }

        if !NodeValue::is_compatible(&from_port_data.value_type, &to_port_data.value_type) {
            warn!(
                "Skipping incompatible loaded link: {} -> {}",
                from_port_data.value_type.display_name(),
                to_port_data.value_type.display_name()
            );
            skipped_link_count += 1;
            continue;
        }

        if graph
            .connections
            .iter()
            .any(|existing| existing.to_port == to_port)
        {
            warn!(
                "Skipping loaded link: input port already connected (node {:?}, input {})",
                to_node, edge.to_index
            );
            skipped_link_count += 1;
            continue;
        }

        graph.connections.push(GraphLink {
            from_node,
            from_index: edge.from_index,
            to_node,
            to_index: edge.to_index,
            from_port,
            to_port,
        });
    }

    for (entity, inputs) in pending.node_inputs.drain(..) {
        let Ok(mut node) = q_nodes.get_mut(entity) else {
            continue;
        };

        for (index, value) in inputs.into_iter().enumerate() {
            if index < node.values.inputs.len() {
                node.values.inputs[index] = value;
            }
        }
    }

    let selected_ids: Vec<u64> = pending.selected_node_ids.drain(..).collect();
    for selected_id in selected_ids {
        if let Some(entity) = pending.node_map.get(&selected_id).copied() {
            commands.entity(entity).try_insert(Selected);
        }
    }

    if let Some(camera) = pending.camera.take() {
        if let Ok((mut transform, mut projection)) = q_camera.single_mut() {
            transform.translation = Vec3::new(
                camera.translation[0],
                camera.translation[1],
                camera.translation[2],
            );
            if let Projection::Orthographic(ref mut ortho) = *projection {
                ortho.scale = camera.ortho_scale.max(0.01);
            }
        }
    }

    runtime.needs_rebaseline = true;
    runtime.autosave_elapsed_secs = 0.0;
    runtime.open_confirm_until_secs = None;

    let source_label = pending
        .source_label
        .clone()
        .unwrap_or_else(|| settings.file_path.clone());

    if pending.placeholder_count > 0 || skipped_link_count > 0 || pending.validation_issue_count > 0
    {
        set_persistence_status(
            &mut status,
            GraphPersistenceStatusSeverity::Warning,
            format!(
                "{} with {} placeholder node(s), {} skipped link(s), and {} validation issue(s).",
                pending_completion_prefix(pending.origin, &source_label),
                pending.placeholder_count,
                skipped_link_count,
                pending.validation_issue_count
            ),
            time.elapsed_secs_f64(),
            settings.status_duration_secs,
        );
    } else {
        set_persistence_status(
            &mut status,
            GraphPersistenceStatusSeverity::Info,
            pending_completion_success(pending.origin, &source_label),
            time.elapsed_secs_f64(),
            settings.status_duration_secs,
        );
    }

    pending.reset();
}

fn pending_completion_prefix(origin: PendingGraphApplyOrigin, source_label: &str) -> String {
    match origin {
        PendingGraphApplyOrigin::Load => format!("Loaded {}", source_label),
        PendingGraphApplyOrigin::Mutation => format!("Applied {}", source_label),
        PendingGraphApplyOrigin::Undo => "Undo restored graph snapshot".to_string(),
        PendingGraphApplyOrigin::Redo => "Redo restored graph snapshot".to_string(),
    }
}

fn pending_completion_success(origin: PendingGraphApplyOrigin, source_label: &str) -> String {
    match origin {
        PendingGraphApplyOrigin::Load => format!("Graph loaded successfully from {}", source_label),
        PendingGraphApplyOrigin::Mutation => format!("Applied {}", source_label),
        PendingGraphApplyOrigin::Undo => "Undo restored graph snapshot".to_string(),
        PendingGraphApplyOrigin::Redo => "Redo restored graph snapshot".to_string(),
    }
}

fn capture_graph_history_snapshot(
    activation: Option<Res<GraphPersistenceActivation>>,
    settings: Res<GraphHistorySettings>,
    live_document: Res<LiveGraphDocumentState>,
    mut history: ResMut<GraphHistoryState>,
    mut mutation_tracker: ResMut<GraphMutationTracker>,
) {
    if !graph_persistence_enabled(activation.as_deref()) || !settings.enabled {
        mutation_tracker.capture_requested = false;
        return;
    }

    let Ok(current_signature) = graph_document_signature(&live_document.document) else {
        mutation_tracker.capture_requested = false;
        return;
    };

    if history.awaiting_rebaseline || history.last_signature.is_none() {
        history.rebaseline_to_document(&live_document.document);
        mutation_tracker.capture_requested = false;
        return;
    }

    if history
        .last_signature
        .as_ref()
        .is_some_and(|signature| signature == &current_signature)
    {
        mutation_tracker.capture_requested = false;
        return;
    }

    if mutation_tracker.capture_requested {
        if let Some(previous) = history.last_document.take() {
            history.past.push(previous);
            history.trim_to_limit(settings.max_entries);
        }
        history.future.clear();
    }

    history.last_document = Some(live_document.document.clone());
    history.last_signature = Some(current_signature);
    mutation_tracker.capture_requested = false;
}

fn refresh_dirty_state(
    graph: Res<Connecting>,
    activation: Option<Res<GraphPersistenceActivation>>,
    q_nodes: Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: Query<(&Transform, &Projection), With<GraphCamera>>,
    live_document: Res<LiveGraphDocumentState>,
    mut runtime: ResMut<GraphPersistenceRuntimeState>,
) {
    if !graph_persistence_enabled(activation.as_deref()) {
        runtime.autosave_elapsed_secs = 0.0;
        runtime.open_confirm_until_secs = None;
        return;
    }

    let current_document =
        build_graph_document(&graph, &q_nodes, &q_camera, &live_document.document);
    let Ok(current_signature) = graph_document_signature(&current_document) else {
        return;
    };

    if !runtime.initialized || runtime.last_saved_signature.is_none() || runtime.needs_rebaseline {
        runtime.last_saved_signature = Some(current_signature);
        runtime.dirty = false;
        runtime.initialized = true;
        runtime.needs_rebaseline = false;
        return;
    }

    runtime.dirty = runtime
        .last_saved_signature
        .as_ref()
        .map(|saved| saved != &current_signature)
        .unwrap_or(false);
}

fn autosave_dirty_graph(
    time: Res<Time>,
    settings: Res<GraphPersistenceSettings>,
    activation: Option<Res<GraphPersistenceActivation>>,
    registry: Res<NodeRegistry>,
    graph: Res<Connecting>,
    q_nodes: Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: Query<(&Transform, &Projection), With<GraphCamera>>,
    live_document: Res<LiveGraphDocumentState>,
    mut runtime: ResMut<GraphPersistenceRuntimeState>,
    mut status: ResMut<GraphPersistenceStatus>,
) {
    if !graph_persistence_enabled(activation.as_deref()) {
        runtime.autosave_elapsed_secs = 0.0;
        return;
    }

    if !settings.autosave_enabled {
        return;
    }

    if !runtime.dirty {
        runtime.autosave_elapsed_secs = 0.0;
        return;
    }

    runtime.autosave_elapsed_secs += time.delta_secs();
    if runtime.autosave_elapsed_secs < settings.autosave_interval_secs {
        return;
    }
    runtime.autosave_elapsed_secs = 0.0;

    match persist_graph_to_path(
        &settings.file_path,
        settings.pretty_json,
        &registry,
        &graph,
        &q_nodes,
        &q_camera,
        &live_document.document,
    ) {
        Ok(prepared) => {
            runtime.last_saved_signature = Some(prepared.signature.clone());
            runtime.dirty = false;

            let backup_result = write_backup_file(
                &prepared.document,
                settings.pretty_json,
                &settings.backup_directory,
                settings.max_backup_files,
            );

            match backup_result {
                Ok(backup_path) => {
                    set_persistence_status(
                        &mut status,
                        if prepared.validation_issue_count > 0 {
                            GraphPersistenceStatusSeverity::Warning
                        } else {
                            GraphPersistenceStatusSeverity::Info
                        },
                        if prepared.validation_issue_count > 0 {
                            format!(
                                "Autosaved graph to {} with {} validation issue(s).",
                                backup_path.display(),
                                prepared.validation_issue_count
                            )
                        } else {
                            format!("Autosaved graph to {}", backup_path.display())
                        },
                        time.elapsed_secs_f64(),
                        settings.status_duration_secs,
                    );
                }
                Err(err) => {
                    warn!("Autosave backup warning: {}", err);
                    set_persistence_status(
                        &mut status,
                        GraphPersistenceStatusSeverity::Warning,
                        format!("Autosave completed but backup failed: {}", err),
                        time.elapsed_secs_f64(),
                        settings.status_duration_secs,
                    );
                }
            }
        }
        Err(err) => {
            warn!("Autosave failed: {}", err);
            set_persistence_status(
                &mut status,
                GraphPersistenceStatusSeverity::Error,
                format!("Autosave failed: {}", err),
                time.elapsed_secs_f64(),
                settings.status_duration_secs,
            );
        }
    }
}

fn expire_persistence_status(
    mut status: ResMut<GraphPersistenceStatus>,
    activation: Option<Res<GraphPersistenceActivation>>,
    time: Res<Time>,
) {
    if !graph_persistence_enabled(activation.as_deref()) {
        status.active = None;
        return;
    }

    if let Some(active) = &status.active {
        if time.elapsed_secs_f64() > active.expires_at_secs {
            status.active = None;
        }
    }
}

fn set_persistence_status(
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

fn graph_persistence_enabled(activation: Option<&GraphPersistenceActivation>) -> bool {
    activation
        .map(|activation| activation.enabled)
        .unwrap_or(true)
}

fn stage_graph_document_apply(
    commands: &mut Commands,
    registry: &NodeRegistry,
    graph: &mut Connecting,
    q_existing_nodes: &Query<Entity, With<GraphNode>>,
    pending: &mut PendingGraphLoad,
    document: GraphDocument,
    origin: PendingGraphApplyOrigin,
    source_label: String,
    validation_issue_count: usize,
) {
    pending.reset();

    for entity in q_existing_nodes.iter() {
        commands.entity(entity).try_despawn();
    }
    graph.connections.clear();

    let GraphDocument {
        nodes, edges, view, ..
    } = document;

    for saved_node in nodes {
        let position = Vec2::new(saved_node.position[0], saved_node.position[1]);

        let spawned = if let Some(definition) = registry.get(&saved_node.definition_id) {
            let expected_inputs = definition.inputs().len();
            let expected_outputs = definition.outputs().len();

            if expected_inputs == saved_node.input_count
                && expected_outputs == saved_node.output_count
            {
                spawn_node_from_definition_entity(commands, &definition, position)
            } else {
                warn!(
                    "Node definition {} port mismatch (saved {}/{}, runtime {}/{}) - using placeholder",
                    saved_node.definition_id,
                    saved_node.input_count,
                    saved_node.output_count,
                    expected_inputs,
                    expected_outputs
                );
                pending.placeholder_count += 1;
                spawn_placeholder_node_entity(
                    commands,
                    &saved_node.definition_id,
                    position,
                    saved_node.input_count,
                    saved_node.output_count,
                )
            }
        } else {
            warn!(
                "Node definition {} not found during apply - using placeholder",
                saved_node.definition_id
            );
            pending.placeholder_count += 1;
            spawn_placeholder_node_entity(
                commands,
                &saved_node.definition_id,
                position,
                saved_node.input_count,
                saved_node.output_count,
            )
        };

        pending.node_map.insert(saved_node.id, spawned);
        pending.node_inputs.push((spawned, saved_node.inputs));
    }

    pending.edges = edges;
    pending.selected_node_ids = view.selected_node_ids;
    pending.camera = view.camera;
    pending.source_label = Some(source_label);
    pending.origin = origin;
    pending.is_pending = true;
    pending.validation_issue_count = validation_issue_count;
}

fn persist_graph_to_path(
    path: &str,
    pretty_json: bool,
    registry: &NodeRegistry,
    graph: &Connecting,
    q_nodes: &Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: &Query<(&Transform, &Projection), With<GraphCamera>>,
    source_document: &GraphDocument,
) -> Result<PreparedGraphWrite, String> {
    let document = build_graph_document(graph, q_nodes, q_camera, source_document);
    let prepared = prepare_graph_document_write(document, pretty_json, registry)?;
    write_graph_payload(path, &prepared.payload)?;
    Ok(prepared)
}

fn build_graph_document(
    graph: &Connecting,
    q_nodes: &Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: &Query<(&Transform, &Projection), With<GraphCamera>>,
    source_document: &GraphDocument,
) -> GraphDocument {
    let mut nodes_data = Vec::new();
    for (entity, node, transform, selected) in q_nodes.iter() {
        nodes_data.push(GraphDocumentNodeSnapshot {
            entity,
            definition_id: node.definition_id.clone(),
            position: [transform.translation.x, transform.translation.y],
            inputs: node.values.inputs.clone(),
            input_count: node.values.inputs.len(),
            output_count: node.values.outputs.len(),
            selected: selected.is_some(),
        });
    }

    let camera = q_camera
        .iter()
        .next()
        .map(|(transform, projection)| GraphDocumentCameraState {
            translation: [
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ],
            ortho_scale: match projection {
                Projection::Orthographic(ortho) => ortho.scale,
                _ => 1.0,
            },
        });

    let edge_snapshots = graph
        .connections
        .iter()
        .map(|link| GraphDocumentEdgeSnapshot {
            from_entity: link.from_node,
            from_index: link.from_index,
            to_entity: link.to_node,
            to_index: link.to_index,
        });

    let mut document =
        build_graph_document_from_snapshots(nodes_data, edge_snapshots, camera, None).document;
    document.prefabs = source_document.prefabs.clone();
    document.subgraphs = source_document.subgraphs.clone();
    document
}

fn write_graph_payload(path: &str, payload: &str) -> Result<(), String> {
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("cannot create save directory {}: {}", parent.display(), err))?;
    }

    fs::write(target, payload)
        .map_err(|err| format!("cannot write graph file {}: {}", target.display(), err))
}

fn write_backup_file(
    document: &GraphDocument,
    pretty_json: bool,
    backup_directory: &str,
    max_backup_files: usize,
) -> Result<PathBuf, String> {
    let backup_dir = Path::new(backup_directory);
    fs::create_dir_all(backup_dir).map_err(|err| {
        format!(
            "cannot create backup directory {}: {}",
            backup_dir.display(),
            err
        )
    })?;

    let backup_name = format!("autosave_{}.json", unix_timestamp_millis());
    let backup_path = backup_dir.join(backup_name);
    let payload = serialize_graph_document(document, pretty_json)?;

    write_graph_payload(
        backup_path
            .to_str()
            .ok_or_else(|| "backup path is not valid UTF-8".to_string())?,
        &payload,
    )?;

    prune_backup_files(backup_dir, max_backup_files)?;

    Ok(backup_path)
}

pub fn latest_backup_file(backup_directory: &str) -> Result<Option<PathBuf>, String> {
    let backup_dir = Path::new(backup_directory);
    if !backup_dir.exists() {
        return Ok(None);
    }

    let mut files = collect_backup_files(backup_dir)?;
    if files.is_empty() {
        return Ok(None);
    }

    files.sort_by_key(|(_, modified)| *modified);
    Ok(files.pop().map(|(path, _)| path))
}

fn prune_backup_files(backup_dir: &Path, max_backup_files: usize) -> Result<(), String> {
    if max_backup_files == 0 {
        return Ok(());
    }

    let mut files = collect_backup_files(backup_dir)?;

    if files.len() <= max_backup_files {
        return Ok(());
    }

    files.sort_by_key(|(_, modified)| *modified);

    let remove_count = files.len().saturating_sub(max_backup_files);
    for (path, _) in files.into_iter().take(remove_count) {
        if let Err(err) = fs::remove_file(&path) {
            warn!("Failed to prune backup file {}: {}", path.display(), err);
        }
    }

    Ok(())
}

fn collect_backup_files(backup_dir: &Path) -> Result<Vec<(PathBuf, SystemTime)>, String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(backup_dir).map_err(|err| {
        format!(
            "cannot read backup directory {}: {}",
            backup_dir.display(),
            err
        )
    })? {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let modified = entry
            .metadata()
            .ok()
            .and_then(|meta| meta.modified().ok())
            .unwrap_or(UNIX_EPOCH);

        files.push((path, modified));
    }

    Ok(files)
}

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
