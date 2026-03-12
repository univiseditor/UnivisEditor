//! Node settings popup rendered over the graph canvas.
use crate::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::picking::prelude::Pickable;
use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use std::fmt::Write as _;
use univis_ui::prelude::UInteraction;

#[derive(Resource, Debug, Clone, Default)]
pub struct NodePopupState {
    pub open_for: Option<Entity>,
    pub anchor_world: Vec2,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct NodeSettingsButton {
    pub node_entity: Entity,
}

#[derive(Component)]
struct NodePopupRoot;

#[derive(Component)]
struct NodePopupPanel;

#[derive(Component)]
struct NodePopupTitleText;

#[derive(Component)]
struct NodePopupContent;

#[derive(Component)]
struct NodePopupCloseButton;

#[derive(Debug, Clone, Copy)]
enum ColorChannel {
    R,
    G,
    B,
    A,
}

#[derive(Component, Debug, Clone, Copy)]
struct NodePopupAdjustButton {
    input_index: usize,
    delta: f64,
    channel: Option<ColorChannel>,
}

#[derive(Component, Debug, Clone, Copy)]
struct NodePopupBoolToggleButton {
    input_index: usize,
}

#[derive(Component, Debug, Clone, Copy)]
struct NodePopupTextFieldInput {
    input_index: usize,
}

#[derive(Component, Debug, Clone)]
struct NodePopupTextFieldState {
    text: String,
    placeholder: String,
    focused: bool,
}

#[derive(Component, Debug, Clone, Copy)]
struct NodePopupTextFieldDisplay;

pub struct NodePopupPlugin;

impl Plugin for NodePopupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NodePopupState>()
            .add_systems(Startup, setup_node_popup_ui)
            .add_systems(
                Update,
                (
                    sync_popup_ui_target,
                    close_popup_when_graph_editing_disabled,
                    sync_popup_overlay,
                    handle_node_settings_button_clicks,
                    handle_popup_close_button,
                    handle_popup_adjust_buttons,
                    handle_popup_bool_toggle_buttons,
                    handle_popup_text_field_focus,
                    clear_popup_text_field_focus_on_other_clicks,
                    handle_popup_text_field_keyboard_input,
                    sync_popup_text_field_visuals,
                    sync_popup_anchor_world,
                    update_popup_panel_position,
                    rebuild_popup_content,
                )
                    .chain(),
            );
    }
}

fn interaction_is_pointer_active(interaction: &UInteraction) -> bool {
    matches!(
        *interaction,
        UInteraction::Pressed | UInteraction::Hovered | UInteraction::Clicked
    )
}

fn setup_node_popup_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Pickable::IGNORE,
            ZIndex(2200),
            NodePopupRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(340.0),
                    max_height: Val::Px(620.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(6.0),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.09, 0.09, 0.12, 0.96)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                NodePopupPanel,
            ))
            .with_children(|panel| {
                panel
                    .spawn((Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },))
                    .with_children(|header| {
                        header.spawn((
                            Text::new("Node Settings"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            NodePopupTitleText,
                        ));

                        header
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.4, 0.2, 0.2)),
                                NodePopupCloseButton,
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("X"),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    });

                panel.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    NodePopupContent,
                ));
            });
        });
}

fn sync_popup_ui_target(
    mut commands: Commands,
    q_graph_camera: Query<Entity, With<GraphCamera>>,
    q_roots: Query<(Entity, Option<&UiTargetCamera>), With<NodePopupRoot>>,
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

fn close_popup_when_graph_editing_disabled(
    activation: Option<Res<GraphEditingUiActivation>>,
    mut popup: ResMut<NodePopupState>,
    mut overlay: ResMut<GraphOverlayState>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        popup.open_for = None;
        if overlay.active_surface == GraphOverlaySurface::NodePopup {
            overlay.active_surface = GraphOverlaySurface::None;
        }
    }
}

fn sync_popup_overlay(mut popup: ResMut<NodePopupState>, overlay: Res<GraphOverlayState>) {
    if popup.open_for.is_some() && overlay.active_surface != GraphOverlaySurface::NodePopup {
        popup.open_for = None;
    }
}

