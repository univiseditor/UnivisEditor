use crate::prelude::*;
use bevy::prelude::*;

use super::graph_editing_enabled;

pub fn request_graph_workflow_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    overlay: Res<GraphOverlayState>,
    activation: Option<Res<GraphEditingUiActivation>>,
    mut command_writer: MessageWriter<GraphCommandRequest>,
) {
    if !graph_editing_enabled(activation.as_deref())
        || overlay.active_surface != GraphOverlaySurface::None
    {
        return;
    }

    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyC) {
        command_writer.write(GraphCommandRequest::CopySelectedNodes);
    }

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyV) {
        let world_position = windows
            .single()
            .ok()
            .and_then(|window| window.cursor_position())
            .and_then(|cursor| {
                camera_query.single().ok().and_then(|(camera, transform)| {
                    camera.viewport_to_world_2d(transform, cursor).ok()
                })
            })
            .unwrap_or(Vec2::ZERO);
        command_writer.write(GraphCommandRequest::PasteNodes {
            position: world_position,
        });
    }

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyD) {
        command_writer.write(GraphCommandRequest::DuplicateSelectedNodes);
    }

    if keys.just_pressed(KeyCode::KeyF) {
        command_writer.write(GraphCommandRequest::FrameSelectedNodes);
    }
}
