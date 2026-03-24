use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use serde::{Deserialize, Serialize};
use univis_editor_persistence::graph_persistence::{
    GraphHistoryState, GraphPersistenceRuntimeState, GraphPersistenceStatus,
};
use univis_editor_runtime::{
    GraphRuntimeDiagnostics, GraphRuntimeIssueSeverity, GraphRuntimeTrace,
    GraphRuntimeTraceSettings, GraphSceneOutputs,
};
use univis_editor_ui::prelude::{
    GraphCamera, GraphConnectionInspectorSummary, GraphPortPreviewSummary,
};
use univis_graph_core::prelude::GraphValidationIssue;
use univis_node_graph::prelude::{
    GraphDocument, GraphDocumentEdge, LiveGraphDocumentState, LiveGraphValidationState,
    NodeRegistry, Selected,
};

#[derive(Resource, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloatingPanelsSettings {
    pub show_diagnostics: bool,
    pub show_scene_preview: bool,
    pub show_connection_inspector: bool,
}

impl Default for FloatingPanelsSettings {
    fn default() -> Self {
        Self {
            show_diagnostics: true,
            show_scene_preview: true,
            show_connection_inspector: true,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
struct EditorDiagnosticsSummary {
    text: String,
}

#[derive(Resource, Debug, Clone, Default)]
struct ScenePreviewSummary {
    text: String,
}

#[derive(Component)]
struct FloatingPanelsRoot;

#[derive(Component)]
struct DiagnosticsPanel;

#[derive(Component)]
struct ScenePreviewPanel;

#[derive(Component)]
struct ConnectionInspectorPanel;

#[derive(Component)]
struct DiagnosticsPanelText;

#[derive(Component)]
struct ScenePreviewPanelText;

#[derive(Component)]
struct ConnectionInspectorPanelText;

#[derive(Component)]
struct PortPreviewTooltip;

#[derive(Component)]
struct PortPreviewTooltipTitle;

#[derive(Component)]
struct PortPreviewTooltipSubtitle;

#[derive(Component)]
struct PortPreviewTooltipValue;

#[derive(Component)]
struct PortPreviewTooltipDetail;

#[derive(Component)]
struct PortPreviewTooltipSwatch;

pub struct FloatingPanelsPlugin;

impl Plugin for FloatingPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FloatingPanelsSettings>()
            .init_resource::<EditorDiagnosticsSummary>()
            .init_resource::<ScenePreviewSummary>()
            .add_systems(Startup, setup_floating_panels_system)
            .add_systems(
                Update,
                (
                    sync_floating_panels_ui_target_system,
                    sync_floating_panel_visibility_system,
                    refresh_editor_diagnostics_summary_system,
                    refresh_scene_preview_summary_system,
                    sync_floating_panel_text_system,
                    sync_port_preview_tooltip_system,
                )
                    .chain(),
            );
    }
}

fn setup_floating_panels_system(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(18.0),
                top: Val::Px(18.0),
                width: Val::Px(320.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            ZIndex(1700),
            FloatingPanelsRoot,
        ))
        .with_children(|root| {
            spawn_panel(
                root,
                "Graph Diagnostics",
                "Gathering diagnostics...",
                DiagnosticsPanel,
                DiagnosticsPanelText,
            );
            spawn_panel(
                root,
                "Scene Preview",
                "No scene sink connected.",
                ScenePreviewPanel,
                ScenePreviewPanelText,
            );
            spawn_panel(
                root,
                "Connection Inspector",
                "Hover a port or click one to pin its connection details.",
                ConnectionInspectorPanel,
                ConnectionInspectorPanelText,
            );
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                top: Val::Px(24.0),
                width: Val::Px(260.0),
                display: Display::None,
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.96)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
            ZIndex(1710),
            PortPreviewTooltip,
        ))
        .with_children(|tooltip| {
            tooltip.spawn((
                Text::new("Port Preview"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PortPreviewTooltipTitle,
            ));
            tooltip.spawn((
                Text::new(""),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.68)),
                PortPreviewTooltipSubtitle,
            ));
            tooltip
                .spawn((Node {
                    width: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            display: Display::None,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::WHITE),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.14)),
                        PortPreviewTooltipSwatch,
                    ));
                    row.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.95, 0.98, 1.0, 0.96)),
                        PortPreviewTooltipValue,
                    ));
                });
            tooltip.spawn((
                Text::new(""),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.76)),
                PortPreviewTooltipDetail,
            ));
        });
}

