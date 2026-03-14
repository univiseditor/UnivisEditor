use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use serde::{Deserialize, Serialize};
use univis_editor_persistence::graph_persistence::{
    GraphHistoryState, GraphPersistenceRuntimeState, GraphPersistenceStatus,
};
use univis_editor_runtime::{GraphRuntimeDiagnostics, GraphSceneOutputs};
use univis_editor_ui::prelude::{GraphCamera, Selected};
use univis_node_graph::prelude::{LiveGraphDocumentState, NodeRegistry};

#[derive(Resource, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloatingPanelsSettings {
    pub show_diagnostics: bool,
    pub show_scene_preview: bool,
}

impl Default for FloatingPanelsSettings {
    fn default() -> Self {
        Self {
            show_diagnostics: true,
            show_scene_preview: true,
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
struct DiagnosticsPanelText;

#[derive(Component)]
struct ScenePreviewPanelText;

pub struct FloatingPanelsPlugin;

impl Plugin for FloatingPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FloatingPanelsSettings>()
            .init_resource::<EditorDiagnosticsSummary>()
            .init_resource::<ScenePreviewSummary>()
            .add_systems(Startup, setup_floating_panels)
            .add_systems(
                Update,
                (
                    sync_floating_panels_ui_target,
                    sync_floating_panel_visibility,
                    refresh_editor_diagnostics_summary,
                    refresh_scene_preview_summary,
                    sync_floating_panel_text,
                )
                    .chain(),
            );
    }
}

fn setup_floating_panels(mut commands: Commands) {
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

fn sync_floating_panels_ui_target(
    mut commands: Commands,
    q_graph_camera: Query<Entity, With<GraphCamera>>,
    q_roots: Query<(Entity, Option<&UiTargetCamera>), With<FloatingPanelsRoot>>,
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

fn sync_floating_panel_visibility(
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
) {
    if !settings.is_changed() {
        return;
    }

    for mut node in roots.iter_mut() {
        node.display = if settings.show_diagnostics || settings.show_scene_preview {
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
}

fn refresh_editor_diagnostics_summary(
    live_document: Res<LiveGraphDocumentState>,
    registry: Res<NodeRegistry>,
    runtime_diagnostics: Res<GraphRuntimeDiagnostics>,
    persistence_runtime: Res<GraphPersistenceRuntimeState>,
    persistence_status: Res<GraphPersistenceStatus>,
    history: Res<GraphHistoryState>,
    mut summary: ResMut<EditorDiagnosticsSummary>,
) {
    let validation_issues = univis_node_graph::graph_validation::validate_graph_document(
        &live_document.document,
        &registry,
    );
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
        "Validation: {}  Runtime issues: {}  Blocked: {}",
        validation_issues.len(),
        runtime_diagnostics.node_issues.len(),
        runtime_diagnostics.blocked_nodes.len()
    ));
    lines.push(format!(
        "Persistence: {}",
        if persistence_runtime.dirty {
            "dirty"
        } else {
            "clean"
        }
    ));

    if let Some(issue) = validation_issues.first() {
        lines.push(format!("Validation issue: {}", issue.message));
    } else if let Some(issue) = runtime_diagnostics.node_issues.first() {
        lines.push(format!("Runtime issue: {}", issue.message));
    } else if let Some(status) = persistence_status.active.as_ref() {
        lines.push(format!("Status: {}", status.text));
    } else {
        lines.push("Status: healthy".to_string());
    }

    summary.text = lines.join("\n");
}

fn refresh_scene_preview_summary(
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

fn sync_floating_panel_text(
    diagnostics: Res<EditorDiagnosticsSummary>,
    scene_preview: Res<ScenePreviewSummary>,
    mut diagnostics_text: Query<
        &mut Text,
        (With<DiagnosticsPanelText>, Without<ScenePreviewPanelText>),
    >,
    mut scene_text: Query<&mut Text, (With<ScenePreviewPanelText>, Without<DiagnosticsPanelText>)>,
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
}
