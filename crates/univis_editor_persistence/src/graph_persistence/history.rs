use bevy::prelude::*;
use univis_editor_commands::GraphCommandRequest;
use univis_node_graph::prelude::*;

use super::apply::stage_graph_document_apply;
use super::io::unix_timestamp_millis;
use super::state::{
    GraphHistorySettings, GraphHistoryState, GraphPersistenceActivation, GraphPersistenceSettings,
    GraphPersistenceStatus, GraphPersistenceStatusSeverity, MutationUiState,
    PendingGraphApplyOrigin, PendingGraphLoad, graph_persistence_enabled, set_persistence_status,
};

pub(super) fn graph_persistence_shortcuts_system(
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

pub(super) fn handle_history_requests_system(
    mut commands: Commands,
    mut command_requests: MessageReader<GraphCommandRequest>,
    activation: Option<Res<GraphPersistenceActivation>>,
    history_settings: Res<GraphHistorySettings>,
    registry: Res<NodeRegistry>,
    q_existing_nodes: Query<Entity, With<GraphNode>>,
    q_existing_connections: Query<Entity, With<GraphConnection>>,
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

    let validation_issue_count =
        validate_graph_document_report(&target_document, &registry).issue_count();
    live_document.document.prefabs = target_document.prefabs.clone();
    live_document.document.subgraphs = target_document.subgraphs.clone();
    ui_state.reset();
    stage_graph_document_apply(
        &mut commands,
        &registry,
        &q_existing_nodes,
        &q_existing_connections,
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
    history.last_signature = crate::format::graph_document_signature(&target_document).ok();
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

pub(super) fn capture_graph_history_snapshot_system(
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

    let Ok(current_signature) = crate::format::graph_document_signature(&live_document.document)
    else {
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
