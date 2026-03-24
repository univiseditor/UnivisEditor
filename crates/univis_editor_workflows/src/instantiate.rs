use bevy::prelude::*;
use univis_editor_commands::GraphCommandRequest;
use univis_editor_persistence::graph_persistence::{
    ApplyGraphDocumentRequest, GraphPersistenceSettings, GraphPersistenceStatus,
};
use univis_editor_ui::node_spawn::spawn_node_from_definition_entity;
use univis_node_graph::{
    commands::GraphMutationTracker,
    node_definition::NodeId,
    prelude::{AuthoredNodeInputs, GraphNode, LiveGraphDocumentState, NodeRegistry},
    value::NodeValue,
};

use crate::status::{
    apply_document_change, publish_workflow_failure, publish_workflow_success,
    WorkflowDocumentChange, WorkflowStatusFailure,
};

pub(super) fn insert_subgraph_instances_system(
    mut command_requests: MessageReader<GraphCommandRequest>,
    live_document: Res<LiveGraphDocumentState>,
    mut apply_writer: MessageWriter<ApplyGraphDocumentRequest>,
    mut status: ResMut<GraphPersistenceStatus>,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    for command in command_requests.read() {
        let GraphCommandRequest::InsertSubgraph {
            subgraph_id,
            position,
        } = command
        else {
            continue;
        };

        let resolved_id = if subgraph_id.trim().is_empty() {
            live_document
                .document
                .subgraphs
                .last()
                .map(|subgraph| subgraph.id.clone())
        } else {
            Some(subgraph_id.clone())
        };

        let Some(resolved_id) = resolved_id else {
            publish_workflow_failure(
                &mut status,
                WorkflowStatusFailure::warning("No saved subgraph is available to insert."),
                &settings,
                &time,
            );
            continue;
        };

        match live_document
            .document
            .merged_with_subgraph_instance(&resolved_id, [position.x, position.y])
            .map(|document| WorkflowDocumentChange {
                document,
                source_label: format!("subgraph {}", resolved_id),
                success_message: format!("Inserted subgraph '{}'.", resolved_id),
            })
            .map_err(|error| {
                WorkflowStatusFailure::document_operation(
                    format!("instantiate subgraph '{}'", resolved_id),
                    error,
                )
            }) {
            Ok(change) => {
                apply_document_change(&mut apply_writer, &mut status, &settings, &time, change)
            }
            Err(failure) => {
                publish_workflow_failure(&mut status, failure, &settings, &time);
            }
        }
    }
}

pub(super) fn spawn_prefab_instance_nodes_system(
    mut commands: Commands,
    mut command_requests: MessageReader<GraphCommandRequest>,
    registry: Res<NodeRegistry>,
    live_document: Res<LiveGraphDocumentState>,
    mut mutation_tracker: ResMut<GraphMutationTracker>,
    mut status: ResMut<GraphPersistenceStatus>,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    for command in command_requests.read() {
        let GraphCommandRequest::SpawnPrefabNode {
            prefab_id,
            position,
        } = command
        else {
            continue;
        };

        let resolved_id = if prefab_id.trim().is_empty() {
            live_document
                .document
                .prefabs
                .last()
                .map(|prefab| prefab.id.clone())
        } else {
            Some(prefab_id.clone())
        };

        let Some(resolved_id) = resolved_id else {
            publish_workflow_failure(
                &mut status,
                WorkflowStatusFailure::warning("No saved prefab is available to instance."),
                &settings,
                &time,
            );
            continue;
        };

        if live_document.document.prefab(&resolved_id).is_none() {
            publish_workflow_failure(
                &mut status,
                WorkflowStatusFailure::warning(format!(
                    "Prefab '{}' does not exist in the current graph.",
                    resolved_id
                )),
                &settings,
                &time,
            );
            continue;
        }

        let definition_id = NodeId::new("scene/prefab_instance");
        let Some(definition) = registry.get(&definition_id) else {
            publish_workflow_failure(
                &mut status,
                WorkflowStatusFailure::error(
                    "The prefab instance node definition is not registered.",
                ),
                &settings,
                &time,
            );
            continue;
        };

        let spawned = spawn_node_from_definition_entity(&mut commands, &definition, *position);
        let prefab_id = resolved_id.clone();
        commands.queue(move |world: &mut World| {
            let Ok(mut entity) = world.get_entity_mut(spawned) else {
                return;
            };

            let prefab_value = NodeValue::string(prefab_id.clone());

            if let Some(mut authored_inputs) = entity.get_mut::<AuthoredNodeInputs>() {
                authored_inputs.ensure_len(1);
                authored_inputs.values[0] = prefab_value.clone();
            }

            if let Some(mut node) = entity.get_mut::<GraphNode>() {
                let _ = node.set_input_projection(0, prefab_value);
            }
        });
        mutation_tracker.mark_changed();

        publish_workflow_success(
            &mut status,
            format!("Spawned prefab instance node for '{}'.", resolved_id),
            &settings,
            &time,
        );
    }
}