fn handle_node_settings_button_clicks(
    mouse: Res<ButtonInput<MouseButton>>,
    activation: Option<Res<GraphEditingUiActivation>>,
    mut popup: ResMut<NodePopupState>,
    mut overlay: ResMut<GraphOverlayState>,
    buttons: Query<(&UInteraction, &NodeSettingsButton)>,
    node_transforms: Query<&Transform, With<GraphNode>>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        popup.open_for = None;
        if overlay.active_surface == GraphOverlaySurface::NodePopup {
            overlay.active_surface = GraphOverlaySurface::None;
        }
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    for (interaction, gear_button) in buttons.iter() {
        if !interaction_is_pointer_active(interaction) {
            continue;
        }

        if popup.open_for == Some(gear_button.node_entity) {
            popup.open_for = None;
            if overlay.active_surface == GraphOverlaySurface::NodePopup {
                overlay.active_surface = GraphOverlaySurface::None;
            }
            return;
        }

        popup.open_for = Some(gear_button.node_entity);
        overlay.active_surface = GraphOverlaySurface::NodePopup;
        if let Ok(transform) = node_transforms.get(gear_button.node_entity) {
            popup.anchor_world = transform.translation.truncate();
        }
        return;
    }
}

fn graph_editing_enabled(activation: Option<&GraphEditingUiActivation>) -> bool {
    activation
        .map(|activation| activation.enabled)
        .unwrap_or(true)
}

fn handle_popup_close_button(
    buttons: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<NodePopupCloseButton>,
        ),
    >,
    mut popup: ResMut<NodePopupState>,
    mut overlay: ResMut<GraphOverlayState>,
) {
    for interaction in buttons.iter() {
        if *interaction == Interaction::Pressed {
            popup.open_for = None;
            if overlay.active_surface == GraphOverlaySurface::NodePopup {
                overlay.active_surface = GraphOverlaySurface::None;
            }
        }
    }
}

fn handle_popup_adjust_buttons(
    buttons: Query<(&Interaction, &NodePopupAdjustButton), (Changed<Interaction>, With<Button>)>,
    popup: Res<NodePopupState>,
    registry: Res<NodeRegistry>,
    mut q_nodes: Query<&mut GraphNode>,
) {
    let Some(node_entity) = popup.open_for else {
        return;
    };

    for (interaction, button) in buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok(mut graph_node) = q_nodes.get_mut(node_entity) else {
            continue;
        };
        let Some(definition) = registry.get(&graph_node.definition_id) else {
            continue;
        };
        let inputs = definition.inputs();
        let Some(port_def) = inputs.get(button.input_index) else {
            continue;
        };

        if button.channel.is_some() {
            let current = graph_node
                .values
                .inputs
                .get(button.input_index)
                .and_then(NodeValue::as_color)
                .or_else(|| {
                    port_def
                        .default_value
                        .as_ref()
                        .and_then(NodeValue::as_color)
                })
                .unwrap_or_else(|| Color::srgba(1.0, 1.0, 1.0, 1.0));

            let mut srgba = current.to_srgba();
            let step = port_def.ui_step.unwrap_or(0.05) as f32;
            let min = port_def.ui_min.unwrap_or(0.0) as f32;
            let max = port_def.ui_max.unwrap_or(1.0) as f32;

            match button.channel {
                Some(ColorChannel::R) => {
                    srgba.red = (srgba.red + (button.delta as f32 * step)).clamp(min, max)
                }
                Some(ColorChannel::G) => {
                    srgba.green = (srgba.green + (button.delta as f32 * step)).clamp(min, max)
                }
                Some(ColorChannel::B) => {
                    srgba.blue = (srgba.blue + (button.delta as f32 * step)).clamp(min, max)
                }
                Some(ColorChannel::A) => {
                    srgba.alpha = (srgba.alpha + (button.delta as f32 * step)).clamp(min, max)
                }
                None => {}
            }

            if button.input_index < graph_node.values.inputs.len() {
                graph_node.values.inputs[button.input_index] = NodeValue::Color(Color::srgba(
                    srgba.red,
                    srgba.green,
                    srgba.blue,
                    srgba.alpha,
                ));
            }
            continue;
        }

        match port_def.value_type {
            ValueType::Float => {
                let current = graph_node
                    .values
                    .inputs
                    .get(button.input_index)
                    .and_then(NodeValue::as_float)
                    .or_else(|| {
                        port_def
                            .default_value
                            .as_ref()
                            .and_then(NodeValue::as_float)
                    })
                    .unwrap_or(0.0);

                let step = port_def.ui_step.unwrap_or(0.1);
                let mut next = current + button.delta * step;
                if let Some(min) = port_def.ui_min {
                    next = next.max(min);
                }
                if let Some(max) = port_def.ui_max {
                    next = next.min(max);
                }

                if button.input_index < graph_node.values.inputs.len() {
                    graph_node.values.inputs[button.input_index] = NodeValue::Float(next);
                }
            }
            ValueType::Int => {
                let current = graph_node
                    .values
                    .inputs
                    .get(button.input_index)
                    .and_then(NodeValue::as_int)
                    .or_else(|| port_def.default_value.as_ref().and_then(NodeValue::as_int))
                    .unwrap_or(0);

                let step = port_def.ui_step.unwrap_or(1.0);
                let mut next = current as f64 + button.delta * step;
                if let Some(min) = port_def.ui_min {
                    next = next.max(min);
                }
                if let Some(max) = port_def.ui_max {
                    next = next.min(max);
                }

                if button.input_index < graph_node.values.inputs.len() {
                    graph_node.values.inputs[button.input_index] =
                        NodeValue::Int(next.round() as i64);
                }
            }
            _ => {}
        }
    }
}

