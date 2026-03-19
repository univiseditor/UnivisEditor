use univis_editor_persistence::graph_persistence::{
    GraphPersistenceActivation, GraphPersistenceRuntimeState, GraphPersistenceSettings,
    GraphPersistenceStatus,
};

#[test]
fn persistence_settings_default_to_workspace_graph_paths() {
    let settings = GraphPersistenceSettings::default();

    assert_eq!(settings.file_path, "assets/graphs/current_graph.json");
    assert_eq!(settings.backup_directory, "assets/graphs/backups");
    assert!(settings.pretty_json);
    assert!(settings.autosave_enabled);
    assert_eq!(settings.max_backup_files, 10);
}

#[test]
fn persistence_runtime_state_starts_clean() {
    let runtime = GraphPersistenceRuntimeState::default();

    assert!(!runtime.dirty);
    assert!(!runtime.initialized);
    assert!(runtime.last_saved_document_signature.is_none());
    assert_eq!(runtime.autosave_elapsed_secs, 0.0);
    assert!(runtime.open_confirm_until_secs.is_none());
    assert!(!runtime.needs_rebaseline);
}

#[test]
fn persistence_activation_and_status_have_safe_defaults() {
    let activation = GraphPersistenceActivation::default();
    let status = GraphPersistenceStatus::default();

    assert!(activation.enabled);
    assert!(status.active.is_none());
}
