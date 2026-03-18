//! Context-menu systems for graph actions and explicit node spawning.
use std::collections::HashMap;

use crate::interaction::{
    graph_editing_enabled, interaction_is_pointer_active, pointer_target_node,
};
use crate::node_spawn::node_icon_for_definition;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use univis_editor_commands::GraphClipboardState;
use univis_ui::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContextMenuMode {
    #[default]
    Actions,
    NodeBrowser,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ContextMenuState {
    pub is_open: bool,
    pub mode: ContextMenuMode,
    pub position: Vec2,
    pub world_position: Vec2,
    pub search_query: String,
}

#[derive(Component)]
pub struct ContextMenuUI;

#[derive(Component)]
pub struct NodeTypeButton {
    pub definition_id: NodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuAction {
    PasteClipboard,
    CopySelected,
    DuplicateSelected,
    DeleteSelected,
    FrameSelected,
    OpenNodeBrowser,
    BackToActions,
}

#[derive(Component)]
pub struct ContextMenuActionButton {
    pub action: ContextMenuAction,
    pub enabled: bool,
}

#[derive(Clone)]
struct MenuNodeEntry {
    definition_id: NodeId,
    title: String,
    description: String,
    color: Color,
    icon_text: String,
    icon_uses_icon_font: bool,
}

struct ActionMenuEntry {
    icon: &'static str,
    label: &'static str,
    description: &'static str,
    action: ContextMenuAction,
    enabled: bool,
}

/// Opens the context menu at the current cursor position. Right-clicking a node
/// focuses that node first so actions apply to the visible target, while
/// right-clicking an input port keeps the existing disconnect behavior.
pub fn open_context_menu_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    mut menu_state: ResMut<ContextMenuState>,
    mut overlay: ResMut<GraphOverlayState>,
    activation: Option<Res<GraphEditingUiActivation>>,
    nodes_interaction: Query<(Entity, &UInteraction), With<GraphNode>>,
    headers_interaction: Query<(Entity, &UInteraction), With<Header>>,
    ports_interaction: Query<(&UInteraction, &GraphPort)>,
    parents: Query<&ChildOf>,
    node_markers: Query<(), With<GraphNode>>,
    selected_nodes: Query<Entity, With<Selected>>,
    mut live_document: ResMut<LiveGraphDocumentState>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        close_context_menu(&mut menu_state, &mut overlay);
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let clicked_on_port = ports_interaction
        .iter()
        .any(|(interaction, _)| interaction_is_pointer_active(interaction));
    if clicked_on_port {
        close_context_menu(&mut menu_state, &mut overlay);
        return;
    }

    if let Some(target_entity) = pointer_target_node(
        &nodes_interaction,
        &headers_interaction,
        &ports_interaction,
        &parents,
        &node_markers,
    ) {
        if selected_nodes.get(target_entity).is_err() {
            for entity in selected_nodes.iter() {
                if entity != target_entity {
                    commands.entity(entity).try_remove::<Selected>();
                }
            }
            commands.entity(target_entity).try_insert(Selected);
            live_document.select_single_entity(target_entity);
        }
    }

    let Ok(window) = windows.single() else { return };
    let Some(position) = window.cursor_position() else {
        return;
    };

    let world_position = camera_query
        .single()
        .ok()
        .and_then(|(camera, transform)| camera.viewport_to_world_2d(transform, position).ok())
        .unwrap_or(menu_state.world_position);

    menu_state.is_open = true;
    menu_state.mode = ContextMenuMode::Actions;
    menu_state.position = position;
    menu_state.world_position = world_position;
    menu_state.search_query.clear();
    overlay.active_surface = GraphOverlaySurface::ContextMenu;
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

/// Builds the floating context menu for clipboard-style actions and the
/// explicit node browser entry point.
pub fn draw_context_menu_system(
    mut commands: Commands,
    menu_state: Res<ContextMenuState>,
    activation: Option<Res<GraphEditingUiActivation>>,
    live_document: Res<LiveGraphDocumentState>,
    clipboard: Option<Res<GraphClipboardState>>,
    existing_menu: Query<Entity, With<ContextMenuUI>>,
    registry: Res<NodeRegistry>,
    theme: Res<Theme>,
    graph_camera: Query<Entity, With<GraphCamera>>,
) {
    let should_rebuild = menu_state.is_changed()
        || live_document.is_changed()
        || clipboard
            .as_ref()
            .is_some_and(|clipboard| clipboard.is_changed());

    if !graph_editing_enabled(activation.as_deref()) {
        for entity in existing_menu.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    if !menu_state.is_open {
        for entity in existing_menu.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    if !should_rebuild && !existing_menu.is_empty() {
        return;
    }

    for entity in existing_menu.iter() {
        commands.entity(entity).try_despawn();
    }

    let (menu_width, menu_height) = match menu_state.mode {
        ContextMenuMode::Actions => (260.0, 296.0),
        ContextMenuMode::NodeBrowser => (280.0, 600.0),
    };
    let target_camera = graph_camera.iter().next();
    let icon_font = theme.icon.font.clone();

    let mut menu_commands = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(menu_state.position.x.min(1280.0 - menu_width)),
            top: Val::Px(menu_state.position.y.min(720.0 - menu_height)),
            width: Val::Px(menu_width),
            max_height: Val::Px(menu_height),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            row_gap: Val::Px(4.0),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.115, 0.118, 0.14)),
        BorderColor::all(Color::srgba(0.34, 0.36, 0.42, 0.8)),
        ZIndex(100),
        ContextMenuUI,
    ));

    if let Some(target_camera) = target_camera {
        menu_commands.insert(UiTargetCamera(target_camera));
    }

    menu_commands.with_children(|parent| match menu_state.mode {
        ContextMenuMode::Actions => {
            spawn_context_menu_title(parent, "Context", "Edit the current graph selection.");

            let selection_count = live_document.document.selected_node_ids().len();
            let has_selection = selection_count > 0;
            let can_paste = clipboard
                .as_ref()
                .is_some_and(|clipboard| clipboard.has_contents());

            let entries = [
                ActionMenuEntry {
                    icon: Icon::CLIPBOARD,
                    label: "Paste",
                    description: "Paste copied nodes at the cursor.",
                    action: ContextMenuAction::PasteClipboard,
                    enabled: can_paste,
                },
                ActionMenuEntry {
                    icon: Icon::COPY,
                    label: "Copy",
                    description: "Copy the current selection into the clipboard.",
                    action: ContextMenuAction::CopySelected,
                    enabled: has_selection,
                },
                ActionMenuEntry {
                    icon: Icon::COPYLEFT,
                    label: "Duplicate",
                    description: "Duplicate the current selection nearby.",
                    action: ContextMenuAction::DuplicateSelected,
                    enabled: has_selection,
                },
                ActionMenuEntry {
                    icon: Icon::TRASH,
                    label: "Delete",
                    description: "Delete the selected nodes and their wires.",
                    action: ContextMenuAction::DeleteSelected,
                    enabled: has_selection,
                },
                ActionMenuEntry {
                    icon: Icon::FRAMER,
                    label: "Frame Selected",
                    description: "Move the camera to the current selection.",
                    action: ContextMenuAction::FrameSelected,
                    enabled: has_selection,
                },
                ActionMenuEntry {
                    icon: Icon::LAYOUT_GRID,
                    label: "Add Node...",
                    description: "Open the node browser as a second step.",
                    action: ContextMenuAction::OpenNodeBrowser,
                    enabled: true,
                },
            ];

            for entry in entries {
                spawn_action_button(parent, &icon_font, entry);
            }

            if selection_count == 0 {
                spawn_context_menu_hint(
                    parent,
                    &icon_font,
                    Icon::BOX,
                    "Right-clicking a node now focuses it before opening this menu.",
                );
            }
        }
        ContextMenuMode::NodeBrowser => {
            spawn_context_menu_title(parent, "Add Node", "Choose a node type explicitly.");
            spawn_action_button(
                parent,
                &icon_font,
                ActionMenuEntry {
                    icon: Icon::ARROW_LEFT,
                    label: "Back",
                    description: "Return to clipboard and selection actions.",
                    action: ContextMenuAction::BackToActions,
                    enabled: true,
                },
            );

            let menu_nodes = if menu_state.search_query.is_empty() {
                registry.get_menu_nodes_sorted()
            } else {
                registry.search(&menu_state.search_query)
            };
            let mut categories: HashMap<String, Vec<MenuNodeEntry>> = HashMap::new();

            for definition in menu_nodes {
                let category = definition.category().as_str().to_string();
                let (icon_text, icon_uses_icon_font) = node_icon_for_definition(&definition);
                categories.entry(category).or_default().push(MenuNodeEntry {
                    definition_id: definition.id(),
                    title: definition.display_name().to_string(),
                    description: definition
                        .description()
                        .unwrap_or("No description yet.")
                        .to_string(),
                    color: definition.color(),
                    icon_text,
                    icon_uses_icon_font,
                });
            }

            let mut sorted_categories: Vec<_> = categories.iter().collect();
            sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

            for (category, nodes) in sorted_categories {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(10.0)),
                            margin: UiRect::top(Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.09, 0.095, 0.115)),
                    ))
                    .with_children(|header| {
                        header.spawn((
                            Text::new(category.clone()),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.58, 0.61, 0.67)),
                        ));
                    });

                for entry in nodes {
                    spawn_node_browser_button(parent, &icon_font, entry);
                }
            }

            if categories.is_empty() {
                spawn_context_menu_hint(
                    parent,
                    &icon_font,
                    Icon::BOX,
                    "No nodes are currently available in the registry.",
                );
            }
        }
    });
}

