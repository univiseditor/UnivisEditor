use bevy::prelude::*;
use univis_editor_commands::GraphCommandRequest;
use univis_editor_persistence::graph_persistence::{
    GraphPersistenceSettings, GraphPersistenceStatus,
};
use univis_node_graph::{
    commands::GraphMutationTracker,
    document::GraphDocumentPrefab,
    prelude::{GraphNode, LiveGraphDocumentState},
    value::NodeValue,
};
use univis_scene::EntityValue;

use crate::status::{publish_workflow_failure, publish_workflow_success, WorkflowStatusFailure};

pub(super) fn capture_prefab_from_selection_system(
    mut command_requests: MessageReader<GraphCommandRequest>,
    q_nodes: Query<&GraphNode>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut mutation_tracker: ResMut<GraphMutationTracker>,
    mut status: ResMut<GraphPersistenceStatus>,
    settings: Res<GraphPersistenceSettings>,
    time: Res<Time>,
) {
    let capture_target = command_requests.read().find_map(|command| match command {
        GraphCommandRequest::CapturePrefabFromSelection => Some(None),
        GraphCommandRequest::UpdatePrefabFromSelection { prefab_id } => {
            Some(Some(prefab_id.clone()))
        }
        _ => None,
    });

    let Some(capture_target) = capture_target else {
        return;
    };

    let Some((name_hint, root)) =
        resolve_selected_entity_value(live_document.selected_entities(), &q_nodes)
    else {
        publish_workflow_failure(
            &mut status,
            WorkflowStatusFailure::warning(
                "Select an entity-producing scene node or scene output first.",
            ),
            &settings,
            &time,
        );
        return;
    };

    let (id, name, updated_existing) = if let Some(prefab_id) = capture_target {
        let Some(existing) = live_document.document.prefab(&prefab_id) else {
            publish_workflow_failure(
                &mut status,
                WorkflowStatusFailure::warning(format!(
                    "Prefab '{}' does not exist in the current graph.",
                    prefab_id
                )),
                &settings,
                &time,
            );
            return;
        };
        (existing.id.clone(), existing.name.clone(), true)
    } else {
        (
            next_asset_id("prefab", &name_hint, |candidate| {
                live_document
                    .document
                    .prefabs
                    .iter()
                    .any(|prefab| prefab.id == candidate)
            }),
            humanize_asset_name(&name_hint, "Prefab"),
            false,
        )
    };

    live_document.document.upsert_prefab(GraphDocumentPrefab {
        id: id.clone(),
        name: name.clone(),
        root,
    });
    mutation_tracker.mark_changed();

    publish_workflow_success(
        &mut status,
        if updated_existing {
            format!(
                "Updated prefab '{}' ({}) from the current selection.",
                name, id
            )
        } else {
            format!("Captured prefab '{}' ({})", name, id)
        },
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
    let capture_target = command_requests.read().find_map(|command| match command {
        GraphCommandRequest::CaptureSubgraphFromSelection => Some(None),
        GraphCommandRequest::UpdateSubgraphFromSelection { subgraph_id } => {
            Some(Some(subgraph_id.clone()))
        }
        _ => None,
    });

    let Some(capture_target) = capture_target else {
        return;
    };

    let Some(boundary) = live_document.document.selected_subgraph_boundary_summary() else {
        publish_workflow_failure(
            &mut status,
            WorkflowStatusFailure::warning("Select one or more nodes before capturing a subgraph."),
            &settings,
            &time,
        );
        return;
    };
    let selected_count = boundary.selected_node_count();

    let (id, name, updated_existing) = if let Some(subgraph_id) = capture_target {
        let Some(existing) = live_document.document.subgraph(&subgraph_id) else {
            publish_workflow_failure(
                &mut status,
                WorkflowStatusFailure::warning(format!(
                    "Subgraph '{}' does not exist in the current graph.",
                    subgraph_id
                )),
                &settings,
                &time,
            );
            return;
        };
        (existing.id.clone(), existing.name.clone(), true)
    } else {
        let name_hint = live_document
            .document
            .selected_node_ids()
            .first()
            .and_then(|node_id| live_document.document.node(*node_id))
            .map(|node| node.definition_id.to_string())
            .unwrap_or_else(|| "subgraph".to_string());
        (
            next_asset_id("subgraph", &name_hint, |candidate| {
                live_document
                    .document
                    .subgraphs
                    .iter()
                    .any(|subgraph| subgraph.id == candidate)
            }),
            humanize_asset_name(&name_hint, "Subgraph"),
            false,
        )
    };

    if let Err(error) = live_document
        .document
        .capture_selected_subgraph(id.clone(), name.clone())
    {
        publish_workflow_failure(
            &mut status,
            WorkflowStatusFailure::document_operation(
                "capture the current selection as a subgraph",
                error,
            ),
            &settings,
            &time,
        );
        return;
    }

    mutation_tracker.mark_changed();
    publish_workflow_success(
        &mut status,
        if updated_existing {
            format!(
                "Updated subgraph '{}' ({}) from {} node(s) with {} internal wire(s); omitted {} incoming and {} outgoing boundary wire(s).",
                name,
                id,
                selected_count,
                boundary.internal_edge_count(),
                boundary.incoming_edge_count(),
                boundary.outgoing_edge_count()
            )
        } else {
            format!(
                "Captured subgraph '{}' ({}) from {} node(s) with {} internal wire(s); omitted {} incoming and {} outgoing boundary wire(s).",
                name,
                id,
                selected_count,
                boundary.internal_edge_count(),
                boundary.incoming_edge_count(),
                boundary.outgoing_edge_count()
            )
        },
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