fn handle_popup_bool_toggle_buttons(
    buttons: Query<
        (&Interaction, &NodePopupBoolToggleButton),
        (Changed<Interaction>, With<Button>),
    >,
    popup: Res<NodePopupState>,
    registry: Res<NodeRegistry>,
    mut q_nodes: Query<&mut GraphNode>,
) {
    let Some(node_entity) = popup.open_for else {
        return;
    };

    for (interaction, button) in buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok(mut graph_node) = q_nodes.get_mut(node_entity) else {
            continue;
        };
        let Some(definition) = registry.get(&graph_node.definition_id) else {
            continue;
        };
        let inputs = definition.inputs();
        let Some(port_def) = inputs.get(button.input_index) else {
            continue;
        };
        if port_def.value_type != ValueType::Bool {
            continue;
        }

        let current = graph_node
            .values
            .inputs
            .get(button.input_index)
            .and_then(NodeValue::as_bool)
            .or_else(|| port_def.default_value.as_ref().and_then(NodeValue::as_bool))
            .unwrap_or(false);

        if button.input_index < graph_node.values.inputs.len() {
            graph_node.values.inputs[button.input_index] = NodeValue::Bool(!current);
        }
    }
}

fn handle_popup_text_field_focus(
    buttons: Query<
        (Entity, &Interaction),
        (
            Changed<Interaction>,
            With<Button>,
            With<NodePopupTextFieldInput>,
        ),
    >,
    mut fields: Query<(Entity, &mut NodePopupTextFieldState)>,
) {
    let focused_entity = buttons
        .iter()
        .find_map(|(entity, interaction)| (*interaction == Interaction::Pressed).then_some(entity));

    let Some(focused_entity) = focused_entity else {
        return;
    };

    for (_entity, mut field) in fields.iter_mut() {
        field.focused = false;
    }

    if let Ok((_entity, mut field)) = fields.get_mut(focused_entity) {
        field.focused = true;
    }
}

fn clear_popup_text_field_focus_on_other_clicks(
    buttons: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            Without<NodePopupTextFieldInput>,
        ),
    >,
    mut fields: Query<&mut NodePopupTextFieldState>,
) {
    let should_clear = buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if !should_clear {
        return;
    }

    for mut field in fields.iter_mut() {
        field.focused = false;
    }
}

