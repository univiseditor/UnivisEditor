pub mod editor;
pub mod interaction;
pub mod menu;
pub mod node_popup;
pub mod node_spawn;
pub mod widgets;
pub mod wire;

use bevy::prelude::*;
use interaction::*;
use menu::*;
use node_popup::NodePopupPlugin;
use univis_ui::prelude::{UnivisTextFieldPlugin, UnivisUiPlugin};
use wire::*;

#[derive(Message, Debug, Clone, Copy, Default)]
pub struct DeleteSelectedNodesRequest;

#[derive(Resource, Debug, Clone, Copy)]
pub struct GraphEditingUiActivation {
    pub enabled: bool,
}

impl Default for GraphEditingUiActivation {
    fn default() -> Self {
        Self { enabled: true }
    }
}

pub struct NodeUiPlugin;

impl Plugin for NodeUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<UnivisUiPlugin>() {
            app.add_plugins(UnivisUiPlugin);
        }
        if !app.is_plugin_added::<UnivisTextFieldPlugin>() {
            app.add_plugins(UnivisTextFieldPlugin);
        }

        app.init_resource::<univis_node_graph::pin::DragState>()
            .init_resource::<univis_node_graph::pin::Connecting>()
            .init_resource::<univis_node_graph::pin::WireConnectionState>()
            .add_message::<DeleteSelectedNodesRequest>()
            .init_resource::<GraphEditingUiActivation>()
            .init_resource::<ContextMenuState>()
            .add_plugins(editor::EditorPlugin)
            .add_plugins(NodePopupPlugin)
            .add_systems(
                Update,
                (camera_controller, node_drag_system, node_highlight_system).chain(),
            )
            .add_systems(
                Update,
                (
                    wire_start_system.in_set(WireSystemsSet::Start),
                    wire_update_system.in_set(WireSystemsSet::Update),
                    wire_complete_system.in_set(WireSystemsSet::Complete),
                    wire_visuals_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    open_context_menu,
                    sync_context_menu_overlay,
                    draw_context_menu,
                    interact_context_menu,
                    execute_spawn_node_commands,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    selection_system,
                    request_delete_selected_nodes,
                    delete_node_system,
                    reset_inputs,
                    disconnect_wire_system,
                )
                    .chain(),
            )
            .add_systems(PostUpdate, sync_live_graph_document_state);
    }
}

pub mod prelude {
    pub use univis_editor_runtime::prelude::*;
    pub use univis_node_graph::prelude::*;

    pub use crate::DeleteSelectedNodesRequest;
    pub use crate::GraphEditingUiActivation;
    pub use crate::NodeUiPlugin;
    pub use crate::editor::*;
    pub use crate::interaction::*;
    pub use crate::menu::*;
    pub use crate::node_popup::*;
    pub use crate::node_spawn::*;
    pub use crate::widgets::prelude::*;
    pub use crate::wire::*;
}
