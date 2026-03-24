//! Helpers for spawning graph nodes from registered definitions.
use crate::inline_editors::{default_inline_section_state, spawn_inline_input_panel};
use crate::internal_prelude::*;
use bevy::prelude::*;
use std::sync::Arc;
use univis_ui::prelude::*;

#[derive(Component)]
pub struct Header;

#[derive(Component)]
pub struct NodeIconFontGlyph;

#[derive(Component)]
pub struct NodeBody(pub Entity);

#[derive(Component)]
pub struct PortLabel {
    pub port_entity: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct MissingNodePlaceholder {
    pub original_definition_id: NodeId,
}

fn authored_inputs_from_ports(inputs: &[PortDefinition]) -> Vec<NodeValue> {
    inputs
        .iter()
        .map(|port| port.default_value().cloned().unwrap_or(NodeValue::None))
        .collect()
}

/// Extension trait for spawning nodes from the registry.
pub trait SpawnNodeExt<'w, 's> {
    fn spawn_node_from_definition(
        &mut self,
        definition_id: &NodeId,
        position: Vec2,
        registry: &NodeRegistry,
    );

    fn spawn_node_with_definition(&mut self, definition: &ArcNodeDefinition, position: Vec2);
}

impl<'w, 's> SpawnNodeExt<'w, 's> for Commands<'w, 's> {
    fn spawn_node_from_definition(
        &mut self,
        definition_id: &NodeId,
        position: Vec2,
        registry: &NodeRegistry,
    ) {
        if let Some(definition) = registry.get(definition_id) {
            let _ = spawn_node_from_definition_entity(self, &definition, position);
        } else {
            warn!("Node definition not found: {}", definition_id);
        }
    }

