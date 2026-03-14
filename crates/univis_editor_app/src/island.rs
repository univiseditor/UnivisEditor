use crate::editor_settings_persistence::EditorWorkflowState;
use crate::panels::FloatingPanelsSettings;
use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use univis_editor_persistence::graph_persistence::{
    latest_backup_file, GraphHistorySettings, GraphPersistenceRuntimeState,
    GraphPersistenceSettings, GraphPersistenceStatus, GraphPersistenceStatusSeverity,
};
use univis_editor_ui::prelude::{EditorSettings, GraphCamera};
use univis_node_graph::commands::{GraphCommandRequest, GraphOverlayState, GraphOverlaySurface};
use univis_node_graph::node_definition::NodeId;
use univis_node_graph::node_registry::NodeRegistry;

#[derive(Resource, Debug, Clone, Default)]
struct CanvasIslandState {
    surface: CanvasIslandSurface,
    search_query: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum CanvasIslandSurface {
    #[default]
    Compact,
    FileMenu,
    EditMenu,
    Search,
    Settings,
}

#[derive(Debug, Clone, Copy)]
enum CanvasIslandMenuAction {
    Save,
    SaveAs,
    Open,
    DeleteSelected,
}

#[derive(Component)]
struct CanvasIslandRoot;

#[derive(Component)]
struct CanvasIslandStatusDot;

#[derive(Component)]
struct CanvasIslandFileNameText;

#[derive(Component)]
struct CanvasIslandStatusText;

#[derive(Component)]
struct CanvasIslandInteractive;

#[derive(Component)]
struct CanvasIslandTriggerButton {
    surface: CanvasIslandSurface,
}

#[derive(Component)]
struct CanvasIslandMenuPanel {
    surface: CanvasIslandSurface,
}

#[derive(Component)]
struct CanvasIslandMenuActionButton {
    action: CanvasIslandMenuAction,
}

#[derive(Component)]
struct CanvasIslandRecentFileButton {
    path: String,
}

#[derive(Component)]
struct CanvasIslandRecoverAutosaveButton {
    path: String,
}

#[derive(Component)]
struct CanvasIslandFileDynamicContent;

#[derive(Debug, Clone, Copy)]
enum CanvasIslandSettingsAction {
    ToggleAutosave,
    TogglePrettyJson,
    ToggleHistory,
    DecreaseHistoryLimit,
    IncreaseHistoryLimit,
    ToggleDiagnosticsPanel,
    ToggleScenePreviewPanel,
    ToggleGridDisplayMode,
    CycleGridPalette,
    DecreaseGridPointSize,
    IncreaseGridPointSize,
    CycleWireStyle,
    ToggleWireColorFromOutput,
    ResetToDefaults,
}

#[derive(Debug, Clone, Copy)]
enum CanvasIslandSettingsValueKind {
    Autosave,
    PrettyJson,
    HistoryEnabled,
    HistoryLimit,
    DiagnosticsPanel,
    ScenePreviewPanel,
    GridDisplayMode,
    GridColorPalette,
    GridPointSize,
    WireStyle,
    WireColorFromOutput,
}

#[derive(Component)]
struct CanvasIslandSettingsActionButton {
    action: CanvasIslandSettingsAction,
}

#[derive(Component)]
struct CanvasIslandSettingsValueText {
    kind: CanvasIslandSettingsValueKind,
}

#[derive(Component)]
struct CanvasIslandSearchQueryText;

#[derive(Component)]
struct CanvasIslandSearchResults;

#[derive(Component)]
struct CanvasIslandSearchResultButton {
    definition_id: NodeId,
}

#[derive(Debug, Clone)]
struct CanvasIslandSearchEntry {
    definition_id: NodeId,
    display_name: String,
    category: String,
}

pub struct CanvasIslandPlugin;

impl Plugin for CanvasIslandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CanvasIslandState>()
            .add_systems(Startup, setup_canvas_island_ui)
            .add_systems(
                Update,
                (
                    handle_canvas_island_shortcuts,
                    handle_canvas_island_trigger_buttons,
                    handle_canvas_island_menu_actions,
                    handle_canvas_island_recent_file_buttons,
                    handle_canvas_island_recover_autosave_buttons,
                    handle_canvas_island_settings_actions,
                    handle_canvas_island_search_typing,
                    handle_canvas_island_search_result_buttons,
                    close_canvas_island_menu_on_outside_click,
                    sync_canvas_island_ui_target,
                    sync_canvas_island_overlay,
                    update_canvas_island_menu_visibility,
                    rebuild_canvas_island_file_panel,
                    rebuild_canvas_island_search_panel,
                    sync_canvas_island_settings_values,
                    sync_canvas_island_status,
                    style_canvas_island_buttons,
                )
                    .chain(),
            );
    }
}

