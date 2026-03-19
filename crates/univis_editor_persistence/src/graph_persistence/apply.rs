use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use univis_editor_ui::node_spawn::{
    spawn_node_from_definition_entity, spawn_placeholder_node_entity,
};
use univis_editor_ui::prelude::GraphCamera;
use univis_node_graph::prelude::*;

use super::state::{
    ApplyGraphDocumentRequest, GraphHistoryState, GraphPersistenceActivation,
    GraphPersistenceRuntimeState, GraphPersistenceSettings, GraphPersistenceStatus,
    GraphPersistenceStatusSeverity, MutationUiState, PendingGraphApplyOrigin, PendingGraphLoad,
    graph_persistence_enabled, set_persistence_status,
};

pub(super) fn handle_apply_graph_document_requests_system(
    mut commands: Commands,
    mut apply_requests: MessageReader<ApplyGraphDocumentRequest>,
    activation: Option<Res<GraphPersistenceActivation>>,
    registry: Res<NodeRegistry>,
    q_existing_nodes: Query<Entity, With<GraphNode>>,
    q_existing_connections: Query<Entity, With<GraphConnection>>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut live_validation: ResMut<LiveGraphValidationState>,
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

    let validation_report = request
        .validation_report
        .unwrap_or_else(|| validate_graph_document_report(&request.document, &registry));
    let validation_issue_count = validation_report.issue_count();
    live_validation.set_report_for_document(&request.document, validation_report.clone());
    live_document.document.prefabs = request.document.prefabs.clone();
    live_document.document.subgraphs = request.document.subgraphs.clone();
    ui_state.reset();
    stage_graph_document_apply(
        &mut commands,
        &registry,
        &q_existing_nodes,
        &q_existing_connections,
        &mut pending,
        request.document,
        PendingGraphApplyOrigin::Mutation,
        request.source_label.clone(),
        validation_issue_count,
        Some(validation_report),
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

pub(super) fn finalize_pending_graph_load_system(
    mut commands: Commands,
    mut pending: ResMut<PendingGraphLoad>,
    activation: Option<Res<GraphPersistenceActivation>>,
    q_ports: Query<(Entity, &GraphPort)>,
    mut q_nodes: Query<(&mut GraphNode, Option<&mut AuthoredNodeInputs>)>,
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
    let mut claimed_inputs = HashSet::new();

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

        if !claimed_inputs.insert((to_node, edge.to_index)) {
            warn!(
                "Skipping loaded link: input port already connected (node {:?}, input {})",
                to_node, edge.to_index
            );
            skipped_link_count += 1;
            continue;
        }

        commands.spawn(GraphConnection {
            from_node,
            from_index: edge.from_index,
            to_node,
            to_index: edge.to_index,
            from_port,
            to_port,
        });
    }

    for (entity, inputs) in pending.node_inputs.drain(..) {
        let Ok((mut node, mut authored_inputs)) = q_nodes.get_mut(entity) else {
            continue;
        };

        for (index, value) in inputs.into_iter().enumerate() {
            if index < node.values.inputs.len() {
                node.values.inputs[index] = value.clone();
                if let Some(authored_inputs) = authored_inputs.as_deref_mut() {
                    if authored_inputs.values.len() < node.values.inputs.len() {
                        authored_inputs
                            .values
                            .resize(node.values.inputs.len(), NodeValue::None);
                    }
                    authored_inputs.values[index] = value;
                }
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
    let requires_warning = pending.requires_resave_after_migration
        || pending.placeholder_count > 0
        || skipped_link_count > 0
        || pending.validation_issue_count > 0;

    if requires_warning {
        let message = if pending.requires_resave_after_migration {
            let migration_note = pending
                .migration_note
                .as_deref()
                .unwrap_or("Loaded a legacy graph payload");

            if pending.placeholder_count > 0
                || skipped_link_count > 0
                || pending.validation_issue_count > 0
            {
                format!(
                    "{}. Save the graph to rewrite it in the current format. Loaded {} with {} placeholder node(s), {} skipped link(s), and {} validation issue(s).",
                    migration_note,
                    source_label,
                    pending.placeholder_count,
                    skipped_link_count,
                    pending.validation_issue_count
                )
            } else {
                format!(
                    "{}. Save the graph to rewrite it in the current format.",
                    migration_note
                )
            }
        } else {
            format!(
                "{} with {} placeholder node(s), {} skipped link(s), and {} validation issue(s).",
                pending_completion_prefix(pending.origin, &source_label),
                pending.placeholder_count,
                skipped_link_count,
                pending.validation_issue_count
            )
        };

        set_persistence_status(
            &mut status,
            GraphPersistenceStatusSeverity::Warning,
            message,
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

pub(super) fn stage_graph_document_apply(
    commands: &mut Commands,
    registry: &NodeRegistry,
    q_existing_nodes: &Query<Entity, With<GraphNode>>,
    q_existing_connections: &Query<Entity, With<GraphConnection>>,
    pending: &mut PendingGraphLoad,
    document: GraphDocument,
    origin: PendingGraphApplyOrigin,
    source_label: String,
    validation_issue_count: usize,
    validation_report: Option<GraphValidationReport>,
) {
    pending.reset();

    for entity in q_existing_nodes.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in q_existing_connections.iter() {
        commands.entity(entity).try_despawn();
    }

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
    pending.validation_report = validation_report;
}
