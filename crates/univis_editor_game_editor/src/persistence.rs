use crate::components::COMPONENT_KIND_ENTITY_ROOT;
use crate::context::*;
use crate::scene::{
    EditorComponentInstance, EditorComponentPayload, EditorEntity, EditorScene, SCENE_SAVE_VERSION,
    SavedComponentLink, SavedEntityUiState, SavedGraphLayoutNode,
};
use crate::schemas::ComponentSchemaRegistry;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_SCENE_FILE_PATH: &str = "assets/scenes/current_scene.json";
const DEFAULT_BACKUP_DIRECTORY: &str = "assets/scenes/backups";

fn default_scene_save_version() -> u32 {
    SCENE_SAVE_VERSION
}

fn default_editor_mode() -> EditorMode {
    EditorMode::ComponentMode
}

fn default_component_enabled() -> bool {
    true
}

#[derive(Resource, Debug, Clone)]
pub struct ScenePersistenceSettings {
    pub file_path: String,
    pub pretty_json: bool,
    pub autosave_enabled: bool,
    pub autosave_interval_secs: f32,
    pub backup_directory: String,
    pub max_backup_files: usize,
    pub status_duration_secs: f32,
    pub confirm_reload_window_secs: f32,
}

impl Default for ScenePersistenceSettings {
    fn default() -> Self {
        Self {
            file_path: DEFAULT_SCENE_FILE_PATH.to_string(),
            pretty_json: true,
            autosave_enabled: true,
            autosave_interval_secs: 90.0,
            backup_directory: DEFAULT_BACKUP_DIRECTORY.to_string(),
            max_backup_files: 10,
            status_duration_secs: 4.0,
            confirm_reload_window_secs: 3.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameEditorStatusSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
struct GameEditorStatusMessage {
    text: String,
    severity: GameEditorStatusSeverity,
    expires_at_secs: f64,
}

#[derive(Resource, Default)]
pub struct GameEditorStatusState {
    active: Option<GameEditorStatusMessage>,
    rendered_key: Option<String>,
}

#[derive(Component)]
pub(crate) struct GameEditorStatusUi;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSaveFile {
    #[serde(default = "default_scene_save_version")]
    pub version: u32,
    #[serde(default = "default_editor_mode")]
    pub editor_mode: EditorMode,
    pub active_entity_id: Option<u64>,
    #[serde(default)]
    pub entities: Vec<SavedEntity>,
    #[serde(default)]
    pub next_entity_id: Option<u64>,
    #[serde(default)]
    pub next_component_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedEntity {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub components: Vec<SavedComponent>,
    #[serde(default)]
    pub graph_layout: Vec<SavedGraphLayoutNode>,
    #[serde(default)]
    pub composition_links: Vec<SavedComponentLink>,
    #[serde(default)]
    pub ui: SavedEntityUiState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedComponent {
    pub id: u64,
    pub kind: String,
    #[serde(default = "default_component_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct SceneSaveFileV0 {
    #[serde(default)]
    pub editor_mode: Option<EditorMode>,
    pub active_entity_id: Option<u64>,
    #[serde(default)]
    pub entities: Vec<SavedEntityV0>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct SavedEntityV0 {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub components: Vec<SavedComponentV0>,
    #[serde(default)]
    pub graph_layout: Vec<SavedGraphLayoutNode>,
    #[serde(default)]
    pub ui: SavedEntityUiState,
}

#[derive(Debug, Clone, Deserialize)]
struct SavedComponentV0 {
    pub id: u64,
    pub kind: String,
    #[serde(default = "default_component_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub payload: Value,
}

struct ParsedSceneBundle {
    scene: EditorScene,
    editor_mode: EditorMode,
    signature: String,
    migration_note: Option<String>,
    placeholder_count: usize,
}

pub(crate) fn game_editor_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mode: Res<EditorModeState>,
    mut save_writer: MessageWriter<SaveSceneRequest>,
    mut save_as_writer: MessageWriter<SaveSceneToPathRequest>,
    mut load_writer: MessageWriter<LoadSceneRequest>,
) {
    if mode.mode != EditorMode::ComponentMode {
        return;
    }

    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift_pressed = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if ctrl_pressed && !shift_pressed && keys.just_pressed(KeyCode::KeyS) {
        save_writer.write(SaveSceneRequest);
    }

    if ctrl_pressed && shift_pressed && keys.just_pressed(KeyCode::KeyS) {
        let stamped_path = format!("assets/scenes/scene_{}.json", unix_timestamp_millis());
        save_as_writer.write(SaveSceneToPathRequest { path: stamped_path });
    }

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyO) {
        load_writer.write(LoadSceneRequest);
    }
}

pub(crate) fn handle_save_scene_requests(
    mut save_requests: MessageReader<SaveSceneRequest>,
    mut save_to_path_requests: MessageReader<SaveSceneToPathRequest>,
    mut settings: ResMut<ScenePersistenceSettings>,
    scene: Res<EditorScene>,
    mode: Res<EditorModeState>,
    mut runtime: ResMut<GameEditorRuntimeState>,
    mut status: ResMut<GameEditorStatusState>,
    time: Res<Time>,
) {
    if mode.mode != EditorMode::ComponentMode {
        return;
    }

    let mut target_paths = Vec::new();

    for request in save_to_path_requests.read() {
        settings.file_path = request.path.clone();
        target_paths.push(request.path.clone());
    }

    let mut save_current = false;
    for _ in save_requests.read() {
        save_current = true;
    }
    if save_current {
        target_paths.push(settings.file_path.clone());
    }

    if target_paths.is_empty() {
        return;
    }

    for path in target_paths {
        match persist_scene_to_path(&path, settings.pretty_json, &scene, mode.mode) {
            Ok((_, signature)) => {
                runtime.last_saved_signature = Some(signature);
                runtime.dirty = false;
                runtime.initialized = true;
                runtime.autosave_elapsed_secs = 0.0;
                runtime.open_confirm_until_secs = None;
                runtime.needs_rebaseline = false;

                set_game_editor_status(
                    &mut status,
                    GameEditorStatusSeverity::Info,
                    format!("Scene saved to {}", path),
                    time.elapsed_secs_f64(),
                    settings.status_duration_secs,
                );
            }
            Err(err) => {
                warn!("Failed to save scene to {}: {}", path, err);
                set_game_editor_status(
                    &mut status,
                    GameEditorStatusSeverity::Error,
                    format!("Save failed: {}", err),
                    time.elapsed_secs_f64(),
                    settings.status_duration_secs,
                );
            }
        }
    }
}

pub(crate) fn handle_load_scene_requests(
    mut load_requests: MessageReader<LoadSceneRequest>,
    mut load_from_path_requests: MessageReader<LoadSceneFromPathRequest>,
    mut settings: ResMut<ScenePersistenceSettings>,
    mut scene: ResMut<EditorScene>,
    schemas: Res<ComponentSchemaRegistry>,
    mut mode_state: ResMut<EditorModeState>,
    mut active_context: ResMut<ActiveEntityContext>,
    mut selected_component: ResMut<SelectedComponentContext>,
    mut runtime: ResMut<GameEditorRuntimeState>,
    mut status: ResMut<GameEditorStatusState>,
    time: Res<Time>,
) {
    let mut requested: Option<(String, bool)> = None;

    for request in load_from_path_requests.read() {
        requested = Some((request.path.clone(), request.force_if_dirty));
    }

    let mut load_current = false;
    for _ in load_requests.read() {
        load_current = true;
    }
    if load_current {
        requested = Some((settings.file_path.clone(), false));
    }

    let Some((path, force_if_dirty)) = requested else {
        return;
    };

    let now = time.elapsed_secs_f64();
    if mode_state.mode == EditorMode::ComponentMode && runtime.dirty && !force_if_dirty {
        let confirmed = runtime
            .open_confirm_until_secs
            .map(|deadline| now <= deadline)
            .unwrap_or(false);

        if !confirmed {
            runtime.open_confirm_until_secs =
                Some(now + settings.confirm_reload_window_secs as f64);
            set_game_editor_status(
                &mut status,
                GameEditorStatusSeverity::Warning,
                "Unsaved changes detected. Press Ctrl+O again to confirm reload.".to_string(),
                now,
                settings.status_duration_secs,
            );
            return;
        }
    }

    runtime.open_confirm_until_secs = None;
    settings.file_path = path.clone();

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            warn!("Failed to read scene file {}: {}", path, err);
            set_game_editor_status(
                &mut status,
                GameEditorStatusSeverity::Error,
                format!("Open failed: cannot read {}", path),
                now,
                settings.status_duration_secs,
            );
            return;
        }
    };

    let parsed = match parse_and_migrate_scene(&content, &schemas) {
        Ok(parsed) => parsed,
        Err(err) => {
            warn!("Failed to parse/migrate scene JSON {}: {}", path, err);
            set_game_editor_status(
                &mut status,
                GameEditorStatusSeverity::Error,
                format!("Open failed: {}", err),
                now,
                settings.status_duration_secs,
            );
            return;
        }
    };

    *scene = parsed.scene;
    scene.ensure_active_entity();

    mode_state.mode = parsed.editor_mode;
    active_context.active_entity_id = scene.active_entity_id;
    selected_component.component_id = scene.get_active_entity().and_then(|entity| {
        entity
            .graph_layout
            .iter()
            .find(|layout| layout.selected)
            .map(|layout| layout.component_id)
    });

    runtime.last_saved_signature = Some(parsed.signature);
    runtime.dirty = false;
    runtime.initialized = true;
    runtime.autosave_elapsed_secs = 0.0;
    runtime.open_confirm_until_secs = None;
    runtime.needs_rebaseline = false;
    runtime.canvas_needs_rebuild = true;
    runtime.links_need_rebuild = true;

    let mut status_text = format!("Scene loaded from {}", path);
    if let Some(note) = parsed.migration_note {
        status_text = note;
    }

    let severity = if parsed.placeholder_count > 0 {
        status_text = format!(
            "{} ({} unknown component type(s) loaded as placeholders)",
            status_text, parsed.placeholder_count
        );
        GameEditorStatusSeverity::Warning
    } else {
        GameEditorStatusSeverity::Info
    };

    set_game_editor_status(
        &mut status,
        severity,
        status_text,
        now,
        settings.status_duration_secs,
    );
}

pub(crate) fn refresh_scene_dirty_state(
    scene: Res<EditorScene>,
    mode: Res<EditorModeState>,
    mut runtime: ResMut<GameEditorRuntimeState>,
) {
    if mode.mode != EditorMode::ComponentMode {
        runtime.open_confirm_until_secs = None;
        runtime.autosave_elapsed_secs = 0.0;
        return;
    }

    let Ok(current_signature) = scene_signature(&scene, mode.mode) else {
        return;
    };

    if !runtime.initialized || runtime.last_saved_signature.is_none() || runtime.needs_rebaseline {
        runtime.last_saved_signature = Some(current_signature);
        runtime.dirty = false;
        runtime.initialized = true;
        runtime.needs_rebaseline = false;
        return;
    }

    runtime.dirty = runtime
        .last_saved_signature
        .as_ref()
        .map(|saved| saved != &current_signature)
        .unwrap_or(false);
}

pub(crate) fn autosave_dirty_scene(
    time: Res<Time>,
    settings: Res<ScenePersistenceSettings>,
    mode: Res<EditorModeState>,
    scene: Res<EditorScene>,
    mut runtime: ResMut<GameEditorRuntimeState>,
    mut status: ResMut<GameEditorStatusState>,
) {
    if mode.mode != EditorMode::ComponentMode || !settings.autosave_enabled {
        return;
    }

    if !runtime.dirty {
        runtime.autosave_elapsed_secs = 0.0;
        return;
    }

    runtime.autosave_elapsed_secs += time.delta_secs();
    if runtime.autosave_elapsed_secs < settings.autosave_interval_secs {
        return;
    }
    runtime.autosave_elapsed_secs = 0.0;

    match persist_scene_to_path(&settings.file_path, settings.pretty_json, &scene, mode.mode) {
        Ok((save_file, signature)) => {
            runtime.last_saved_signature = Some(signature);
            runtime.dirty = false;

            let backup_result = write_backup_file(
                &save_file,
                settings.pretty_json,
                &settings.backup_directory,
                settings.max_backup_files,
            );

            match backup_result {
                Ok(backup_path) => {
                    set_game_editor_status(
                        &mut status,
                        GameEditorStatusSeverity::Info,
                        format!("Autosaved scene to {}", backup_path.display()),
                        time.elapsed_secs_f64(),
                        settings.status_duration_secs,
                    );
                }
                Err(err) => {
                    warn!("Autosave backup warning: {}", err);
                    set_game_editor_status(
                        &mut status,
                        GameEditorStatusSeverity::Warning,
                        format!("Autosave completed but backup failed: {}", err),
                        time.elapsed_secs_f64(),
                        settings.status_duration_secs,
                    );
                }
            }
        }
        Err(err) => {
            warn!("Autosave failed: {}", err);
            set_game_editor_status(
                &mut status,
                GameEditorStatusSeverity::Error,
                format!("Autosave failed: {}", err),
                time.elapsed_secs_f64(),
                settings.status_duration_secs,
            );
        }
    }
}

pub(crate) fn draw_game_editor_status_ui(
    mut commands: Commands,
    mut status: ResMut<GameEditorStatusState>,
    time: Res<Time>,
    existing: Query<Entity, With<GameEditorStatusUi>>,
) {
    if let Some(active) = &status.active {
        if time.elapsed_secs_f64() > active.expires_at_secs {
            status.active = None;
            status.rendered_key = None;
        }
    }

    let existing_entities: Vec<Entity> = existing.iter().collect();

    let Some(active) = &status.active else {
        for entity in existing_entities {
            commands.entity(entity).despawn();
        }
        return;
    };

    let key = format!("{:?}:{}", active.severity, active.text);
    if status.rendered_key.as_ref() == Some(&key) && !existing_entities.is_empty() {
        return;
    }

    for entity in existing_entities {
        commands.entity(entity).despawn();
    }

    let background = match active.severity {
        GameEditorStatusSeverity::Info => Color::srgba(0.1, 0.35, 0.18, 0.92),
        GameEditorStatusSeverity::Warning => Color::srgba(0.45, 0.3, 0.06, 0.92),
        GameEditorStatusSeverity::Error => Color::srgba(0.5, 0.15, 0.15, 0.95),
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(14.0),
                top: Val::Px(58.0),
                width: Val::Px(560.0),
                min_height: Val::Px(34.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(background),
            ZIndex(1001),
            GameEditorStatusUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(active.text.clone()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    status.rendered_key = Some(key);
}

fn set_game_editor_status(
    status: &mut GameEditorStatusState,
    severity: GameEditorStatusSeverity,
    text: String,
    now_secs: f64,
    duration_secs: f32,
) {
    status.active = Some(GameEditorStatusMessage {
        text,
        severity,
        expires_at_secs: now_secs + duration_secs as f64,
    });
    status.rendered_key = None;
}

fn parse_and_migrate_scene(
    content: &str,
    schemas: &ComponentSchemaRegistry,
) -> Result<ParsedSceneBundle, String> {
    let value: Value =
        serde_json::from_str(content).map_err(|err| format!("invalid JSON: {}", err))?;

    let version = value
        .get("version")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(0);

    let (save_file, migration_note) = match version {
        0 => {
            let legacy: SceneSaveFileV0 = serde_json::from_value(value)
                .map_err(|err| format!("invalid scene schema v0 payload: {}", err))?;

            let entities = legacy
                .entities
                .into_iter()
                .map(|entity| SavedEntity {
                    id: entity.id,
                    name: entity.name,
                    components: entity
                        .components
                        .into_iter()
                        .map(|component| SavedComponent {
                            id: component.id,
                            kind: component.kind,
                            enabled: component.enabled,
                            payload: component.payload,
                        })
                        .collect(),
                    graph_layout: entity.graph_layout,
                    composition_links: vec![],
                    ui: entity.ui,
                })
                .collect::<Vec<_>>();

            let max_entity_id = entities.iter().map(|entity| entity.id).max().unwrap_or(0);
            let max_component_id = entities
                .iter()
                .flat_map(|entity| entity.components.iter().map(|component| component.id))
                .max()
                .unwrap_or(0);

            (
                SceneSaveFile {
                    version: SCENE_SAVE_VERSION,
                    editor_mode: legacy.editor_mode.unwrap_or(EditorMode::ComponentMode),
                    active_entity_id: legacy.active_entity_id,
                    entities,
                    next_entity_id: Some(max_entity_id.saturating_add(1)),
                    next_component_id: Some(max_component_id.saturating_add(1)),
                },
                Some("Migrated scene schema from v0 to v1.".to_string()),
            )
        }
        SCENE_SAVE_VERSION => {
            let current: SceneSaveFile = serde_json::from_value(value)
                .map_err(|err| format!("invalid scene schema v1 payload: {}", err))?;
            (current, None)
        }
        other => {
            return Err(format!(
                "unsupported scene schema version {} (latest supported {})",
                other, SCENE_SAVE_VERSION
            ));
        }
    };

    let signature = serde_json::to_string(&save_file)
        .map_err(|err| format!("signature serialization failed: {}", err))?;

    let (scene, placeholder_count, auto_linked_entities, root_created_entities) =
        save_file_to_scene(&save_file, schemas);

    let migration_note = {
        let mut notes = Vec::new();
        if let Some(note) = migration_note {
            notes.push(note);
        }
        if root_created_entities > 0 {
            notes.push(format!(
                "Added missing root component for {} entit(y/ies).",
                root_created_entities
            ));
        }
        if auto_linked_entities > 0 {
            notes.push(format!(
                "Auto-linked {} entit(y/ies) to root due to missing composition links.",
                auto_linked_entities
            ));
        }

        if notes.is_empty() {
            None
        } else {
            Some(notes.join(" "))
        }
    };

    Ok(ParsedSceneBundle {
        scene,
        editor_mode: save_file.editor_mode,
        signature,
        migration_note,
        placeholder_count,
    })
}

fn save_file_to_scene(
    save_file: &SceneSaveFile,
    schemas: &ComponentSchemaRegistry,
) -> (EditorScene, usize, usize, usize) {
    let mut placeholder_count = 0usize;
    let mut auto_linked_entities = 0usize;
    let mut root_created_entities = 0usize;
    let mut entities = Vec::with_capacity(save_file.entities.len());
    let mut max_entity_id = 0u64;
    let mut max_component_id = 0u64;

    for saved_entity in &save_file.entities {
        max_entity_id = max_entity_id.max(saved_entity.id);

        let mut components = Vec::with_capacity(saved_entity.components.len());
        for saved_component in &saved_entity.components {
            max_component_id = max_component_id.max(saved_component.id);

            let mut component = match schemas
                .deserialize_known_payload(&saved_component.kind, saved_component.payload.clone())
            {
                Ok(payload) => EditorComponentInstance {
                    id: saved_component.id,
                    kind: saved_component.kind.clone(),
                    enabled: saved_component.enabled,
                    payload,
                },
                Err(_) => {
                    placeholder_count += 1;
                    schemas.fallback_component(
                        saved_component.id,
                        saved_component.kind.clone(),
                        saved_component.enabled,
                        saved_component.payload.clone(),
                    )
                }
            };

            schemas.validate_component(&mut component);
            components.push(component);
        }

        let mut graph_layout = saved_entity.graph_layout.clone();
        let mut composition_links = saved_entity.composition_links.clone();

        let root_component_id = if let Some(root_id) = components
            .iter()
            .find(|component| component.kind == COMPONENT_KIND_ENTITY_ROOT)
            .map(|component| component.id)
        {
            root_id
        } else {
            max_component_id = max_component_id.saturating_add(1);
            let root_id = max_component_id;
            components.push(EditorComponentInstance {
                id: root_id,
                kind: COMPONENT_KIND_ENTITY_ROOT.to_string(),
                enabled: true,
                payload: EditorComponentPayload::EntityRoot(crate::components::EEntityRoot),
            });
            graph_layout.push(SavedGraphLayoutNode {
                component_id: root_id,
                position: [160.0, 140.0],
                selected: false,
            });
            root_created_entities += 1;
            root_id
        };

        let component_ids: std::collections::HashSet<u64> =
            components.iter().map(|component| component.id).collect();

        for component in &components {
            if graph_layout
                .iter()
                .all(|layout| layout.component_id != component.id)
            {
                let index = graph_layout.len() as f32;
                graph_layout.push(SavedGraphLayoutNode {
                    component_id: component.id,
                    position: [-260.0 + index * 40.0, 140.0 - index * 35.0],
                    selected: false,
                });
            }
        }

        let mut latest_parent_by_child: HashMap<u64, (u64, usize)> = HashMap::new();
        for (order, link) in composition_links.iter().enumerate() {
            if link.child_component_id == root_component_id
                || link.child_component_id == link.parent_component_id
            {
                continue;
            }

            if !component_ids.contains(&link.child_component_id)
                || !component_ids.contains(&link.parent_component_id)
            {
                continue;
            }

            latest_parent_by_child
                .insert(link.child_component_id, (link.parent_component_id, order));
        }

        let mut normalized_links: Vec<(u64, u64, usize)> = latest_parent_by_child
            .into_iter()
            .map(|(child, (parent, order))| (child, parent, order))
            .collect();
        normalized_links.sort_by_key(|(_, _, order)| *order);

        composition_links = normalized_links
            .into_iter()
            .map(
                |(child_component_id, parent_component_id, _)| SavedComponentLink {
                    child_component_id,
                    parent_component_id,
                },
            )
            .collect();

        if components.len() > 1 && composition_links.is_empty() {
            auto_linked_entities += 1;
            composition_links = components
                .iter()
                .filter(|component| component.id != root_component_id)
                .map(|component| SavedComponentLink {
                    child_component_id: component.id,
                    parent_component_id: root_component_id,
                })
                .collect();
        }

        entities.push(EditorEntity {
            id: saved_entity.id,
            name: saved_entity.name.clone(),
            components,
            graph_layout,
            composition_links,
            ui: saved_entity.ui.clone(),
        });
    }

    let next_entity_id = save_file
        .next_entity_id
        .unwrap_or_else(|| max_entity_id.saturating_add(1))
        .max(max_entity_id.saturating_add(1));

    let next_component_id = save_file
        .next_component_id
        .unwrap_or_else(|| max_component_id.saturating_add(1))
        .max(max_component_id.saturating_add(1));

    let mut scene = EditorScene {
        version: SCENE_SAVE_VERSION,
        entities,
        active_entity_id: save_file.active_entity_id,
        next_entity_id,
        next_component_id,
    };
    scene.ensure_active_entity();

    (
        scene,
        placeholder_count,
        auto_linked_entities,
        root_created_entities,
    )
}

fn scene_to_save_file(scene: &EditorScene, mode: EditorMode) -> Result<SceneSaveFile, String> {
    let mut entities = Vec::with_capacity(scene.entities.len());

    for entity in &scene.entities {
        let mut components = Vec::with_capacity(entity.components.len());
        for component in &entity.components {
            components.push(SavedComponent {
                id: component.id,
                kind: component.kind.clone(),
                enabled: component.enabled,
                payload: component_payload_to_json(&component.payload)?,
            });
        }

        entities.push(SavedEntity {
            id: entity.id,
            name: entity.name.clone(),
            components,
            graph_layout: entity.graph_layout.clone(),
            composition_links: entity.composition_links.clone(),
            ui: entity.ui.clone(),
        });
    }

    Ok(SceneSaveFile {
        version: SCENE_SAVE_VERSION,
        editor_mode: mode,
        active_entity_id: scene.active_entity_id,
        entities,
        next_entity_id: Some(scene.next_entity_id),
        next_component_id: Some(scene.next_component_id),
    })
}

fn scene_signature(scene: &EditorScene, mode: EditorMode) -> Result<String, String> {
    let save_file = scene_to_save_file(scene, mode)?;
    serde_json::to_string(&save_file)
        .map_err(|err| format!("signature serialization failed: {}", err))
}

fn component_payload_to_json(payload: &EditorComponentPayload) -> Result<Value, String> {
    match payload {
        EditorComponentPayload::EntityRoot(value) => serde_json::to_value(value)
            .map_err(|err| format!("cannot serialize root payload: {}", err)),
        EditorComponentPayload::Transform(value) => serde_json::to_value(value)
            .map_err(|err| format!("cannot serialize transform payload: {}", err)),
        EditorComponentPayload::Sprite(value) => serde_json::to_value(value)
            .map_err(|err| format!("cannot serialize sprite payload: {}", err)),
        EditorComponentPayload::Camera2D(value) => serde_json::to_value(value)
            .map_err(|err| format!("cannot serialize camera payload: {}", err)),
        EditorComponentPayload::Unknown(value) => Ok(value.clone()),
    }
}

fn persist_scene_to_path(
    path: &str,
    pretty_json: bool,
    scene: &EditorScene,
    mode: EditorMode,
) -> Result<(SceneSaveFile, String), String> {
    let save_file = scene_to_save_file(scene, mode)?;
    let signature = serde_json::to_string(&save_file)
        .map_err(|err| format!("signature serialization failed: {}", err))?;
    write_scene_save_file(path, &save_file, pretty_json)?;
    Ok((save_file, signature))
}

fn write_scene_save_file(
    path: &str,
    save_file: &SceneSaveFile,
    pretty_json: bool,
) -> Result<(), String> {
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("cannot create save directory {}: {}", parent.display(), err))?;
    }

    let payload = if pretty_json {
        serde_json::to_string_pretty(save_file)
            .map_err(|err| format!("cannot serialize JSON payload: {}", err))?
    } else {
        serde_json::to_string(save_file)
            .map_err(|err| format!("cannot serialize JSON payload: {}", err))?
    };

    fs::write(target, payload)
        .map_err(|err| format!("cannot write scene file {}: {}", target.display(), err))
}

fn write_backup_file(
    save_file: &SceneSaveFile,
    pretty_json: bool,
    backup_directory: &str,
    max_backup_files: usize,
) -> Result<PathBuf, String> {
    let backup_dir = Path::new(backup_directory);
    fs::create_dir_all(backup_dir).map_err(|err| {
        format!(
            "cannot create backup directory {}: {}",
            backup_dir.display(),
            err
        )
    })?;

    let backup_name = format!("autosave_{}.json", unix_timestamp_millis());
    let backup_path = backup_dir.join(backup_name);

    write_scene_save_file(
        backup_path
            .to_str()
            .ok_or_else(|| "backup path is not valid UTF-8".to_string())?,
        save_file,
        pretty_json,
    )?;

    prune_backup_files(backup_dir, max_backup_files)?;

    Ok(backup_path)
}

fn prune_backup_files(backup_dir: &Path, max_backup_files: usize) -> Result<(), String> {
    if max_backup_files == 0 {
        return Ok(());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(backup_dir).map_err(|err| {
        format!(
            "cannot read backup directory {}: {}",
            backup_dir.display(),
            err
        )
    })? {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let modified = entry
            .metadata()
            .ok()
            .and_then(|meta| meta.modified().ok())
            .unwrap_or(UNIX_EPOCH);

        files.push((path, modified));
    }

    if files.len() <= max_backup_files {
        return Ok(());
    }

    files.sort_by_key(|(_, modified)| *modified);

    let remove_count = files.len().saturating_sub(max_backup_files);
    for (path, _) in files.into_iter().take(remove_count) {
        if let Err(err) = fs::remove_file(&path) {
            warn!("Failed to prune backup file {}: {}", path.display(), err);
        }
    }

    Ok(())
}

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{COMPONENT_KIND_SPRITE, COMPONENT_KIND_TRANSFORM};
    use crate::scene::EditorScene;

    #[test]
    fn migrate_v0_to_v1_scene() {
        let mut schemas = ComponentSchemaRegistry::default();
        schemas.register_defaults();

        let v0 = r#"
        {
            "active_entity_id": 1,
            "entities": [
                {
                    "id": 1,
                    "name": "Root",
                    "components": [
                        {
                            "id": 11,
                            "kind": "editor/transform",
                            "enabled": true,
                            "payload": {"x": 4.0, "y": 2.0}
                        }
                    ],
                    "graph_layout": [
                        {"component_id": 11, "position": [10.0, 20.0], "selected": true}
                    ]
                }
            ]
        }
        "#;

        let parsed = parse_and_migrate_scene(v0, &schemas).expect("v0 migration should work");
        assert_eq!(parsed.scene.version, SCENE_SAVE_VERSION);
        assert_eq!(parsed.scene.active_entity_id, Some(1));
        assert!(parsed.migration_note.is_some());
    }

    #[test]
    fn scene_roundtrip_signature() {
        let scene = EditorScene::default();
        let signature =
            scene_signature(&scene, EditorMode::ComponentMode).expect("signature should serialize");
        assert!(!signature.is_empty());
    }

    #[test]
    fn unknown_component_becomes_placeholder() {
        let mut schemas = ComponentSchemaRegistry::default();
        schemas.register_defaults();

        let file = SceneSaveFile {
            version: SCENE_SAVE_VERSION,
            editor_mode: EditorMode::ComponentMode,
            active_entity_id: Some(1),
            entities: vec![SavedEntity {
                id: 1,
                name: "Entity".to_string(),
                components: vec![
                    SavedComponent {
                        id: 1,
                        kind: COMPONENT_KIND_TRANSFORM.to_string(),
                        enabled: true,
                        payload: serde_json::json!({"x": 1.0, "y": 2.0}),
                    },
                    SavedComponent {
                        id: 2,
                        kind: "editor/unknown".to_string(),
                        enabled: true,
                        payload: serde_json::json!({"data": 1}),
                    },
                ],
                graph_layout: vec![],
                composition_links: vec![],
                ui: SavedEntityUiState::default(),
            }],
            next_entity_id: Some(2),
            next_component_id: Some(3),
        };

        let (scene, placeholders, _, _) = save_file_to_scene(&file, &schemas);
        assert_eq!(placeholders, 1);
        let active = scene
            .get_active_entity()
            .expect("active entity should exist");
        assert_eq!(active.components.len(), 3);
        assert_eq!(active.components[0].kind, COMPONENT_KIND_TRANSFORM);
        assert_eq!(active.components[1].kind, "editor/unknown");

        let save_file = scene_to_save_file(&scene, EditorMode::ComponentMode)
            .expect("scene serialization should succeed");
        assert_eq!(
            save_file.entities[0].components[0].kind,
            COMPONENT_KIND_TRANSFORM
        );
        assert_ne!(
            save_file.entities[0].components[0].kind,
            COMPONENT_KIND_SPRITE
        );
    }

    #[test]
    fn entities_without_links_are_auto_attached_to_root() {
        let mut schemas = ComponentSchemaRegistry::default();
        schemas.register_defaults();

        let file = SceneSaveFile {
            version: SCENE_SAVE_VERSION,
            editor_mode: EditorMode::ComponentMode,
            active_entity_id: Some(1),
            entities: vec![SavedEntity {
                id: 1,
                name: "Entity".to_string(),
                components: vec![
                    SavedComponent {
                        id: 10,
                        kind: COMPONENT_KIND_TRANSFORM.to_string(),
                        enabled: true,
                        payload: serde_json::json!({"x": 1.0, "y": 2.0}),
                    },
                    SavedComponent {
                        id: 11,
                        kind: COMPONENT_KIND_SPRITE.to_string(),
                        enabled: true,
                        payload: serde_json::json!({"sprite_id": "hero"}),
                    },
                ],
                graph_layout: vec![],
                composition_links: vec![],
                ui: SavedEntityUiState::default(),
            }],
            next_entity_id: Some(2),
            next_component_id: Some(12),
        };

        let (scene, _, auto_linked_entities, _) = save_file_to_scene(&file, &schemas);
        assert_eq!(auto_linked_entities, 1);

        let active = scene
            .get_active_entity()
            .expect("active entity should exist");
        let root_id = active
            .root_component_id()
            .expect("root component should be created");
        assert_eq!(active.composition_links.len(), 2);
        assert!(
            active
                .composition_links
                .iter()
                .all(|link| link.parent_component_id == root_id)
        );
    }
}