fn handle_popup_text_field_keyboard_input(
    mut keyboard: MessageReader<KeyboardInput>,
    popup: Res<NodePopupState>,
    mut fields: Query<(&NodePopupTextFieldInput, &mut NodePopupTextFieldState)>,
    mut q_nodes: Query<&mut GraphNode>,
) {
    let Some(node_entity) = popup.open_for else {
        return;
    };

    let Some((field_input, mut field_state)) = fields.iter_mut().find(|(_, field)| field.focused)
    else {
        return;
    };

    let Ok(mut graph_node) = q_nodes.get_mut(node_entity) else {
        return;
    };

    let mut changed = false;
    for event in keyboard.read() {
        if event.state != ButtonState::Pressed || event.repeat {
            continue;
        }

        match event.key_code {
            KeyCode::Backspace => {
                changed |= field_state.text.pop().is_some();
            }
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Escape => {
                field_state.focused = false;
            }
            _ => {
                if let Some(text) = &event.text {
                    let filtered: String = text.chars().filter(|ch| !ch.is_control()).collect();
                    if !filtered.is_empty() {
                        field_state.text.push_str(&filtered);
                        changed = true;
                    }
                } else if let Key::Character(text) = &event.logical_key {
                    let filtered: String = text.chars().filter(|ch| !ch.is_control()).collect();
                    if !filtered.is_empty() {
                        field_state.text.push_str(&filtered);
                        changed = true;
                    }
                }
            }
        }
    }

    if changed && field_input.input_index < graph_node.values.inputs.len() {
        graph_node.values.inputs[field_input.input_index] =
            NodeValue::string(field_state.text.clone());
    }
}

fn sync_popup_text_field_visuals(
    mut fields: Query<
        (
            &NodePopupTextFieldState,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<NodePopupTextFieldState>,
    >,
    mut texts: Query<&mut Text, With<NodePopupTextFieldDisplay>>,
    mut text_colors: Query<&mut TextColor, With<NodePopupTextFieldDisplay>>,
) {
    for (field, children, mut background, mut border) in fields.iter_mut() {
        background.0 = if field.focused {
            Color::srgb(0.2, 0.2, 0.3)
        } else {
            Color::srgb(0.14, 0.14, 0.18)
        };

        *border = BorderColor::all(if field.focused {
            Color::srgb(0.3, 0.7, 1.0)
        } else {
            Color::srgb(0.24, 0.24, 0.3)
        });

        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = if field.text.is_empty() {
                    field.placeholder.clone()
                } else if field.focused {
                    format!("{}|", field.text)
                } else {
                    field.text.clone()
                };
            }
            if let Ok(mut text_color) = text_colors.get_mut(child) {
                text_color.0 = if field.text.is_empty() {
                    Color::srgb(0.55, 0.55, 0.62)
                } else {
                    Color::WHITE
                };
            }
        }
    }
}

fn sync_popup_anchor_world(
    mut popup: ResMut<NodePopupState>,
    q_nodes: Query<&Transform, With<GraphNode>>,
) {
    let Some(node_entity) = popup.open_for else {
        return;
    };

    let Ok(transform) = q_nodes.get(node_entity) else {
        popup.open_for = None;
        return;
    };
    popup.anchor_world = transform.translation.truncate();
}

fn update_popup_panel_position(
    popup: Res<NodePopupState>,
    windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    mut panel_query: Query<&mut Node, With<NodePopupPanel>>,
) {
    let Ok(mut panel_node) = panel_query.single_mut() else {
        return;
    };

    let Some(_) = popup.open_for else {
        panel_node.display = Display::None;
        return;
    };

    let Ok(window) = windows.single() else {
        panel_node.display = Display::None;
        return;
    };
    let Ok((camera, camera_transform)) = q_camera.single() else {
        panel_node.display = Display::None;
        return;
    };

    let Ok(screen_pos) = camera.world_to_viewport(camera_transform, popup.anchor_world.extend(0.0))
    else {
        panel_node.display = Display::None;
        return;
    };

    let panel_width = 340.0;
    let panel_height = 620.0;
    let max_left = (window.width() - panel_width - 6.0).max(6.0);
    let max_top = (window.height() - panel_height - 6.0).max(6.0);

    let left = (screen_pos.x + 18.0).clamp(6.0, max_left);
    let top = (window.height() - screen_pos.y + 18.0).clamp(6.0, max_top);

    panel_node.display = Display::Flex;
    panel_node.left = Val::Px(left);
    panel_node.top = Val::Px(top);
}

