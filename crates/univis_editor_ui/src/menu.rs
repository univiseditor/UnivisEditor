//! Context-menu systems for graph actions and explicit node spawning.
use std::collections::HashMap;

use crate::interaction::{
    graph_editing_enabled, interaction_is_pointer_active, pointer_target_node,
};
use crate::node_spawn::node_icon_for_definition;
use crate::prelude::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
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
    pub show_category_flyout: bool,
    pub active_category: Option<String>,
    pub search_query: String,
}

#[derive(Component)]
pub struct ContextMenuUI;

#[derive(Component)]
pub struct ContextMenuFlyoutUI;

#[derive(Component)]
pub struct ContextMenuScrollArea;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuScrollSurface {
    Main,
    CategoryFlyout,
    NodeFlyout,
}

#[derive(Component)]
pub struct ContextMenuScrollbarTrack;

#[derive(Component)]
pub struct ContextMenuScrollbarThumb;

#[derive(Component)]
pub struct NodeTypeButton {
    pub definition_id: NodeId,
}

#[derive(Component)]
pub struct ContextMenuCategoryButton {
    pub category: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuAction {
    PasteClipboard,
    CopySelected,
    DuplicateSelected,
    DeleteSelected,
    FrameSelected,
    OpenNodeBrowser,
    BackToCategories,
    BackToActions,
}

#[derive(Component)]
pub struct ContextMenuActionButton {
    pub action: ContextMenuAction,
    pub enabled: bool,
}

#[derive(Component, Clone, Copy)]
pub struct ContextMenuButtonVisualStyle {
    pub base_background: Color,
    pub hover_background: Color,
    pub pressed_background: Color,
    pub active_background: Color,
    pub disabled_background: Color,
    pub border_color: Color,
    pub active_border_color: Color,
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

#[derive(Debug, Clone, Copy)]
struct ContextMenuFrame {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawnContextMenuState {
    mode: ContextMenuMode,
    position: Vec2,
    active_category: Option<String>,
    search_query: String,
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
    menu_state.show_category_flyout = false;
    menu_state.active_category = None;
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
        menu_state.show_category_flyout = false;
        menu_state.active_category = None;
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
    windows: Query<&Window>,
    existing_menu: Query<Entity, (With<ContextMenuUI>, Without<ContextMenuFlyoutUI>)>,
    existing_flyout: Query<Entity, With<ContextMenuFlyoutUI>>,
    registry: Res<NodeRegistry>,
    theme: Res<Theme>,
    graph_camera: Query<Entity, With<GraphCamera>>,
    mut last_drawn_state: Local<Option<DrawnContextMenuState>>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        for entity in existing_menu.iter() {
            commands.entity(entity).try_despawn();
        }
        for entity in existing_flyout.iter() {
            commands.entity(entity).try_despawn();
        }
        *last_drawn_state = None;
        return;
    }

    if !menu_state.is_open {
        for entity in existing_menu.iter() {
            commands.entity(entity).try_despawn();
        }
        for entity in existing_flyout.iter() {
            commands.entity(entity).try_despawn();
        }
        *last_drawn_state = None;
        return;
    }

    let window_size = window_size(&windows).unwrap_or(Vec2::new(1280.0, 720.0));
    let menu_frame = context_menu_frame(menu_state.mode, menu_state.position, window_size);
    let target_camera = graph_camera.iter().next();
    let icon_font = theme.icon.font.clone();
    let main_state = DrawnContextMenuState {
        mode: menu_state.mode,
        position: menu_state.position,
        active_category: if menu_state.mode == ContextMenuMode::NodeBrowser {
            menu_state.active_category.clone()
        } else {
            None
        },
        search_query: if menu_state.mode == ContextMenuMode::NodeBrowser {
            menu_state.search_query.clone()
        } else {
            String::new()
        },
    };
    let should_rebuild_main =
        existing_menu.is_empty() || last_drawn_state.as_ref() != Some(&main_state);

    if should_rebuild_main {
        for entity in existing_menu.iter() {
            commands.entity(entity).try_despawn();
        }

        let mut menu_commands = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(menu_frame.left),
                top: Val::Px(menu_frame.top),
                width: Val::Px(menu_frame.width),
                height: Val::Px(menu_frame.height),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::Px(10.0)),
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

