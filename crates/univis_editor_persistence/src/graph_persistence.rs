//! نظام حفظ/تحميل الجراف إلى JSON

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use univis_editor_core::mode::{EditorMode, EditorModeState};
use univis_editor_core::prelude::*;
use univis_editor_ui::node_spawn::{
    spawn_node_from_definition_entity, spawn_placeholder_node_entity,
};
use univis_editor_ui::prelude::GraphCamera;

const GRAPH_SAVE_VERSION: u32 = 1;
const DEFAULT_SAVE_FILE_PATH: &str = "assets/graphs/current_graph.json";
const DEFAULT_BACKUP_DIRECTORY: &str = "assets/graphs/backups";

fn default_graph_save_version() -> u32 {
    GRAPH_SAVE_VERSION
}

/// إعدادات الحفظ/التحميل
#[derive(Resource, Debug, Clone)]
pub struct GraphPersistenceSettings {
    /// مسار ملف الجراف الحالي
    pub file_path: String,
    /// تنسيق JSON بشكل مقروء
    pub pretty_json: bool,
    /// تفعيل الحفظ التلقائي
    pub autosave_enabled: bool,
    /// فترة الحفظ التلقائي بالثواني
    pub autosave_interval_secs: f32,
    /// مجلد النسخ الاحتياطية
    pub backup_directory: String,
    /// عدد النسخ الاحتياطية القصوى
    pub max_backup_files: usize,
    /// مدة عرض رسائل الحالة في الواجهة
    pub status_duration_secs: f32,
    /// نافذة تأكيد التحميل عند وجود تغييرات غير محفوظة
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

/// حالة runtime لنظام الحفظ/التحميل
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PersistenceStatusSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
struct PersistenceStatusMessage {
    text: String,
    severity: PersistenceStatusSeverity,
    expires_at_secs: f64,
}

#[derive(Resource, Default)]
struct GraphPersistenceStatusState {
    active: Option<PersistenceStatusMessage>,
    rendered_key: Option<String>,
}

#[derive(Component)]
struct GraphPersistenceStatusUi;

/// رسالة طلب حفظ الجراف في المسار الحالي
#[derive(Message, Debug, Clone, Copy, Default)]
pub struct SaveGraphRequest;

/// رسالة طلب تحميل الجراف من المسار الحالي
#[derive(Message, Debug, Clone, Copy, Default)]
pub struct LoadGraphRequest;

/// رسالة طلب حفظ الجراف في مسار معين (Save As)
#[derive(Message, Debug, Clone)]
pub struct SaveGraphToPathRequest {
    pub path: String,
}

/// رسالة طلب تحميل الجراف من مسار معين (Open Path)
#[derive(Message, Debug, Clone)]
pub struct LoadGraphFromPathRequest {
    pub path: String,
    pub force_if_dirty: bool,
}

/// ملف الحفظ - schema v1
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphSaveFile {
    #[serde(default = "default_graph_save_version")]
    pub version: u32,
    #[serde(default)]
    pub nodes: Vec<SavedNode>,
    #[serde(default)]
    pub links: Vec<SavedLink>,
    #[serde(default)]
    pub ui: SavedUiState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedNode {
    pub id: u64,
    pub definition_id: NodeId,
    pub position: [f32; 2],
    #[serde(default)]
    pub inputs: Vec<NodeValue>,
    pub input_count: usize,
    pub output_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedLink {
    pub from_node_id: u64,
    pub from_index: usize,
    pub to_node_id: u64,
    pub to_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedUiState {
    pub camera: Option<SavedCameraState>,
    #[serde(default)]
    pub selected_node_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedCameraState {
    pub translation: [f32; 3],
    pub ortho_scale: f32,
}

/// schema v0 (بدون version، وعدادات منافذ اختيارية)
#[derive(Debug, Clone, Deserialize, Default)]
struct GraphSaveFileV0 {
    #[serde(default)]
    nodes: Vec<SavedNodeV0>,
    #[serde(default)]
    links: Vec<SavedLink>,
    #[serde(default)]
    ui: SavedUiState,
}

#[derive(Debug, Clone, Deserialize)]
struct SavedNodeV0 {
    id: u64,
    definition_id: NodeId,
    position: [f32; 2],
    #[serde(default)]
    inputs: Vec<NodeValue>,
    #[serde(default)]
    input_count: Option<usize>,
    #[serde(default)]
    output_count: Option<usize>,
}

/// حالة تحميل مرحلية لإعادة بناء الوصلات بعد Spawn
#[derive(Resource, Default)]
struct PendingGraphLoad {
    is_pending: bool,
    source_path: Option<String>,
    node_map: HashMap<u64, Entity>,
    node_inputs: Vec<(Entity, Vec<NodeValue>)>,
    links: Vec<SavedLink>,
    selected_node_ids: Vec<u64>,
    camera: Option<SavedCameraState>,
    placeholder_count: usize,
}

impl PendingGraphLoad {
    fn reset(&mut self) {
        self.is_pending = false;
        self.source_path = None;
        self.node_map.clear();
        self.node_inputs.clear();
        self.links.clear();
        self.selected_node_ids.clear();
        self.camera = None;
        self.placeholder_count = 0;
    }
}

/// Plugin الحفظ/التحميل
pub struct GraphPersistencePlugin;

impl Plugin for GraphPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphPersistenceSettings>()
            .init_resource::<GraphPersistenceRuntimeState>()
            .init_resource::<GraphPersistenceStatusState>()
            .init_resource::<PendingGraphLoad>()
            .add_message::<SaveGraphRequest>()
            .add_message::<LoadGraphRequest>()
            .add_message::<SaveGraphToPathRequest>()
            .add_message::<LoadGraphFromPathRequest>()
            .add_systems(
                PreUpdate,
                (
                    graph_persistence_shortcuts,
                    handle_save_graph_requests,
                    handle_load_graph_requests,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    finalize_pending_graph_load,
                    refresh_dirty_state,
                    autosave_dirty_graph,
                    draw_persistence_status_ui,
                )
                    .chain(),
            );
    }
}

/// اختصارات لوحة المفاتيح
/// - Ctrl+S: حفظ المسار الحالي
/// - Ctrl+Shift+S: Save As (مسار timestamped)
/// - Ctrl+O: تحميل المسار الحالي (مع تأكيد إذا dirty)
fn graph_persistence_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mode: Option<Res<EditorModeState>>,
    mut save_writer: MessageWriter<SaveGraphRequest>,
    mut save_as_writer: MessageWriter<SaveGraphToPathRequest>,
    mut load_writer: MessageWriter<LoadGraphRequest>,
) {
    if !graph_persistence_enabled(mode.as_deref()) {
        return;
    }

    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift_pressed = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if ctrl_pressed && !shift_pressed && keys.just_pressed(KeyCode::KeyS) {
        save_writer.write(SaveGraphRequest);
    }

    if ctrl_pressed && shift_pressed && keys.just_pressed(KeyCode::KeyS) {
        let stamped_path = format!("assets/graphs/graph_{}.json", unix_timestamp_millis());
        save_as_writer.write(SaveGraphToPathRequest { path: stamped_path });
    }

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyO) {
        load_writer.write(LoadGraphRequest);
    }
}

fn handle_save_graph_requests(
    mut save_requests: MessageReader<SaveGraphRequest>,
    mut save_to_path_requests: MessageReader<SaveGraphToPathRequest>,
    mode: Option<Res<EditorModeState>>,
    mut settings: ResMut<GraphPersistenceSettings>,
    graph: Res<Connecting>,
    q_nodes: Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: Query<(&Transform, &Projection), With<GraphCamera>>,
    mut runtime: ResMut<GraphPersistenceRuntimeState>,
    mut status: ResMut<GraphPersistenceStatusState>,
    time: Res<Time>,
) {
    if !graph_persistence_enabled(mode.as_deref()) {
        return;
    }

    let mut target_paths = Vec::new();

    for request in save_to_path_requests.read() {
        settings.file_path = request.path.clone();
        target_paths.push(request.path.clone());
    }

    let mut save_current = false;
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
        match persist_graph_to_path(&path, settings.pretty_json, &graph, &q_nodes, &q_camera) {
            Ok((_, signature)) => {
                runtime.last_saved_signature = Some(signature);
                runtime.dirty = false;
                runtime.initialized = true;
                runtime.autosave_elapsed_secs = 0.0;
                runtime.open_confirm_until_secs = None;

                set_persistence_status(
                    &mut status,
                    PersistenceStatusSeverity::Info,
                    format!("Graph saved to {}", path),
                    time.elapsed_secs_f64(),
                    settings.status_duration_secs,
                );
            }
            Err(err) => {
                warn!("Failed to save graph to {}: {}", path, err);
                set_persistence_status(
                    &mut status,
                    PersistenceStatusSeverity::Error,
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
    mode: Option<Res<EditorModeState>>,
    mut settings: ResMut<GraphPersistenceSettings>,
    registry: Res<NodeRegistry>,
    mut graph: ResMut<Connecting>,
    q_existing_nodes: Query<Entity, With<GraphNode>>,
    mut pending: ResMut<PendingGraphLoad>,
    mut runtime: ResMut<GraphPersistenceRuntimeState>,
    mut status: ResMut<GraphPersistenceStatusState>,
    time: Res<Time>,
) {
    if !graph_persistence_enabled(mode.as_deref()) {
        return;
    }

    let mut requested: Option<(String, bool)> = None;

    for request in load_from_path_requests.read() {
        requested = Some((request.path.clone(), request.force_if_dirty));
    }

    let mut load_current = false;
    for _ in load_requests.read() {
        load_current = true;
    }
    if load_current {
        requested = Some((settings.file_path.clone(), false));
    }

    let Some((path, force_if_dirty)) = requested else {
        return;
    };

    let now = time.elapsed_secs_f64();
    if runtime.dirty && !force_if_dirty {
        let confirmed = runtime
            .open_confirm_until_secs
            .map(|deadline| now <= deadline)
            .unwrap_or(false);

        if !confirmed {
            runtime.open_confirm_until_secs =
                Some(now + settings.confirm_reload_window_secs as f64);
            set_persistence_status(
                &mut status,
                PersistenceStatusSeverity::Warning,
                "Unsaved changes detected. Press Ctrl+O again to confirm reload.".to_string(),
                now,
                settings.status_duration_secs,
            );
            return;
        }
    }

    runtime.open_confirm_until_secs = None;
    settings.file_path = path.clone();

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            warn!("Failed to read graph file {}: {}", path, err);
            set_persistence_status(
                &mut status,
                PersistenceStatusSeverity::Error,
                format!("Open failed: cannot read {}", path),
                now,
                settings.status_duration_secs,
            );
            return;
        }
    };

    let (save_file, migration_note) = match parse_and_migrate_graph(&content) {
        Ok(result) => result,
        Err(err) => {
            warn!("Failed to parse/migrate graph JSON {}: {}", path, err);
            set_persistence_status(
                &mut status,
                PersistenceStatusSeverity::Error,
                format!("Open failed: {}", err),
                now,
                settings.status_duration_secs,
            );
            return;
        }
    };

    pending.reset();

    for entity in q_existing_nodes.iter() {
        commands.entity(entity).despawn();
    }
    graph.connections.clear();

    for saved_node in save_file.nodes {
        let position = Vec2::new(saved_node.position[0], saved_node.position[1]);

        let spawned = if let Some(definition) = registry.get(&saved_node.definition_id) {
            let expected_inputs = definition.inputs().len();
            let expected_outputs = definition.outputs().len();

            if expected_inputs == saved_node.input_count
                && expected_outputs == saved_node.output_count
            {
                spawn_node_from_definition_entity(&mut commands, &definition, position)
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
                    &mut commands,
                    &saved_node.definition_id,
                    position,
                    saved_node.input_count,
                    saved_node.output_count,
                )
            }
        } else {
            warn!(
                "Node definition {} not found during load - using placeholder",
                saved_node.definition_id
            );
            pending.placeholder_count += 1;
            spawn_placeholder_node_entity(
                &mut commands,
                &saved_node.definition_id,
                position,
                saved_node.input_count,
                saved_node.output_count,
            )
        };

        pending.node_map.insert(saved_node.id, spawned);
        pending.node_inputs.push((spawned, saved_node.inputs));
    }

    pending.links = save_file.links;
    pending.selected_node_ids = save_file.ui.selected_node_ids;
    pending.camera = save_file.ui.camera;
    pending.source_path = Some(path.clone());
    pending.is_pending = true;

    if let Some(note) = migration_note {
        set_persistence_status(
            &mut status,
            PersistenceStatusSeverity::Info,
            note,
            now,
            settings.status_duration_secs,
        );
    } else {
        set_persistence_status(
            &mut status,
            PersistenceStatusSeverity::Info,
            format!("Loading graph from {}", path),
            now,
            settings.status_duration_secs,
        );
    }
}

/// مرحلة ثانية بعد Spawn لإعادة الوصلات وحالة UI
fn finalize_pending_graph_load(
    mut commands: Commands,
    mut pending: ResMut<PendingGraphLoad>,
    mode: Option<Res<EditorModeState>>,
    mut graph: ResMut<Connecting>,
    q_ports: Query<(Entity, &GraphPort)>,
    mut q_nodes: Query<&mut GraphNode>,
    mut q_camera: Query<(&mut Transform, &mut Projection), With<GraphCamera>>,
    mut runtime: ResMut<GraphPersistenceRuntimeState>,
    mut status: ResMut<GraphPersistenceStatusState>,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    if !graph_persistence_enabled(mode.as_deref()) {
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
    for link in &pending.links {
        let Some(from_node) = pending.node_map.get(&link.from_node_id).copied() else {
            skipped_link_count += 1;
            continue;
        };
        let Some(to_node) = pending.node_map.get(&link.to_node_id).copied() else {
            skipped_link_count += 1;
            continue;
        };

        let Some(from_port) = output_ports.get(&(from_node, link.from_index)).copied() else {
            skipped_link_count += 1;
            continue;
        };
        let Some(to_port) = input_ports.get(&(to_node, link.to_index)).copied() else {
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
                to_node, link.to_index
            );
            skipped_link_count += 1;
            continue;
        }

        graph.connections.push(GraphLink {
            from_node,
            from_index: link.from_index,
            to_node,
            to_index: link.to_index,
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
            commands.entity(entity).insert(Selected);
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

    let source_path = pending
        .source_path
        .clone()
        .unwrap_or_else(|| settings.file_path.clone());

    if pending.placeholder_count > 0 || skipped_link_count > 0 {
        set_persistence_status(
            &mut status,
            PersistenceStatusSeverity::Warning,
            format!(
                "Loaded {} with {} placeholder node(s) and {} skipped link(s).",
                source_path, pending.placeholder_count, skipped_link_count
            ),
            time.elapsed_secs_f64(),
            settings.status_duration_secs,
        );
    } else {
        set_persistence_status(
            &mut status,
            PersistenceStatusSeverity::Info,
            format!("Graph loaded successfully from {}", source_path),
            time.elapsed_secs_f64(),
            settings.status_duration_secs,
        );
    }

    pending.reset();
}

/// تحديث dirty state عبر مقارنة توقيع المشهد الحالي مع آخر نسخة محفوظة
fn refresh_dirty_state(
    graph: Res<Connecting>,
    mode: Option<Res<EditorModeState>>,
    q_nodes: Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: Query<(&Transform, &Projection), With<GraphCamera>>,
    mut runtime: ResMut<GraphPersistenceRuntimeState>,
) {
    if !graph_persistence_enabled(mode.as_deref()) {
        runtime.autosave_elapsed_secs = 0.0;
        runtime.open_confirm_until_secs = None;
        return;
    }

    let current_file = build_graph_save_file(&graph, &q_nodes, &q_camera);
    let Ok(current_signature) = graph_signature(&current_file) else {
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

/// حفظ تلقائي عند وجود تغييرات غير محفوظة
fn autosave_dirty_graph(
    time: Res<Time>,
    settings: Res<GraphPersistenceSettings>,
    mode: Option<Res<EditorModeState>>,
    graph: Res<Connecting>,
    q_nodes: Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: Query<(&Transform, &Projection), With<GraphCamera>>,
    mut runtime: ResMut<GraphPersistenceRuntimeState>,
    mut status: ResMut<GraphPersistenceStatusState>,
) {
    if !graph_persistence_enabled(mode.as_deref()) {
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
        &graph,
        &q_nodes,
        &q_camera,
    ) {
        Ok((save_file, signature)) => {
            runtime.last_saved_signature = Some(signature);
            runtime.dirty = false;

            let backup_result = write_backup_file(
                &save_file,
                settings.pretty_json,
                &settings.backup_directory,
                settings.max_backup_files,
            );

            match backup_result {
                Ok(backup_path) => {
                    set_persistence_status(
                        &mut status,
                        PersistenceStatusSeverity::Info,
                        format!("Autosaved graph to {}", backup_path.display()),
                        time.elapsed_secs_f64(),
                        settings.status_duration_secs,
                    );
                }
                Err(err) => {
                    warn!("Autosave backup warning: {}", err);
                    set_persistence_status(
                        &mut status,
                        PersistenceStatusSeverity::Warning,
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
                PersistenceStatusSeverity::Error,
                format!("Autosave failed: {}", err),
                time.elapsed_secs_f64(),
                settings.status_duration_secs,
            );
        }
    }
}

/// عرض رسالة حالة بصرية على الشاشة
fn draw_persistence_status_ui(
    mut commands: Commands,
    mut status: ResMut<GraphPersistenceStatusState>,
    mode: Option<Res<EditorModeState>>,
    time: Res<Time>,
    existing: Query<Entity, With<GraphPersistenceStatusUi>>,
) {
    if !graph_persistence_enabled(mode.as_deref()) {
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    if let Some(active) = &status.active {
        if time.elapsed_secs_f64() > active.expires_at_secs {
            status.active = None;
            status.rendered_key = None;
        }
    }

    let existing_entities: Vec<Entity> = existing.iter().collect();

    let Some(active) = &status.active else {
        for entity in existing_entities {
            commands.entity(entity).despawn();
        }
        return;
    };

    let key = format!("{:?}:{}", active.severity, active.text);
    if status.rendered_key.as_ref() == Some(&key) && !existing_entities.is_empty() {
        return;
    }

    for entity in existing_entities {
        commands.entity(entity).despawn();
    }

    let background = match active.severity {
        PersistenceStatusSeverity::Info => Color::srgba(0.1, 0.35, 0.18, 0.92),
        PersistenceStatusSeverity::Warning => Color::srgba(0.45, 0.3, 0.06, 0.92),
        PersistenceStatusSeverity::Error => Color::srgba(0.5, 0.15, 0.15, 0.95),
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(14.0),
                top: Val::Px(14.0),
                width: Val::Px(560.0),
                min_height: Val::Px(34.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(background),
            ZIndex(999),
            GraphPersistenceStatusUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(active.text.clone()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    status.rendered_key = Some(key);
}

fn set_persistence_status(
    status: &mut GraphPersistenceStatusState,
    severity: PersistenceStatusSeverity,
    text: String,
    now_secs: f64,
    duration_secs: f32,
) {
    status.active = Some(PersistenceStatusMessage {
        text,
        severity,
        expires_at_secs: now_secs + duration_secs as f64,
    });
    status.rendered_key = None;
}

fn graph_persistence_enabled(mode: Option<&EditorModeState>) -> bool {
    mode.map(|mode| mode.mode == EditorMode::LegacyGraph)
        .unwrap_or(true)
}

fn parse_and_migrate_graph(content: &str) -> Result<(GraphSaveFile, Option<String>), String> {
    let value: Value =
        serde_json::from_str(content).map_err(|err| format!("invalid JSON: {}", err))?;

    let version = value
        .get("version")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(0);

    match version {
        0 => {
            let legacy: GraphSaveFileV0 = serde_json::from_value(value)
                .map_err(|err| format!("invalid schema v0 payload: {}", err))?;

            let nodes = legacy
                .nodes
                .into_iter()
                .map(|node| {
                    let inferred_input_count = node.input_count.unwrap_or(node.inputs.len());
                    let inferred_output_count = node.output_count.unwrap_or(0);

                    SavedNode {
                        id: node.id,
                        definition_id: node.definition_id,
                        position: node.position,
                        inputs: node.inputs,
                        input_count: inferred_input_count,
                        output_count: inferred_output_count,
                    }
                })
                .collect();

            Ok((
                GraphSaveFile {
                    version: GRAPH_SAVE_VERSION,
                    nodes,
                    links: legacy.links,
                    ui: legacy.ui,
                },
                Some("Migrated graph schema from v0 to v1.".to_string()),
            ))
        }
        GRAPH_SAVE_VERSION => {
            let current: GraphSaveFile = serde_json::from_value(value)
                .map_err(|err| format!("invalid schema v1 payload: {}", err))?;
            Ok((current, None))
        }
        other => Err(format!(
            "unsupported schema version {} (latest supported {})",
            other, GRAPH_SAVE_VERSION
        )),
    }
}

fn persist_graph_to_path(
    path: &str,
    pretty_json: bool,
    graph: &Connecting,
    q_nodes: &Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: &Query<(&Transform, &Projection), With<GraphCamera>>,
) -> Result<(GraphSaveFile, String), String> {
    let save_file = build_graph_save_file(graph, q_nodes, q_camera);
    let signature = graph_signature(&save_file)?;
    write_graph_save_file(path, &save_file, pretty_json)?;
    Ok((save_file, signature))
}

fn build_graph_save_file(
    graph: &Connecting,
    q_nodes: &Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: &Query<(&Transform, &Projection), With<GraphCamera>>,
) -> GraphSaveFile {
    let mut nodes_data = Vec::new();
    for (entity, node, transform, selected) in q_nodes.iter() {
        nodes_data.push((
            entity,
            node.definition_id.clone(),
            [transform.translation.x, transform.translation.y],
            node.values.inputs.clone(),
            node.values.inputs.len(),
            node.values.outputs.len(),
            selected.is_some(),
        ));
    }
    nodes_data.sort_by_key(|(entity, ..)| entity.index());

    let mut entity_to_saved_id: HashMap<Entity, u64> = HashMap::new();
    let mut nodes = Vec::with_capacity(nodes_data.len());
    let mut selected_node_ids = Vec::new();

    for (idx, (entity, definition_id, position, inputs, input_count, output_count, is_selected)) in
        nodes_data.into_iter().enumerate()
    {
        let saved_id = (idx as u64) + 1;
        entity_to_saved_id.insert(entity, saved_id);

        if is_selected {
            selected_node_ids.push(saved_id);
        }

        nodes.push(SavedNode {
            id: saved_id,
            definition_id,
            position,
            inputs,
            input_count,
            output_count,
        });
    }

    let mut links = Vec::new();
    for link in &graph.connections {
        let Some(from_node_id) = entity_to_saved_id.get(&link.from_node).copied() else {
            continue;
        };
        let Some(to_node_id) = entity_to_saved_id.get(&link.to_node).copied() else {
            continue;
        };

        links.push(SavedLink {
            from_node_id,
            from_index: link.from_index,
            to_node_id,
            to_index: link.to_index,
        });
    }

    let camera = q_camera
        .iter()
        .next()
        .map(|(transform, projection)| SavedCameraState {
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

    GraphSaveFile {
        version: GRAPH_SAVE_VERSION,
        nodes,
        links,
        ui: SavedUiState {
            camera,
            selected_node_ids,
        },
    }
}

fn graph_signature(save_file: &GraphSaveFile) -> Result<String, String> {
    serde_json::to_string(save_file)
        .map_err(|err| format!("signature serialization failed: {}", err))
}

fn write_graph_save_file(
    path: &str,
    save_file: &GraphSaveFile,
    pretty_json: bool,
) -> Result<(), String> {
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("cannot create save directory {}: {}", parent.display(), err))?;
    }

    let payload = if pretty_json {
        serde_json::to_string_pretty(save_file)
            .map_err(|err| format!("cannot serialize JSON payload: {}", err))?
    } else {
        serde_json::to_string(save_file)
            .map_err(|err| format!("cannot serialize JSON payload: {}", err))?
    };

    fs::write(target, payload)
        .map_err(|err| format!("cannot write graph file {}: {}", target.display(), err))
}

fn write_backup_file(
    save_file: &GraphSaveFile,
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

    write_graph_save_file(
        backup_path
            .to_str()
            .ok_or_else(|| "backup path is not valid UTF-8".to_string())?,
        save_file,
        pretty_json,
    )?;

    prune_backup_files(backup_dir, max_backup_files)?;

    Ok(backup_path)
}

fn prune_backup_files(backup_dir: &Path, max_backup_files: usize) -> Result<(), String> {
    if max_backup_files == 0 {
        return Ok(());
    }

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

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
