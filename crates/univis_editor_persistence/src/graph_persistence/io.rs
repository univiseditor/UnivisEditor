use bevy::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use univis_editor_ui::prelude::GraphCamera;
use univis_node_graph::prelude::*;

use crate::format::{
    parse_graph_document_payload, prepare_graph_document_write, serialize_graph_document,
    ParsedGraphDocument, PreparedGraphWrite,
};

use super::apply::stage_graph_document_apply;
use super::state::{
    graph_persistence_enabled, set_persistence_status, GraphPersistenceActivation,
    GraphPersistenceRuntimeState, GraphPersistenceSettings, GraphPersistenceStatus,
    GraphPersistenceStatusSeverity, LoadGraphFromPathRequest, LoadGraphRequest,
    LoadGraphRuntimeParams, MutationUiState, PendingGraphApplyOrigin, SaveGraphRequest,
    SaveGraphToPathRequest,
};

pub(super) fn handle_save_graph_requests_system(
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

pub(super) fn handle_load_graph_requests_system(
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

pub(super) fn refresh_dirty_state_system(
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
    let Ok(current_signature) = crate::format::graph_document_signature(&current_document) else {
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

pub(super) fn autosave_dirty_graph_system(
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

pub(super) fn expire_persistence_status_system(
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

pub(crate) fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
