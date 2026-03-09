//! أنظمة التفاعل مع المستخدم

use crate::prelude::*;
use bevy::ecs::relationship::Relationship;
use bevy::{input::mouse::MouseWheel, platform::collections::HashSet, prelude::*};
use univis_editor_core::mode::{EditorMode, EditorModeState};
use univis_ui::prelude::*;

fn find_graph_node_ancestor(
    start_entity: Entity,
    parents: &Query<&ChildOf>,
    node_markers: &Query<(), With<GraphNode>>,
) -> Option<Entity> {
    let mut current = start_entity;

    loop {
        if node_markers.get(current).is_ok() {
            return Some(current);
        }

        let Ok(parent) = parents.get(current) else {
            return None;
        };
        current = parent.get();
    }
}

fn interaction_is_pointer_active(interaction: &UInteraction) -> bool {
    matches!(
        *interaction,
        UInteraction::Pressed | UInteraction::Hovered | UInteraction::Clicked
    )
}

/// نظام تحديد العُقد
pub fn selection_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    nodes_interaction: Query<(Entity, &UInteraction), With<GraphNode>>,
    headers_interaction: Query<(Entity, &UInteraction), With<Header>>,
    ports_interaction: Query<(&UInteraction, &GraphPort)>,
    parents: Query<&ChildOf>,
    node_markers: Query<(), With<GraphNode>>,
    selected_nodes: Query<Entity, With<Selected>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let clicked_node = nodes_interaction
            .iter()
            .find_map(|(entity, interaction)| {
                interaction_is_pointer_active(interaction).then_some(entity)
            })
            .or_else(|| {
                headers_interaction
                    .iter()
                    .find_map(|(header_entity, interaction)| {
                        if !interaction_is_pointer_active(interaction) {
                            return None;
                        }

                        find_graph_node_ancestor(header_entity, &parents, &node_markers)
                    })
            })
            .or_else(|| {
                ports_interaction.iter().find_map(|(interaction, port)| {
                    if !interaction_is_pointer_active(interaction) {
                        return None;
                    }

                    (node_markers.get(port.node_entity).is_ok()).then_some(port.node_entity)
                })
            });

        if let Some(target_entity) = clicked_node {
            for entity in selected_nodes.iter() {
                if entity != target_entity {
                    commands.entity(entity).remove::<Selected>();
                }
            }
            commands.entity(target_entity).insert(Selected);
        } else {
            for entity in selected_nodes.iter() {
                commands.entity(entity).remove::<Selected>();
            }
        }
    }
}

/// نظام إبراز العُقد المحددة
pub fn node_highlight_system(
    drag_state: Res<DragState>,
    mut nodes: Query<(Entity, &mut UBorder, Option<&Selected>), With<GraphNode>>,
) {
    for (entity, mut border, selected) in nodes.iter_mut() {
        if Some(entity) == drag_state.active_entity {
            border.color = Color::WHITE;
            border.width = 2.0;
        } else if selected.is_some() {
            border.color = Color::srgb(1.0, 0.5, 0.0);
            border.width = 2.0;
        } else {
            border.color = Color::NONE;
            border.width = 1.0;
        }
    }
}

/// نظام حذف العُقد
pub fn delete_node_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mode: Option<Res<EditorModeState>>,
    selected_nodes: Query<Entity, With<Selected>>,
    mut connect: ResMut<Connecting>,
) {
    if mode
        .as_ref()
        .map(|mode| mode.mode != EditorMode::LegacyGraph)
        .unwrap_or(false)
    {
        return;
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        for entity in selected_nodes.iter() {
            commands.entity(entity).despawn();
            connect
                .connections
                .retain(|link| link.from_node != entity && link.to_node != entity);
        }
    }
}

/// نظام إعادة ضبط المداخل غير المتصلة
pub fn reset_inputs(mut q_nodes: Query<(Entity, &mut GraphNode)>, graph: Res<Connecting>) {
    let mut connected_pins: HashSet<(Entity, usize)> = HashSet::new();

    for link in &graph.connections {
        connected_pins.insert((link.to_node, link.to_index));
    }

    for (entity, mut node) in q_nodes.iter_mut() {
        for (index, input) in node.values.inputs.iter_mut().enumerate() {
            if !connected_pins.contains(&(entity, index)) {
                *input = NodeValue::None;
            }
        }
    }
}

