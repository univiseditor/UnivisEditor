//! نظام إنشاء العُقد - النظام الجديد القابل للتوسع

use crate::prelude::*;
use bevy::prelude::*;
use std::sync::Arc;
use univis_ui::prelude::*;

#[derive(Component)]
pub struct Header;

#[derive(Component)]
pub struct NodeBody(pub Entity);

/// مكون للعُقد البديلة عند فقدان التعريف الأصلي وقت التحميل
#[derive(Component, Debug, Clone)]
pub struct MissingNodePlaceholder {
    pub original_definition_id: NodeId,
}

/// Trait لإنشاء العُقد من Registry
pub trait SpawnNodeExt<'w, 's> {
    /// إنشاء عقدة من معرف التعريف
    fn spawn_node_from_definition(
        &mut self,
        definition_id: &NodeId,
        position: Vec2,
        registry: &NodeRegistry,
    );

    /// إنشاء عقدة من ArcNodeDefinition مباشرة
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

/// إنشاء عقدة معروفة من تعريف موجود وإرجاع الكيان الناتج
pub fn spawn_node_from_definition_entity<'w, 's>(
    commands: &mut Commands<'w, 's>,
    definition: &ArcNodeDefinition,
    position: Vec2,
) -> Entity {
    let title = definition.display_name().to_string();
    let color = definition.color();
    let inputs = definition.inputs();
    let outputs = definition.outputs();
    let input_count = inputs.len();
    let output_count = outputs.len();
    let definition_id = definition.id();
    let has_custom_body = definition.has_custom_body();
    let has_popup_inputs = inputs.iter().any(|port| port.editable_in_popup);

    // استنساخ الـ Arc لاستخدامه داخل الـ closure
    let definition_clone = Arc::clone(definition);
    let node_width = 270.0;

    // إنشاء العقدة مع المكونات الجديدة
    let root_entity = commands
        .spawn((
            UWorldRoot {
                size: Vec2::ZERO,
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 1.0),
            GraphNode::new(definition_id, input_count, output_count),
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

    // بناء الهيكل الداخلي
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
                            justify_content: UJustifyContent::SpaceBetween,
                            ..default()
                        },
                    ))
                    .with_children(|header| {
                        header.spawn(UTextLabel {
                            text: title,
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        });

                        if has_popup_inputs {
                            header
                                .spawn((
                                    UNode {
                                        width: UVal::Px(22.0),
                                        height: UVal::Px(22.0),
                                        background_color: Color::srgb(0.12, 0.12, 0.16),
                                        border_radius: UCornerRadius::all(4.0),
                                        ..default()
                                    },
                                    ULayout {
                                        justify_content: UJustifyContent::Center,
                                        align_items: UAlignItems::Center,
                                        ..default()
                                    },
                                    UInteraction::default(),
                                    NodeSettingsButton {
                                        node_entity: root_entity,
                                    },
                                ))
                                .with_children(|button| {
                                    button.spawn(UTextLabel {
                                        text: "G".to_string(),
                                        font_size: 11.0,
                                        color: Color::WHITE,
                                        ..default()
                                    });
                                });
                        }
                    });

                // Body
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
                        // Inputs
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

                        // Center Content
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
                            // استدعاء build_body إذا كانت العقدة تحتاج محتوى مخصص
                            if has_custom_body {
                                definition_clone.build_body(center, root_entity);
                            } else {
                                // العقدة الافتراضية - body فارغ
                                center.spawn((
                                    NodeBody(root_entity),
                                    UNode::default(),
                                    UInteraction::default(),
                                ));
                            }
                        });

                        // Outputs
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

/// إنشاء عقدة بديلة عند فقدان تعريف العقدة الأصلي وقت التحميل
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

    let root_entity = commands
        .spawn((
            UWorldRoot {
                size: Vec2::ZERO,
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 1.0),
            GraphNode::new(original_definition_id.clone(), input_count, output_count),
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

/// إنشاء عقدة Component Mode (بدون أسلاك منطقية في MVP)
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

    let root_entity = commands
        .spawn((
            UWorldRoot {
                size: Vec2::ZERO,
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 1.0),
            GraphNode::new(
                NodeId::new(format!("component/{}", component_kind)),
                inputs.len(),
                outputs.len(),
            ),
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

/// إنشاء Placeholder لعقدة Component مفقودة
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

/// إنشاء منفذ العقدة (النظام الجديد)
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
            // المنفذ (الدائرة)
            row.spawn((
                GraphPort {
                    node_entity,
                    port_type,
                    index,
                    value_type: port_def.value_type.clone(),
                },
                UNode {
                    width: UVal::Px(12.0),
                    height: UVal::Px(12.0),
                    background_color: port_color,
                    border_radius: UCornerRadius::all(6.0),
                    ..default()
                },
                UInteraction::default(),
            ));

            // اسم المنفذ
            row.spawn(UTextLabel {
                text: port_name,
                font_size: 15.0,
                color: if port_def.requirement.is_some() {
                    port_color
                } else {
                    Color::srgb(0.7, 0.7, 0.7)
                },
                ..default()
            });
        });
}