        menu_commands.with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollPosition::default(),
                    BackgroundColor(Color::NONE),
                    ContextMenuScrollArea,
                    ContextMenuScrollSurface::Main,
                ))
                .with_children(|parent| match menu_state.mode {
                    ContextMenuMode::Actions => {
                        spawn_context_menu_title(
                            parent,
                            "Context",
                            "Edit the current graph selection.",
                        );

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
                                description: "Hover to browse node categories first.",
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
                        let browser_title = menu_state
                            .active_category
                            .as_deref()
                            .map(|category| format!("Add Node • {category}"))
                            .unwrap_or_else(|| "Add Node".to_string());
                        let browser_description = menu_state
                            .active_category
                            .as_deref()
                            .map(|_| "Choose a node from the active category.")
                            .unwrap_or("Choose a node type explicitly.");
                        spawn_context_menu_title(parent, &browser_title, browser_description);
                        spawn_action_button(
                            parent,
                            &icon_font,
                            ActionMenuEntry {
                                icon: Icon::ARROW_LEFT,
                                label: if menu_state.active_category.is_some() {
                                    "Categories"
                                } else {
                                    "Back"
                                },
                                description: if menu_state.active_category.is_some() {
                                    "Return to the category flyout."
                                } else {
                                    "Return to clipboard and selection actions."
                                },
                                action: if menu_state.active_category.is_some() {
                                    ContextMenuAction::BackToCategories
                                } else {
                                    ContextMenuAction::BackToActions
                                },
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
                            let (icon_text, icon_uses_icon_font) =
                                node_icon_for_definition(&definition);
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

                        if let Some(active_category) = menu_state.active_category.as_deref() {
                            if let Some(nodes) = categories.get(active_category) {
                                for entry in nodes {
                                    spawn_node_browser_button(parent, &icon_font, entry);
                                }
                            }
                        } else {
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

            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(4.0),
                    top: Val::Px(8.0),
                    bottom: Val::Px(8.0),
                    width: Val::Px(4.0),
                    border_radius: BorderRadius::all(Val::Px(999.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.04)),
                ContextMenuScrollbarTrack,
                ContextMenuScrollSurface::Main,
            ));

            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(4.0),
                    top: Val::Px(8.0),
                    width: Val::Px(4.0),
                    height: Val::Px(40.0),
                    border_radius: BorderRadius::all(Val::Px(999.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.28)),
                ContextMenuScrollbarThumb,
                ContextMenuScrollSurface::Main,
            ));
        });