    fn spawn_node_with_definition(&mut self, definition: &ArcNodeDefinition, position: Vec2) {
        let _ = spawn_node_from_definition_entity(self, definition, position);
    }
}

pub fn spawn_node_from_definition_entity<'w, 's>(
    commands: &mut Commands<'w, 's>,
    definition: &ArcNodeDefinition,
    position: Vec2,
) -> Entity {
    let title = definition.display_name().to_string();
    let category = definition.category().as_str().to_string();
    let color = definition.color();
    let title_color = definition.title_color();
    let inputs = definition.inputs();
    let outputs = definition.outputs();
    let input_count = inputs.len();
    let output_count = outputs.len();
    let definition_id = definition.id();
    let has_custom_body = definition.has_custom_body();
    let authored_inputs = authored_inputs_from_ports(&inputs);
    let (header_icon, header_icon_uses_icon_font) = node_icon_for_definition(definition);

    let definition_clone = Arc::clone(definition);
    let node_width = 300.0;
    let authored_inputs = AuthoredNodeInputs {
        values: authored_inputs.clone(),
    };
    let mut graph_node = GraphNode::new(definition_id, input_count, output_count);
    graph_node.sync_input_projection_from_authored(&authored_inputs);

    let root_entity = commands
        .spawn((
            UWorldRoot {
                size: Vec2::ZERO,
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 1.0),
            graph_node,
            authored_inputs.clone(),
            UInteraction::default(),
            UBorder {
                color: Color::WHITE,
                width: 1.0,
                ..default()
            },
            UNode {
                border_radius: UCornerRadius::all(10.0),
                padding: USides::all(3.0),
                shape_mode: UShapeMode::Round,
                ..default()
            },
        ))
        .id();

    if let Some(section_state) = default_inline_section_state(&inputs) {
        commands.entity(root_entity).insert(section_state);
    }

    commands.entity(root_entity).with_children(|parent| {
        parent
            .spawn((
                UNode {
                    width: UVal::Px(node_width),
                    height: UVal::Content,
                    background_color: Color::srgb(0.15, 0.15, 0.18),
                    border_radius: UCornerRadius::all(8.0),
                    ..default()
                },
                ULayout {
                    flex_direction: UFlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|container| {
                // Header
                container
                    .spawn((
                        UNode {
                            width: UVal::Percent(1.0),
                            height: UVal::Px(40.0),
                            background_color: color,
                            padding: USides::axes(10.0, 5.0),
                            border_radius: UCornerRadius::top(8.0),
                            ..default()
                        },
                        Header,
                        UInteraction::default(),
                        ULayout {
                            align_items: UAlignItems::Center,
                            justify_content: UJustifyContent::SpaceBetween,
                            ..default()
                        },
                    ))
                    .with_children(|header| {
                        header
                            .spawn((
                                UNode::default(),
                                ULayout {
                                    align_items: UAlignItems::Center,
                                    gap: 8.0,
                                    ..default()
                                },
                            ))
                            .with_children(|left| {
                                left.spawn((
                                    UNode {
                                        width: UVal::Px(28.0),
                                        height: UVal::Px(28.0),
                                        background_color: Color::srgba(0.08, 0.09, 0.12, 0.22),
                                        border_radius: UCornerRadius::all(14.0),
                                        ..default()
                                    },
                                    ULayout {
                                        justify_content: UJustifyContent::Center,
                                        align_items: UAlignItems::Center,
                                        ..default()
                                    },
                                ))
                                .with_children(|badge| {
                                    let mut entity = badge.spawn(UTextLabel {
                                        text: header_icon.clone(),
                                        font_size: 14.0,
                                        color: title_color,
                                        ..default()
                                    });
                                    if header_icon_uses_icon_font {
                                        entity.insert(NodeIconFontGlyph);
                                    }
                                });

                                left.spawn((
                                    UNode::default(),
                                    ULayout {
                                        flex_direction: UFlexDirection::Column,
                                        justify_content: UJustifyContent::Center,
                                        ..default()
                                    },
                                ))
                                .with_children(|text| {
                                    text.spawn(UTextLabel {
                                        text: title.clone(),
                                        font_size: 17.0,
                                        color: title_color,
                                        ..default()
                                    });

                                    text.spawn(UTextLabel {
                                        text: category.clone(),
                                        font_size: 10.5,
                                        color: Color::srgba(1.0, 1.0, 1.0, 0.62),
                                        ..default()
                                    });
                                });
                            });
                    });

                // Body
                container
                    .spawn((
                        UNode {
                            width: UVal::Percent(1.0),
                            padding: USides::column(10.0),
                            ..default()
                        },
                        ULayout {
                            flex_direction: UFlexDirection::Row,
                            justify_content: UJustifyContent::SpaceBetween,
                            align_items: UAlignItems::Start,
                            gap: 12.0,
                            ..default()
                        },
                    ))
                    .with_children(|body| {
                        body.spawn((
                            UNode {
                                width: UVal::Flex(1.0),
                                height: UVal::Content,
                                ..default()
                            },
                            ULayout {
                                flex_direction: UFlexDirection::Column,
                                gap: 6.0,
                                ..default()
                            },
                        ))
                        .with_children(|main| {
                            if has_custom_body {
                                definition_clone.build_body(main, root_entity);
                            }

                            let spawned_inputs = spawn_inline_input_panel(
                                main,
                                root_entity,
                                &inputs,
                                &authored_inputs.values,
                            );

                            if !has_custom_body && !spawned_inputs {
                                main.spawn((
                                    NodeBody(root_entity),
                                    UNode::default(),
                                    UInteraction::default(),
                                ));
                            }
                        });

                        // Outputs
                        body.spawn((
                            UNode {
                                width: UVal::Content,
                                ..default()
                            },
                            ULayout {
                                flex_direction: UFlexDirection::Column,
                                gap: 4.0,
                                ..default()
                            },
                        ))
                        .with_children(|col| {
                            for (i, port_def) in outputs.iter().enumerate() {
                                spawn_port_ui_new(col, root_entity, PortType::Output, i, port_def);
                            }
                        });
                    });
            });
    });

    root_entity
}

pub(crate) fn fallback_node_category_icon(category: &str) -> &'static str {
    match category {
        NodeCategory::INPUT => Icon::TYPE,
        NodeCategory::MATH => Icon::CIRCLE_PLUS,
        NodeCategory::LOGIC => Icon::GIT_BRANCH,
        NodeCategory::SCENE => Icon::BOX,
        NodeCategory::ADVANCED => Icon::CPU,
        _ => Icon::CODESANDBOX,
    }
}

pub(crate) fn node_icon_for_definition(definition: &ArcNodeDefinition) -> (String, bool) {
    if let Some(icon) = definition.icon() {
        (icon.to_string(), false)
    } else {
        (
            fallback_node_category_icon(definition.category().as_str()).to_string(),
            true,
        )
    }
}

pub fn sync_node_icon_font_glyphs_system(
    theme: Res<Theme>,
    mut labels: Query<&mut UTextLabel, Added<NodeIconFontGlyph>>,
) {
    for mut label in labels.iter_mut() {
        label.font = theme.icon.font.clone();
    }
}

pub fn spawn_placeholder_node_entity<'w, 's>(
    commands: &mut Commands<'w, 's>,
    original_definition_id: &NodeId,
    position: Vec2,
    input_count: usize,
    output_count: usize,
) -> Entity {
    let inputs: Vec<PortDefinition> = (0..input_count)
        .map(|i| PortDefinition::new(format!("In {}", i + 1), ValueType::Any))
        .collect();
    let outputs: Vec<PortDefinition> = (0..output_count)
        .map(|i| PortDefinition::new(format!("Out {}", i + 1), ValueType::Any))
        .collect();
    let title = format!("Missing: {}", original_definition_id);
    let node_width = 270.0;
    let authored_inputs = AuthoredNodeInputs {
        values: vec![NodeValue::None; input_count],
    };
    let mut graph_node = GraphNode::new(original_definition_id.clone(), input_count, output_count);
    graph_node.sync_input_projection_from_authored(&authored_inputs);

    let root_entity = commands
        .spawn((
            UWorldRoot {
                size: Vec2::ZERO,
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 1.0),
            graph_node,
            authored_inputs,
            MissingNodePlaceholder {
                original_definition_id: original_definition_id.clone(),
            },
            UInteraction::default(),
            UBorder {
                color: Color::WHITE,
                width: 1.0,
                ..default()
            },
            UNode {
                border_radius: UCornerRadius::all(10.0),
                padding: USides::all(3.0),
                shape_mode: UShapeMode::Round,
                ..default()
            },
        ))
        .id();

    commands.entity(root_entity).with_children(|parent| {
        parent
            .spawn((
                UNode {
                    width: UVal::Px(node_width),
                    height: UVal::Content,
                    background_color: Color::srgb(0.15, 0.15, 0.18),
                    border_radius: UCornerRadius::all(8.0),
                    ..default()
                },
                ULayout {
                    flex_direction: UFlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|container| {
                container
                    .spawn((
                        UNode {
                            width: UVal::Percent(1.0),
                            height: UVal::Px(40.0),
                            background_color: Color::srgb(0.75, 0.2, 0.2),
                            padding: USides::axes(10.0, 5.0),
                            border_radius: UCornerRadius::top(8.0),
                            ..default()
                        },
                        Header,
                        UInteraction::default(),
                        ULayout {
                            justify_content: UJustifyContent::SpaceBetween,
                            ..default()
                        },
                    ))
                    .with_children(|header| {
                        header.spawn(UTextLabel {
                            text: title,
                            font_size: 17.0,
                            color: Color::WHITE,
                            ..default()
                        });
                    });

                container
                    .spawn((
                        UNode {
                            width: UVal::Percent(1.0),
                            padding: USides::all(10.0),
                            ..default()
                        },
                        ULayout {
                            flex_direction: UFlexDirection::Row,
                            justify_content: UJustifyContent::SpaceBetween,
                            ..default()
                        },
                    ))
                    .with_children(|body| {
                        body.spawn((
                            ULayout {
                                flex_direction: UFlexDirection::Column,
                                ..default()
                            },
                            UNode::default(),
                        ))
                        .with_children(|col| {
                            for (i, port_def) in inputs.iter().enumerate() {
                                spawn_port_ui_new(col, root_entity, PortType::Input, i, port_def);
                            }
                        });

                        body.spawn((
                            UNode {
                                width: UVal::Flex(1.0),
                                height: UVal::Content,
                                ..default()
                            },
                            ULayout {
                                justify_content: UJustifyContent::Center,
                                align_items: UAlignItems::Center,
                                flex_direction: UFlexDirection::Column,
                                ..default()
                            },
                        ))
                        .with_children(|center| {
                            center
                                .spawn((
                                    NodeBody(root_entity),
                                    UNode::default(),
                                    UInteraction::default(),
                                ))
                                .with_children(|status| {
                                    status.spawn(UTextLabel {
                                        text: "Definition not found".to_string(),
                                        font_size: 11.0,
                                        color: Color::srgb(1.0, 0.8, 0.8),
                                        ..default()
                                    });
                                });
                        });

                        body.spawn((
                            ULayout {
                                flex_direction: UFlexDirection::Column,
                                ..default()
                            },
                            UNode::default(),
                        ))
                        .with_children(|col| {
                            for (i, port_def) in outputs.iter().enumerate() {
                                spawn_port_ui_new(col, root_entity, PortType::Output, i, port_def);
                            }
                        });
                    });
            });
    });

    root_entity
}

pub fn spawn_component_mode_node_entity<'w, 's>(
    commands: &mut Commands<'w, 's>,
    component_kind: &str,
    title: &str,
    position: Vec2,
    missing: bool,
    accepts_children: bool,
    can_be_child: bool,
) -> Entity {
    let node_width = 270.0;
    let header_color = if missing {
        Color::srgb(0.72, 0.2, 0.2)
    } else {
        Color::srgb(0.2, 0.45, 0.75)
    };
    let subtitle = if missing {
        format!("Unavailable Component Type: {}", component_kind)
    } else {
        format!("Component Type: {}", component_kind)
    };
    let inputs: Vec<PortDefinition> = if accepts_children {
        vec![PortDefinition::new("Children", ValueType::Any)]
    } else {
        vec![]
    };
    let outputs: Vec<PortDefinition> = if can_be_child {
        vec![PortDefinition::new("Parent", ValueType::Any)]
    } else {
        vec![]
    };
    let authored_inputs = authored_inputs_from_ports(&inputs);
    let authored_inputs = AuthoredNodeInputs {
        values: authored_inputs,
    };
    let mut graph_node = GraphNode::new(
        NodeId::new(format!("component/{}", component_kind)),
        inputs.len(),
        outputs.len(),
    );
    graph_node.sync_input_projection_from_authored(&authored_inputs);

    let root_entity = commands
        .spawn((
            UWorldRoot {
                size: Vec2::ZERO,
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 1.0),
            graph_node,
            authored_inputs,
            UInteraction::default(),
            UBorder {
                color: Color::WHITE,
                width: 1.0,
                ..default()
            },
            UNode {
                border_radius: UCornerRadius::all(10.0),
                padding: USides::all(3.0),
                shape_mode: UShapeMode::Round,
                ..default()
            },
        ))
        .id();

    commands.entity(root_entity).with_children(|parent| {
        parent
            .spawn((
                UNode {
                    width: UVal::Px(node_width),
                    height: UVal::Content,
                    background_color: Color::srgb(0.15, 0.15, 0.18),
                    border_radius: UCornerRadius::all(8.0),
                    ..default()
                },
                ULayout {
                    flex_direction: UFlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|container| {
                container
                    .spawn((
                        UNode {
                            width: UVal::Percent(1.0),
                            height: UVal::Px(40.0),
                            background_color: header_color,
                            padding: USides::axes(10.0, 5.0),
                            border_radius: UCornerRadius::top(8.0),
                            ..default()
                        },
                        Header,
                        UInteraction::default(),
                        ULayout {
                            justify_content: UJustifyContent::SpaceBetween,
                            ..default()
                        },
                    ))
                    .with_children(|header| {
                        header.spawn(UTextLabel {
                            text: title.to_string(),
                            font_size: 18.0,
                            color: Color::WHITE,
                            ..default()
                        });
                    });

                container
                    .spawn((
                        UNode {
                            width: UVal::Percent(1.0),
                            padding: USides::all(10.0),
                            ..default()
                        },
                        ULayout {
                            flex_direction: UFlexDirection::Row,
                            justify_content: UJustifyContent::SpaceBetween,
                            ..default()
                        },
                    ))
                    .with_children(|body| {
                        body.spawn((
                            ULayout {
                                flex_direction: UFlexDirection::Column,
                                ..default()
                            },
                            UNode {
                                width: UVal::Flex(1.),
                                ..default()
                            },
                        ))
                        .with_children(|col| {
                            for (i, port_def) in inputs.iter().enumerate() {
                                spawn_port_ui_new(col, root_entity, PortType::Input, i, port_def);
                            }
                        });

                        body.spawn((
                            UNode {
                                width: UVal::Flex(2.0),
                                height: UVal::Content,
                                ..default()
                            },
                            ULayout {
                                justify_content: UJustifyContent::Center,
                                align_items: UAlignItems::Center,
                                flex_direction: UFlexDirection::Column,
                                ..default()
                            },
                        ))
                        .with_children(|center| {
                            center.spawn(UTextLabel {
                                text: subtitle,
                                font_size: 12.0,
                                color: if missing {
                                    Color::srgb(1.0, 0.78, 0.78)
                                } else {
                                    Color::srgb(0.74, 0.84, 0.95)
                                },
                                ..default()
                            });
                        });

                        body.spawn((
                            ULayout {
                                flex_direction: UFlexDirection::Column,
                                ..default()
                            },
                            UNode {
                                width: UVal::Flex(1.),
                                ..default()
                            },
                        ))
                        .with_children(|col| {
                            for (i, port_def) in outputs.iter().enumerate() {
                                spawn_port_ui_new(col, root_entity, PortType::Output, i, port_def);
                            }
                        });
                    });
            });
    });

    root_entity
}

pub fn spawn_missing_component_mode_node_entity<'w, 's>(
    commands: &mut Commands<'w, 's>,
    component_kind: &str,
    position: Vec2,
) -> Entity {
    spawn_component_mode_node_entity(
        commands,
        component_kind,
        &format!("Missing: {}", component_kind),
        position,
        true,
        true,
        true,
    )
}

fn spawn_port_ui_new(
    parent: &mut ChildSpawnerCommands,
    node_entity: Entity,
    port_type: PortType,
    index: usize,
    port_def: &PortDefinition,
) {
    let port_color = port_def.resolve_color();
    let is_input = port_type == PortType::Input;
    let port_name = port_def.display_label();
    parent
        .spawn((
            UNode::default(),
            ULayout {
                flex_direction: if is_input {
                    UFlexDirection::Row
                } else {
                    UFlexDirection::RowReverse
                },
                align_items: UAlignItems::Start,
                justify_content: UJustifyContent::Start,
                gap: 6.0,
                ..default()
            },
        ))
        .with_children(|row| {
            let port_entity = if is_input {
                row.spawn((
                    GraphPort {
                        node_entity,
                        port_type,
                        index,
                        value_type: port_def.value_type().clone(),
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
                .id()
            } else {
                row.spawn((
                    GraphPort {
                        node_entity,
                        port_type,
                        index,
                        value_type: port_def.value_type().clone(),
                    },
                    OutputConnections::default(),
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
                .id()
            };

            row.spawn((
                PortLabel { port_entity },
                UTextLabel {
                    text: port_name,
                    font_size: 12.0,
                    color: if port_def.requirement().is_some() {
                        port_color
                    } else {
                        Color::srgb(0.7, 0.7, 0.7)
                    },
                    ..default()
                },
            ));
        });
}