/// نظام فصل الوصلات
pub fn disconnect_wire_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut connect: ResMut<Connecting>,
    ports: Query<(Entity, &UInteraction, &GraphPort)>,
) {
    if mouse_button.just_pressed(MouseButton::Right) {
        for (port_entity, interaction, port_data) in ports.iter() {
            if interaction_is_pointer_active(interaction) && port_data.port_type == PortType::Input
            {
                connect
                    .connections
                    .retain(|link| link.to_port != port_entity);
            }
        }
    }
}

/// نظام سحب العُقد
pub fn node_drag_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    q_camera: Query<&Projection, With<GraphCamera>>,
    mut drag_state: ResMut<DragState>,
    headers: Query<(Entity, &UInteraction), With<Header>>,
    parents: Query<&ChildOf>,
    node_markers: Query<(), With<GraphNode>>,
    mut node_transforms: Query<&mut Transform, With<GraphNode>>,
) {
    let window = if let Ok(w) = windows.single() {
        w
    } else {
        return;
    };
    let current_mouse_pos = if let Some(pos) = window.cursor_position() {
        pos
    } else {
        return;
    };

    let zoom_factor = {
        let Ok(projection) = q_camera.single() else {
            return;
        };
        match projection {
            Projection::Orthographic(ortho) => ortho.scale,
            _ => 1.0,
        }
    };

    if mouse_button.just_pressed(MouseButton::Left) {
        let dragged_entity = headers.iter().find_map(|(header_entity, interaction)| {
            if !interaction_is_pointer_active(interaction) {
                return None;
            }

            find_graph_node_ancestor(header_entity, &parents, &node_markers)
        });

        if let Some(entity) = dragged_entity {
            drag_state.active_entity = Some(entity);
            drag_state.last_mouse_pos = current_mouse_pos;
        }
    }

    if mouse_button.pressed(MouseButton::Left) {
        if let Some(entity) = drag_state.active_entity {
            if let Ok(mut trans) = node_transforms.get_mut(entity) {
                let delta_screen = current_mouse_pos - drag_state.last_mouse_pos;
                let delta_world = delta_screen * zoom_factor;
                trans.translation.x = trans.translation.x + delta_world.x;
                trans.translation.y = trans.translation.y - delta_world.y;
                drag_state.last_mouse_pos = current_mouse_pos;
            } else {
                drag_state.active_entity = None;
            }
        }
    }

    if mouse_button.just_released(MouseButton::Left) {
        drag_state.active_entity = None;
    }
}

/// نظام تحكم الكاميرا
pub fn camera_controller(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scroll_evr: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    mut query: Query<
        (&mut Transform, &mut Projection, &Camera, &GlobalTransform),
        With<GraphCamera>,
    >,
) {
    let zoom_speed = 0.1;
    let camera_speed = 400.0;

    let mut scroll_amount = 0.0;
    for ev in scroll_evr.read() {
        scroll_amount += ev.y;
    }

    let Ok((mut transform, mut projection, camera, cam_global_transform)) = query.single_mut()
    else {
        return;
    };

    let Ok(window) = windows.single() else { return };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    if let Projection::Orthographic(ref mut ortho) = *projection {
        let mouse_world_pos = camera
            .viewport_to_world_2d(cam_global_transform, cursor_position)
            .unwrap_or(Vec2::ZERO);

        let scale_before = ortho.scale;

        if scroll_amount != 0.0 {
            ortho.scale *= 1.0 - (scroll_amount * zoom_speed * 0.1);
            ortho.scale = ortho.scale.clamp(0.2, 5.0);
        }

        let scale_after = ortho.scale;

        let cam_pos_vec = transform.translation.truncate();
        let vector_to_mouse = mouse_world_pos - cam_pos_vec;
        let ratio = scale_after / scale_before;
        transform.translation =
            (mouse_world_pos - vector_to_mouse * ratio).extend(transform.translation.z);

        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            transform.translation +=
                direction.normalize() * camera_speed * scale_after * time.delta_secs();
        }
    }
}

/// نظام القيم القابلة للسحب (placeholder)
pub fn draggable_value_system(_query: Query<Entity>) {
    // TODO: تنفيذ نظام السحب للقيم
}
