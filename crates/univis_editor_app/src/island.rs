use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use univis_node_graph::commands::{
    GraphCommandRequest, GraphOverlayState, GraphOverlaySurface,
};
use univis_node_graph::node_definition::NodeId;
use univis_node_graph::node_registry::NodeRegistry;
use univis_editor_persistence::graph_persistence::{
    GraphPersistenceRuntimeState, GraphPersistenceSettings, GraphPersistenceStatus,
    GraphPersistenceStatusSeverity,
};
use univis_editor_ui::prelude::GraphCamera;

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
struct CanvasIslandSearchQueryText;

#[derive(Component)]
struct CanvasIslandSearchResults;

#[derive(Component)]
struct CanvasIslandSearchResultButton {
    definition_id: NodeId,
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
                    handle_canvas_island_search_typing,
                    handle_canvas_island_search_result_buttons,
                    close_canvas_island_menu_on_outside_click,
                    sync_canvas_island_ui_target,
                    sync_canvas_island_overlay,
                    update_canvas_island_menu_visibility,
                    rebuild_canvas_island_search_panel,
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
                shell.spawn((
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
            });

            spawn_canvas_island_menu_panel(
                root,
                CanvasIslandSurface::FileMenu,
                "File",
                &[
                    ("Save", "Ctrl+S", CanvasIslandMenuAction::Save),
                    ("Save As", "Ctrl+Shift+S", CanvasIslandMenuAction::SaveAs),
                    ("Open", "Ctrl+O", CanvasIslandMenuAction::Open),
                ],
            );

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

        commands.entity(entity).insert(UiTargetCamera(graph_camera));
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
            panel.spawn((
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

        panel.spawn((
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

    let inside_island = interactions
        .iter()
        .any(|interaction| *interaction == Interaction::Hovered || *interaction == Interaction::Pressed);

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
            commands.entity(child).despawn();
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

    commands.entity(results_entity).with_children(|results| {
        if nodes.is_empty() {
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

        for definition in nodes {
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
                        definition_id: definition.id(),
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(definition.display_name()),
                        TextFont {
                            font_size: 12.5,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    button.spawn((
                        Text::new(definition.category().as_str()),
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
        (&Interaction, &CanvasIslandTriggerButton, &mut BackgroundColor),
        (
            With<Button>,
            With<CanvasIslandTriggerButton>,
            Without<CanvasIslandMenuActionButton>,
        ),
    >,
    mut menu_buttons: Query<
        (&Interaction, &CanvasIslandMenuActionButton, &mut BackgroundColor),
        (
            With<Button>,
            With<CanvasIslandMenuActionButton>,
            Without<CanvasIslandTriggerButton>,
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