        *last_drawn_state = Some(main_state.clone());
    }

    let should_show_flyout =
        menu_state.mode == ContextMenuMode::Actions && menu_state.show_category_flyout;
    if !should_show_flyout {
        for entity in existing_flyout.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    if should_rebuild_main || existing_flyout.is_empty() || menu_state.is_changed() {
        for entity in existing_flyout.iter() {
            commands.entity(entity).try_despawn();
        }

        let category_frame = category_flyout_frame(menu_frame, window_size);
        let categories = collect_menu_categories(&registry);
        let nodes_by_category = collect_menu_nodes_by_category(&registry, &menu_state.search_query);

        let mut flyout_commands = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(category_frame.left),
                top: Val::Px(category_frame.top),
                width: Val::Px(category_frame.width),
                height: Val::Px(category_frame.height),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.105, 0.108, 0.13)),
            BorderColor::all(Color::srgba(0.34, 0.36, 0.42, 0.8)),
            ZIndex(101),
            ContextMenuUI,
            ContextMenuFlyoutUI,
        ));

        if let Some(target_camera) = target_camera {
            flyout_commands.insert(UiTargetCamera(target_camera));
        }

        flyout_commands.with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollPosition::default(),
                    BackgroundColor(Color::NONE),
                    ContextMenuScrollArea,
                    ContextMenuScrollSurface::CategoryFlyout,
                ))
                .with_children(|parent| {
                    spawn_context_menu_title(parent, "Categories", "Choose a node family first.");
                    for category in categories {
                        spawn_category_button(parent, &icon_font, &category);
                    }
                });

            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(4.0),
                    top: Val::Px(8.0),
                    bottom: Val::Px(8.0),
                    width: Val::Px(4.0),
                    border_radius: BorderRadius::all(Val::Px(999.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.04)),
                ContextMenuScrollbarTrack,
                ContextMenuScrollSurface::CategoryFlyout,
            ));

            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(4.0),
                    top: Val::Px(8.0),
                    width: Val::Px(4.0),
                    height: Val::Px(40.0),
                    border_radius: BorderRadius::all(Val::Px(999.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.28)),
                ContextMenuScrollbarThumb,
                ContextMenuScrollSurface::CategoryFlyout,
            ));
        });

        if let Some(active_category) = menu_state.active_category.as_deref() {
            let node_entries = nodes_by_category
                .get(active_category)
                .cloned()
                .unwrap_or_default();
            let node_frame = node_flyout_frame(category_frame, window_size);

            let mut node_flyout_commands = commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(node_frame.left),
                    top: Val::Px(node_frame.top),
                    width: Val::Px(node_frame.width),
                    height: Val::Px(node_frame.height),
                    flex_direction: FlexDirection::Column,
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.098, 0.102, 0.125)),
                BorderColor::all(Color::srgba(0.44, 0.68, 1.0, 0.24)),
                ZIndex(102),
                ContextMenuUI,
                ContextMenuFlyoutUI,
            ));

            if let Some(target_camera) = target_camera {
                node_flyout_commands.insert(UiTargetCamera(target_camera));
            }

            node_flyout_commands.with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(8.0)),
                            row_gap: Val::Px(4.0),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollPosition::default(),
                        BackgroundColor(Color::NONE),
                        ContextMenuScrollArea,
                        ContextMenuScrollSurface::NodeFlyout,
                    ))
                    .with_children(|parent| {
                        let description = if node_entries.is_empty() {
                            "No nodes are available in this category yet."
                        } else {
                            "Choose a node to spawn directly."
                        };
                        spawn_context_menu_title(parent, active_category, description);

                        for entry in &node_entries {
                            spawn_node_browser_button(parent, &icon_font, entry);
                        }

                        if node_entries.is_empty() {
                            spawn_context_menu_hint(
                                parent,
                                &icon_font,
                                Icon::BOX,
                                "This category is currently empty.",
                            );
                        }
                    });

                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(4.0),
                        top: Val::Px(8.0),
                        bottom: Val::Px(8.0),
                        width: Val::Px(4.0),
                        border_radius: BorderRadius::all(Val::Px(999.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.04)),
                    ContextMenuScrollbarTrack,
                    ContextMenuScrollSurface::NodeFlyout,
                ));

                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(4.0),
                        top: Val::Px(8.0),
                        width: Val::Px(4.0),
                        height: Val::Px(40.0),
                        border_radius: BorderRadius::all(Val::Px(999.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.28)),
                    ContextMenuScrollbarThumb,
                    ContextMenuScrollSurface::NodeFlyout,
                ));
            });
        }
    }
}

pub fn interact_context_menu_system(
    activation: Option<Res<GraphEditingUiActivation>>,
    mut action_query: Query<
        (&Interaction, &ContextMenuActionButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut category_query: Query<
        (&Interaction, &ContextMenuCategoryButton),
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
                menu_state.mode = ContextMenuMode::Actions;
                menu_state.active_category = None;
                menu_state.show_category_flyout = true;
            }
            ContextMenuAction::BackToCategories => {
                menu_state.mode = ContextMenuMode::Actions;
                menu_state.active_category = None;
                menu_state.show_category_flyout = true;
            }
            ContextMenuAction::BackToActions => {
                menu_state.mode = ContextMenuMode::Actions;
                menu_state.active_category = None;
                menu_state.show_category_flyout = false;
            }
        }
    }

    for (interaction, category_button) in category_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        menu_state.active_category = Some(category_button.category.clone());
        menu_state.show_category_flyout = true;
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

