use crate::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use super::graph_editing_enabled;

pub fn frame_selected_nodes_system(
    activation: Option<Res<GraphEditingUiActivation>>,
    mut command_requests: MessageReader<GraphCommandRequest>,
    live_document: Res<LiveGraphDocumentState>,
    windows: Query<&Window>,
    mut q_camera: Query<(&mut Transform, &mut Projection), With<GraphCamera>>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        command_requests.clear();
        return;
    }

    if !command_requests
        .read()
        .any(|command| matches!(command, GraphCommandRequest::FrameSelectedNodes))
    {
        return;
    }

    let selected_nodes = live_document
        .document
        .selected_node_ids()
        .iter()
        .filter_map(|node_id| live_document.document.node(*node_id))
        .collect::<Vec<_>>();
    if selected_nodes.is_empty() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut camera_transform, mut projection)) = q_camera.single_mut() else {
        return;
    };

    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for node in selected_nodes {
        let position = Vec2::new(node.position[0], node.position[1]);
        min = min.min(position);
        max = max.max(position);
    }

    let node_extent = Vec2::new(300.0, 160.0);
    min -= node_extent * 0.5;
    max += node_extent * 0.5;

    let center = (min + max) * 0.5;
    let framed_size = (max - min) + Vec2::new(220.0, 180.0);

    camera_transform.translation.x = center.x;
    camera_transform.translation.y = center.y;

    if let Projection::Orthographic(ref mut ortho) = *projection {
        let width_scale = framed_size.x / (window.width() * 0.72).max(1.0);
        let height_scale = framed_size.y / (window.height() * 0.72).max(1.0);
        ortho.scale = width_scale.max(height_scale).max(0.45).clamp(0.2, 5.0);
    }
}

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

    let Ok(window) = windows.single() else {
        return;
    };
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
