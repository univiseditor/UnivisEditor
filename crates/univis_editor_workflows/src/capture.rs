use bevy::prelude::*;
use univis_editor_commands::GraphCommandRequest;
use univis_editor_persistence::graph_persistence::{
    GraphPersistenceSettings, GraphPersistenceStatus, GraphPersistenceStatusSeverity,
};
use univis_node_graph::{
    commands::GraphMutationTracker,
    document::GraphDocumentPrefab,
    prelude::{GraphNode, LiveGraphDocumentState},
    value::NodeValue,
};
use univis_scene::EntityValue;

use crate::status::set_asset_status;

pub(super) fn capture_prefab_from_selection_system(
    mut command_requests: MessageReader<GraphCommandRequest>,
    q_nodes: Query<&GraphNode>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut mutation_tracker: ResMut<GraphMutationTracker>,
    mut status: ResMut<GraphPersistenceStatus>,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    if !command_requests
        .read()
        .any(|command| matches!(command, GraphCommandRequest::CapturePrefabFromSelection))
    {
        return;
    }

    let Some((name_hint, root)) =
        resolve_selected_entity_value(live_document.selected_entities(), &q_nodes)
    else {
        set_asset_status(
            &mut status,
            GraphPersistenceStatusSeverity::Warning,
            "Select an entity-producing scene node or scene output first.".to_string(),
            &settings,
            &time,
        );
        return;
    };

    let id = next_asset_id("prefab", &name_hint, |candidate| {
        live_document
            .document
            .prefabs
            .iter()
            .any(|prefab| prefab.id == candidate)
    });
    let name = humanize_asset_name(&name_hint, "Prefab");
    live_document.document.upsert_prefab(GraphDocumentPrefab {
        id: id.clone(),
        name: name.clone(),
        root,
    });
    mutation_tracker.mark_changed();

    set_asset_status(
        &mut status,
        GraphPersistenceStatusSeverity::Info,
        format!("Captured prefab '{}' ({})", name, id),
        &settings,
        &time,
    );
}

pub(super) fn capture_subgraph_from_selection_system(
    mut command_requests: MessageReader<GraphCommandRequest>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut mutation_tracker: ResMut<GraphMutationTracker>,
    mut status: ResMut<GraphPersistenceStatus>,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    if !command_requests
        .read()
        .any(|command| matches!(command, GraphCommandRequest::CaptureSubgraphFromSelection))
    {
        return;
    }

    let Some(boundary) = live_document.document.selected_subgraph_boundary_summary() else {
        set_asset_status(
            &mut status,
            GraphPersistenceStatusSeverity::Warning,
            "Select one or more nodes before capturing a subgraph.".to_string(),
            &settings,
            &time,
        );
        return;
    };
    let selected_count = boundary.selected_node_count();

    let name_hint = live_document
        .document
        .selected_node_ids()
        .first()
        .and_then(|node_id| live_document.document.node(*node_id))
        .map(|node| node.definition_id.to_string())
        .unwrap_or_else(|| "subgraph".to_string());
    let id = next_asset_id("subgraph", &name_hint, |candidate| {
        live_document
            .document
            .subgraphs
            .iter()
            .any(|subgraph| subgraph.id == candidate)
    });
    let name = humanize_asset_name(&name_hint, "Subgraph");

    if !live_document
        .document
        .capture_selected_subgraph(id.clone(), name.clone())
    {
        set_asset_status(
            &mut status,
            GraphPersistenceStatusSeverity::Warning,
            "Unable to capture the current selection as a subgraph.".to_string(),
            &settings,
            &time,
        );
        return;
    }

    mutation_tracker.mark_changed();
    set_asset_status(
        &mut status,
        GraphPersistenceStatusSeverity::Info,
        format!(
            "Captured subgraph '{}' ({}) from {} node(s) with {} internal wire(s); omitted {} incoming and {} outgoing boundary wire(s).",
            name,
            id,
            selected_count,
            boundary.internal_edge_count(),
            boundary.incoming_edge_count(),
            boundary.outgoing_edge_count()
        ),
        &settings,
        &time,
    );
}

fn resolve_selected_entity_value(
    selected_entities: Vec<Entity>,
    q_nodes: &Query<&GraphNode>,
) -> Option<(String, EntityValue)> {
    for entity in selected_entities {
        let Ok(node) = q_nodes.get(entity) else {
            continue;
        };

        if node.definition_id.as_str() == "scene/scene" {
            if let Some(root) = node
                .values
                .inputs
                .first()
                .and_then(NodeValue::as_entity)
                .cloned()
            {
                let name_hint = root.name.clone().unwrap_or_else(|| "scene".to_string());
                return Some((name_hint, root));
            }
        }

        if let Some(root) = node
            .values
            .outputs
            .iter()
            .find_map(NodeValue::as_entity)
            .cloned()
        {
            let name_hint = root
                .name
                .clone()
                .unwrap_or_else(|| node.definition_id.to_string());
            return Some((name_hint, root));
        }
    }

    None
}

fn next_asset_id(prefix: &str, hint: &str, mut exists: impl FnMut(&str) -> bool) -> String {
    let base = slugify_asset_token(hint);
    let base = if base.is_empty() {
        prefix.to_string()
    } else {
        format!("{}_{}", prefix, base)
    };

    if !exists(&base) {
        return base;
    }

    let mut index = 2usize;
    loop {
        let candidate = format!("{}_{}", base, index);
        if !exists(&candidate) {
            return candidate;
        }
        index += 1;
    }
}

fn slugify_asset_token(value: &str) -> String {
    let mut token = String::new();
    let mut last_was_underscore = false;

    for ch in value.chars() {
        let lowered = ch.to_ascii_lowercase();
        if lowered.is_ascii_alphanumeric() {
            token.push(lowered);
            last_was_underscore = false;
        } else if !last_was_underscore {
            token.push('_');
            last_was_underscore = true;
        }
    }

    token.trim_matches('_').to_string()
}

fn humanize_asset_name(hint: &str, fallback: &str) -> String {
    let trimmed = hint
        .rsplit('/')
        .next()
        .unwrap_or(hint)
        .replace('_', " ")
        .trim()
        .to_string();

    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed
    }
}
