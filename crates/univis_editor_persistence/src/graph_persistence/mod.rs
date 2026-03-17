//! Graph persistence resources and systems.

mod apply;
mod history;
mod io;
mod state;

use bevy::prelude::*;
use univis_editor_ui::overlay::GraphOverlayState;

use self::apply::{
    finalize_pending_graph_load_system, handle_apply_graph_document_requests_system,
};
use self::history::{
    capture_graph_history_snapshot_system, graph_persistence_shortcuts_system,
    handle_history_requests_system,
};
pub use self::io::latest_backup_file;
use self::io::{
    autosave_dirty_graph_system, expire_persistence_status_system,
    handle_load_graph_requests_system, handle_save_graph_requests_system,
    refresh_dirty_state_system,
};
use self::state::PendingGraphLoad;
pub use self::state::{
    ApplyGraphDocumentRequest, GraphHistorySettings, GraphHistoryState, GraphPersistenceActivation,
    GraphPersistenceRuntimeState, GraphPersistenceSettings, GraphPersistenceStatus,
    GraphPersistenceStatusMessage, GraphPersistenceStatusSeverity, LoadGraphFromPathRequest,
    LoadGraphRequest, SaveGraphRequest, SaveGraphToPathRequest,
};

/// Plugin that wires save/load and autosave systems into the app.
pub struct GraphPersistencePlugin;

impl Plugin for GraphPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphPersistenceSettings>()
            .init_resource::<GraphPersistenceRuntimeState>()
            .init_resource::<GraphHistorySettings>()
            .init_resource::<GraphHistoryState>()
            .init_resource::<GraphPersistenceActivation>()
            .init_resource::<GraphPersistenceStatus>()
            .init_resource::<GraphOverlayState>()
            .init_resource::<PendingGraphLoad>()
            .add_message::<SaveGraphRequest>()
            .add_message::<LoadGraphRequest>()
            .add_message::<SaveGraphToPathRequest>()
            .add_message::<LoadGraphFromPathRequest>()
            .add_message::<ApplyGraphDocumentRequest>()
            .add_systems(
                PreUpdate,
                (
                    graph_persistence_shortcuts_system,
                    handle_history_requests_system,
                    handle_save_graph_requests_system,
                    handle_load_graph_requests_system,
                )
                    .chain(),
            )
            .add_systems(PreUpdate, handle_apply_graph_document_requests_system)
            .add_systems(
                PostUpdate,
                (
                    finalize_pending_graph_load_system,
                    capture_graph_history_snapshot_system
                        .after(univis_editor_ui::interaction::sync_live_graph_document_state),
                    refresh_dirty_state_system,
                    autosave_dirty_graph_system,
                    expire_persistence_status_system,
                )
                    .chain(),
            );
    }
}