pub fn close_context_menu_on_outside_click_system(
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<ContextMenuState>,
    mut overlay: ResMut<GraphOverlayState>,
) {
    if !menu_state.is_open {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        close_context_menu(&mut menu_state, &mut overlay);
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let cursor_position = windows
        .single()
        .ok()
        .and_then(|window| window.cursor_position());
    let Some(cursor_position) = cursor_position else {
        close_context_menu(&mut menu_state, &mut overlay);
        return;
    };

    let menu_frame = context_menu_frame(
        menu_state.mode,
        menu_state.position,
        window_size(&windows).unwrap_or(Vec2::new(1280.0, 720.0)),
    );
    let inside_menu = cursor_position.x >= menu_frame.left
        && cursor_position.x <= menu_frame.left + menu_frame.width
        && cursor_position.y >= menu_frame.top
        && cursor_position.y <= menu_frame.top + menu_frame.height;
    let inside_flyout =
        if menu_state.mode == ContextMenuMode::Actions && menu_state.show_category_flyout {
            let flyout_frame = category_flyout_frame(
                menu_frame,
                window_size(&windows).unwrap_or(Vec2::new(1280.0, 720.0)),
            );
            let inside_category_flyout = cursor_position.x >= flyout_frame.left
                && cursor_position.x <= flyout_frame.left + flyout_frame.width
                && cursor_position.y >= flyout_frame.top
                && cursor_position.y <= flyout_frame.top + flyout_frame.height;
            let inside_node_flyout = menu_state.active_category.as_ref().is_some_and(|_| {
                let node_frame = node_flyout_frame(
                    flyout_frame,
                    window_size(&windows).unwrap_or(Vec2::new(1280.0, 720.0)),
                );
                cursor_position.x >= node_frame.left
                    && cursor_position.x <= node_frame.left + node_frame.width
                    && cursor_position.y >= node_frame.top
                    && cursor_position.y <= node_frame.top + node_frame.height
            });
            inside_category_flyout || inside_node_flyout
        } else {
            false
        };

    if !inside_menu && !inside_flyout {
        close_context_menu(&mut menu_state, &mut overlay);
    }
}

pub fn sync_context_menu_hover_state_system(
    menu_state: ResMut<ContextMenuState>,
    windows: Query<&Window>,
    action_buttons: Query<(&Interaction, &ContextMenuActionButton), With<Button>>,
    category_buttons: Query<(&Interaction, &ContextMenuCategoryButton), With<Button>>,
    node_buttons: Query<&Interaction, (With<NodeTypeButton>, With<Button>)>,
) {
    let mut menu_state = menu_state;
    if !menu_state.is_open || menu_state.mode != ContextMenuMode::Actions {
        if menu_state.show_category_flyout {
            menu_state.show_category_flyout = false;
        }
        if menu_state.active_category.is_some() {
            menu_state.active_category = None;
        }
        return;
    }

    let window_size = window_size(&windows).unwrap_or(Vec2::new(1280.0, 720.0));
    let main_frame = context_menu_frame(menu_state.mode, menu_state.position, window_size);
    let category_frame = category_flyout_frame(main_frame, window_size);
    let add_node_hovered = action_buttons.iter().any(|(interaction, button)| {
        button.action == ContextMenuAction::OpenNodeBrowser
            && matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
    });
    let hovered_category = category_buttons.iter().find_map(|(interaction, button)| {
        matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
            .then(|| button.category.clone())
    });
    let category_hovered = hovered_category.is_some();
    let node_hovered = node_buttons
        .iter()
        .any(|interaction| matches!(*interaction, Interaction::Hovered | Interaction::Pressed));
    let cursor_position = windows
        .single()
        .ok()
        .and_then(|window| window.cursor_position());
    let cursor_inside_category_flyout = cursor_position.is_some_and(|cursor| {
        cursor_inside_frame(cursor, category_frame)
            || cursor_inside_flyout_bridge(cursor, category_frame, main_frame)
    });
    let cursor_inside_node_flyout = menu_state.active_category.as_ref().is_some_and(|_| {
        let node_frame = node_flyout_frame(category_frame, window_size);
        cursor_position.is_some_and(|cursor| {
            cursor_inside_frame(cursor, node_frame)
                || cursor_inside_flyout_bridge(cursor, node_frame, category_frame)
        })
    });

    let next_show = add_node_hovered
        || category_hovered
        || node_hovered
        || cursor_inside_category_flyout
        || cursor_inside_node_flyout;
    let next_category = if let Some(category) = hovered_category {
        Some(category)
    } else if cursor_inside_category_flyout || cursor_inside_node_flyout || node_hovered {
        menu_state.active_category.clone()
    } else {
        None
    };

    if menu_state.active_category != next_category {
        menu_state.active_category = next_category;
    }
    if menu_state.show_category_flyout != next_show {
        menu_state.show_category_flyout = next_show;
    }
}

pub fn sync_context_menu_button_visuals_system(
    menu_state: Res<ContextMenuState>,
    mut button_sets: ParamSet<(
        Query<
            (
                &Interaction,
                &ContextMenuActionButton,
                &ContextMenuButtonVisualStyle,
                &mut BackgroundColor,
                &mut BorderColor,
            ),
            With<Button>,
        >,
        Query<
            (
                &Interaction,
                &ContextMenuCategoryButton,
                &ContextMenuButtonVisualStyle,
                &mut BackgroundColor,
                &mut BorderColor,
            ),
            With<Button>,
        >,
        Query<
            (
                &Interaction,
                &ContextMenuButtonVisualStyle,
                &mut BackgroundColor,
                &mut BorderColor,
            ),
            (With<NodeTypeButton>, With<Button>),
        >,
    )>,
) {
    for (interaction, button, style, mut background, mut border) in button_sets.p0().iter_mut() {
        let active =
            button.action == ContextMenuAction::OpenNodeBrowser && menu_state.show_category_flyout;
        apply_context_menu_button_visuals(
            *interaction,
            button.enabled,
            active,
            style,
            &mut background,
            &mut border,
        );
    }

    for (interaction, button, style, mut background, mut border) in button_sets.p1().iter_mut() {
        let active = menu_state.active_category.as_deref() == Some(button.category.as_str());
        apply_context_menu_button_visuals(
            *interaction,
            true,
            active,
            style,
            &mut background,
            &mut border,
        );
    }

    for (interaction, style, mut background, mut border) in button_sets.p2().iter_mut() {
        apply_context_menu_button_visuals(
            *interaction,
            true,
            false,
            style,
            &mut background,
            &mut border,
        );
    }
}

pub fn handle_context_menu_scroll_system(
    activation: Option<Res<GraphEditingUiActivation>>,
    menu_state: Res<ContextMenuState>,
    windows: Query<&Window>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut scroll_areas: Query<
        (
            &ContextMenuScrollSurface,
            &mut ScrollPosition,
            &Node,
            &ComputedNode,
        ),
        With<ContextMenuScrollArea>,
    >,
) {
    if !graph_editing_enabled(activation.as_deref()) || !menu_state.is_open {
        mouse_wheel.clear();
        return;
    }

    let cursor_position = windows
        .single()
        .ok()
        .and_then(|window| window.cursor_position());
    let Some(cursor_position) = cursor_position else {
        mouse_wheel.clear();
        return;
    };

    let window_size = window_size(&windows).unwrap_or(Vec2::new(1280.0, 720.0));
    let Some(target_surface) =
        context_menu_scroll_target_surface(&menu_state, cursor_position, window_size)
    else {
        mouse_wheel.clear();
        return;
    };

    let Some((_, mut scroll_position, node, computed)) = scroll_areas
        .iter_mut()
        .find(|(surface, ..)| **surface == target_surface)
    else {
        mouse_wheel.clear();
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    for event in mouse_wheel.read() {
        let mut delta = -Vec2::new(event.x, event.y);
        if event.unit == MouseScrollUnit::Line {
            delta *= 24.0;
        }

        if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
            scroll_position.y = (scroll_position.y + delta.y).clamp(0.0, max_offset.y.max(0.0));
        }
    }
}

pub fn sync_context_menu_scrollbar_system(
    menu_state: Res<ContextMenuState>,
    scroll_areas: Query<
        (&ContextMenuScrollSurface, &ScrollPosition, &ComputedNode),
        With<ContextMenuScrollArea>,
    >,
    mut track_query: Query<
        (&ContextMenuScrollSurface, &mut Node, &mut BackgroundColor),
        (
            With<ContextMenuScrollbarTrack>,
            Without<ContextMenuScrollbarThumb>,
        ),
    >,
    mut thumb_query: Query<
        (&ContextMenuScrollSurface, &mut Node, &mut BackgroundColor),
        (
            With<ContextMenuScrollbarThumb>,
            Without<ContextMenuScrollbarTrack>,
        ),
    >,
) {
    for surface in [
        ContextMenuScrollSurface::Main,
        ContextMenuScrollSurface::CategoryFlyout,
        ContextMenuScrollSurface::NodeFlyout,
    ] {
        let area = scroll_areas
            .iter()
            .find(|(scroll_surface, ..)| **scroll_surface == surface);
        let track = track_query
            .iter_mut()
            .find(|(scroll_surface, ..)| **scroll_surface == surface);
        let thumb = thumb_query
            .iter_mut()
            .find(|(scroll_surface, ..)| **scroll_surface == surface);

        let (
            Some((_, scroll_position, computed)),
            Some((_, mut track_node, mut track_color)),
            Some((_, mut thumb_node, mut thumb_color)),
        ) = (area, track, thumb)
        else {
            continue;
        };

        let viewport_height = computed.size().y * computed.inverse_scale_factor();
        let content_height = computed.content_size().y * computed.inverse_scale_factor();
        let max_offset = (content_height - viewport_height).max(0.0);
        let has_scroll = menu_state.is_open && content_height > viewport_height + 1.0;

        track_node.display = if has_scroll {
            Display::Flex
        } else {
            Display::None
        };
        thumb_node.display = if has_scroll {
            Display::Flex
        } else {
            Display::None
        };

        if !has_scroll {
            continue;
        }

        let track_top = 8.0;
        let track_bottom = 8.0;
        let track_height = (viewport_height - track_top - track_bottom).max(0.0);
        let visible_ratio = (viewport_height / content_height).clamp(0.0, 1.0);
        let thumb_height = (track_height * visible_ratio).max(32.0).min(track_height);
        let thumb_range = (track_height - thumb_height).max(0.0);
        let thumb_offset = if max_offset > 0.0 {
            thumb_range * (scroll_position.y / max_offset).clamp(0.0, 1.0)
        } else {
            0.0
        };

        thumb_node.top = Val::Px(track_top + thumb_offset);
        thumb_node.height = Val::Px(thumb_height);
        *track_color = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05));
        *thumb_color = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.3));
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
        Color::srgba(0.14, 0.16, 0.2, 0.96)
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
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
            ContextMenuButtonVisualStyle {
                base_background: Color::srgba(0.14, 0.16, 0.2, 0.96),
                hover_background: Color::srgba(0.19, 0.24, 0.31, 0.98),
                pressed_background: Color::srgba(0.24, 0.31, 0.42, 1.0),
                active_background: Color::srgba(0.2, 0.28, 0.4, 0.98),
                disabled_background: Color::srgba(0.13, 0.14, 0.17, 0.55),
                border_color: Color::srgba(1.0, 1.0, 1.0, 0.08),
                active_border_color: Color::srgba(0.52, 0.75, 1.0, 0.72),
            },
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

