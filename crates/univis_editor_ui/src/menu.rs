//! Context-menu systems for spawning registered nodes.
use crate::prelude::*;
use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use univis_ui::prelude::*;

#[derive(Resource, Default)]
pub struct ContextMenuState {
    pub is_open: bool,
    pub position: Vec2,
    pub search_query: String,
}

#[derive(Component)]
pub struct ContextMenuUI;

#[derive(Component)]
pub struct SearchInput;

#[derive(Component)]
pub struct CategoryHeader {
    pub category: String,
}

/// Opens or closes the node spawn context menu at the current cursor position.
pub fn open_context_menu_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut menu_state: ResMut<ContextMenuState>,
    mut overlay: ResMut<GraphOverlayState>,
    activation: Option<Res<GraphEditingUiActivation>>,
    ports: Query<&UInteraction, With<GraphPort>>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        menu_state.is_open = false;
        if overlay.active_surface == GraphOverlaySurface::ContextMenu {
            overlay.active_surface = GraphOverlaySurface::None;
        }
        return;
    }

    if mouse_button.just_pressed(MouseButton::Right) {
        let clicked_on_port = ports.iter().any(|interaction| {
            matches!(
                *interaction,
                UInteraction::Pressed | UInteraction::Hovered | UInteraction::Clicked
            )
        });
        if !clicked_on_port {
            if !menu_state.is_open {
                if let Ok(window) = windows.single() {
                    if let Some(pos) = window.cursor_position() {
                        menu_state.is_open = true;
                        menu_state.position = pos;
                        menu_state.search_query.clear();
                        overlay.active_surface = GraphOverlaySurface::ContextMenu;
                    }
                }
            } else {
                menu_state.is_open = false;
                if overlay.active_surface == GraphOverlaySurface::ContextMenu {
                    overlay.active_surface = GraphOverlaySurface::None;
                }
            }
        }
    }
}

/// Keeps the context menu in sync with the shared overlay focus state.
pub fn sync_context_menu_overlay_system(
    mut menu_state: ResMut<ContextMenuState>,
    overlay: Res<GraphOverlayState>,
) {
    if menu_state.is_open && overlay.active_surface != GraphOverlaySurface::ContextMenu {
        menu_state.is_open = false;
    }
}

/// Builds the floating node spawn menu for the active search and category state.
pub fn draw_context_menu_system(
    mut commands: Commands,
    menu_state: Res<ContextMenuState>,
    activation: Option<Res<GraphEditingUiActivation>>,
    existing_menu: Query<Entity, With<ContextMenuUI>>,
    registry: Res<NodeRegistry>,
    graph_camera: Query<Entity, With<GraphCamera>>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        for entity in existing_menu.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    if menu_state.is_open {
        if existing_menu.is_empty() {
            let menu_nodes = if menu_state.search_query.is_empty() {
                registry.get_menu_nodes_sorted()
            } else {
                registry.search(&menu_state.search_query)
            };

            let mut categories: std::collections::HashMap<String, Vec<(NodeId, String, Color)>> =
                std::collections::HashMap::new();

            for definition in menu_nodes {
                let category = definition.category().as_str().to_string();
                categories.entry(category).or_insert_with(Vec::new).push((
                    definition.id(),
                    definition.display_name().to_string(),
                    definition.color(),
                ));
            }

            let menu_width = 220.0;
            let menu_height = 600.0_f32.min(600.0);
            let target_camera = graph_camera.iter().next();

            let mut menu_commands = commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(menu_state.position.x.min(1280.0 - menu_width)),
                    top: Val::Px(menu_state.position.y.min(720.0 - menu_height)),
                    width: Val::Px(menu_width),
                    max_height: Val::Px(menu_height),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(5.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    overflow: Overflow::scroll_y(), // 🎯 Scrolling!
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 0.8)),
                // BorderWidth(Val::Px(1.0)),
                ZIndex(100),
                ContextMenuUI,
            ));

            if let Some(target_camera) = target_camera {
                menu_commands.insert(UiTargetCamera(target_camera));
            }

            menu_commands.with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(32.0),
                            padding: UiRect::horizontal(Val::Px(8.0)),
                            margin: UiRect::bottom(Val::Px(5.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.18, 0.22)),
                    ))
                    .with_children(|search_container| {
                        search_container.spawn((
                            Text::new("🔍 Search..."),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.5)),
                        ));
                    });

                let mut sorted_categories: Vec<_> = categories.iter().collect();
                sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

                for (category, nodes) in sorted_categories {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(25.0),
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(10.0)),
                                margin: UiRect::top(Val::Px(5.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.1, 0.1, 0.12)),
                        ))
                        .with_children(|header| {
                            header.spawn((
                                Text::new(category.clone()),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                            ));
                        });

                    for (node_id, title, color) in nodes {
                        parent
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(28.0),
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                    margin: UiRect::top(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(*color),
                                NodeTypeButton {
                                    definition_id: node_id.clone(),
                                },
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(title.clone()),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                }

                if categories.is_empty() {
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|empty| {
                            empty.spawn((
                                Text::new("No nodes found"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                            ));
                        });
                }
            });
        }
    }

    if !menu_state.is_open {
        for entity in existing_menu.iter() {
            commands.entity(entity).try_despawn();
        }
    }
}

#[derive(Component)]
pub struct NodeTypeButton {
    pub definition_id: NodeId,
}

pub fn interact_context_menu_system(
    activation: Option<Res<GraphEditingUiActivation>>,
    mut interaction_query: Query<
        (&Interaction, &NodeTypeButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut menu_state: ResMut<ContextMenuState>,
    mut overlay: ResMut<GraphOverlayState>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    mut command_writer: MessageWriter<GraphCommandRequest>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        return;
    }

    for (interaction, button_data) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let Ok(window) = windows.single() else { return };
            let Ok((camera, cam_transform)) = camera_query.single() else {
                return;
            };

            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
                    command_writer.write(GraphCommandRequest::SpawnNode {
                        definition_id: button_data.definition_id.clone(),
                        position: world_pos,
                    });
                    menu_state.is_open = false;
                    if overlay.active_surface == GraphOverlaySurface::ContextMenu {
                        overlay.active_surface = GraphOverlaySurface::None;
                    }
                }
            }
        }
    }
}

pub fn execute_spawn_node_commands_system(
    mut commands: Commands,
    activation: Option<Res<GraphEditingUiActivation>>,
    registry: Res<NodeRegistry>,
    mut command_requests: MessageReader<GraphCommandRequest>,
    mut mutations: ResMut<GraphMutationTracker>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        command_requests.clear();
        return;
    }

    for command in command_requests.read() {
        let GraphCommandRequest::SpawnNode {
            definition_id,
            position,
        } = command
        else {
            continue;
        };

        commands.spawn_node_from_definition(definition_id, *position, &registry);
        mutations.mark_changed();
    }
}

fn graph_editing_enabled(activation: Option<&GraphEditingUiActivation>) -> bool {
    activation
        .map(|activation| activation.enabled)
        .unwrap_or(true)
}