fn spawn_panel<P: Component, T: Component>(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    placeholder: &str,
    panel_marker: P,
    marker: T,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.07, 0.09, 0.9)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
            panel_marker,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(title),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            panel.spawn((
                Text::new(placeholder),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.82)),
                marker,
            ));
        });
}

fn sync_floating_panels_ui_target_system(
    mut commands: Commands,
    q_graph_camera: Query<Entity, With<GraphCamera>>,
    q_roots: Query<
        (Entity, Option<&UiTargetCamera>),
        Or<(With<FloatingPanelsRoot>, With<PortPreviewTooltip>)>,
    >,
) {
    let Some(graph_camera) = q_graph_camera.iter().next() else {
        return;
    };

    for (entity, target_camera) in q_roots.iter() {
        if target_camera.map(|target| target.entity()) == Some(graph_camera) {
            continue;
        }

        commands
            .entity(entity)
            .try_insert(UiTargetCamera(graph_camera));
    }
}

fn sync_floating_panel_visibility_system(
    settings: Res<FloatingPanelsSettings>,
    mut roots: Query<
        &mut Node,
        (
            With<FloatingPanelsRoot>,
            Without<DiagnosticsPanel>,
            Without<ScenePreviewPanel>,
        ),
    >,
    mut diagnostics_panels: Query<
        &mut Node,
        (
            With<DiagnosticsPanel>,
            Without<ScenePreviewPanel>,
            Without<FloatingPanelsRoot>,
        ),
    >,
    mut scene_preview_panels: Query<
        &mut Node,
        (
            With<ScenePreviewPanel>,
            Without<DiagnosticsPanel>,
            Without<FloatingPanelsRoot>,
        ),
    >,
    mut connection_inspector_panels: Query<
        &mut Node,
        (
            With<ConnectionInspectorPanel>,
            Without<DiagnosticsPanel>,
            Without<ScenePreviewPanel>,
            Without<FloatingPanelsRoot>,
        ),
    >,
) {
    if !settings.is_changed() {
        return;
    }

    for mut node in roots.iter_mut() {
        node.display = if settings.show_diagnostics
            || settings.show_scene_preview
            || settings.show_connection_inspector
        {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in diagnostics_panels.iter_mut() {
        node.display = if settings.show_diagnostics {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in scene_preview_panels.iter_mut() {
        node.display = if settings.show_scene_preview {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in connection_inspector_panels.iter_mut() {
        node.display = if settings.show_connection_inspector {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn format_node_label(document: &GraphDocument, registry: &NodeRegistry, node_id: u64) -> String {
    let Some(node) = document.nodes.iter().find(|node| node.id == node_id) else {
        return format!("Node#{node_id}");
    };

    let display_name = registry
        .get(&node.definition_id)
        .map(|definition| definition.display_name().to_string())
        .unwrap_or_else(|| node.definition_id.as_str().to_string());

    format!("{display_name}#{node_id}")
}

fn format_node_list(
    document: &GraphDocument,
    registry: &NodeRegistry,
    node_ids: &[u64],
    limit: usize,
) -> String {
    let mut labels = node_ids
        .iter()
        .take(limit)
        .map(|node_id| format_node_label(document, registry, *node_id))
        .collect::<Vec<_>>();

    if node_ids.len() > limit {
        labels.push(format!("+{}", node_ids.len() - limit));
    }

    labels.join(", ")
}

fn format_edge_list(
    document: &GraphDocument,
    registry: &NodeRegistry,
    edges: &[GraphDocumentEdge],
    limit: usize,
) -> String {
    let mut labels = edges
        .iter()
        .take(limit)
        .map(|edge| {
            format!(
                "{}:{} -> {}:{}",
                format_node_label(document, registry, edge.from_node_id),
                edge.from_index + 1,
                format_node_label(document, registry, edge.to_node_id),
                edge.to_index + 1
            )
        })
        .collect::<Vec<_>>();

    if edges.len() > limit {
        labels.push(format!("+{}", edges.len() - limit));
    }

    labels.join(", ")
}

fn collect_unused_branch_nodes(document: &GraphDocument, sink_node_ids: &[u64]) -> Vec<u64> {
    if sink_node_ids.is_empty() {
        return Vec::new();
    }

    let mut reverse_edges: HashMap<u64, Vec<u64>> = HashMap::new();
    for edge in &document.edges {
        reverse_edges
            .entry(edge.to_node_id)
            .or_default()
            .push(edge.from_node_id);
    }

    let mut used_nodes = HashSet::new();
    let mut pending = sink_node_ids.to_vec();
    while let Some(node_id) = pending.pop() {
        if !used_nodes.insert(node_id) {
            continue;
        }

        if let Some(upstream_nodes) = reverse_edges.get(&node_id) {
            pending.extend(upstream_nodes.iter().copied());
        }
    }

    document
        .nodes
        .iter()
        .filter_map(|node| (!used_nodes.contains(&node.id)).then_some(node.id))
        .collect()
}

fn runtime_issue_line(
    document: &GraphDocument,
    registry: &NodeRegistry,
    live_document: &LiveGraphDocumentState,
    issue: &univis_editor_runtime::GraphRuntimeNodeIssue,
) -> String {
    let severity = match issue.severity {
        GraphRuntimeIssueSeverity::Warning => "Warning",
        GraphRuntimeIssueSeverity::Error => "Error",
    };

    if let Some(node_id) = live_document.node_id_for_entity(issue.node) {
        format!(
            "{severity}: {} -> {}",
            format_node_label(document, registry, node_id),
            issue.message
        )
    } else {
        format!("{severity}: {} -> {}", issue.definition_id, issue.message)
    }
}

fn validation_issue_line(
    document: &GraphDocument,
    registry: &NodeRegistry,
    issue: &GraphValidationIssue,
) -> String {
    if issue.node_ids.is_empty() {
        return format!("Validation: {}", issue.message);
    }

    format!(
        "Validation: {} [{}]",
        issue.message,
        format_node_list(document, registry, &issue.node_ids, 3)
    )
}

fn refresh_editor_diagnostics_summary_system(
    live_document: Res<LiveGraphDocumentState>,
    validation_state: Res<LiveGraphValidationState>,
    registry: Res<NodeRegistry>,
    runtime_diagnostics: Res<GraphRuntimeDiagnostics>,
    runtime_trace_settings: Res<GraphRuntimeTraceSettings>,
    runtime_trace: Res<GraphRuntimeTrace>,
    scene_outputs: Res<GraphSceneOutputs>,
    persistence_runtime: Res<GraphPersistenceRuntimeState>,
    persistence_status: Res<GraphPersistenceStatus>,
    history: Res<GraphHistoryState>,
    mut summary: ResMut<EditorDiagnosticsSummary>,
) {
    let validation_report = &validation_state.report;
    let validation_issues = &validation_report.issues;
    let blocked_node_ids = validation_report.topology.blocked_nodes.clone();
    let blocked_node_set = blocked_node_ids.iter().copied().collect::<HashSet<_>>();
    let sink_node_ids = scene_outputs
        .sinks
        .iter()
        .filter_map(|sink| live_document.node_id_for_entity(sink.node_entity))
        .collect::<Vec<_>>();
    let unused_node_ids = collect_unused_branch_nodes(&live_document.document, &sink_node_ids);
    let unused_node_set = unused_node_ids.iter().copied().collect::<HashSet<_>>();
    let mut lines = Vec::new();

    lines.push(format!(
        "Nodes: {}  Edges: {}",
        live_document.document.nodes.len(),
        live_document.document.edges.len()
    ));
    lines.push(format!(
        "History: undo {}  redo {}",
        history.past.len(),
        history.future.len()
    ));
    lines.push(format!(
        "Assets: prefabs {}  subgraphs {}",
        live_document.document.prefab_count(),
        live_document.document.subgraph_count()
    ));
    lines.push(format!(
        "Validation: {}  Runtime issues: {}  Blocked: {}  Ordered: {}  Unused: {}",
        validation_report.issue_count(),
        runtime_diagnostics.node_issues.len(),
        blocked_node_ids.len(),
        validation_report.topology.ordered_nodes.len(),
        unused_node_ids.len()
    ));
    lines.push(format!(
        "Persistence: {}",
        if persistence_runtime.dirty {
            "dirty"
        } else {
            "clean"
        }
    ));

    if sink_node_ids.is_empty() {
        lines.push("Reachability: no scene sink, so unused-branch analysis is paused.".to_string());
    } else {
        lines.push(format!(
            "Reachability: {} scene sink(s) active.",
            sink_node_ids.len()
        ));
    }

    if let Some(selected_node_id) = live_document.document.selected_node_ids().first().copied() {
        lines.push(format!(
            "Selected: {}",
            format_node_label(&live_document.document, &registry, selected_node_id)
        ));

        let mut selected_findings = Vec::new();
        if blocked_node_set.contains(&selected_node_id) {
            selected_findings.push(
                "Selected issue: blocked by a cycle or unresolved dependency path.".to_string(),
            );
        }
        if !sink_node_ids.is_empty() && unused_node_set.contains(&selected_node_id) {
            selected_findings.push(
                "Selected issue: this branch does not contribute to any scene sink.".to_string(),
            );
        }

        selected_findings.extend(
            validation_issues
                .iter()
                .filter(|issue| issue.node_ids.contains(&selected_node_id))
                .take(2)
                .map(|issue| validation_issue_line(&live_document.document, &registry, issue)),
        );
        selected_findings.extend(
            runtime_diagnostics
                .node_issues
                .iter()
                .filter(|issue| {
                    live_document.node_id_for_entity(issue.node) == Some(selected_node_id)
                })
                .take(2)
                .map(|issue| {
                    runtime_issue_line(&live_document.document, &registry, &live_document, issue)
                }),
        );

        if selected_findings.is_empty() {
            lines.push("Selected status: healthy".to_string());
        } else {
            lines.extend(selected_findings);
        }
    } else {
        lines.push("Selected: none".to_string());
    }

    if let Some(boundary) = live_document.document.selected_subgraph_boundary_summary() {
        lines.push(format!(
            "Selection boundary: {} node(s), {} internal wire(s), {} incoming omitted, {} outgoing omitted",
            boundary.selected_node_count(),
            boundary.internal_edge_count(),
            boundary.incoming_edge_count(),
            boundary.outgoing_edge_count()
        ));

        if !boundary.incoming_edges.is_empty() {
            lines.push(format!(
                "Selection incoming: {}",
                format_edge_list(
                    &live_document.document,
                    &registry,
                    &boundary.incoming_edges,
                    3
                )
            ));
        }

        if !boundary.outgoing_edges.is_empty() {
            lines.push(format!(
                "Selection outgoing: {}",
                format_edge_list(
                    &live_document.document,
                    &registry,
                    &boundary.outgoing_edges,
                    3
                )
            ));
        }
    }

    if !blocked_node_ids.is_empty() {
        lines.push(format!(
            "Blocked path: {}",
            format_node_list(&live_document.document, &registry, &blocked_node_ids, 4)
        ));
    }

    if !unused_node_ids.is_empty() {
        lines.push(format!(
            "Unused branch: {}",
            format_node_list(&live_document.document, &registry, &unused_node_ids, 4)
        ));
    }

    for issue in validation_issues.iter().take(2) {
        lines.push(validation_issue_line(
            &live_document.document,
            &registry,
            issue,
        ));
    }

    for issue in runtime_diagnostics.node_issues.iter().take(2) {
        lines.push(runtime_issue_line(
            &live_document.document,
            &registry,
            &live_document,
            issue,
        ));
    }

    if validation_report.is_valid()
        && runtime_diagnostics.node_issues.is_empty()
        && blocked_node_ids.is_empty()
        && unused_node_ids.is_empty()
    {
        if let Some(status) = persistence_status.active.as_ref() {
            lines.push(format!("Status: {}", status.text));
        } else {
            lines.push("Status: healthy".to_string());
        }
    } else if let Some(status) = persistence_status.active.as_ref() {
        lines.push(format!("Status: {}", status.text));
    }

    if runtime_trace_settings.enabled {
        lines.push(format!(
            "Trace: ON (showing {} recent node pass(es))",
            runtime_trace.entries.len()
        ));
        for entry in runtime_trace.entries.iter().rev().take(4) {
            let node_label = live_document
                .node_id_for_entity(entry.node)
                .map(|node_id| format_node_label(&live_document.document, &registry, node_id))
                .unwrap_or_else(|| entry.definition_id.clone());
            let reasons = if entry.reasons.is_empty() {
                "no explicit reason".to_string()
            } else {
                entry.reasons.join(", ")
            };
            lines.push(format!(
                "Trace: {} [{}] -> {}{}",
                node_label,
                reasons,
                entry.result,
                if entry.outputs_changed {
                    " (outputs changed)"
                } else {
                    ""
                }
            ));
        }
    }

    summary.text = lines.join("\n");
}

fn refresh_scene_preview_summary_system(
    selected_nodes: Query<(), With<Selected>>,
    scene_outputs: Res<GraphSceneOutputs>,
    live_document: Res<LiveGraphDocumentState>,
    mut summary: ResMut<ScenePreviewSummary>,
) {
    let selected_sink = scene_outputs
        .sinks
        .iter()
        .find(|sink| selected_nodes.get(sink.node_entity).is_ok());
    let scene_sink = selected_sink.or_else(|| scene_outputs.sinks.first());

    let Some(scene_sink) = scene_sink else {
        summary.text = "No scene sink node in the current graph.".to_string();
        return;
    };

    let Some(scene) = scene_sink.scene.as_ref() else {
        summary.text = "Scene sink exists, but its Entity input is not connected.".to_string();
        return;
    };

    let stats = scene_sink.stats.unwrap_or_else(|| scene.stats());
    let node_id = live_document
        .node_id_for_entity(scene_sink.node_entity)
        .unwrap_or_default();
    let child_names = scene.child_names(4);

    summary.text = vec![
        format!("Sink node id: {}", node_id),
        format!(
            "Entities: {}  Depth: {}  Named: {}",
            stats.entity_count, stats.max_depth, stats.named_entity_count
        ),
        format!(
            "Sprites: {}  Text: {}  Cameras: {}",
            stats.sprite_count, stats.text_count, stats.camera_count
        ),
        format!(
            "Root: {}",
            scene.root_name().unwrap_or("<unnamed root>").to_string()
        ),
        format!(
            "Children: {}",
            if child_names.is_empty() {
                "<none>".to_string()
            } else {
                child_names.join(", ")
            }
        ),
    ]
    .join("\n");
}

fn sync_port_preview_tooltip_system(
    summary: Res<GraphPortPreviewSummary>,
    windows: Query<&Window>,
    mut tooltips: Query<&mut Node, (With<PortPreviewTooltip>, Without<PortPreviewTooltipSwatch>)>,
    mut titles: Query<
        &mut Text,
        (
            With<PortPreviewTooltipTitle>,
            Without<PortPreviewTooltipSubtitle>,
            Without<PortPreviewTooltipValue>,
            Without<PortPreviewTooltipDetail>,
        ),
    >,
    mut subtitles: Query<
        &mut Text,
        (
            With<PortPreviewTooltipSubtitle>,
            Without<PortPreviewTooltipTitle>,
            Without<PortPreviewTooltipValue>,
            Without<PortPreviewTooltipDetail>,
        ),
    >,
    mut values: Query<
        &mut Text,
        (
            With<PortPreviewTooltipValue>,
            Without<PortPreviewTooltipTitle>,
            Without<PortPreviewTooltipSubtitle>,
            Without<PortPreviewTooltipDetail>,
        ),
    >,
    mut details: Query<
        &mut Text,
        (
            With<PortPreviewTooltipDetail>,
            Without<PortPreviewTooltipTitle>,
            Without<PortPreviewTooltipSubtitle>,
            Without<PortPreviewTooltipValue>,
        ),
    >,
    mut swatches: Query<
        (&mut Node, &mut BackgroundColor),
        (With<PortPreviewTooltipSwatch>, Without<PortPreviewTooltip>),
    >,
) {
    if !summary.is_changed() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    for mut node in tooltips.iter_mut() {
        node.display = if summary.visible {
            Display::Flex
        } else {
            Display::None
        };

        if !summary.visible {
            continue;
        }

        let max_left = (window.width() - 276.0).max(8.0);
        let max_top = (window.height() - 168.0).max(8.0);
        node.left = Val::Px(summary.screen_position.x.clamp(8.0, max_left));
        node.top = Val::Px(summary.screen_position.y.clamp(8.0, max_top));
    }

    if !summary.visible {
        return;
    }

    for mut text in titles.iter_mut() {
        text.0 = summary.title.clone();
    }
    for mut text in subtitles.iter_mut() {
        text.0 = summary.subtitle.clone();
    }
    for mut text in values.iter_mut() {
        text.0 = summary.value_text.clone();
    }
    for mut text in details.iter_mut() {
        text.0 = summary.detail.clone();
    }
    for (mut node, mut background) in swatches.iter_mut() {
        if let Some(color) = summary.swatch {
            node.display = Display::Flex;
            *background = BackgroundColor(color);
        } else {
            node.display = Display::None;
        }
    }
}

fn sync_floating_panel_text_system(
    diagnostics: Res<EditorDiagnosticsSummary>,
    scene_preview: Res<ScenePreviewSummary>,
    connection_inspector: Res<GraphConnectionInspectorSummary>,
    mut diagnostics_text: Query<
        &mut Text,
        (With<DiagnosticsPanelText>, Without<ScenePreviewPanelText>),
    >,
    mut scene_text: Query<&mut Text, (With<ScenePreviewPanelText>, Without<DiagnosticsPanelText>)>,
    mut connection_text: Query<
        &mut Text,
        (
            With<ConnectionInspectorPanelText>,
            Without<DiagnosticsPanelText>,
            Without<ScenePreviewPanelText>,
        ),
    >,
) {
    if diagnostics.is_changed() {
        for mut text in diagnostics_text.iter_mut() {
            text.0 = diagnostics.text.clone();
        }
    }

    if scene_preview.is_changed() {
        for mut text in scene_text.iter_mut() {
            text.0 = scene_preview.text.clone();
        }
    }

    if connection_inspector.is_changed() {
        for mut text in connection_text.iter_mut() {
            text.0 = connection_inspector.text.clone();
        }
    }
}
