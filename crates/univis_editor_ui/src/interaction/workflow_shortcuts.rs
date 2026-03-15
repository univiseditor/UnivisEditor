use crate::prelude::*;
use bevy::prelude::*;

use super::graph_editing_enabled;

pub fn request_graph_workflow_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
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

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyD) {
        command_writer.write(GraphCommandRequest::DuplicateSelectedNodes);
    }

    if keys.just_pressed(KeyCode::KeyF) {
        command_writer.write(GraphCommandRequest::FrameSelectedNodes);
    }
}