fn setup_canvas_island_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(14.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            ZIndex(1800),
            CanvasIslandRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    border_radius: BorderRadius::all(Val::Px(999.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.9)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
            ))
            .with_children(|shell| {
                shell
                    .spawn((
                        Node {
                            min_width: Val::Px(180.0),
                            height: Val::Px(32.0),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            border_radius: BorderRadius::all(Val::Px(999.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                    ))
                    .with_children(|status| {
                        status.spawn((
                            Node {
                                width: Val::Px(8.0),
                                height: Val::Px(8.0),
                                border_radius: BorderRadius::all(Val::Px(999.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.35, 0.85, 0.45)),
                            CanvasIslandStatusDot,
                        ));

                        status.spawn((
                            Text::new("current_graph.json"),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            CanvasIslandFileNameText,
                        ));

                        status.spawn((
                            Text::new("Saved"),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.55)),
                            CanvasIslandStatusText,
                        ));
                    });

                shell
                    .spawn((
                        Button,
                        Node {
                            height: Val::Px(32.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(999.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                        CanvasIslandInteractive,
                        CanvasIslandTriggerButton {
                            surface: CanvasIslandSurface::FileMenu,
                        },
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("File"),
                            TextFont {
                                font_size: 12.5,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                shell
                    .spawn((
                        Button,
                        Node {
                            height: Val::Px(32.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(999.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                        CanvasIslandInteractive,
                        CanvasIslandTriggerButton {
                            surface: CanvasIslandSurface::EditMenu,
                        },
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Edit"),
                            TextFont {
                                font_size: 12.5,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                shell
                    .spawn((
                        Button,
                        Node {
                            height: Val::Px(32.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(999.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                        CanvasIslandInteractive,
                        CanvasIslandTriggerButton {
                            surface: CanvasIslandSurface::Search,
                        },
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Search"),
                            TextFont {
                                font_size: 12.5,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                shell
                    .spawn((
                        Button,
                        Node {
                            height: Val::Px(32.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(999.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                        CanvasIslandInteractive,
                        CanvasIslandTriggerButton {
                            surface: CanvasIslandSurface::Settings,
                        },
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Settings"),
                            TextFont {
                                font_size: 12.5,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });

            spawn_canvas_island_file_panel(root);

            spawn_canvas_island_menu_panel(
                root,
                CanvasIslandSurface::EditMenu,
                "Edit",
                &[(
                    "Delete Selected",
                    "Delete",
                    CanvasIslandMenuAction::DeleteSelected,
                )],
            );

            spawn_canvas_island_search_panel(root);
            spawn_canvas_island_settings_panel(root);
        });
}

fn spawn_canvas_island_file_panel(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            width: Val::Px(280.0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            row_gap: Val::Px(6.0),
            border_radius: BorderRadius::all(Val::Px(22.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.94)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        CanvasIslandMenuPanel {
            surface: CanvasIslandSurface::FileMenu,
        },
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("File"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        ));

        for (label, shortcut, action) in [
            ("Save", "Ctrl+S", CanvasIslandMenuAction::Save),
            ("Save As", "Ctrl+Shift+S", CanvasIslandMenuAction::SaveAs),
            ("Open", "Ctrl+O", CanvasIslandMenuAction::Open),
        ] {
            panel
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(34.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(14.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                    CanvasIslandInteractive,
                    CanvasIslandMenuActionButton { action },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 12.5,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    button.spawn((
                        Text::new(shortcut),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
                    ));
                });
        }

        panel.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            CanvasIslandFileDynamicContent,
        ));
    });
}

fn sync_canvas_island_ui_target(
    mut commands: Commands,
    q_graph_camera: Query<Entity, With<GraphCamera>>,
    q_roots: Query<(Entity, Option<&UiTargetCamera>), With<CanvasIslandRoot>>,
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

fn spawn_canvas_island_menu_panel(
    root: &mut ChildSpawnerCommands,
    surface: CanvasIslandSurface,
    title: &str,
    items: &[(&str, &str, CanvasIslandMenuAction)],
) {
    root.spawn((
        Node {
            width: Val::Px(240.0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            row_gap: Val::Px(6.0),
            border_radius: BorderRadius::all(Val::Px(22.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.94)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        CanvasIslandMenuPanel { surface },
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new(title),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        ));

        for (label, shortcut, action) in items {
            panel
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(34.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(14.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                    CanvasIslandInteractive,
                    CanvasIslandMenuActionButton { action: *action },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(*label),
                        TextFont {
                            font_size: 12.5,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    button.spawn((
                        Text::new(*shortcut),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
                    ));
                });
        }
    });
}

fn spawn_canvas_island_search_panel(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            width: Val::Px(320.0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(8.0),
            border_radius: BorderRadius::all(Val::Px(22.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.96)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        CanvasIslandMenuPanel {
            surface: CanvasIslandSurface::Search,
        },
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("Search"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        ));

        panel
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(34.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(14.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
            ))
            .with_children(|query| {
                query.spawn((
                    Text::new("Type to search nodes..."),
                    TextFont {
                        font_size: 12.5,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                    CanvasIslandSearchQueryText,
                ));
            });

        panel.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            CanvasIslandSearchResults,
        ));
    });
}

fn spawn_canvas_island_settings_panel(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            width: Val::Px(320.0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(8.0),
            border_radius: BorderRadius::all(Val::Px(22.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.96)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        CanvasIslandMenuPanel {
            surface: CanvasIslandSurface::Settings,
        },
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("Settings"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        ));

        spawn_canvas_island_setting_toggle_row(
            panel,
            "Autosave",
            CanvasIslandSettingsAction::ToggleAutosave,
            CanvasIslandSettingsValueKind::Autosave,
        );
        spawn_canvas_island_setting_toggle_row(
            panel,
            "Pretty JSON",
            CanvasIslandSettingsAction::TogglePrettyJson,
            CanvasIslandSettingsValueKind::PrettyJson,
        );
        spawn_canvas_island_setting_toggle_row(
            panel,
            "History",
            CanvasIslandSettingsAction::ToggleHistory,
            CanvasIslandSettingsValueKind::HistoryEnabled,
        );
        spawn_canvas_island_setting_stepper_row(
            panel,
            "History Limit",
            CanvasIslandSettingsAction::DecreaseHistoryLimit,
            CanvasIslandSettingsValueKind::HistoryLimit,
            CanvasIslandSettingsAction::IncreaseHistoryLimit,
        );
        spawn_canvas_island_setting_toggle_row(
            panel,
            "Diagnostics Panel",
            CanvasIslandSettingsAction::ToggleDiagnosticsPanel,
            CanvasIslandSettingsValueKind::DiagnosticsPanel,
        );
        spawn_canvas_island_setting_toggle_row(
            panel,
            "Scene Preview",
            CanvasIslandSettingsAction::ToggleScenePreviewPanel,
            CanvasIslandSettingsValueKind::ScenePreviewPanel,
        );
        spawn_canvas_island_setting_toggle_row(
            panel,
            "Grid Mode",
            CanvasIslandSettingsAction::ToggleGridDisplayMode,
            CanvasIslandSettingsValueKind::GridDisplayMode,
        );
        spawn_canvas_island_setting_toggle_row(
            panel,
            "Grid Palette",
            CanvasIslandSettingsAction::CycleGridPalette,
            CanvasIslandSettingsValueKind::GridColorPalette,
        );
        spawn_canvas_island_setting_stepper_row(
            panel,
            "Point Size",
            CanvasIslandSettingsAction::DecreaseGridPointSize,
            CanvasIslandSettingsValueKind::GridPointSize,
            CanvasIslandSettingsAction::IncreaseGridPointSize,
        );
        spawn_canvas_island_setting_toggle_row(
            panel,
            "Wire Style",
            CanvasIslandSettingsAction::CycleWireStyle,
            CanvasIslandSettingsValueKind::WireStyle,
        );
        spawn_canvas_island_setting_toggle_row(
            panel,
            "Wire Color by Output",
            CanvasIslandSettingsAction::ToggleWireColorFromOutput,
            CanvasIslandSettingsValueKind::WireColorFromOutput,
        );
        spawn_canvas_island_setting_action_row(
            panel,
            "Reset Settings",
            "Reset",
            CanvasIslandSettingsAction::ResetToDefaults,
        );
    });
}

fn spawn_canvas_island_setting_toggle_row(
    panel: &mut ChildSpawnerCommands,
    label: &str,
    action: CanvasIslandSettingsAction,
    value_kind: CanvasIslandSettingsValueKind,
) {
    panel
        .spawn((
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(34.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.5,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            row.spawn((
                Button,
                Node {
                    min_width: Val::Px(72.0),
                    height: Val::Px(24.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(999.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
                CanvasIslandInteractive,
                CanvasIslandSettingsActionButton { action },
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("..."),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    CanvasIslandSettingsValueText { kind: value_kind },
                ));
            });
        });
}

fn spawn_canvas_island_setting_stepper_row(
    panel: &mut ChildSpawnerCommands,
    label: &str,
    decrease_action: CanvasIslandSettingsAction,
    value_kind: CanvasIslandSettingsValueKind,
    increase_action: CanvasIslandSettingsAction,
) {
    panel
        .spawn((
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(34.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.5,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            row.spawn((Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            },))
                .with_children(|controls| {
                    for (text, action) in [("-", decrease_action), ("+", increase_action)] {
                        controls
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(999.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
                                CanvasIslandInteractive,
                                CanvasIslandSettingsActionButton { action },
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new(text),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }

                    controls.spawn((
                        Text::new("0"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        CanvasIslandSettingsValueText { kind: value_kind },
                    ));
                });
        });
}

fn spawn_canvas_island_setting_action_row(
    panel: &mut ChildSpawnerCommands,
    label: &str,
    button_label: &str,
    action: CanvasIslandSettingsAction,
) {
    panel
        .spawn((
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(34.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.5,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            row.spawn((
                Button,
                Node {
                    min_width: Val::Px(82.0),
                    height: Val::Px(24.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(999.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.86, 0.28, 0.24, 0.18)),
                CanvasIslandInteractive,
                CanvasIslandSettingsActionButton { action },
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new(button_label),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

fn handle_canvas_island_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut island: ResMut<CanvasIslandState>,
    mut overlay: ResMut<GraphOverlayState>,
) {
    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl_pressed && keys.just_pressed(KeyCode::KeyK) {
        island.surface = if island.surface == CanvasIslandSurface::Search {
            if overlay.active_surface == GraphOverlaySurface::CanvasIslandMenu {
                overlay.active_surface = GraphOverlaySurface::None;
            }
            CanvasIslandSurface::Compact
        } else {
            island.search_query.clear();
            overlay.active_surface = GraphOverlaySurface::CanvasIslandMenu;
            CanvasIslandSurface::Search
        };
    }

    if ctrl_pressed && keys.just_pressed(KeyCode::Comma) {
        island.surface = if island.surface == CanvasIslandSurface::Settings {
            if overlay.active_surface == GraphOverlaySurface::CanvasIslandMenu {
                overlay.active_surface = GraphOverlaySurface::None;
            }
            CanvasIslandSurface::Compact
        } else {
            overlay.active_surface = GraphOverlaySurface::CanvasIslandMenu;
            CanvasIslandSurface::Settings
        };
    }
}

fn handle_canvas_island_trigger_buttons(
    mut island: ResMut<CanvasIslandState>,
    mut overlay: ResMut<GraphOverlayState>,
    buttons: Query<(&Interaction, &CanvasIslandTriggerButton), Changed<Interaction>>,
) {
    for (interaction, trigger) in buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        island.surface = if island.surface == trigger.surface {
            if overlay.active_surface == GraphOverlaySurface::CanvasIslandMenu {
                overlay.active_surface = GraphOverlaySurface::None;
            }
            CanvasIslandSurface::Compact
        } else {
            if trigger.surface == CanvasIslandSurface::Search {
                island.search_query.clear();
            }
            overlay.active_surface = GraphOverlaySurface::CanvasIslandMenu;
            trigger.surface
        };
    }
}

fn handle_canvas_island_menu_actions(
    mut island: ResMut<CanvasIslandState>,
    mut overlay: ResMut<GraphOverlayState>,
    buttons: Query<(&Interaction, &CanvasIslandMenuActionButton), Changed<Interaction>>,
    mut command_writer: MessageWriter<GraphCommandRequest>,
) {
    for (interaction, action) in buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action.action {
            CanvasIslandMenuAction::Save => {
                command_writer.write(GraphCommandRequest::SaveGraph);
            }
            CanvasIslandMenuAction::SaveAs => {
                command_writer.write(GraphCommandRequest::SaveGraphToPath {
                    path: format!("assets/graphs/graph_{}.json", unix_timestamp_millis()),
                });
            }
            CanvasIslandMenuAction::Open => {
                command_writer.write(GraphCommandRequest::LoadGraph);
            }
            CanvasIslandMenuAction::DeleteSelected => {
                command_writer.write(GraphCommandRequest::DeleteSelectedNodes);
            }
        };

        island.surface = CanvasIslandSurface::Compact;
        if overlay.active_surface == GraphOverlaySurface::CanvasIslandMenu {
            overlay.active_surface = GraphOverlaySurface::None;
        }
    }
}

fn handle_canvas_island_recent_file_buttons(
    mut island: ResMut<CanvasIslandState>,
    mut overlay: ResMut<GraphOverlayState>,
    buttons: Query<
        (&Interaction, &CanvasIslandRecentFileButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut command_writer: MessageWriter<GraphCommandRequest>,
) {
    for (interaction, button) in buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        command_writer.write(GraphCommandRequest::LoadGraphFromPath {
            path: button.path.clone(),
            force_if_dirty: true,
        });
        island.surface = CanvasIslandSurface::Compact;
        if overlay.active_surface == GraphOverlaySurface::CanvasIslandMenu {
            overlay.active_surface = GraphOverlaySurface::None;
        }
    }
}

fn handle_canvas_island_recover_autosave_buttons(
    mut island: ResMut<CanvasIslandState>,
    mut overlay: ResMut<GraphOverlayState>,
    buttons: Query<
        (&Interaction, &CanvasIslandRecoverAutosaveButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut command_writer: MessageWriter<GraphCommandRequest>,
) {
    for (interaction, button) in buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        command_writer.write(GraphCommandRequest::LoadGraphFromPath {
            path: button.path.clone(),
            force_if_dirty: true,
        });
        island.surface = CanvasIslandSurface::Compact;
        if overlay.active_surface == GraphOverlaySurface::CanvasIslandMenu {
            overlay.active_surface = GraphOverlaySurface::None;
        }
    }
}

fn handle_canvas_island_settings_actions(
    buttons: Query<(&Interaction, &CanvasIslandSettingsActionButton), Changed<Interaction>>,
    mut persistence_settings: ResMut<GraphPersistenceSettings>,
    mut history_settings: ResMut<GraphHistorySettings>,
    mut floating_panels: ResMut<FloatingPanelsSettings>,
    mut editor_settings: ResMut<EditorSettings>,
) {
    for (interaction, button) in buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.action {
            CanvasIslandSettingsAction::ToggleAutosave => {
                persistence_settings.autosave_enabled = !persistence_settings.autosave_enabled;
            }
            CanvasIslandSettingsAction::TogglePrettyJson => {
                persistence_settings.pretty_json = !persistence_settings.pretty_json;
            }
            CanvasIslandSettingsAction::ToggleHistory => {
                history_settings.enabled = !history_settings.enabled;
            }
            CanvasIslandSettingsAction::DecreaseHistoryLimit => {
                history_settings.max_entries =
                    history_settings.max_entries.saturating_sub(16).max(16);
            }
            CanvasIslandSettingsAction::IncreaseHistoryLimit => {
                history_settings.max_entries = (history_settings.max_entries + 16).min(256);
            }
            CanvasIslandSettingsAction::ToggleDiagnosticsPanel => {
                floating_panels.show_diagnostics = !floating_panels.show_diagnostics;
            }
            CanvasIslandSettingsAction::ToggleScenePreviewPanel => {
                floating_panels.show_scene_preview = !floating_panels.show_scene_preview;
            }
            CanvasIslandSettingsAction::ToggleGridDisplayMode => {
                editor_settings.grid_display_mode = editor_settings.grid_display_mode.toggle();
            }
            CanvasIslandSettingsAction::CycleGridPalette => {
                editor_settings.grid_color_palette = editor_settings.grid_color_palette.next();
            }
            CanvasIslandSettingsAction::DecreaseGridPointSize => {
                editor_settings.grid_point_size =
                    (editor_settings.grid_point_size - 0.1).clamp(0.5, 3.0);
            }
            CanvasIslandSettingsAction::IncreaseGridPointSize => {
                editor_settings.grid_point_size =
                    (editor_settings.grid_point_size + 0.1).clamp(0.5, 3.0);
            }
            CanvasIslandSettingsAction::CycleWireStyle => {
                editor_settings.wire_style = editor_settings.wire_style.next();
            }
            CanvasIslandSettingsAction::ToggleWireColorFromOutput => {
                editor_settings.wire_color_from_output = !editor_settings.wire_color_from_output;
            }
            CanvasIslandSettingsAction::ResetToDefaults => {
                let persistence_defaults = GraphPersistenceSettings::default();
                persistence_settings.pretty_json = persistence_defaults.pretty_json;
                persistence_settings.autosave_enabled = persistence_defaults.autosave_enabled;
                *history_settings = GraphHistorySettings::default();
                *floating_panels = FloatingPanelsSettings::default();
                *editor_settings = EditorSettings::default();
            }
        }
    }
}

fn handle_canvas_island_search_typing(
    keys: Res<ButtonInput<KeyCode>>,
    mut island: ResMut<CanvasIslandState>,
) {
    if island.surface != CanvasIslandSurface::Search {
        return;
    }

    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl_pressed {
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        island.search_query.pop();
    }

    if keys.just_pressed(KeyCode::Space) {
        island.search_query.push(' ');
    }

    if let Some(ch) = just_pressed_search_char(&keys) {
        island.search_query.push(ch);
    }
}

fn handle_canvas_island_search_result_buttons(
    mut island: ResMut<CanvasIslandState>,
    mut overlay: ResMut<GraphOverlayState>,
    mut command_writer: MessageWriter<GraphCommandRequest>,
    buttons: Query<
        (&Interaction, &CanvasIslandSearchResultButton),
        (Changed<Interaction>, With<Button>),
    >,
    camera_query: Query<&Transform, With<GraphCamera>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let spawn_position = camera_transform.translation.truncate();

    for (interaction, button) in buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        command_writer.write(GraphCommandRequest::SpawnNode {
            definition_id: button.definition_id.clone(),
            position: spawn_position,
        });
        island.surface = CanvasIslandSurface::Compact;
        island.search_query.clear();
        if overlay.active_surface == GraphOverlaySurface::CanvasIslandMenu {
            overlay.active_surface = GraphOverlaySurface::None;
        }
    }
}

fn close_canvas_island_menu_on_outside_click(
    mut island: ResMut<CanvasIslandState>,
    mut overlay: ResMut<GraphOverlayState>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    interactions: Query<&Interaction, With<CanvasIslandInteractive>>,
) {
    if island.surface == CanvasIslandSurface::Compact {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        island.surface = CanvasIslandSurface::Compact;
        if overlay.active_surface == GraphOverlaySurface::CanvasIslandMenu {
            overlay.active_surface = GraphOverlaySurface::None;
        }
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) && !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let inside_island = interactions.iter().any(|interaction| {
        *interaction == Interaction::Hovered || *interaction == Interaction::Pressed
    });

    if !inside_island {
        island.surface = CanvasIslandSurface::Compact;
        if overlay.active_surface == GraphOverlaySurface::CanvasIslandMenu {
            overlay.active_surface = GraphOverlaySurface::None;
        }
    }
}

fn sync_canvas_island_overlay(
    mut island: ResMut<CanvasIslandState>,
    overlay: Res<GraphOverlayState>,
) {
    if island.surface != CanvasIslandSurface::Compact
        && overlay.active_surface != GraphOverlaySurface::CanvasIslandMenu
    {
        island.surface = CanvasIslandSurface::Compact;
    }
}

fn update_canvas_island_menu_visibility(
    island: Res<CanvasIslandState>,
    mut panels: Query<(&CanvasIslandMenuPanel, &mut Node)>,
) {
    if !island.is_changed() {
        return;
    }

    for (panel, mut node) in panels.iter_mut() {
        node.display = if island.surface == panel.surface {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn rebuild_canvas_island_file_panel(
    mut commands: Commands,
    island: Res<CanvasIslandState>,
    workflow: Res<EditorWorkflowState>,
    persistence_settings: Res<GraphPersistenceSettings>,
    runtime: Res<GraphPersistenceRuntimeState>,
    dynamic_content_entity: Query<Entity, With<CanvasIslandFileDynamicContent>>,
    children_query: Query<&Children>,
) {
    if !island.is_changed()
        && !workflow.is_changed()
        && !persistence_settings.is_changed()
        && !runtime.is_changed()
    {
        return;
    }

    let Ok(content_entity) = dynamic_content_entity.single() else {
        return;
    };

    if let Ok(existing_children) = children_query.get(content_entity) {
        for child in existing_children.iter() {
            commands.entity(child).try_despawn();
        }
    }

    if island.surface != CanvasIslandSurface::FileMenu {
        return;
    }

    let latest_backup_path = latest_backup_file(&persistence_settings.backup_directory)
        .ok()
        .flatten()
        .and_then(|path| path.to_str().map(|path| path.to_string()));
    let recent_files: Vec<String> = workflow
        .recent_files
        .iter()
        .filter(|path| Path::new(path).is_file())
        .cloned()
        .collect();

    commands.queue(move |world: &mut World| {
        let Ok(mut content_entity_mut) = world.get_entity_mut(content_entity) else {
            return;
        };

        content_entity_mut.with_children(|content| {
            content.spawn((
                Text::new("Recovery"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.42)),
            ));

            match latest_backup_path.as_deref() {
                Some(path) => {
                    let file_name = Path::new(path)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(path)
                        .to_string();

                    content
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(42.0),
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::FlexStart,
                                border_radius: BorderRadius::all(Val::Px(14.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.36, 0.58, 0.96, 0.16)),
                            CanvasIslandInteractive,
                            CanvasIslandRecoverAutosaveButton {
                                path: path.to_string(),
                            },
                        ))
                        .with_children(|button| {
                            button
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(2.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::NONE),
                                ))
                                .with_children(|column| {
                                    column.spawn((
                                        Text::new("Recover Latest Autosave"),
                                        TextFont {
                                            font_size: 12.5,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));

                                    column.spawn((
                                        Text::new(file_name),
                                        TextFont {
                                            font_size: 10.5,
                                            ..default()
                                        },
                                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                                    ));
                                });
                        });
                }
                None => {
                    content.spawn((
                        Text::new("No autosave backup found"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.55)),
                    ));
                }
            }

            content.spawn((
                Text::new("Recent Files"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.42)),
            ));

            if recent_files.is_empty() {
                content.spawn((
                    Text::new("No recent files"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.55)),
                ));
            } else {
                for path in &recent_files {
                    let file_name = Path::new(path)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(path)
                        .to_string();

                    content
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(42.0),
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::FlexStart,
                                border_radius: BorderRadius::all(Val::Px(14.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                            CanvasIslandInteractive,
                            CanvasIslandRecentFileButton {
                                path: path.to_string(),
                            },
                        ))
                        .with_children(|button| {
                            button
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(2.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::NONE),
                                ))
                                .with_children(|column| {
                                    column.spawn((
                                        Text::new(file_name),
                                        TextFont {
                                            font_size: 12.5,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));

                                    column.spawn((
                                        Text::new(path),
                                        TextFont {
                                            font_size: 10.5,
                                            ..default()
                                        },
                                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.42)),
                                    ));
                                });
                        });
                }
            }
        });
    });
}

fn rebuild_canvas_island_search_panel(
    mut commands: Commands,
    island: Res<CanvasIslandState>,
    registry: Res<NodeRegistry>,
    query_text_entity: Query<Entity, With<CanvasIslandSearchQueryText>>,
    results_entity: Query<Entity, With<CanvasIslandSearchResults>>,
    mut text_query: Query<&mut Text>,
    children_query: Query<&Children>,
) {
    if !island.is_changed() {
        return;
    }

    let Ok(query_entity) = query_text_entity.single() else {
        return;
    };
    let Ok(results_entity) = results_entity.single() else {
        return;
    };

    if let Ok(mut text) = text_query.get_mut(query_entity) {
        text.0 = if island.search_query.is_empty() {
            "Type to search nodes...".to_string()
        } else {
            island.search_query.clone()
        };
    }

    if let Ok(existing_children) = children_query.get(results_entity) {
        for child in existing_children.iter() {
            commands.entity(child).try_despawn();
        }
    }

    if island.surface != CanvasIslandSurface::Search {
        return;
    }

    let mut nodes = if island.search_query.is_empty() {
        registry.get_menu_nodes_sorted()
    } else {
        let mut matches: Vec<_> = registry
            .search(&island.search_query)
            .into_iter()
            .filter(|definition| definition.show_in_menu())
            .collect();
        matches.sort_by(|a, b| a.display_name().cmp(b.display_name()));
        matches
    };
    nodes.truncate(8);

    let search_entries: Vec<_> = nodes
        .into_iter()
        .map(|definition| CanvasIslandSearchEntry {
            definition_id: definition.id(),
            display_name: definition.display_name().to_string(),
            category: definition.category().as_str().to_string(),
        })
        .collect();

    commands.queue(move |world: &mut World| {
        let Ok(mut results_entity_mut) = world.get_entity_mut(results_entity) else {
            return;
        };

        results_entity_mut.with_children(|results| {
            if search_entries.is_empty() {
                results.spawn((
                    Text::new("No matching nodes"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.55)),
                ));
                return;
            }

            for entry in &search_entries {
                results
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(36.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(14.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                        CanvasIslandInteractive,
                        CanvasIslandSearchResultButton {
                            definition_id: entry.definition_id.clone(),
                        },
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(&entry.display_name),
                            TextFont {
                                font_size: 12.5,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        button.spawn((
                            Text::new(&entry.category),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
                        ));
                    });
            }
        });
    });
}

fn sync_canvas_island_settings_values(
    persistence_settings: Res<GraphPersistenceSettings>,
    history_settings: Res<GraphHistorySettings>,
    floating_panels: Res<FloatingPanelsSettings>,
    editor_settings: Res<EditorSettings>,
    mut texts: Query<(&CanvasIslandSettingsValueText, &mut Text)>,
) {
    if !persistence_settings.is_changed()
        && !history_settings.is_changed()
        && !floating_panels.is_changed()
        && !editor_settings.is_changed()
    {
        return;
    }

    for (marker, mut text) in texts.iter_mut() {
        text.0 = match marker.kind {
            CanvasIslandSettingsValueKind::Autosave => if persistence_settings.autosave_enabled {
                "ON"
            } else {
                "OFF"
            }
            .to_string(),
            CanvasIslandSettingsValueKind::PrettyJson => if persistence_settings.pretty_json {
                "ON"
            } else {
                "OFF"
            }
            .to_string(),
            CanvasIslandSettingsValueKind::HistoryEnabled => if history_settings.enabled {
                "ON"
            } else {
                "OFF"
            }
            .to_string(),
            CanvasIslandSettingsValueKind::HistoryLimit => history_settings.max_entries.to_string(),
            CanvasIslandSettingsValueKind::DiagnosticsPanel => {
                if floating_panels.show_diagnostics {
                    "ON"
                } else {
                    "OFF"
                }
                .to_string()
            }
            CanvasIslandSettingsValueKind::ScenePreviewPanel => {
                if floating_panels.show_scene_preview {
                    "ON"
                } else {
                    "OFF"
                }
                .to_string()
            }
            CanvasIslandSettingsValueKind::GridDisplayMode => {
                editor_settings.grid_display_mode.label().to_string()
            }
            CanvasIslandSettingsValueKind::GridColorPalette => {
                editor_settings.grid_color_palette.label().to_string()
            }
            CanvasIslandSettingsValueKind::GridPointSize => {
                format!("{:.1}x", editor_settings.grid_point_size)
            }
            CanvasIslandSettingsValueKind::WireStyle => {
                editor_settings.wire_style.label().to_string()
            }
            CanvasIslandSettingsValueKind::WireColorFromOutput => {
                if editor_settings.wire_color_from_output {
                    "ON"
                } else {
                    "OFF"
                }
                .to_string()
            }
        };
    }
}

fn sync_canvas_island_status(
    settings: Res<GraphPersistenceSettings>,
    runtime: Res<GraphPersistenceRuntimeState>,
    persistence_status: Res<GraphPersistenceStatus>,
    mut file_name_text: Query<
        &mut Text,
        (
            With<CanvasIslandFileNameText>,
            Without<CanvasIslandStatusText>,
        ),
    >,
    mut status_text: Query<
        &mut Text,
        (
            With<CanvasIslandStatusText>,
            Without<CanvasIslandFileNameText>,
        ),
    >,
    mut dot: Query<&mut BackgroundColor, With<CanvasIslandStatusDot>>,
) {
    let file_name = Path::new(&settings.file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("graph.json");

    if let Ok(mut text) = file_name_text.single_mut() {
        text.0 = file_name.to_string();
    }

    if let Ok(mut text) = status_text.single_mut() {
        text.0 = if let Some(active) = &persistence_status.active {
            active.text.clone()
        } else if runtime.dirty {
            "Unsaved".to_string()
        } else {
            "Saved".to_string()
        };
    }

    if let Ok(mut background) = dot.single_mut() {
        background.0 = if let Some(active) = &persistence_status.active {
            match active.severity {
                GraphPersistenceStatusSeverity::Info => Color::srgb(0.35, 0.85, 0.45),
                GraphPersistenceStatusSeverity::Warning => Color::srgb(0.93, 0.62, 0.18),
                GraphPersistenceStatusSeverity::Error => Color::srgb(0.92, 0.34, 0.34),
            }
        } else if runtime.dirty {
            Color::srgb(0.93, 0.62, 0.18)
        } else {
            Color::srgb(0.35, 0.85, 0.45)
        };
    }
}

fn style_canvas_island_buttons(
    island: Res<CanvasIslandState>,
    mut trigger_buttons: Query<
        (
            &Interaction,
            &CanvasIslandTriggerButton,
            &mut BackgroundColor,
        ),
        (
            With<Button>,
            With<CanvasIslandTriggerButton>,
            Without<CanvasIslandMenuActionButton>,
        ),
    >,
    mut menu_buttons: Query<
        (
            &Interaction,
            &CanvasIslandMenuActionButton,
            &mut BackgroundColor,
        ),
        (
            With<Button>,
            With<CanvasIslandMenuActionButton>,
            Without<CanvasIslandTriggerButton>,
        ),
    >,
    mut settings_buttons: Query<
        (
            &Interaction,
            &CanvasIslandSettingsActionButton,
            &mut BackgroundColor,
        ),
        (
            With<Button>,
            With<CanvasIslandSettingsActionButton>,
            Without<CanvasIslandTriggerButton>,
            Without<CanvasIslandMenuActionButton>,
        ),
    >,
) {
    for (interaction, trigger, mut background) in trigger_buttons.iter_mut() {
        background.0 = if island.surface == trigger.surface {
            Color::srgba(0.9, 0.93, 1.0, 0.18)
        } else {
            match *interaction {
                Interaction::Pressed => Color::srgba(0.9, 0.93, 1.0, 0.16),
                Interaction::Hovered => Color::srgba(1.0, 1.0, 1.0, 0.1),
                Interaction::None => Color::srgba(1.0, 1.0, 1.0, 0.06),
            }
        };
    }

    for (interaction, _, mut background) in menu_buttons.iter_mut() {
        background.0 = match *interaction {
            Interaction::Pressed => Color::srgba(0.9, 0.93, 1.0, 0.18),
            Interaction::Hovered => Color::srgba(1.0, 1.0, 1.0, 0.1),
            Interaction::None => Color::srgba(1.0, 1.0, 1.0, 0.05),
        };
    }

    for (interaction, _, mut background) in settings_buttons.iter_mut() {
        background.0 = match *interaction {
            Interaction::Pressed => Color::srgba(0.9, 0.93, 1.0, 0.18),
            Interaction::Hovered => Color::srgba(1.0, 1.0, 1.0, 0.1),
            Interaction::None => Color::srgba(1.0, 1.0, 1.0, 0.08),
        };
    }
}

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn just_pressed_search_char(keys: &ButtonInput<KeyCode>) -> Option<char> {
    const LETTERS: &[(KeyCode, char)] = &[
        (KeyCode::KeyA, 'a'),
        (KeyCode::KeyB, 'b'),
        (KeyCode::KeyC, 'c'),
        (KeyCode::KeyD, 'd'),
        (KeyCode::KeyE, 'e'),
        (KeyCode::KeyF, 'f'),
        (KeyCode::KeyG, 'g'),
        (KeyCode::KeyH, 'h'),
        (KeyCode::KeyI, 'i'),
        (KeyCode::KeyJ, 'j'),
        (KeyCode::KeyK, 'k'),
        (KeyCode::KeyL, 'l'),
        (KeyCode::KeyM, 'm'),
        (KeyCode::KeyN, 'n'),
        (KeyCode::KeyO, 'o'),
        (KeyCode::KeyP, 'p'),
        (KeyCode::KeyQ, 'q'),
        (KeyCode::KeyR, 'r'),
        (KeyCode::KeyS, 's'),
        (KeyCode::KeyT, 't'),
        (KeyCode::KeyU, 'u'),
        (KeyCode::KeyV, 'v'),
        (KeyCode::KeyW, 'w'),
        (KeyCode::KeyX, 'x'),
        (KeyCode::KeyY, 'y'),
        (KeyCode::KeyZ, 'z'),
    ];
    const DIGITS: &[(KeyCode, char)] = &[
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
    ];

    for (key, ch) in LETTERS {
        if keys.just_pressed(*key) {
            return Some(*ch);
        }
    }

    for (key, ch) in DIGITS {
        if keys.just_pressed(*key) {
            return Some(*ch);
        }
    }

    None
}
