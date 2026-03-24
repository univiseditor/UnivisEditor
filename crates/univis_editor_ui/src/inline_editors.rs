use crate::node_spawn::{NodeIconFontGlyph, PortLabel};
use crate::internal_prelude::*;
use bevy::prelude::*;
use univis_ui::prelude::*;

#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct InlineNodeSectionState {
    pub transform_open: bool,
}

impl InlineNodeSectionState {
    fn is_open(self, section: InlineNodeSectionKind) -> bool {
        match section {
            InlineNodeSectionKind::Transform => self.transform_open,
        }
    }

    fn toggle(&mut self, section: InlineNodeSectionKind) {
        match section {
            InlineNodeSectionKind::Transform => {
                self.transform_open = !self.transform_open;
            }
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineNodeSectionKind {
    Transform,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct InlineNodeSectionToggleButton {
    pub node_entity: Entity,
    pub section: InlineNodeSectionKind,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct InlineNodeSectionContent {
    pub node_entity: Entity,
    pub section: InlineNodeSectionKind,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct InlineNodeSectionIcon {
    pub node_entity: Entity,
    pub section: InlineNodeSectionKind,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum InlineColorChannel {
    R,
    G,
    B,
    A,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum InlineNumericInputKind {
    Float,
    Int,
    Color(InlineColorChannel),
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct InlineNodeNumericInput {
    pub node_entity: Entity,
    pub input_index: usize,
    pub kind: InlineNumericInputKind,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct InlineNodeToggleInput {
    pub node_entity: Entity,
    pub input_index: usize,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct InlineNodeTextInput {
    pub node_entity: Entity,
    pub input_index: usize,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct InlineNodeColorSwatch {
    pub node_entity: Entity,
    pub input_index: usize,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct InlineNodeEditorContainer {
    pub port_entity: Entity,
}

pub(crate) fn default_inline_section_state(
    inputs: &[PortDefinition],
) -> Option<InlineNodeSectionState> {
    let transform_count = inputs
        .iter()
        .filter(|port| {
            inline_editor_section_for_port(port) == Some(InlineNodeSectionKind::Transform)
        })
        .count();
    if transform_count == 0 {
        return None;
    }

    let inline_count = inputs
        .iter()
        .filter(|port| supports_inline_editor(port))
        .count();

    Some(InlineNodeSectionState {
        transform_open: transform_count == inline_count,
    })
}

pub(crate) fn spawn_inline_input_panel(
    parent: &mut ChildSpawnerCommands,
    node_entity: Entity,
    inputs: &[PortDefinition],
    authored_inputs: &[NodeValue],
) -> bool {
    if inputs.is_empty() {
        return false;
    }

    parent
        .spawn((
            UNode {
                width: UVal::Percent(1.0),
                margin: USides::top(6.0),
                ..default()
            },
            ULayout {
                flex_direction: UFlexDirection::Column,
                gap: 4.0,
                ..default()
            },
        ))
        .with_children(|panel| {
            let mut index = 0usize;
            while index < inputs.len() {
                let port_def = &inputs[index];

                if inline_editor_section_for_port(port_def)
                    == Some(InlineNodeSectionKind::Transform)
                {
                    let start = index;
                    while index < inputs.len()
                        && inline_editor_section_for_port(&inputs[index])
                            == Some(InlineNodeSectionKind::Transform)
                    {
                        index += 1;
                    }

                    spawn_section_header(panel, node_entity, InlineNodeSectionKind::Transform);
                    panel
                        .spawn((
                            UNode {
                                width: UVal::Percent(1.0),
                                ..default()
                            },
                            ULayout {
                                flex_direction: UFlexDirection::Column,
                                gap: 4.0,
                                ..default()
                            },
                            InlineNodeSectionContent {
                                node_entity,
                                section: InlineNodeSectionKind::Transform,
                            },
                        ))
                        .with_children(|section| {
                            for row_index in start..index {
                                spawn_inline_input_row(
                                    section,
                                    node_entity,
                                    row_index,
                                    &inputs[row_index],
                                    authored_inputs,
                                );
                            }
                        });
                    continue;
                }

                spawn_inline_input_row(panel, node_entity, index, port_def, authored_inputs);
                index += 1;
            }
        });

    true
}

pub(crate) fn set_node_input_value(
    graph_node: &mut GraphNode,
    authored_inputs: Option<&mut AuthoredNodeInputs>,
    input_index: usize,
    value: NodeValue,
) -> bool {
    set_graph_node_authored_input_value(graph_node, authored_inputs, input_index, value)
}

pub(crate) fn handle_inline_node_section_toggle_system(
    mut states: Query<&mut InlineNodeSectionState>,
    buttons: Query<(&UInteraction, &InlineNodeSectionToggleButton), Changed<UInteraction>>,
) {
    for (interaction, toggle) in buttons.iter() {
        if !matches!(*interaction, UInteraction::Pressed | UInteraction::Clicked) {
            continue;
        }

        let Ok(mut state) = states.get_mut(toggle.node_entity) else {
            continue;
        };
        state.toggle(toggle.section);
    }
}

pub(crate) fn sync_inline_editor_visibility_system(
    connections: Query<&InputConnection>,
    mut editors: Query<(&InlineNodeEditorContainer, &mut ULayout)>,
) {
    for (binding, mut layout) in editors.iter_mut() {
        let is_connected = connections
            .get(binding.port_entity)
            .ok()
            .and_then(|connection| connection.source_node)
            .is_some();
        layout.display = if is_connected {
            UDisplay::None
        } else {
            UDisplay::Flex
        };
    }
}

pub(crate) fn sync_inline_node_section_visuals_system(
    state_query: Query<&InlineNodeSectionState>,
    mut content_query: Query<(&InlineNodeSectionContent, &mut ULayout)>,
    mut icon_query: Query<(&InlineNodeSectionIcon, &mut UTextLabel)>,
) {
    for (content, mut layout) in content_query.iter_mut() {
        let is_open = state_query
            .get(content.node_entity)
            .map(|state| state.is_open(content.section))
            .unwrap_or(true);
        layout.display = if is_open {
            UDisplay::Flex
        } else {
            UDisplay::None
        };
    }

    for (icon, mut label) in icon_query.iter_mut() {
        let is_open = state_query
            .get(icon.node_entity)
            .map(|state| state.is_open(icon.section))
            .unwrap_or(true);
        let next = if is_open {
            Icon::CHEVRON_DOWN
        } else {
            Icon::CHEVRON_RIGHT
        };
        if label.text != next {
            label.text = next.to_string();
        }
    }
}

pub(crate) fn sync_inline_numeric_inputs_system(
    mut widgets: Query<(&InlineNodeNumericInput, &UDragValue), Changed<UDragValue>>,
    mut nodes: Query<(&mut GraphNode, Option<&mut AuthoredNodeInputs>)>,
    mut mutations: ResMut<GraphMutationTracker>,
) {
    for (binding, drag) in widgets.iter_mut() {
        let Ok((mut graph_node, mut authored_inputs)) = nodes.get_mut(binding.node_entity) else {
            continue;
        };

        let value = match binding.kind {
            InlineNumericInputKind::Float => NodeValue::float(drag.value as f64),
            InlineNumericInputKind::Int => NodeValue::int(drag.value.round() as i64),
            InlineNumericInputKind::Color(channel) => {
                let current = graph_node
                    .values
                    .inputs
                    .get(binding.input_index)
                    .and_then(NodeValue::as_color)
                    .unwrap_or(Color::WHITE);
                let mut rgba = current.to_srgba();
                match channel {
                    InlineColorChannel::R => rgba.red = drag.value,
                    InlineColorChannel::G => rgba.green = drag.value,
                    InlineColorChannel::B => rgba.blue = drag.value,
                    InlineColorChannel::A => rgba.alpha = drag.value,
                }
                NodeValue::Color(Color::srgba(rgba.red, rgba.green, rgba.blue, rgba.alpha))
            }
        };

        if set_node_input_value(
            &mut graph_node,
            authored_inputs.as_deref_mut(),
            binding.input_index,
            value,
        ) {
            mutations.mark_changed();
        }
    }
}

pub(crate) fn sync_inline_toggle_inputs_system(
    mut widgets: Query<(&InlineNodeToggleInput, &UToggle), Changed<UToggle>>,
    mut nodes: Query<(&mut GraphNode, Option<&mut AuthoredNodeInputs>)>,
    mut mutations: ResMut<GraphMutationTracker>,
) {
    for (binding, toggle) in widgets.iter_mut() {
        let Ok((mut graph_node, mut authored_inputs)) = nodes.get_mut(binding.node_entity) else {
            continue;
        };

        if set_node_input_value(
            &mut graph_node,
            authored_inputs.as_deref_mut(),
            binding.input_index,
            NodeValue::bool(toggle.checked),
        ) {
            mutations.mark_changed();
        }
    }
}

pub(crate) fn sync_inline_text_inputs_system(
    mut widgets: Query<(&InlineNodeTextInput, &UTextField), Changed<UTextField>>,
    mut nodes: Query<(&mut GraphNode, Option<&mut AuthoredNodeInputs>)>,
    mut mutations: ResMut<GraphMutationTracker>,
) {
    for (binding, field) in widgets.iter_mut() {
        let Ok((mut graph_node, mut authored_inputs)) = nodes.get_mut(binding.node_entity) else {
            continue;
        };

        if set_node_input_value(
            &mut graph_node,
            authored_inputs.as_deref_mut(),
            binding.input_index,
            NodeValue::string(field.text.clone()),
        ) {
            mutations.mark_changed();
        }
    }
}

pub(crate) fn sync_inline_widgets_from_authored_inputs_system(
    node_inputs: Query<&AuthoredNodeInputs>,
    mut numeric_widgets: Query<(&InlineNodeNumericInput, &mut UDragValue)>,
    mut toggle_widgets: Query<(&InlineNodeToggleInput, &mut UToggle)>,
    mut text_widgets: Query<(&InlineNodeTextInput, &mut UTextField)>,
    mut swatches: Query<(&InlineNodeColorSwatch, &mut UNode)>,
) {
    for (binding, mut drag) in numeric_widgets.iter_mut() {
        let Ok(authored_inputs) = node_inputs.get(binding.node_entity) else {
            continue;
        };
        let Some(value) = authored_inputs.values.get(binding.input_index) else {
            continue;
        };

        let next = match binding.kind {
            InlineNumericInputKind::Float => value.as_float().unwrap_or(0.0) as f32,
            InlineNumericInputKind::Int => value.as_int().unwrap_or(0) as f32,
            InlineNumericInputKind::Color(channel) => {
                let rgba = value.as_color().unwrap_or(Color::WHITE).to_srgba();
                match channel {
                    InlineColorChannel::R => rgba.red,
                    InlineColorChannel::G => rgba.green,
                    InlineColorChannel::B => rgba.blue,
                    InlineColorChannel::A => rgba.alpha,
                }
            }
        };

        if (drag.value - next).abs() > f32::EPSILON {
            drag.value = next;
        }
    }

    for (binding, mut toggle) in toggle_widgets.iter_mut() {
        let Ok(authored_inputs) = node_inputs.get(binding.node_entity) else {
            continue;
        };
        let Some(next) = authored_inputs
            .values
            .get(binding.input_index)
            .and_then(NodeValue::as_bool)
        else {
            continue;
        };

        if toggle.checked != next {
            toggle.checked = next;
        }
    }

    for (binding, mut field) in text_widgets.iter_mut() {
        let Ok(authored_inputs) = node_inputs.get(binding.node_entity) else {
            continue;
        };
        let Some(next) = authored_inputs
            .values
            .get(binding.input_index)
            .and_then(NodeValue::as_string)
        else {
            continue;
        };

        if field.text != next {
            field.text = next.to_string();
            field.cursor_position = field.text.len();
        }
    }

    for (binding, mut node) in swatches.iter_mut() {
        let Ok(authored_inputs) = node_inputs.get(binding.node_entity) else {
            continue;
        };
        let next = authored_inputs
            .values
            .get(binding.input_index)
            .and_then(NodeValue::as_color)
            .unwrap_or(Color::WHITE);
        if node.background_color != next {
            node.background_color = next;
        }
    }
}

fn supports_inline_editor(port_def: &PortDefinition) -> bool {
    port_def.editable_inline
        && matches!(
            port_def.value_type,
            ValueType::Float
                | ValueType::Int
                | ValueType::Bool
                | ValueType::String
                | ValueType::Color
        )
}

fn inline_editor_section_for_port(port_def: &PortDefinition) -> Option<InlineNodeSectionKind> {
    if !supports_inline_editor(port_def) {
        return None;
    }

    match port_def.name.as_str() {
        "Tx" | "Ty" | "Tz" | "Rotation" | "Sx" | "Sy" | "Sz" => {
            Some(InlineNodeSectionKind::Transform)
        }
        _ => None,
    }
}

fn initial_input_value(
    authored_inputs: &[NodeValue],
    index: usize,
    port_def: &PortDefinition,
) -> NodeValue {
    authored_inputs
        .get(index)
        .cloned()
        .or_else(|| port_def.default_value.clone())
        .unwrap_or(NodeValue::None)
}

fn scalar_range(port_def: &PortDefinition) -> (f32, f32) {
    let min = port_def.ui_min.unwrap_or(-100_000.0) as f32;
    let max = port_def.ui_max.unwrap_or(100_000.0) as f32;
    if min < max {
        (min, max)
    } else {
        (-100_000.0, 100_000.0)
    }
}

fn spawn_section_header(
    parent: &mut ChildSpawnerCommands,
    node_entity: Entity,
    section: InlineNodeSectionKind,
) {
    parent
        .spawn((
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(28.0),
                padding: USides::column(3.0),
                ..default()
            },
            ULayout {
                align_items: UAlignItems::Center,
                gap: 8.0,
                ..default()
            },
            UInteraction::default(),
            InlineNodeSectionToggleButton {
                node_entity,
                section,
            },
        ))
        .with_children(|row| {
            row.spawn((
                UTextLabel {
                    text: Icon::CHEVRON_RIGHT.to_string(),
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
                NodeIconFontGlyph,
                InlineNodeSectionIcon {
                    node_entity,
                    section,
                },
            ));

            row.spawn(UTextLabel {
                text: "Transform".to_string(),
                font_size: 12.5,
                color: Color::srgb(0.88, 0.9, 0.95),
                ..default()
            });
        });
}

fn spawn_inline_input_row(
    parent: &mut ChildSpawnerCommands,
    node_entity: Entity,
    index: usize,
    port_def: &PortDefinition,
    authored_inputs: &[NodeValue],
) {
    let port_color = port_def.resolve_color();
    let port_name = port_def.display_label();
    let has_editor = supports_inline_editor(port_def);

    parent
        .spawn((
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(30.0),
                padding: USides::column(2.0),
                ..default()
            },
            ULayout {
                align_items: UAlignItems::Center,
                justify_content: if has_editor {
                    UJustifyContent::SpaceBetween
                } else {
                    UJustifyContent::Start
                },
                gap: 8.0,
                ..default()
            },
        ))
        .with_children(|row| {
            let mut port_entity = None;
            row.spawn((
                UNode {
                    width: UVal::Flex(1.0),
                    ..default()
                },
                ULayout {
                    align_items: UAlignItems::Center,
                    gap: 7.0,
                    ..default()
                },
            ))
            .with_children(|left| {
                let created_port_entity = left
                    .spawn((
                        GraphPort {
                            node_entity,
                            port_type: PortType::Input,
                            index,
                            value_type: port_def.value_type.clone(),
                        },
                        InputConnection::default(),
                        UNode {
                            width: UVal::Px(12.0),
                            height: UVal::Px(12.0),
                            background_color: port_color,
                            border_radius: UCornerRadius::all(6.0),
                            ..default()
                        },
                        UBorder {
                            color: Color::srgba(1.0, 1.0, 1.0, 0.08),
                            width: 1.0,
                            radius: UCornerRadius::all(6.0),
                            ..default()
                        },
                        UInteraction::default(),
                    ))
                    .id();
                port_entity = Some(created_port_entity);

                left.spawn((
                    PortLabel {
                        port_entity: created_port_entity,
                    },
                    UTextLabel {
                        text: port_name,
                        font_size: 13.5,
                        color: if port_def.requirement.is_some() {
                            port_color
                        } else {
                            Color::srgb(0.74, 0.74, 0.78)
                        },
                        ..default()
                    },
                ));
            });

            if has_editor {
                let Some(port_entity) = port_entity else {
                    return;
                };
                row.spawn((
                    UNode::default(),
                    ULayout {
                        align_items: UAlignItems::Center,
                        gap: 6.0,
                        ..default()
                    },
                    InlineNodeEditorContainer { port_entity },
                ))
                .with_children(|editor| {
                    spawn_inline_editor_widget(
                        editor,
                        node_entity,
                        index,
                        port_def,
                        &initial_input_value(authored_inputs, index, port_def),
                    );
                });
            }
        });
}

fn spawn_inline_editor_widget(
    parent: &mut ChildSpawnerCommands,
    node_entity: Entity,
    input_index: usize,
    port_def: &PortDefinition,
    value: &NodeValue,
) {
    match port_def.value_type {
        ValueType::Float => spawn_scalar_drag_value(
            parent,
            node_entity,
            input_index,
            port_def,
            value.as_float().unwrap_or(0.0) as f32,
            InlineNumericInputKind::Float,
        ),
        ValueType::Int => spawn_scalar_drag_value(
            parent,
            node_entity,
            input_index,
            port_def,
            value.as_int().unwrap_or(0) as f32,
            InlineNumericInputKind::Int,
        ),
        ValueType::Bool => {
            parent.spawn((
                UToggle::new()
                    .with_checked(value.as_bool().unwrap_or(false))
                    .with_size(46.0, 22.0),
                InlineNodeToggleInput {
                    node_entity,
                    input_index,
                },
            ));
        }
        ValueType::String => {
            let mut field = UTextField::new()
                .with_text(value.as_string().unwrap_or(""))
                .with_placeholder(port_def.name.clone())
                .with_size(142.0, 28.0);
            field.font_size = 14.0;
            field.padding = 8.0;

            parent.spawn((
                field,
                InlineNodeTextInput {
                    node_entity,
                    input_index,
                },
            ));
        }
        ValueType::Color => {
            let color = value.as_color().unwrap_or(Color::WHITE);
            parent.spawn((
                UNode {
                    width: UVal::Px(18.0),
                    height: UVal::Px(18.0),
                    background_color: color,
                    border_radius: UCornerRadius::all(5.0),
                    ..default()
                },
                InlineNodeColorSwatch {
                    node_entity,
                    input_index,
                },
            ));

            let rgba = color.to_srgba();
            for (channel, channel_value) in [
                (InlineColorChannel::R, rgba.red),
                (InlineColorChannel::G, rgba.green),
                (InlineColorChannel::B, rgba.blue),
                (InlineColorChannel::A, rgba.alpha),
            ] {
                spawn_scalar_drag_value(
                    parent,
                    node_entity,
                    input_index,
                    port_def,
                    channel_value,
                    InlineNumericInputKind::Color(channel),
                );
            }
        }
        _ => {}
    }
}

fn spawn_scalar_drag_value(
    parent: &mut ChildSpawnerCommands,
    node_entity: Entity,
    input_index: usize,
    port_def: &PortDefinition,
    value: f32,
    kind: InlineNumericInputKind,
) {
    let (min, max) = match kind {
        InlineNumericInputKind::Color(_) => (
            port_def.ui_min.unwrap_or(0.0) as f32,
            port_def.ui_max.unwrap_or(1.0) as f32,
        ),
        _ => scalar_range(port_def),
    };
    let step = match kind {
        InlineNumericInputKind::Int => port_def.ui_step.unwrap_or(1.0) as f32,
        InlineNumericInputKind::Color(_) => port_def.ui_step.unwrap_or(0.05) as f32,
        InlineNumericInputKind::Float => port_def.ui_step.unwrap_or(0.1) as f32,
    };
    let decimals = match kind {
        InlineNumericInputKind::Int => 0,
        InlineNumericInputKind::Color(_) => 2,
        InlineNumericInputKind::Float => 3,
    };
    let width = match kind {
        InlineNumericInputKind::Color(_) => 42.0,
        InlineNumericInputKind::Int => 88.0,
        InlineNumericInputKind::Float => 104.0,
    };

    parent.spawn((
        UNode {
            width: UVal::Px(width),
            height: UVal::Px(26.0),
            ..default()
        },
        UDragValue::new()
            .with_range(min, max)
            .with_step(step)
            .with_decimals(decimals)
            .with_value(value),
        InlineNodeNumericInput {
            node_entity,
            input_index,
            kind,
        },
    ));
}
