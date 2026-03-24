pub mod connection_diagnostics;
pub mod editor;
pub mod inline_editors;
pub mod interaction;
pub mod menu;
pub mod node_spawn;
pub mod overlay;
pub mod widgets;
pub mod wire;

use bevy::prelude::*;
use connection_diagnostics::*;
use inline_editors::*;
use interaction::*;
use menu::*;
use node_spawn::sync_node_icon_font_glyphs_system;
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
            .init_resource::<univis_node_graph::graph_validation::LiveGraphValidationState>()
            .init_resource::<WireDragFeedback>()
            .init_resource::<GraphConnectionUiDiagnostics>()
            .init_resource::<GraphConnectionInspectorSummary>()
            .init_resource::<GraphPortPreviewSummary>()
            .add_message::<DeleteSelectedNodesRequest>()
            .init_resource::<GraphEditingUiActivation>()
            .init_resource::<overlay::GraphOverlayState>()
            .init_resource::<ContextMenuState>()
            .add_plugins(editor::EditorPlugin)
            .add_systems(
                Update,
                (
                    sanitize_graph_editor_state,
                    sync_port_connection_caches_system,
                    sync_node_icon_font_glyphs_system,
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
                    wire_drag_feedback_system.in_set(WireSystemsSet::Update),
                    wire_complete_system.in_set(WireSystemsSet::Complete),
                    refresh_connection_ui_diagnostics_system,
                    wire_visuals_system,
                    sync_port_diagnostic_visuals_system,
                    sync_connection_inspector_summary_system,
                    sync_port_preview_summary_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    open_context_menu_system,
                    sync_context_menu_overlay_system,
                    sync_context_menu_hover_state_system,
                    draw_context_menu_system,
                    sync_context_menu_button_visuals_system,
                    handle_context_menu_scroll_system,
                    sync_context_menu_scrollbar_system,
                    interact_context_menu_system,
                    close_context_menu_on_outside_click_system,
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
            .add_systems(
                PostUpdate,
                (
                    handle_inline_node_section_toggle_system,
                    sync_inline_node_section_visuals_system,
                    sync_inline_editor_visibility_system,
                    sync_inline_numeric_inputs_system,
                    sync_inline_toggle_inputs_system,
                    sync_inline_text_inputs_system,
                    sync_inline_widgets_from_authored_inputs_system,
                    sync_live_graph_document_state,
                    univis_node_graph::graph_validation::refresh_live_graph_validation_state_system,
                )
                    .chain(),
            );
    }
}

pub(crate) mod internal_prelude {
    pub use univis_editor_commands::prelude::*;
    pub use univis_node_graph::prelude::*;

    pub use crate::DeleteSelectedNodesRequest;
    pub use crate::GraphEditingUiActivation;
    pub use crate::connection_diagnostics::*;
    pub use crate::editor::*;
    pub use crate::node_spawn::*;
    pub use crate::overlay::*;
}

pub mod prelude {
    pub use crate::DeleteSelectedNodesRequest;
    pub use crate::GraphEditingUiActivation;
    pub use crate::NodeUiPlugin;
    pub use crate::connection_diagnostics::*;
    pub use crate::editor::*;
    pub use crate::interaction::sync_live_graph_document_state;
    pub use crate::widgets::prelude::*;
    pub use crate::wire::*;
}
