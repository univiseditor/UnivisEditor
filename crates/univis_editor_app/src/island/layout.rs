use super::*;

pub(super) fn setup_canvas_island_ui_system(mut commands: Commands) {
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

                for (label, surface) in [
                    ("File", CanvasIslandSurface::FileMenu),
                    ("Edit", CanvasIslandSurface::EditMenu),
                    ("Assets", CanvasIslandSurface::Assets),
                    ("Search", CanvasIslandSurface::Search),
                    ("Settings", CanvasIslandSurface::Settings),
                ] {
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
                            CanvasIslandTriggerButton { surface },
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
                        });
                }
            });

            spawn_canvas_island_file_panel(root);
            spawn_canvas_island_assets_panel(root);

            spawn_canvas_island_menu_panel(
                root,
                CanvasIslandSurface::EditMenu,
                "Edit",
                &[
                    (
                        "Delete Selected",
                        "Delete",
                        CanvasIslandMenuAction::DeleteSelected,
                    ),
                    (
                        "Duplicate Selected",
                        "Ctrl+D",
                        CanvasIslandMenuAction::DuplicateSelected,
                    ),
                    ("Frame Selected", "F", CanvasIslandMenuAction::FrameSelected),
                    (
                        "Capture Prefab",
                        "Store",
                        CanvasIslandMenuAction::CapturePrefab,
                    ),
                    (
                        "Capture Subgraph",
                        "Store",
                        CanvasIslandMenuAction::CaptureSubgraph,
                    ),
                    (
                        "Insert Latest Subgraph",
                        "Insert",
                        CanvasIslandMenuAction::InsertLatestSubgraph,
                    ),
                    (
                        "Spawn Latest Prefab Node",
                        "Spawn",
                        CanvasIslandMenuAction::SpawnLatestPrefabNode,
                    ),
                ],
            );

            spawn_canvas_island_search_panel(root);
            spawn_canvas_island_settings_panel(root);
        });
}

fn spawn_canvas_island_assets_panel(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            width: Val::Px(340.0),
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
            surface: CanvasIslandSurface::Assets,
        },
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("Assets"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        ));

        panel.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            CanvasIslandAssetsDynamicContent,
        ));
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
            "Connection Inspector",
            CanvasIslandSettingsAction::ToggleConnectionInspectorPanel,
            CanvasIslandSettingsValueKind::ConnectionInspectorPanel,
        );
        spawn_canvas_island_setting_toggle_row(
            panel,
            "Runtime Trace",
            CanvasIslandSettingsAction::ToggleRuntimeTrace,
            CanvasIslandSettingsValueKind::RuntimeTrace,
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