fn rebuild_popup_content(
    mut commands: Commands,
    mut popup: ResMut<NodePopupState>,
    registry: Res<NodeRegistry>,
    q_nodes: Query<Ref<GraphNode>>,
    content_query: Query<Entity, With<NodePopupContent>>,
    title_query: Query<Entity, With<NodePopupTitleText>>,
    mut title_text_query: Query<&mut Text>,
    children_query: Query<&Children>,
    popup_text_fields: Query<&NodePopupTextFieldState>,
) {
    let Ok(content_entity) = content_query.single() else {
        return;
    };
    let Ok(title_entity) = title_query.single() else {
        return;
    };

    let Some(node_entity) = popup.open_for else {
        if !popup.is_changed() {
            return;
        }
        if let Ok(mut title) = title_text_query.get_mut(title_entity) {
            title.0 = "Node Settings".to_string();
        }
        if let Ok(existing_children) = children_query.get(content_entity) {
            for child in existing_children.iter() {
                commands.entity(child).despawn();
            }
        }
        return;
    };

    let Ok(graph_node) = q_nodes.get(node_entity) else {
        popup.open_for = None;
        return;
    };
    let text_field_focused = popup_text_fields.iter().any(|field| field.focused);
    if !popup.is_changed() && (!graph_node.is_changed() || text_field_focused) {
        return;
    }
    let Some(definition) = registry.get(&graph_node.definition_id) else {
        popup.open_for = None;
        return;
    };
    let inputs = definition.inputs();

    if let Ok(existing_children) = children_query.get(content_entity) {
        for child in existing_children.iter() {
            commands.entity(child).despawn();
        }
    }

    if let Ok(mut title) = title_text_query.get_mut(title_entity) {
        title.0 = format!("{} Settings", definition.display_name());
    }

    let editable_indices: Vec<usize> = inputs
        .iter()
        .enumerate()
        .filter_map(|(idx, port)| port.editable_in_popup.then_some(idx))
        .collect();

    if editable_indices.is_empty() {
        commands.entity(content_entity).with_children(|content| {
            content.spawn((
                Text::new("No editable parameters."),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.85)),
            ));
        });
        return;
    }

    commands.entity(content_entity).with_children(|content| {
        for index in editable_indices {
            let Some(port_def) = inputs.get(index) else {
                continue;
            };
            match port_def.value_type {
                ValueType::Float => {
                    let value = graph_node
                        .values
                        .inputs
                        .get(index)
                        .and_then(NodeValue::as_float)
                        .or_else(|| {
                            port_def
                                .default_value
                                .as_ref()
                                .and_then(NodeValue::as_float)
                        })
                        .unwrap_or(0.0);
                    spawn_numeric_row(
                        content,
                        &port_def.name,
                        &format!("{:.3}", value),
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: -1.0,
                            channel: None,
                        },
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: 1.0,
                            channel: None,
                        },
                    );
                }
                ValueType::Int => {
                    let value = graph_node
                        .values
                        .inputs
                        .get(index)
                        .and_then(NodeValue::as_int)
                        .or_else(|| port_def.default_value.as_ref().and_then(NodeValue::as_int))
                        .unwrap_or(0);
                    spawn_numeric_row(
                        content,
                        &port_def.name,
                        &value.to_string(),
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: -1.0,
                            channel: None,
                        },
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: 1.0,
                            channel: None,
                        },
                    );
                }
                ValueType::Bool => {
                    let value = graph_node
                        .values
                        .inputs
                        .get(index)
                        .and_then(NodeValue::as_bool)
                        .or_else(|| port_def.default_value.as_ref().and_then(NodeValue::as_bool))
                        .unwrap_or(false);

                    content
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                display: Display::Flex,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(6.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.85)),
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(format!("{}: {}", port_def.name, value)),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(70.0),
                                    height: Val::Px(24.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(if value {
                                    Color::srgb(0.2, 0.45, 0.22)
                                } else {
                                    Color::srgb(0.45, 0.2, 0.2)
                                }),
                                NodePopupBoolToggleButton { input_index: index },
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(if value { "ON" } else { "OFF" }),
                                    TextFont {
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });
                }
                ValueType::Color => {
                    let color = graph_node
                        .values
                        .inputs
                        .get(index)
                        .and_then(NodeValue::as_color)
                        .or_else(|| {
                            port_def
                                .default_value
                                .as_ref()
                                .and_then(NodeValue::as_color)
                        })
                        .unwrap_or_else(|| Color::srgba(1.0, 1.0, 1.0, 1.0))
                        .to_srgba();

                    content.spawn((
                        Text::new(format!(
                            "{}: ({:.2}, {:.2}, {:.2}, {:.2})",
                            port_def.name, color.red, color.green, color.blue, color.alpha
                        )),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    spawn_numeric_row(
                        content,
                        "R",
                        &format!("{:.2}", color.red),
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: -1.0,
                            channel: Some(ColorChannel::R),
                        },
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: 1.0,
                            channel: Some(ColorChannel::R),
                        },
                    );
                    spawn_numeric_row(
                        content,
                        "G",
                        &format!("{:.2}", color.green),
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: -1.0,
                            channel: Some(ColorChannel::G),
                        },
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: 1.0,
                            channel: Some(ColorChannel::G),
                        },
                    );
                    spawn_numeric_row(
                        content,
                        "B",
                        &format!("{:.2}", color.blue),
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: -1.0,
                            channel: Some(ColorChannel::B),
                        },
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: 1.0,
                            channel: Some(ColorChannel::B),
                        },
                    );
                    spawn_numeric_row(
                        content,
                        "A",
                        &format!("{:.2}", color.alpha),
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: -1.0,
                            channel: Some(ColorChannel::A),
                        },
                        NodePopupAdjustButton {
                            input_index: index,
                            delta: 1.0,
                            channel: Some(ColorChannel::A),
                        },
                    );
                }
                ValueType::String => {
                    let value = graph_node
                        .values
                        .inputs
                        .get(index)
                        .and_then(NodeValue::as_string)
                        .or_else(|| {
                            port_def
                                .default_value
                                .as_ref()
                                .and_then(NodeValue::as_string)
                        })
                        .unwrap_or("");
                    spawn_text_row(content, &port_def.name, index, value);
                }
                _ => {
                    let mut line = String::new();
                    let _ = write!(
                        &mut line,
                        "{}: unsupported field type ({})",
                        port_def.name,
                        port_def.value_type.display_name()
                    );
                    content.spawn((
                        Text::new(line),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.7, 0.4)),
                    ));
                }
            }
        }
    });
}

