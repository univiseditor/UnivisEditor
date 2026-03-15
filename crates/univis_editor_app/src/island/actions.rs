use super::*;

pub(super) fn handle_canvas_island_shortcuts_system(
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

pub(super) fn handle_canvas_island_trigger_buttons_system(
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

pub(super) fn handle_canvas_island_menu_actions_system(
    mut island: ResMut<CanvasIslandState>,
    mut overlay: ResMut<GraphOverlayState>,
    buttons: Query<(&Interaction, &CanvasIslandMenuActionButton), Changed<Interaction>>,
    mut command_writer: MessageWriter<GraphCommandRequest>,
    live_document: Res<univis_node_graph::prelude::LiveGraphDocumentState>,
    camera_query: Query<&Transform, With<GraphCamera>>,
) {
    let spawn_position = camera_query
        .iter()
        .next()
        .map(|transform| transform.translation.truncate())
        .unwrap_or(Vec2::ZERO);

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
            CanvasIslandMenuAction::DuplicateSelected => {
                command_writer.write(GraphCommandRequest::DuplicateSelectedNodes);
            }
            CanvasIslandMenuAction::FrameSelected => {
                command_writer.write(GraphCommandRequest::FrameSelectedNodes);
            }
            CanvasIslandMenuAction::CapturePrefab => {
                command_writer.write(GraphCommandRequest::CapturePrefabFromSelection);
            }
            CanvasIslandMenuAction::CaptureSubgraph => {
                command_writer.write(GraphCommandRequest::CaptureSubgraphFromSelection);
            }
            CanvasIslandMenuAction::InsertLatestSubgraph => {
                command_writer.write(GraphCommandRequest::InsertSubgraph {
                    subgraph_id: live_document
                        .document
                        .subgraphs
                        .last()
                        .map(|subgraph| subgraph.id.clone())
                        .unwrap_or_default(),
                    position: spawn_position,
                });
            }
            CanvasIslandMenuAction::SpawnLatestPrefabNode => {
                command_writer.write(GraphCommandRequest::SpawnPrefabNode {
                    prefab_id: live_document
                        .document
                        .prefabs
                        .last()
                        .map(|prefab| prefab.id.clone())
                        .unwrap_or_default(),
                    position: spawn_position,
                });
            }
        };

        island.surface = CanvasIslandSurface::Compact;
        if overlay.active_surface == GraphOverlaySurface::CanvasIslandMenu {
            overlay.active_surface = GraphOverlaySurface::None;
        }
    }
}

pub(super) fn handle_canvas_island_recent_file_buttons_system(
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

pub(super) fn handle_canvas_island_recover_autosave_buttons_system(
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

pub(super) fn handle_canvas_island_settings_actions_system(
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

pub(super) fn handle_canvas_island_search_typing_system(
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

pub(super) fn handle_canvas_island_search_result_buttons_system(
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

pub(super) fn close_canvas_island_menu_on_outside_click_system(
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
