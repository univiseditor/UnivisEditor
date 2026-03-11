//! نظام نافذة إعدادات العقدة (Popup) داخل world-space UI

use crate::prelude::*;
use bevy::prelude::*;
use std::fmt::Write as _;
use univis_ui::prelude::*;

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

pub struct NodePopupPlugin;

impl Plugin for NodePopupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NodePopupState>()
            .add_systems(Startup, setup_node_popup_ui)
            .add_systems(
                Update,
                (
                    close_popup_when_graph_editing_disabled,
                    sync_popup_overlay,
                    handle_node_settings_button_clicks,
                    handle_popup_close_button,
                    handle_popup_adjust_buttons,
                    handle_popup_bool_toggle_buttons,
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
            UWorldRoot {
                size: Vec2::new(340.0, 620.0),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 25.0),
            Visibility::Hidden,
            NodePopupPanel,
            UNode {
                width: UVal::Px(320.0),
                height: UVal::Content,
                background_color: Color::srgba(0.09, 0.09, 0.12, 0.96),
                padding: USides::all(8.0),
                border_radius: UCornerRadius::all(8.0),
                ..default()
            },
            UBorder {
                color: Color::srgba(1.0, 1.0, 1.0, 0.06),
                width: 1.0,
                radius: UCornerRadius::all(8.0),
                ..default()
            },
            ULayout {
                display: UDisplay::Flex,
                flex_direction: UFlexDirection::Column,
                gap: 6.0,
                ..default()
            },
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    UNode {
                        width: UVal::Percent(1.0),
                        height: UVal::Px(30.0),
                        ..default()
                    },
                    ULayout {
                        display: UDisplay::Flex,
                        justify_content: UJustifyContent::SpaceBetween,
                        align_items: UAlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|header| {
                    header.spawn((
                        UTextLabel {
                            text: "Node Settings".to_string(),
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..default()
                        },
                        NodePopupTitleText,
                    ));

                    header
                        .spawn((
                            UNode {
                                width: UVal::Px(24.0),
                                height: UVal::Px(24.0),
                                background_color: Color::srgb(0.4, 0.2, 0.2),
                                border_radius: UCornerRadius::all(4.0),
                                ..default()
                            },
                            ULayout {
                                display: UDisplay::Flex,
                                justify_content: UJustifyContent::Center,
                                align_items: UAlignItems::Center,
                                ..default()
                            },
                            UInteraction::default(),
                            NodePopupCloseButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn(UTextLabel {
                                text: "X".to_string(),
                                font_size: 12.0,
                                color: Color::WHITE,
                                ..default()
                            });
                        });
                });

            panel.spawn((
                UNode {
                    width: UVal::Percent(1.0),
                    ..default()
                },
                ULayout {
                    display: UDisplay::Flex,
                    flex_direction: UFlexDirection::Column,
                    gap: 4.0,
                    ..default()
                },
                NodePopupContent,
            ));
        });
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

fn sync_popup_overlay(
    mut popup: ResMut<NodePopupState>,
    overlay: Res<GraphOverlayState>,
) {
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
    activation.map(|activation| activation.enabled).unwrap_or(true)
}

fn handle_popup_close_button(
    buttons: Query<&UInteraction, (Changed<UInteraction>, With<NodePopupCloseButton>)>,
    mut popup: ResMut<NodePopupState>,
    mut overlay: ResMut<GraphOverlayState>,
) {
    for interaction in buttons.iter() {
        if *interaction == UInteraction::Pressed {
            popup.open_for = None;
            if overlay.active_surface == GraphOverlaySurface::NodePopup {
                overlay.active_surface = GraphOverlaySurface::None;
            }
        }
    }
}

