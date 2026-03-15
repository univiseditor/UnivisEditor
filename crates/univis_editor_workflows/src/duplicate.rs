use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use univis_editor_commands::GraphCommandRequest;
use univis_editor_persistence::graph_persistence::{
    ApplyGraphDocumentRequest, GraphPersistenceSettings, GraphPersistenceStatus,
    GraphPersistenceStatusSeverity,
};
use univis_node_graph::prelude::LiveGraphDocumentState;

use crate::status::set_asset_status;

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

    let selected_ids = live_document.document.selected_node_ids().to_vec();
    if selected_ids.is_empty() {
        set_asset_status(
            &mut status,
            GraphPersistenceStatusSeverity::Warning,
            "Select one or more nodes before duplicating.".to_string(),
            &settings,
            &time,
        );
        return;
    }

    let selected_set: HashSet<u64> = selected_ids.iter().copied().collect();
    let nodes_to_duplicate = live_document
        .document
        .nodes
        .iter()
        .filter(|node| selected_set.contains(&node.id))
        .cloned()
        .collect::<Vec<_>>();

    if nodes_to_duplicate.is_empty() {
        set_asset_status(
            &mut status,
            GraphPersistenceStatusSeverity::Warning,
            "The current selection could not be duplicated.".to_string(),
            &settings,
            &time,
        );
        return;
    }

    let edges_to_duplicate = live_document
        .document
        .edges
        .iter()
        .filter(|edge| {
            selected_set.contains(&edge.from_node_id) && selected_set.contains(&edge.to_node_id)
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut document = live_document.document.clone();
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
        let _ = document.insert_node(node);
    }

    for edge in edges_to_duplicate {
        let Some(from_node_id) = id_map.get(&edge.from_node_id).copied() else {
            continue;
        };
        let Some(to_node_id) = id_map.get(&edge.to_node_id).copied() else {
            continue;
        };
        let _ = document.connect(from_node_id, edge.from_index, to_node_id, edge.to_index);
    }

    document.set_selected_nodes(id_map.values().copied().collect::<Vec<_>>());
    apply_writer.write(ApplyGraphDocumentRequest {
        document,
        source_label: "duplicated selection".to_string(),
        track_for_undo: true,
    });

    set_asset_status(
        &mut status,
        GraphPersistenceStatusSeverity::Info,
        format!("Duplicated {} node(s).", selected_ids.len()),
        &settings,
        &time,
    );
}
