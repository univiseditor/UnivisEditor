use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use univis_editor_commands::{GraphClipboardSnapshot, GraphClipboardState, GraphCommandRequest};
use univis_editor_persistence::graph_persistence::{
    ApplyGraphDocumentRequest, GraphPersistenceSettings, GraphPersistenceStatus,
};
use univis_node_graph::document::{GraphDocument, GRAPH_DOCUMENT_VERSION};
use univis_node_graph::prelude::LiveGraphDocumentState;

use crate::status::{
    apply_document_change, publish_workflow_failure, publish_workflow_success,
    WorkflowDocumentChange, WorkflowStatusFailure,
};

type EditorGraphDocument = GraphDocument;

pub(super) fn copy_selected_nodes_to_clipboard_system(
    mut command_requests: MessageReader<GraphCommandRequest>,
    live_document: Res<LiveGraphDocumentState>,
    mut clipboard: ResMut<GraphClipboardState>,
    mut status: ResMut<GraphPersistenceStatus>,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    if !command_requests
        .read()
        .any(|command| matches!(command, GraphCommandRequest::CopySelectedNodes))
    {
        return;
    }

    let Some(snapshot) = build_clipboard_snapshot(&live_document.document) else {
        publish_workflow_failure(
            &mut status,
            WorkflowStatusFailure::warning("Select one or more nodes before copying."),
            &settings,
            &time,
        );
        return;
    };

    let copied_count = snapshot.document.nodes.len();
    clipboard.snapshot = Some(snapshot);

    publish_workflow_success(
        &mut status,
        format!("Copied {} node(s) to the clipboard.", copied_count),
        &settings,
        &time,
    );
}

pub(super) fn paste_nodes_from_clipboard_system(
    mut command_requests: MessageReader<GraphCommandRequest>,
    live_document: Res<LiveGraphDocumentState>,
    clipboard: Res<GraphClipboardState>,
    mut apply_writer: MessageWriter<ApplyGraphDocumentRequest>,
    mut status: ResMut<GraphPersistenceStatus>,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    for command in command_requests.read() {
        let GraphCommandRequest::PasteNodes { position } = command else {
            continue;
        };

        let Some(snapshot) = clipboard.snapshot.as_ref() else {
            publish_workflow_failure(
                &mut status,
                WorkflowStatusFailure::warning("The clipboard is empty."),
                &settings,
                &time,
            );
            continue;
        };

        match build_clipboard_paste_change(&live_document.document, snapshot, *position) {
            Ok(change) => {
                apply_document_change(&mut apply_writer, &mut status, &settings, &time, change)
            }
            Err(failure) => publish_workflow_failure(&mut status, failure, &settings, &time),
        }
    }
}

fn build_clipboard_paste_change(
    current_document: &EditorGraphDocument,
    snapshot: &GraphClipboardSnapshot,
    origin: Vec2,
) -> Result<WorkflowDocumentChange, WorkflowStatusFailure> {
    let document = merge_clipboard_snapshot(current_document, snapshot, origin)?;
    let pasted_count = document.selected_node_ids().len();
    Ok(WorkflowDocumentChange {
        document,
        source_label: "pasted clipboard".to_string(),
        success_message: format!("Pasted {} node(s).", pasted_count),
    })
}

fn build_clipboard_snapshot(document: &EditorGraphDocument) -> Option<GraphClipboardSnapshot> {
    let boundary = document.selected_subgraph_boundary_summary()?;
    let selected = boundary
        .selected_node_ids
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let mut nodes = document
        .nodes
        .iter()
        .filter(|node| selected.contains(&node.id))
        .cloned()
        .collect::<Vec<_>>();
    if nodes.is_empty() {
        return None;
    }

    let min_x = nodes
        .iter()
        .map(|node| node.position[0])
        .fold(f32::INFINITY, f32::min);
    let min_y = nodes
        .iter()
        .map(|node| node.position[1])
        .fold(f32::INFINITY, f32::min);

    for node in &mut nodes {
        node.position[0] -= min_x;
        node.position[1] -= min_y;
    }

    Some(GraphClipboardSnapshot {
        document: EditorGraphDocument {
            version: GRAPH_DOCUMENT_VERSION,
            nodes,
            edges: boundary.internal_edges,
            prefabs: document.prefabs.clone(),
            subgraphs: document.subgraphs.clone(),
            view: Default::default(),
        },
    })
}

fn merge_clipboard_snapshot(
    current_document: &EditorGraphDocument,
    snapshot: &GraphClipboardSnapshot,
    origin: Vec2,
) -> Result<EditorGraphDocument, WorkflowStatusFailure> {
    if snapshot.document.nodes.is_empty() {
        return Err(WorkflowStatusFailure::warning(
            "The clipboard snapshot does not contain any nodes.",
        ));
    }

    let min_x = snapshot
        .document
        .nodes
        .iter()
        .map(|node| node.position[0])
        .fold(f32::INFINITY, f32::min);
    let min_y = snapshot
        .document
        .nodes
        .iter()
        .map(|node| node.position[1])
        .fold(f32::INFINITY, f32::min);

    let mut merged = current_document.clone();
    let mut next_node_id = merged.next_node_id();
    let mut selected_node_ids = Vec::new();
    let mut node_id_map = HashMap::new();

    for node in &snapshot.document.nodes {
        let new_id = next_node_id;
        next_node_id += 1;
        node_id_map.insert(node.id, new_id);

        let mut cloned = node.clone();
        cloned.id = new_id;
        cloned.position[0] = origin.x + (cloned.position[0] - min_x);
        cloned.position[1] = origin.y + (cloned.position[1] - min_y);
        selected_node_ids.push(new_id);
        merged.insert_node(cloned).map_err(|error| {
            WorkflowStatusFailure::document_operation("paste the clipboard contents", error)
        })?;
    }

    for edge in &snapshot.document.edges {
        let from_node_id = node_id_map.get(&edge.from_node_id).copied().ok_or_else(|| {
            WorkflowStatusFailure::error(
                "Failed to paste the clipboard contents because one pasted edge refers to a missing source node.",
            )
        })?;
        let to_node_id = node_id_map.get(&edge.to_node_id).copied().ok_or_else(|| {
            WorkflowStatusFailure::error(
                "Failed to paste the clipboard contents because one pasted edge refers to a missing target node.",
            )
        })?;

        merged
            .connect(from_node_id, edge.from_index, to_node_id, edge.to_index)
            .map_err(|error| {
                WorkflowStatusFailure::document_operation("paste the clipboard contents", error)
            })?;
    }

    for prefab in &snapshot.document.prefabs {
        merged.upsert_prefab(prefab.clone());
    }
    for subgraph in &snapshot.document.subgraphs {
        merged.upsert_subgraph(subgraph.clone());
    }

    merged.set_selected_nodes(selected_node_ids);
    Ok(merged)
}
