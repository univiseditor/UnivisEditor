use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use univis_editor_commands::GraphCommandRequest;
use univis_editor_persistence::graph_persistence::{
    ApplyGraphDocumentRequest, GraphPersistenceSettings, GraphPersistenceStatus,
};
use univis_node_graph::{document::GraphDocument, prelude::LiveGraphDocumentState};

use crate::status::{
    apply_document_change, publish_workflow_failure, WorkflowDocumentChange, WorkflowStatusFailure,
};

pub(super) fn duplicate_selected_nodes_system(
    mut command_requests: MessageReader<GraphCommandRequest>,
    live_document: Res<LiveGraphDocumentState>,
    mut apply_writer: MessageWriter<ApplyGraphDocumentRequest>,
    mut status: ResMut<GraphPersistenceStatus>,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    if !command_requests
        .read()
        .any(|command| matches!(command, GraphCommandRequest::DuplicateSelectedNodes))
    {
        return;
    }

    match duplicate_selected_nodes(&live_document.document) {
        Ok(change) => {
            apply_document_change(&mut apply_writer, &mut status, &settings, &time, change)
        }
        Err(failure) => publish_workflow_failure(&mut status, failure, &settings, &time),
    }
}

fn duplicate_selected_nodes(
    source_document: &GraphDocument,
) -> Result<WorkflowDocumentChange, WorkflowStatusFailure> {
    let selected_ids = source_document.selected_node_ids().to_vec();
    if selected_ids.is_empty() {
        return Err(WorkflowStatusFailure::warning(
            "Select one or more nodes before duplicating.",
        ));
    }

    let selected_set: HashSet<u64> = selected_ids.iter().copied().collect();
    let nodes_to_duplicate = source_document
        .nodes
        .iter()
        .filter(|node| selected_set.contains(&node.id))
        .cloned()
        .collect::<Vec<_>>();

    if nodes_to_duplicate.is_empty() {
        return Err(WorkflowStatusFailure::warning(
            "The current selection could not be duplicated.",
        ));
    }

    let edges_to_duplicate = source_document
        .edges
        .iter()
        .filter(|edge| {
            selected_set.contains(&edge.from_node_id) && selected_set.contains(&edge.to_node_id)
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut document = source_document.clone();
    let mut next_id = document.next_node_id();
    let mut id_map = HashMap::new();
    let duplicate_offset = [48.0_f32, -48.0_f32];

    for mut node in nodes_to_duplicate {
        let original_id = node.id;
        node.id = next_id;
        next_id += 1;
        node.position[0] += duplicate_offset[0];
        node.position[1] += duplicate_offset[1];
        id_map.insert(original_id, node.id);
        document.insert_node(node).map_err(|error| {
            WorkflowStatusFailure::document_operation("duplicate the selected nodes", error)
        })?;
    }

    for edge in edges_to_duplicate {
        let from_node_id = id_map.get(&edge.from_node_id).copied().ok_or_else(|| {
            WorkflowStatusFailure::error(
                "Failed to duplicate the selected connections because a cloned source node was missing.",
            )
        })?;
        let to_node_id = id_map.get(&edge.to_node_id).copied().ok_or_else(|| {
            WorkflowStatusFailure::error(
                "Failed to duplicate the selected connections because a cloned target node was missing.",
            )
        })?;
        document
            .connect(from_node_id, edge.from_index, to_node_id, edge.to_index)
            .map_err(|error| {
                WorkflowStatusFailure::document_operation(
                    "duplicate the selected connections",
                    error,
                )
            })?;
    }

    document.set_selected_nodes(id_map.values().copied().collect::<Vec<_>>());
    Ok(WorkflowDocumentChange {
        document,
        source_label: "duplicated selection".to_string(),
        success_message: format!("Duplicated {} node(s).", selected_ids.len()),
    })
}