fn spawn_numeric_row(
    content: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    minus: NodePopupAdjustButton,
    plus: NodePopupAdjustButton,
) {
    content
        .spawn((
            Node {
                width: Val::Percent(100.0),
                display: Display::Flex,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.85)),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{}: {}", label, value)),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            row.spawn((Node {
                display: Display::Flex,
                column_gap: Val::Px(4.0),
                ..default()
            },))
                .with_children(|controls| {
                    spawn_adjust_button(controls, "-", minus);
                    spawn_adjust_button(controls, "+", plus);
                });
        });
}

fn spawn_adjust_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    marker: NodePopupAdjustButton,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(26.0),
                height: Val::Px(24.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.22, 0.26, 0.34)),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_text_row(
    content: &mut ChildSpawnerCommands,
    label: &str,
    input_index: usize,
    value: &str,
) {
    content
        .spawn((
            Node {
                width: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.85)),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            row.spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(30.0),
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.14, 0.14, 0.18)),
                BorderColor::all(Color::srgb(0.24, 0.24, 0.3)),
                NodePopupTextFieldInput { input_index },
                NodePopupTextFieldState {
                    text: value.to_string(),
                    placeholder: "type value".to_string(),
                    focused: false,
                },
            ))
            .with_children(|field| {
                field.spawn((
                    Text::new(if value.is_empty() {
                        "type value"
                    } else {
                        value
                    }),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(if value.is_empty() {
                        Color::srgb(0.55, 0.55, 0.62)
                    } else {
                        Color::WHITE
                    }),
                    NodePopupTextFieldDisplay,
                ));
            });
        });
}
