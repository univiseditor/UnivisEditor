use crate::editor_settings_persistence::EditorWorkflowState;
use crate::panels::FloatingPanelsSettings;
use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use univis_editor_commands::GraphCommandRequest;
use univis_editor_persistence::graph_persistence::{
    GraphHistorySettings, GraphPersistenceRuntimeState, GraphPersistenceSettings,
    GraphPersistenceStatus, GraphPersistenceStatusSeverity, latest_backup_file,
};
use univis_editor_runtime::GraphRuntimeTraceSettings;
use univis_editor_ui::overlay::{GraphOverlayState, GraphOverlaySurface};
use univis_editor_ui::prelude::{EditorSettings, GraphCamera};
use univis_node_graph::node_definition::NodeId;
use univis_node_graph::node_registry::NodeRegistry;

mod actions;
mod layout;
mod render;
mod state;

use actions::*;
use layout::*;
use render::*;
use state::*;

pub struct CanvasIslandPlugin;

impl Plugin for CanvasIslandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CanvasIslandState>()
            .add_systems(Startup, setup_canvas_island_ui_system)
            .add_systems(
                Update,
                (
                    handle_canvas_island_shortcuts_system,
                    handle_canvas_island_trigger_buttons_system,
                    handle_canvas_island_menu_actions_system,
                    handle_canvas_island_recent_file_buttons_system,
                    handle_canvas_island_recover_autosave_buttons_system,
                    handle_canvas_island_settings_actions_system,
                    handle_canvas_island_search_typing_system,
                    handle_canvas_island_search_result_buttons_system,
                    close_canvas_island_menu_on_outside_click_system,
                    sync_canvas_island_ui_target_system,
                    sync_canvas_island_overlay_system,
                    update_canvas_island_menu_visibility_system,
                    rebuild_canvas_island_file_panel_system,
                    rebuild_canvas_island_search_panel_system,
                    sync_canvas_island_settings_values_system,
                    sync_canvas_island_status_system,
                    style_canvas_island_buttons_system,
                )
                    .chain(),
            );
    }
}