fn spawn_category_button(
    parent: &mut ChildSpawnerCommands,
    icon_font: &Handle<Font>,
    category: &str,
) {
    let accent = category_color(category);

    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(38.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.13, 0.15, 0.19, 0.96)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
            ContextMenuButtonVisualStyle {
                base_background: Color::srgba(0.13, 0.15, 0.19, 0.96),
                hover_background: Color::srgba(0.18, 0.22, 0.29, 0.98),
                pressed_background: Color::srgba(0.23, 0.29, 0.39, 1.0),
                active_background: Color::srgba(0.21, 0.28, 0.4, 0.98),
                disabled_background: Color::srgba(0.13, 0.15, 0.19, 0.96),
                border_color: Color::srgba(1.0, 1.0, 1.0, 0.08),
                active_border_color: accent,
            },
            ContextMenuCategoryButton {
                category: category.to_string(),
            },
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
                        width: Val::Px(4.0),
                        height: Val::Px(18.0),
                        border_radius: BorderRadius::all(Val::Px(999.0)),
                        ..default()
                    },
                    BackgroundColor(accent),
                ));
                left.spawn((
                    Text::new(category_icon(category)),
                    TextFont {
                        font: icon_font.clone(),
                        font_size: 12.5,
                        ..default()
                    },
                    TextColor(accent),
                ));
                left.spawn((
                    Text::new(category.to_string()),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            row.spawn((
                Text::new(Icon::CHEVRON_RIGHT),
                TextFont {
                    font: icon_font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.48)),
            ));
        });
}

