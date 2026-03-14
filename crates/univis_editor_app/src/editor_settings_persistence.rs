use crate::panels::FloatingPanelsSettings;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use univis_editor_persistence::graph_persistence::{
    GraphHistorySettings, GraphPersistenceSettings,
};
use univis_editor_ui::prelude::EditorSettings;

const DEFAULT_EDITOR_SETTINGS_PATH: &str = ".univis/editor_settings.json";
const EDITOR_SETTINGS_VERSION: u32 = 2;
const MAX_RECENT_FILES: usize = 6;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EditorWorkflowState {
    pub recent_files: Vec<String>,
}

impl EditorWorkflowState {
    pub fn remember_file(&mut self, path: impl Into<String>) {
        let path = path.into();
        if path.trim().is_empty() {
            return;
        }

        self.recent_files.retain(|existing| existing != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(MAX_RECENT_FILES);
    }
}

#[derive(Resource, Debug, Clone)]
struct EditorSettingsStorage {
    path: PathBuf,
    last_saved: Option<EditorSettingsSnapshot>,
}

impl Default for EditorSettingsStorage {
    fn default() -> Self {
        Self {
            path: PathBuf::from(DEFAULT_EDITOR_SETTINGS_PATH),
            last_saved: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct EditorSettingsSnapshot {
    version: u32,
    persistence: PersistenceSettingsSnapshot,
    history: HistorySettingsSnapshot,
    workflow: EditorWorkflowState,
    panels: FloatingPanelsSettings,
    editor: EditorSettings,
}

impl Default for EditorSettingsSnapshot {
    fn default() -> Self {
        Self {
            version: EDITOR_SETTINGS_VERSION,
            persistence: PersistenceSettingsSnapshot::default(),
            history: HistorySettingsSnapshot::default(),
            workflow: EditorWorkflowState::default(),
            panels: FloatingPanelsSettings::default(),
            editor: EditorSettings::default(),
        }
    }
}

impl EditorSettingsSnapshot {
    fn capture(
        persistence: &GraphPersistenceSettings,
        history: &GraphHistorySettings,
        workflow: &EditorWorkflowState,
        panels: &FloatingPanelsSettings,
        editor: &EditorSettings,
    ) -> Self {
        Self {
            version: EDITOR_SETTINGS_VERSION,
            persistence: PersistenceSettingsSnapshot::capture(persistence),
            history: HistorySettingsSnapshot::capture(history),
            workflow: workflow.clone(),
            panels: panels.clone(),
            editor: editor.clone(),
        }
    }

    fn apply(
        &self,
        persistence: &mut GraphPersistenceSettings,
        history: &mut GraphHistorySettings,
        workflow: &mut EditorWorkflowState,
        panels: &mut FloatingPanelsSettings,
        editor: &mut EditorSettings,
    ) {
        self.persistence.apply(persistence);
        self.history.apply(history);
        *workflow = self.workflow.clone();
        *panels = self.panels.clone();
        *editor = self.editor.clone();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct PersistenceSettingsSnapshot {
    pretty_json: bool,
    autosave_enabled: bool,
}

impl Default for PersistenceSettingsSnapshot {
    fn default() -> Self {
        Self {
            pretty_json: GraphPersistenceSettings::default().pretty_json,
            autosave_enabled: GraphPersistenceSettings::default().autosave_enabled,
        }
    }
}

impl PersistenceSettingsSnapshot {
    fn capture(settings: &GraphPersistenceSettings) -> Self {
        Self {
            pretty_json: settings.pretty_json,
            autosave_enabled: settings.autosave_enabled,
        }
    }

    fn apply(&self, settings: &mut GraphPersistenceSettings) {
        settings.pretty_json = self.pretty_json;
        settings.autosave_enabled = self.autosave_enabled;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
struct HistorySettingsSnapshot {
    enabled: bool,
    max_entries: usize,
}

impl Default for HistorySettingsSnapshot {
    fn default() -> Self {
        let defaults = GraphHistorySettings::default();
        Self {
            enabled: defaults.enabled,
            max_entries: defaults.max_entries,
        }
    }
}

impl HistorySettingsSnapshot {
    fn capture(settings: &GraphHistorySettings) -> Self {
        Self {
            enabled: settings.enabled,
            max_entries: settings.max_entries,
        }
    }

    fn apply(&self, settings: &mut GraphHistorySettings) {
        settings.enabled = self.enabled;
        settings.max_entries = self.max_entries;
    }
}

pub struct EditorSettingsPersistencePlugin;

impl Plugin for EditorSettingsPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorSettingsStorage>()
            .init_resource::<EditorWorkflowState>()
            .add_systems(PreStartup, load_editor_settings)
            .add_systems(
                PostUpdate,
                (track_recent_files, persist_editor_settings).chain(),
            );
    }
}

fn load_editor_settings(
    mut storage: ResMut<EditorSettingsStorage>,
    mut persistence_settings: ResMut<GraphPersistenceSettings>,
    mut history_settings: ResMut<GraphHistorySettings>,
    mut workflow_state: ResMut<EditorWorkflowState>,
    mut panel_settings: ResMut<FloatingPanelsSettings>,
    mut editor_settings: ResMut<EditorSettings>,
) {
    let path = storage.path.clone();
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            storage.last_saved = Some(EditorSettingsSnapshot::capture(
                &persistence_settings,
                &history_settings,
                &workflow_state,
                &panel_settings,
                &editor_settings,
            ));
            return;
        }
        Err(err) => {
            warn!(
                "Failed to read editor settings from {}: {}",
                path.display(),
                err
            );
            storage.last_saved = Some(EditorSettingsSnapshot::capture(
                &persistence_settings,
                &history_settings,
                &workflow_state,
                &panel_settings,
                &editor_settings,
            ));
            return;
        }
    };

    let snapshot = match serde_json::from_str::<EditorSettingsSnapshot>(&content) {
        Ok(snapshot) => snapshot,
        Err(err) => {
            warn!(
                "Failed to parse editor settings from {}: {}",
                path.display(),
                err
            );
            storage.last_saved = Some(EditorSettingsSnapshot::capture(
                &persistence_settings,
                &history_settings,
                &workflow_state,
                &panel_settings,
                &editor_settings,
            ));
            return;
        }
    };

    snapshot.apply(
        &mut persistence_settings,
        &mut history_settings,
        &mut workflow_state,
        &mut panel_settings,
        &mut editor_settings,
    );
    storage.last_saved = Some(EditorSettingsSnapshot::capture(
        &persistence_settings,
        &history_settings,
        &workflow_state,
        &panel_settings,
        &editor_settings,
    ));
}

fn track_recent_files(
    persistence_settings: Res<GraphPersistenceSettings>,
    mut workflow_state: ResMut<EditorWorkflowState>,
) {
    if !persistence_settings.is_changed() {
        return;
    }

    let target = Path::new(&persistence_settings.file_path);
    if !target.is_file() {
        return;
    }

    workflow_state.remember_file(persistence_settings.file_path.clone());
}

fn persist_editor_settings(
    persistence_settings: Res<GraphPersistenceSettings>,
    history_settings: Res<GraphHistorySettings>,
    workflow_state: Res<EditorWorkflowState>,
    panel_settings: Res<FloatingPanelsSettings>,
    editor_settings: Res<EditorSettings>,
    mut storage: ResMut<EditorSettingsStorage>,
) {
    if !persistence_settings.is_changed()
        && !history_settings.is_changed()
        && !workflow_state.is_changed()
        && !panel_settings.is_changed()
        && !editor_settings.is_changed()
    {
        return;
    }

    let snapshot = EditorSettingsSnapshot::capture(
        &persistence_settings,
        &history_settings,
        &workflow_state,
        &panel_settings,
        &editor_settings,
    );
    if storage.last_saved.as_ref() == Some(&snapshot) {
        return;
    }

    if let Err(err) = write_editor_settings(&storage.path, &snapshot) {
        warn!(
            "Failed to persist editor settings to {}: {}",
            storage.path.display(),
            err
        );
        return;
    }

    storage.last_saved = Some(snapshot);
}

fn write_editor_settings(path: &Path, snapshot: &EditorSettingsSnapshot) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "failed to create settings directory {}: {}",
                    parent.display(),
                    err
                )
            })?;
        }
    }

    let payload = serde_json::to_string_pretty(snapshot)
        .map_err(|err| format!("failed to serialize editor settings: {}", err))?;
    fs::write(path, payload)
        .map_err(|err| format!("failed to write settings file {}: {}", path.display(), err))
}
