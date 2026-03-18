use super::*;
use univis_ui::prelude::{Icon, Theme};

pub(super) fn sync_canvas_island_ui_target_system(
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

pub(super) fn sync_canvas_island_overlay_system(
    mut island: ResMut<CanvasIslandState>,
    overlay: Res<GraphOverlayState>,
) {
    if island.surface != CanvasIslandSurface::Compact
        && overlay.active_surface != GraphOverlaySurface::CanvasIslandMenu
    {
        island.surface = CanvasIslandSurface::Compact;
    }
}

pub(super) fn update_canvas_island_menu_visibility_system(
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

pub(super) fn rebuild_canvas_island_file_panel_system(
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

pub(super) fn rebuild_canvas_island_assets_panel_system(
    mut commands: Commands,
    island: Res<CanvasIslandState>,
    live_document: Res<univis_node_graph::prelude::LiveGraphDocumentState>,
    theme: Res<Theme>,
    dynamic_content_entity: Query<Entity, With<CanvasIslandAssetsDynamicContent>>,
    children_query: Query<&Children>,
) {
    if !island.is_changed() && !live_document.is_changed() {
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

    if island.surface != CanvasIslandSurface::Assets {
        return;
    }

    let prefab_entries: Vec<_> = live_document
        .document
        .prefabs
        .iter()
        .rev()
        .map(|prefab| {
            let root_name = prefab
                .root
                .name
                .as_deref()
                .filter(|name| !name.is_empty())
                .unwrap_or("<unnamed root>")
                .to_string();
            let entity_count = count_entity_value_nodes(&prefab.root);
            (
                prefab.id.clone(),
                prefab.name.clone(),
                format!("root: {}  •  {} entity node(s)", root_name, entity_count),
            )
        })
        .collect();
    let subgraph_entries: Vec<_> = live_document
        .document
        .subgraphs
        .iter()
        .rev()
        .map(|subgraph| {
            (
                subgraph.id.clone(),
                subgraph.name.clone(),
                format!(
                    "{} node(s)  •  {} wire(s)",
                    subgraph.document.nodes.len(),
                    subgraph.document.edges.len()
                ),
            )
        })
        .collect();
    let icon_font = theme.icon.font.clone();

    commands.queue(move |world: &mut World| {
        let Ok(mut content_entity_mut) = world.get_entity_mut(content_entity) else {
            return;
        };

        content_entity_mut.with_children(|content| {
            content.spawn((
                Text::new("Prefabs"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.42)),
            ));

            if prefab_entries.is_empty() {
                content.spawn((
                    Text::new("No prefabs captured yet. Use Edit > Capture Prefab."),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.55)),
                ));
            } else {
                for (prefab_id, name, summary) in &prefab_entries {
                    content
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(46.0),
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(14.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                        ))
                        .with_children(|card| {
                            card.spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(2.0),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|column| {
                                column.spawn((
                                    Text::new(name.clone()),
                                    TextFont {
                                        font_size: 12.5,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                                column.spawn((
                                    Text::new(summary.clone()),
                                    TextFont {
                                        font_size: 10.5,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
                                ));
                            });

                            card.spawn((
                                Node {
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(6.0),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|actions| {
                                actions
                                    .spawn((
                                        Button,
                                        Node {
                                            min_width: Val::Px(78.0),
                                            height: Val::Px(28.0),
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            column_gap: Val::Px(5.0),
                                            border_radius: BorderRadius::all(Val::Px(999.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(1.0, 0.83, 0.42, 0.16)),
                                        CanvasIslandInteractive,
                                        CanvasIslandUpdatePrefabAssetButton {
                                            prefab_id: prefab_id.clone(),
                                        },
                                    ))
                                    .with_children(|button| {
                                        button.spawn((
                                            Text::new(Icon::REFRESH_CW),
                                            TextFont {
                                                font: icon_font.clone(),
                                                font_size: 12.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgba(1.0, 0.9, 0.62, 0.92)),
                                        ));
                                        button.spawn((
                                            Text::new("Update"),
                                            TextFont {
                                                font_size: 10.5,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });

                                actions
                                    .spawn((
                                        Button,
                                        Node {
                                            min_width: Val::Px(72.0),
                                            height: Val::Px(28.0),
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            column_gap: Val::Px(5.0),
                                            border_radius: BorderRadius::all(Val::Px(999.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.36, 0.58, 0.96, 0.16)),
                                        CanvasIslandInteractive,
                                        CanvasIslandPrefabAssetButton {
                                            prefab_id: prefab_id.clone(),
                                        },
                                    ))
                                    .with_children(|button| {
                                        button.spawn((
                                            Text::new(Icon::BOX),
                                            TextFont {
                                                font: icon_font.clone(),
                                                font_size: 12.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgba(0.84, 0.91, 1.0, 0.92)),
                                        ));
                                        button.spawn((
                                            Text::new("Spawn"),
                                            TextFont {
                                                font_size: 10.5,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            });
                        });
                }
            }

            content.spawn((
                Text::new("Subgraphs"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.42)),
            ));

            if subgraph_entries.is_empty() {
                content.spawn((
                    Text::new("No subgraphs captured yet. Use Edit > Capture Subgraph."),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.55)),
                ));
            } else {
                for (subgraph_id, name, summary) in &subgraph_entries {
                    content
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(46.0),
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(14.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                        ))
                        .with_children(|card| {
                            card.spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(2.0),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|column| {
                                column.spawn((
                                    Text::new(name.clone()),
                                    TextFont {
                                        font_size: 12.5,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                                column.spawn((
                                    Text::new(summary.clone()),
                                    TextFont {
                                        font_size: 10.5,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
                                ));
                            });

                            card.spawn((
                                Node {
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(6.0),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|actions| {
                                actions
                                    .spawn((
                                        Button,
                                        Node {
                                            min_width: Val::Px(78.0),
                                            height: Val::Px(28.0),
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            column_gap: Val::Px(5.0),
                                            border_radius: BorderRadius::all(Val::Px(999.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(1.0, 0.83, 0.42, 0.16)),
                                        CanvasIslandInteractive,
                                        CanvasIslandUpdateSubgraphAssetButton {
                                            subgraph_id: subgraph_id.clone(),
                                        },
                                    ))
                                    .with_children(|button| {
                                        button.spawn((
                                            Text::new(Icon::REFRESH_CW),
                                            TextFont {
                                                font: icon_font.clone(),
                                                font_size: 12.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgba(1.0, 0.9, 0.62, 0.92)),
                                        ));
                                        button.spawn((
                                            Text::new("Update"),
                                            TextFont {
                                                font_size: 10.5,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });

                                actions
                                    .spawn((
                                        Button,
                                        Node {
                                            min_width: Val::Px(68.0),
                                            height: Val::Px(28.0),
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            column_gap: Val::Px(5.0),
                                            border_radius: BorderRadius::all(Val::Px(999.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.36, 0.58, 0.96, 0.16)),
                                        CanvasIslandInteractive,
                                        CanvasIslandSubgraphAssetButton {
                                            subgraph_id: subgraph_id.clone(),
                                        },
                                    ))
                                    .with_children(|button| {
                                        button.spawn((
                                            Text::new(Icon::LAYOUT_GRID),
                                            TextFont {
                                                font: icon_font.clone(),
                                                font_size: 12.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgba(0.84, 0.91, 1.0, 0.92)),
                                        ));
                                        button.spawn((
                                            Text::new("Insert"),
                                            TextFont {
                                                font_size: 10.5,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            });
                        });
                }
            }
        });
    });
}

pub(super) fn rebuild_canvas_island_search_panel_system(
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

pub(super) fn sync_canvas_island_settings_values_system(
    persistence_settings: Res<GraphPersistenceSettings>,
    history_settings: Res<GraphHistorySettings>,
    floating_panels: Res<FloatingPanelsSettings>,
    runtime_trace_settings: Res<GraphRuntimeTraceSettings>,
    editor_settings: Res<EditorSettings>,
    mut texts: Query<(&CanvasIslandSettingsValueText, &mut Text)>,
) {
    if !persistence_settings.is_changed()
        && !history_settings.is_changed()
        && !floating_panels.is_changed()
        && !runtime_trace_settings.is_changed()
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
            CanvasIslandSettingsValueKind::ConnectionInspectorPanel => {
                if floating_panels.show_connection_inspector {
                    "ON"
                } else {
                    "OFF"
                }
                .to_string()
            }
            CanvasIslandSettingsValueKind::RuntimeTrace => if runtime_trace_settings.enabled {
                "ON"
            } else {
                "OFF"
            }
            .to_string(),
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

pub(super) fn sync_canvas_island_status_system(
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

pub(super) fn style_canvas_island_buttons_system(
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

fn count_entity_value_nodes(entity: &univis_scene::EntityValue) -> usize {
    1 + entity
        .children
        .iter()
        .map(count_entity_value_nodes)
        .sum::<usize>()
}
