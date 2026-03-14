use bevy::prelude::*;
use univis_editor_persistence::graph_persistence::{
    ApplyGraphDocumentRequest, GraphPersistenceSettings, GraphPersistenceStatus,
    GraphPersistenceStatusMessage, GraphPersistenceStatusSeverity,
};
use univis_editor_ui::node_spawn::spawn_node_from_definition_entity;
use univis_node_graph::{
    commands::{GraphCommandRequest, GraphMutationTracker},
    document::GraphDocumentPrefab,
    node_definition::NodeId,
    prelude::{GraphNode, LiveGraphDocumentState, NodeRegistry},
    value::NodeValue,
};
use univis_scene::EntityValue;

pub struct GraphAssetWorkflowPlugin;

impl Plugin for GraphAssetWorkflowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                capture_prefab_from_selection,
                capture_subgraph_from_selection,
                insert_subgraph_instances,
                spawn_prefab_instance_nodes,
            )
                .chain(),
        );
    }
}

fn capture_prefab_from_selection(
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

fn capture_subgraph_from_selection(
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

    let selected_count = live_document.document.selected_node_ids().len();
    if selected_count == 0 {
        set_asset_status(
            &mut status,
            GraphPersistenceStatusSeverity::Warning,
            "Select one or more nodes before capturing a subgraph.".to_string(),
            &settings,
            &time,
        );
        return;
    }

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
            "Captured subgraph '{}' ({}) from {} node(s)",
            name, id, selected_count
        ),
        &settings,
        &time,
    );
}

fn insert_subgraph_instances(
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
            set_asset_status(
                &mut status,
                GraphPersistenceStatusSeverity::Warning,
                "No saved subgraph is available to insert.".to_string(),
                &settings,
                &time,
            );
            continue;
        };

        let Some(document) = live_document
            .document
            .merged_with_subgraph_instance(&resolved_id, [position.x, position.y])
        else {
            set_asset_status(
                &mut status,
                GraphPersistenceStatusSeverity::Warning,
                format!("Subgraph '{}' could not be instantiated.", resolved_id),
                &settings,
                &time,
            );
            continue;
        };

        apply_writer.write(ApplyGraphDocumentRequest {
            document,
            source_label: format!("subgraph {}", resolved_id),
            track_for_undo: true,
        });
    }
}

fn spawn_prefab_instance_nodes(
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
            set_asset_status(
                &mut status,
                GraphPersistenceStatusSeverity::Warning,
                "No saved prefab is available to instance.".to_string(),
                &settings,
                &time,
            );
            continue;
        };

        if live_document.document.prefab(&resolved_id).is_none() {
            set_asset_status(
                &mut status,
                GraphPersistenceStatusSeverity::Warning,
                format!(
                    "Prefab '{}' does not exist in the current graph.",
                    resolved_id
                ),
                &settings,
                &time,
            );
            continue;
        }

        let definition_id = NodeId::new("scene/prefab_instance");
        let Some(definition) = registry.get(&definition_id) else {
            set_asset_status(
                &mut status,
                GraphPersistenceStatusSeverity::Error,
                "The prefab instance node definition is not registered.".to_string(),
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
            let Some(mut node) = entity.get_mut::<GraphNode>() else {
                return;
            };
            if let Some(input) = node.values.inputs.first_mut() {
                *input = NodeValue::string(prefab_id.clone());
            }
        });
        mutation_tracker.mark_changed();

        set_asset_status(
            &mut status,
            GraphPersistenceStatusSeverity::Info,
            format!("Spawned prefab instance node for '{}'.", resolved_id),
            &settings,
            &time,
        );
    }
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

fn set_asset_status(
    status: &mut GraphPersistenceStatus,
    severity: GraphPersistenceStatusSeverity,
    text: String,
    settings: &GraphPersistenceSettings,
    time: &Time,
) {
    status.active = Some(GraphPersistenceStatusMessage {
        text,
        severity,
        expires_at_secs: time.elapsed_secs_f64() + settings.status_duration_secs as f64,
    });
}