pub fn interact_context_menu_system(
    activation: Option<Res<GraphEditingUiActivation>>,
    mut action_query: Query<
        (&Interaction, &ContextMenuActionButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut node_query: Query<(&Interaction, &NodeTypeButton), (Changed<Interaction>, With<Button>)>,
    mut menu_state: ResMut<ContextMenuState>,
    mut overlay: ResMut<GraphOverlayState>,
    mut command_writer: MessageWriter<GraphCommandRequest>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        return;
    }

    for (interaction, button) in action_query.iter_mut() {
        if *interaction != Interaction::Pressed || !button.enabled {
            continue;
        }

        match button.action {
            ContextMenuAction::PasteClipboard => {
                command_writer.write(GraphCommandRequest::PasteNodes {
                    position: menu_state.world_position,
                });
                close_context_menu(&mut menu_state, &mut overlay);
            }
            ContextMenuAction::CopySelected => {
                command_writer.write(GraphCommandRequest::CopySelectedNodes);
                close_context_menu(&mut menu_state, &mut overlay);
            }
            ContextMenuAction::DuplicateSelected => {
                command_writer.write(GraphCommandRequest::DuplicateSelectedNodes);
                close_context_menu(&mut menu_state, &mut overlay);
            }
            ContextMenuAction::DeleteSelected => {
                command_writer.write(GraphCommandRequest::DeleteSelectedNodes);
                close_context_menu(&mut menu_state, &mut overlay);
            }
            ContextMenuAction::FrameSelected => {
                command_writer.write(GraphCommandRequest::FrameSelectedNodes);
                close_context_menu(&mut menu_state, &mut overlay);
            }
            ContextMenuAction::OpenNodeBrowser => {
                menu_state.mode = ContextMenuMode::NodeBrowser;
            }
            ContextMenuAction::BackToActions => {
                menu_state.mode = ContextMenuMode::Actions;
            }
        }
    }

    for (interaction, button_data) in node_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        command_writer.write(GraphCommandRequest::SpawnNode {
            definition_id: button_data.definition_id.clone(),
            position: menu_state.world_position,
        });
        close_context_menu(&mut menu_state, &mut overlay);
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

fn spawn_context_menu_title(parent: &mut ChildSpawnerCommands, title: &str, description: &str) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|title_row| {
            title_row.spawn((
                Text::new(title),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            title_row.spawn((
                Text::new(description),
                TextFont {
                    font_size: 10.5,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.54)),
            ));
        });
}

