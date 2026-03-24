use crate::internal_prelude::*;
use bevy::prelude::*;
use univis_ui::prelude::*;

use super::{find_graph_node_ancestor, interaction_is_pointer_active};

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

pub fn node_drag_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    q_camera: Query<&Projection, With<GraphCamera>>,
    mut drag_state: ResMut<DragState>,
    headers: Query<(Entity, &UInteraction), With<Header>>,
    parents: Query<&ChildOf>,
    node_markers: Query<(), With<GraphNode>>,
    mut node_transforms: Query<&mut Transform, With<GraphNode>>,
    mut mutations: ResMut<GraphMutationTracker>,
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
                trans.translation.x += delta_world.x;
                trans.translation.y -= delta_world.y;
                drag_state.last_mouse_pos = current_mouse_pos;
            } else {
                drag_state.active_entity = None;
            }
        }
    }

    if mouse_button.just_released(MouseButton::Left) {
        if drag_state.active_entity.is_some() {
            mutations.mark_changed();
        }
        drag_state.active_entity = None;
    }
}

pub fn draggable_value_system(_query: Query<Entity>) {}
