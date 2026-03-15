use bevy::prelude::*;
use univis_editor_persistence::graph_persistence::{
    GraphPersistenceSettings, GraphPersistenceStatus, GraphPersistenceStatusMessage,
    GraphPersistenceStatusSeverity,
};

pub(super) fn set_asset_status(
    status: &mut GraphPersistenceStatus,
    severity: GraphPersistenceStatusSeverity,
    text: String,
    settings: &GraphPersistenceSettings,
    time: &Time,
) {
    status.active = Some(GraphPersistenceStatusMessage {
        text,
        severity,
        expires_at_secs: time.elapsed_secs_f64() + settings.status_duration_secs as f64,
    });
}