fn handle_popup_adjust_buttons(
    buttons: Query<(&UInteraction, &NodePopupAdjustButton), Changed<UInteraction>>,
    popup: Res<NodePopupState>,
    registry: Res<NodeRegistry>,
    mut q_nodes: Query<&mut GraphNode>,
) {
    let Some(node_entity) = popup.open_for else {
        return;
    };

    for (interaction, button) in buttons.iter() {
        if *interaction != UInteraction::Pressed {
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
    buttons: Query<(&UInteraction, &NodePopupBoolToggleButton), Changed<UInteraction>>,
    popup: Res<NodePopupState>,
    registry: Res<NodeRegistry>,
    mut q_nodes: Query<&mut GraphNode>,
) {
    let Some(node_entity) = popup.open_for else {
        return;
    };

    for (interaction, button) in buttons.iter() {
        if *interaction != UInteraction::Pressed {
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
    mut panel_query: Query<(&mut Transform, &mut Visibility), With<NodePopupPanel>>,
) {
    let Ok((mut transform, mut visibility)) = panel_query.single_mut() else {
        return;
    };

    let Some(_) = popup.open_for else {
        *visibility = Visibility::Hidden;
        return;
    };

    *visibility = Visibility::Inherited;
    transform.translation = Vec3::new(
        popup.anchor_world.x + 210.0,
        popup.anchor_world.y - 40.0,
        25.0,
    );
}

fn rebuild_popup_content(
    mut commands: Commands,
    mut popup: ResMut<NodePopupState>,
    registry: Res<NodeRegistry>,
    q_nodes: Query<Ref<GraphNode>>,
    content_query: Query<Entity, With<NodePopupContent>>,
    title_query: Query<Entity, With<NodePopupTitleText>>,
    mut title_text_query: Query<&mut UTextLabel>,
    children_query: Query<&Children>,
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
            title.text = "Node Settings".to_string();
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
    if !popup.is_changed() && !graph_node.is_changed() {
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
        title.text = format!("{} Settings", definition.display_name());
    }

    let editable_indices: Vec<usize> = inputs
        .iter()
        .enumerate()
        .filter_map(|(idx, port)| port.editable_in_popup.then_some(idx))
        .collect();

    if editable_indices.is_empty() {
        commands.entity(content_entity).with_children(|content| {
            content.spawn(UTextLabel {
                text: "No editable parameters.".to_string(),
                font_size: 12.0,
                color: Color::srgb(0.8, 0.8, 0.85),
                ..default()
            });
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
                            UNode {
                                width: UVal::Percent(1.0),
                                background_color: Color::srgba(0.15, 0.15, 0.2, 0.85),
                                padding: USides::all(6.0),
                                border_radius: UCornerRadius::all(4.0),
                                ..default()
                            },
                            ULayout {
                                display: UDisplay::Flex,
                                justify_content: UJustifyContent::SpaceBetween,
                                align_items: UAlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            row.spawn(UTextLabel {
                                text: format!("{}: {}", port_def.name, value),
                                font_size: 12.0,
                                color: Color::WHITE,
                                ..default()
                            });

                            row.spawn((
                                UNode {
                                    width: UVal::Px(70.0),
                                    height: UVal::Px(24.0),
                                    background_color: if value {
                                        Color::srgb(0.2, 0.45, 0.22)
                                    } else {
                                        Color::srgb(0.45, 0.2, 0.2)
                                    },
                                    border_radius: UCornerRadius::all(4.0),
                                    ..default()
                                },
                                ULayout {
                                    display: UDisplay::Flex,
                                    justify_content: UJustifyContent::Center,
                                    align_items: UAlignItems::Center,
                                    ..default()
                                },
                                UInteraction::default(),
                                NodePopupBoolToggleButton { input_index: index },
                            ))
                            .with_children(|btn| {
                                btn.spawn(UTextLabel {
                                    text: if value {
                                        "ON".to_string()
                                    } else {
                                        "OFF".to_string()
                                    },
                                    font_size: 11.0,
                                    color: Color::WHITE,
                                    ..default()
                                });
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

                    content.spawn(UTextLabel {
                        text: format!(
                            "{}: ({:.2}, {:.2}, {:.2}, {:.2})",
                            port_def.name, color.red, color.green, color.blue, color.alpha
                        ),
                        font_size: 12.0,
                        color: Color::WHITE,
                        ..default()
                    });

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
                _ => {
                    let mut line = String::new();
                    let _ = write!(
                        &mut line,
                        "{}: unsupported field type ({})",
                        port_def.name,
                        port_def.value_type.display_name()
                    );
                    content.spawn(UTextLabel {
                        text: line,
                        font_size: 11.0,
                        color: Color::srgb(0.9, 0.7, 0.4),
                        ..default()
                    });
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
            UNode {
                width: UVal::Percent(1.0),
                background_color: Color::srgba(0.15, 0.15, 0.2, 0.85),
                padding: USides::all(6.0),
                border_radius: UCornerRadius::all(4.0),
                ..default()
            },
            ULayout {
                display: UDisplay::Flex,
                justify_content: UJustifyContent::SpaceBetween,
                align_items: UAlignItems::Center,
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn(UTextLabel {
                text: format!("{}: {}", label, value),
                font_size: 12.0,
                color: Color::WHITE,
                ..default()
            });

            row.spawn((
                UNode::default(),
                ULayout {
                    display: UDisplay::Flex,
                    gap: 4.0,
                    ..default()
                },
            ))
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
            UNode {
                width: UVal::Px(26.0),
                height: UVal::Px(24.0),
                background_color: Color::srgb(0.22, 0.26, 0.34),
                border_radius: UCornerRadius::all(4.0),
                ..default()
            },
            ULayout {
                display: UDisplay::Flex,
                justify_content: UJustifyContent::Center,
                align_items: UAlignItems::Center,
                ..default()
            },
            UInteraction::default(),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn(UTextLabel {
                text: text.to_string(),
                font_size: 13.0,
                color: Color::WHITE,
                ..default()
            });
        });
}