fn spawn_node_browser_button(
    parent: &mut ChildSpawnerCommands,
    icon_font: &Handle<Font>,
    entry: &MenuNodeEntry,
) {
    let accent_background = color_with_alpha(entry.color, 0.22);

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
            BackgroundColor(Color::srgba(0.13, 0.16, 0.2, 0.98)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
            ContextMenuButtonVisualStyle {
                base_background: Color::srgba(0.13, 0.16, 0.2, 0.98),
                hover_background: Color::srgba(0.18, 0.22, 0.29, 1.0),
                pressed_background: Color::srgba(0.22, 0.28, 0.38, 1.0),
                active_background: Color::srgba(0.18, 0.22, 0.29, 1.0),
                disabled_background: Color::srgba(0.13, 0.16, 0.2, 0.98),
                border_color: Color::srgba(1.0, 1.0, 1.0, 0.08),
                active_border_color: entry.color,
            },
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
                            width: Val::Px(4.0),
                            height: Val::Px(20.0),
                            border_radius: BorderRadius::all(Val::Px(999.0)),
                            ..default()
                        },
                        BackgroundColor(entry.color),
                    ));
                    left.spawn((
                        Node {
                            width: Val::Px(22.0),
                            height: Val::Px(22.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(999.0)),
                            ..default()
                        },
                        BackgroundColor(accent_background),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(entry.icon_text.clone()),
                            icon_font_style.clone(),
                            TextColor(entry.color),
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
    menu_state.show_category_flyout = false;
    menu_state.active_category = None;
    menu_state.search_query.clear();
    if overlay.active_surface == GraphOverlaySurface::ContextMenu {
        overlay.active_surface = GraphOverlaySurface::None;
    }
}

fn context_menu_frame(mode: ContextMenuMode, anchor: Vec2, window_size: Vec2) -> ContextMenuFrame {
    let margin = 12.0;
    let preferred = context_menu_preferred_size(mode);
    let max_width = (window_size.x - margin * 2.0).max(180.0);
    let max_height = (window_size.y - margin * 2.0).max(180.0);
    let width = preferred.x.min(max_width);
    let height = preferred.y.min(max_height);

    let available_below = (window_size.y - margin - anchor.y).max(0.0);
    let available_above = (anchor.y - margin).max(0.0);
    let top = if available_below >= height || available_below >= available_above {
        anchor.y.min(window_size.y - margin - height).max(margin)
    } else {
        (anchor.y - height).max(margin)
    };
    let left = anchor.x.min(window_size.x - margin - width).max(margin);

    ContextMenuFrame {
        left,
        top,
        width,
        height,
    }
}

fn category_flyout_frame(main_frame: ContextMenuFrame, window_size: Vec2) -> ContextMenuFrame {
    let margin = 12.0;
    let width = 220.0_f32.min((window_size.x - margin * 2.0).max(180.0));
    let height = 280.0_f32.min((window_size.y - margin * 2.0).max(180.0));
    let right_candidate = main_frame.left + main_frame.width + 8.0;
    let has_room_on_right = right_candidate + width <= window_size.x - margin;
    let left = if has_room_on_right {
        right_candidate
    } else {
        (main_frame.left - 8.0 - width).max(margin)
    };
    let preferred_top = main_frame.top + main_frame.height - height - 32.0;
    let top = preferred_top
        .min(window_size.y - margin - height)
        .max(margin);

    ContextMenuFrame {
        left,
        top,
        width,
        height,
    }
}

fn node_flyout_frame(category_frame: ContextMenuFrame, window_size: Vec2) -> ContextMenuFrame {
    let margin = 12.0;
    let width = 300.0_f32.min((window_size.x - margin * 2.0).max(220.0));
    let height = 360.0_f32.min((window_size.y - margin * 2.0).max(220.0));
    let right_candidate = category_frame.left + category_frame.width + 6.0;
    let has_room_on_right = right_candidate + width <= window_size.x - margin;
    let left = if has_room_on_right {
        right_candidate
    } else {
        (category_frame.left - 6.0 - width).max(margin)
    };
    let top = category_frame
        .top
        .min(window_size.y - margin - height)
        .max(margin);

    ContextMenuFrame {
        left,
        top,
        width,
        height,
    }
}

fn context_menu_preferred_size(mode: ContextMenuMode) -> Vec2 {
    match mode {
        ContextMenuMode::Actions => Vec2::new(280.0, 360.0),
        ContextMenuMode::NodeBrowser => Vec2::new(320.0, 640.0),
    }
}

fn window_size(windows: &Query<&Window>) -> Option<Vec2> {
    windows
        .single()
        .ok()
        .map(|window| Vec2::new(window.width(), window.height()))
}

fn collect_menu_categories(registry: &NodeRegistry) -> Vec<String> {
    let mut categories = registry
        .get_menu_nodes_sorted()
        .into_iter()
        .map(|definition| definition.category().as_str().to_string())
        .collect::<Vec<_>>();
    categories.sort();
    categories.dedup();
    categories
}

fn collect_menu_nodes_by_category(
    registry: &NodeRegistry,
    search_query: &str,
) -> HashMap<String, Vec<MenuNodeEntry>> {
    let menu_nodes = if search_query.is_empty() {
        registry.get_menu_nodes_sorted()
    } else {
        registry.search(search_query)
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

    categories
}

fn cursor_inside_frame(cursor: Vec2, frame: ContextMenuFrame) -> bool {
    cursor.x >= frame.left
        && cursor.x <= frame.left + frame.width
        && cursor.y >= frame.top
        && cursor.y <= frame.top + frame.height
}

fn cursor_inside_flyout_bridge(
    cursor: Vec2,
    frame: ContextMenuFrame,
    origin_frame: ContextMenuFrame,
) -> bool {
    let bridge = 12.0;
    let (left, right) = if frame.left >= origin_frame.left + origin_frame.width {
        (frame.left - bridge, frame.left + frame.width)
    } else {
        (frame.left, frame.left + frame.width + bridge)
    };

    cursor.x >= left
        && cursor.x <= right
        && cursor.y >= frame.top
        && cursor.y <= frame.top + frame.height
}

fn context_menu_scroll_target_surface(
    menu_state: &ContextMenuState,
    cursor: Vec2,
    window_size: Vec2,
) -> Option<ContextMenuScrollSurface> {
    let main_frame = context_menu_frame(menu_state.mode, menu_state.position, window_size);

    if menu_state.mode == ContextMenuMode::Actions && menu_state.show_category_flyout {
        let category_frame = category_flyout_frame(main_frame, window_size);
        if menu_state.active_category.is_some() {
            let node_frame = node_flyout_frame(category_frame, window_size);
            if cursor_inside_frame(cursor, node_frame) {
                return Some(ContextMenuScrollSurface::NodeFlyout);
            }
        }
        if cursor_inside_frame(cursor, category_frame) {
            return Some(ContextMenuScrollSurface::CategoryFlyout);
        }
    }

    if cursor_inside_frame(cursor, main_frame) {
        Some(ContextMenuScrollSurface::Main)
    } else {
        None
    }
}

fn color_with_alpha(color: Color, alpha: f32) -> Color {
    let color = color.to_srgba();
    Color::srgba(color.red, color.green, color.blue, alpha)
}

fn apply_context_menu_button_visuals(
    interaction: Interaction,
    enabled: bool,
    active: bool,
    style: &ContextMenuButtonVisualStyle,
    background: &mut BackgroundColor,
    border: &mut BorderColor,
) {
    let background_color = if !enabled {
        style.disabled_background
    } else if interaction == Interaction::Pressed {
        style.pressed_background
    } else if interaction == Interaction::Hovered {
        style.hover_background
    } else if active {
        style.active_background
    } else {
        style.base_background
    };
    let border_color = if enabled && (active || interaction != Interaction::None) {
        style.active_border_color
    } else {
        style.border_color
    };

    background.0 = background_color;
    *border = BorderColor::all(border_color);
}

fn category_icon(category: &str) -> &'static str {
    match category {
        "Input" => Icon::TYPE,
        "Math" => Icon::CIRCLE_PLUS,
        "Logic" => Icon::GIT_BRANCH,
        "Scene" => Icon::BOX,
        "Advanced" => Icon::CPU,
        _ => Icon::CODESANDBOX,
    }
}

fn category_color(category: &str) -> Color {
    match category {
        "Input" => Color::srgb(0.48, 0.76, 1.0),
        "Math" => Color::srgb(1.0, 0.76, 0.34),
        "Logic" => Color::srgb(0.54, 0.9, 0.58),
        "Scene" => Color::srgb(0.98, 0.54, 0.4),
        "Advanced" => Color::srgb(0.78, 0.62, 1.0),
        _ => Color::srgb(0.72, 0.78, 0.9),
    }
}