fn spawn_context_menu_hint(
    parent: &mut ChildSpawnerCommands,
    icon_font: &Handle<Font>,
    icon: &'static str,
    text: &str,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                margin: UiRect::top(Val::Px(4.0)),
                column_gap: Val::Px(8.0),
                align_items: AlignItems::FlexStart,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.09, 0.11, 0.6)),
        ))
        .with_children(|hint| {
            hint.spawn((
                Text::new(icon),
                TextFont {
                    font: icon_font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.67, 0.71, 0.78)),
            ));
            hint.spawn((
                Text::new(text),
                TextFont {
                    font_size: 10.5,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.56)),
            ));
        });
}

fn spawn_action_button(
    parent: &mut ChildSpawnerCommands,
    icon_font: &Handle<Font>,
    entry: ActionMenuEntry,
) {
    let background = if entry.enabled {
        Color::srgba(0.19, 0.21, 0.26, 0.88)
    } else {
        Color::srgba(0.13, 0.14, 0.17, 0.55)
    };
    let label_color = if entry.enabled {
        Color::WHITE
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.38)
    };
    let description_color = if entry.enabled {
        Color::srgba(1.0, 1.0, 1.0, 0.5)
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.28)
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(42.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(7.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(background),
            ContextMenuActionButton {
                action: entry.action,
                enabled: entry.enabled,
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(entry.icon),
                TextFont {
                    font: icon_font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(label_color),
            ));
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|text| {
                text.spawn((
                    Text::new(entry.label),
                    TextFont {
                        font_size: 12.5,
                        ..default()
                    },
                    TextColor(label_color),
                ));
                text.spawn((
                    Text::new(entry.description),
                    TextFont {
                        font_size: 10.5,
                        ..default()
                    },
                    TextColor(description_color),
                ));
            });
        });
}

