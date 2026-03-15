pub mod connection_diagnostics;
pub mod editor;
pub mod interaction;
pub mod menu;
pub mod node_popup;
pub mod node_spawn;
pub mod overlay;
pub mod widgets;
pub mod wire;

use bevy::prelude::*;
use connection_diagnostics::*;
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
            .init_resource::<interaction::BoxSelectionState>()
            .init_resource::<univis_node_graph::pin::WireConnectionState>()
            .init_resource::<GraphConnectionUiDiagnostics>()
            .init_resource::<GraphConnectionInspectorSummary>()
            .add_message::<DeleteSelectedNodesRequest>()
            .init_resource::<GraphEditingUiActivation>()
            .init_resource::<overlay::GraphOverlayState>()
            .init_resource::<ContextMenuState>()
            .add_plugins(editor::EditorPlugin)
            .add_plugins(NodePopupPlugin)
            .add_systems(
                Update,
                (
                    sanitize_graph_editor_state,
                    sync_port_connection_caches_system,
                ),
            )
            .add_systems(Update, sync_box_selection_overlay)
            .add_systems(
                Update,
                (
                    camera_controller,
                    box_selection_input_system,
                    node_drag_system,
                    node_highlight_system,
                    request_graph_workflow_shortcuts,
                    frame_selected_nodes_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    wire_start_system.in_set(WireSystemsSet::Start),
                    wire_update_system.in_set(WireSystemsSet::Update),
                    wire_complete_system.in_set(WireSystemsSet::Complete),
                    refresh_connection_ui_diagnostics_system,
                    wire_visuals_system,
                    sync_port_diagnostic_visuals_system,
                    sync_connection_inspector_summary_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    open_context_menu_system,
                    sync_context_menu_overlay_system,
                    draw_context_menu_system,
                    interact_context_menu_system,
                    execute_spawn_node_commands_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    selection_system,
                    request_delete_selected_nodes,
                    delete_node_system,
                    disconnect_wire_system,
                )
                    .chain(),
            )
            .add_systems(PostUpdate, sync_live_graph_document_state);
    }
}

pub mod prelude {
    pub use univis_editor_commands::prelude::*;
    pub use univis_editor_runtime::prelude::*;
    pub use univis_node_graph::prelude::*;

    pub use crate::connection_diagnostics::*;
    pub use crate::editor::*;
    pub use crate::interaction::*;
    pub use crate::menu::*;
    pub use crate::node_popup::*;
    pub use crate::node_spawn::*;
    pub use crate::overlay::*;
    pub use crate::widgets::prelude::*;
    pub use crate::wire::*;
    pub use crate::DeleteSelectedNodesRequest;
    pub use crate::GraphEditingUiActivation;
    pub use crate::NodeUiPlugin;
}