fn spawn_node_browser_button(
    parent: &mut ChildSpawnerCommands,
    icon_font: &Handle<Font>,
    entry: &MenuNodeEntry,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(38.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                margin: UiRect::top(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(entry.color),
            NodeTypeButton {
                definition_id: entry.definition_id.clone(),
            },
        ))
        .with_children(|btn| {
            let mut icon_font_style = TextFont {
                font_size: 13.0,
                ..default()
            };
            if entry.icon_uses_icon_font {
                icon_font_style.font = icon_font.clone();
            }

            btn.spawn((
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|row| {
                row.spawn((
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|left| {
                    left.spawn((
                        Node {
                            width: Val::Px(22.0),
                            height: Val::Px(22.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(999.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.08, 0.09, 0.12, 0.22)),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(entry.icon_text.clone()),
                            icon_font_style.clone(),
                            TextColor(Color::WHITE),
                        ));
                    });

                    left.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|text| {
                        text.spawn((
                            Text::new(entry.title.clone()),
                            TextFont {
                                font_size: 12.5,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        text.spawn((
                            Text::new(entry.description.clone()),
                            TextFont {
                                font_size: 10.5,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.52)),
                        ));
                    });
                });

                row.spawn((
                    Text::new(Icon::CHEVRON_RIGHT),
                    TextFont {
                        font: icon_font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                ));
            });
        });
}

fn close_context_menu(menu_state: &mut ContextMenuState, overlay: &mut GraphOverlayState) {
    menu_state.is_open = false;
    menu_state.mode = ContextMenuMode::Actions;
    menu_state.search_query.clear();
    if overlay.active_surface == GraphOverlaySurface::ContextMenu {
        overlay.active_surface = GraphOverlaySurface::None;
    }
}
