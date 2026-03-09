use crate::components::{
    COMPONENT_KIND_CAMERA2D, COMPONENT_KIND_ENTITY_ROOT, COMPONENT_KIND_SPRITE,
    COMPONENT_KIND_TRANSFORM, component_display_name,
};
use crate::context::*;
use crate::scene::{EditorComponentPayload, EditorScene};
use crate::schemas::{ComponentFieldSchema, ComponentSchemaRegistry};
use bevy::picking::prelude::Pickable;
use bevy::prelude::*;

#[derive(Component)]
pub struct GameEditorUiRoot;

#[derive(Component)]
pub(crate) struct TopBarModeText;

#[derive(Component)]
pub(crate) struct TopBarEntityText;

#[derive(Component)]
pub(crate) struct TopBarDirtyText;

#[derive(Component)]
pub struct InspectorPanel;

#[derive(Component)]
pub(crate) struct TopBarActionButton {
    action: TopBarAction,
}

#[derive(Clone, Copy)]
enum TopBarAction {
    ToggleMode,
    PrevEntity,
    NextEntity,
    Save,
    SaveAs,
    Open,
}

#[derive(Component)]
pub(crate) struct AddComponentButton {
    kind: String,
}

#[derive(Component)]
pub(crate) struct InspectorSelectButton {
    component_id: u64,
}

#[derive(Component)]
pub(crate) struct InspectorRemoveButton {
    component_id: u64,
}

#[derive(Component)]
pub(crate) struct InspectorAdjustButton {
    component_id: u64,
    field_key: String,
    delta: f32,
}

pub(crate) fn setup_game_editor_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Pickable::IGNORE,
            ZIndex(1000),
            Visibility::Hidden,
            GameEditorUiRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(42.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.9)),
            ))
            .with_children(|bar| {
                spawn_top_bar_button(bar, "Mode", TopBarAction::ToggleMode);
                spawn_top_bar_button(bar, "< Entity", TopBarAction::PrevEntity);
                spawn_top_bar_button(bar, "Entity >", TopBarAction::NextEntity);
                spawn_top_bar_button(bar, "Save", TopBarAction::Save);
                spawn_top_bar_button(bar, "Save As", TopBarAction::SaveAs);
                spawn_top_bar_button(bar, "Open", TopBarAction::Open);

                bar.spawn((
                    Text::new("Mode: LegacyGraph"),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    TopBarModeText,
                ));

                bar.spawn((
                    Text::new("Active: None"),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.85, 0.9)),
                    TopBarEntityText,
                ));

                bar.spawn((
                    Text::new("Dirty: no"),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.9, 0.5)),
                    TopBarDirtyText,
                ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(44.0),
                    width: Val::Px(320.0),
                    height: Val::Percent(100.0),
                    overflow: Overflow::scroll_y(),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.95)),
                InspectorPanel,
            ));
        });
}

pub(crate) fn set_game_editor_ui_visibility(
    mode: Res<EditorModeState>,
    mut roots: Query<&mut Visibility, With<GameEditorUiRoot>>,
) {
    let visible = mode.mode == EditorMode::ComponentMode;
    for mut visibility in roots.iter_mut() {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn spawn_top_bar_button(parent: &mut ChildSpawnerCommands, label: &str, action: TopBarAction) {
    parent
        .spawn((
            Button,
            Node {
                height: Val::Px(28.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
            BorderRadius::all(Val::Px(4.0)),
            TopBarActionButton { action },
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub(crate) fn handle_top_bar_buttons(
    mut top_bar_buttons: Query<
        (&Interaction, &TopBarActionButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut mode_requests: MessageWriter<SetEditorModeRequest>,
    mut save_requests: MessageWriter<SaveSceneRequest>,
    mut load_requests: MessageWriter<LoadSceneRequest>,
    mut save_to_path_requests: MessageWriter<SaveSceneToPathRequest>,
    mut select_entity_requests: MessageWriter<SelectActiveEntityRequest>,
    scene: Res<EditorScene>,
    mode: Res<EditorModeState>,
) {
    for (interaction, button) in top_bar_buttons.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.action {
            TopBarAction::ToggleMode => {
                let target = if mode.mode == EditorMode::LegacyGraph {
                    EditorMode::ComponentMode
                } else {
                    EditorMode::LegacyGraph
                };
                mode_requests.write(SetEditorModeRequest { mode: target });
            }
            TopBarAction::PrevEntity => {
                if let Some(entity_id) = cycle_entity_id(&scene, scene.active_entity_id, true) {
                    select_entity_requests.write(SelectActiveEntityRequest { entity_id });
                }
            }
            TopBarAction::NextEntity => {
                if let Some(entity_id) = cycle_entity_id(&scene, scene.active_entity_id, false) {
                    select_entity_requests.write(SelectActiveEntityRequest { entity_id });
                }
            }
            TopBarAction::Save => {
                save_requests.write(SaveSceneRequest);
            }
            TopBarAction::SaveAs => {
                let path = format!("assets/scenes/scene_{}.json", unix_timestamp_millis());
                save_to_path_requests.write(SaveSceneToPathRequest { path });
            }
            TopBarAction::Open => {
                load_requests.write(LoadSceneRequest);
            }
        }
    }
}

pub(crate) fn update_top_bar_labels(
    mode: Res<EditorModeState>,
    scene: Res<EditorScene>,
    runtime: Res<GameEditorRuntimeState>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<TopBarModeText>>,
        Query<&mut Text, With<TopBarEntityText>>,
        Query<&mut Text, With<TopBarDirtyText>>,
    )>,
) {
    if let Ok(mut text) = text_sets.p0().single_mut() {
        text.0 = format!("Mode: {:?}", mode.mode);
    }

    if let Ok(mut text) = text_sets.p1().single_mut() {
        text.0 = format!("Active: {}", scene.active_entity_name());
    }

    if let Ok(mut text) = text_sets.p2().single_mut() {
        text.0 = if runtime.dirty {
            "Dirty: yes".to_string()
        } else {
            "Dirty: no".to_string()
        };
    }
}

pub(crate) fn rebuild_inspector_panel(
    mut commands: Commands,
    panel: Query<Entity, With<InspectorPanel>>,
    children: Query<&Children>,
    scene: Res<EditorScene>,
    selected_component: Res<SelectedComponentContext>,
    diagnostics: Res<ComponentCompositionDiagnostics>,
    schemas: Res<ComponentSchemaRegistry>,
) {
    let Ok(panel_entity) = panel.single() else {
        return;
    };

    if let Ok(existing_children) = children.get(panel_entity) {
        for child in existing_children.iter() {
            commands.entity(child).despawn();
        }
    }
    commands.entity(panel_entity).with_children(|panel| {
        panel.spawn((
            Text::new("Component Inspector"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.95, 0.98)),
        ));

        panel.spawn((
            Text::new(format!("Entity: {}", scene.active_entity_name())),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.86)),
        ));

        if diagnostics.invalid_link_count > 0 || !diagnostics.orphan_component_ids.is_empty() {
            panel.spawn((
                Text::new(format!(
                    "Composition Warnings: invalid links={}, orphan components={}",
                    diagnostics.invalid_link_count,
                    diagnostics.orphan_component_ids.len()
                )),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.75, 0.35)),
            ));
        }

        panel.spawn((
            Text::new("Add Component"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.9, 1.0)),
        ));

        spawn_add_component_button(panel, COMPONENT_KIND_TRANSFORM, "+ Transform");
        spawn_add_component_button(panel, COMPONENT_KIND_SPRITE, "+ Sprite");
        spawn_add_component_button(panel, COMPONENT_KIND_CAMERA2D, "+ Camera2D");

        if let Some(active_entity) = scene.get_active_entity() {
            panel.spawn((
                Text::new("Components"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.7)),
            ));

            for component in &active_entity.components {
                panel
                    .spawn((
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|row| {
                        let mut label_tags = Vec::new();
                        if component.kind == COMPONENT_KIND_ENTITY_ROOT {
                            label_tags.push("root");
                        }
                        if diagnostics.orphan_component_ids.contains(&component.id) {
                            label_tags.push("orphan");
                        }
                        let tags_text = if label_tags.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", label_tags.join(","))
                        };

                        row.spawn((
                            Button,
                            Node {
                                padding: UiRect::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.26, 0.35)),
                            BorderRadius::all(Val::Px(3.0)),
                            InspectorSelectButton {
                                component_id: component.id,
                            },
                        ))
                        .with_children(|btn| {
                            let marker = if selected_component.component_id == Some(component.id) {
                                "*"
                            } else {
                                " "
                            };
                            btn.spawn((
                                Text::new(format!(
                                    "{} {} #{}{}",
                                    marker,
                                    component_display_name(&component.kind),
                                    component.id,
                                    tags_text
                                )),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                        if component.kind != COMPONENT_KIND_ENTITY_ROOT {
                            row.spawn((
                                Button,
                                Node {
                                    padding: UiRect::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.45, 0.15, 0.15)),
                                BorderRadius::all(Val::Px(3.0)),
                                InspectorRemoveButton {
                                    component_id: component.id,
                                },
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Delete"),
                                    TextFont {
                                        font_size: 10.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                        }
                    });
            }

            if let Some(component_id) = selected_component.component_id {
                if let Some(component) = active_entity.get_component(component_id) {
                    panel.spawn((
                        Text::new(format!("Selected: {}", component.display_name())),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.95, 0.7)),
                    ));

                    if let Some(schema) = schemas.get(&component.kind) {
                        for field in &schema.fields {
                            spawn_adjust_row(panel, component.id, field, component);
                        }
                    }
                }
            }
        }
    });
}

fn spawn_adjust_row(
    parent: &mut ChildSpawnerCommands,
    component_id: u64,
    field: &ComponentFieldSchema,
    component: &crate::scene::EditorComponentInstance,
) {
    parent
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(format!(
                    "{}: {}",
                    field.label,
                    field_value_text(component, &field.key)
                )),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
            ));

            row.spawn((
                Button,
                Node {
                    padding: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                BorderRadius::all(Val::Px(3.0)),
                InspectorAdjustButton {
                    component_id,
                    field_key: field.key.clone(),
                    delta: -field.step,
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("-"),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            row.spawn((
                Button,
                Node {
                    padding: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.35, 0.25)),
                BorderRadius::all(Val::Px(3.0)),
                InspectorAdjustButton {
                    component_id,
                    field_key: field.key.clone(),
                    delta: field.step,
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("+"),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

fn spawn_add_component_button(parent: &mut ChildSpawnerCommands, kind: &str, label: &str) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.18, 0.28, 0.2)),
            BorderRadius::all(Val::Px(3.0)),
            AddComponentButton {
                kind: kind.to_string(),
            },
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub(crate) fn handle_add_component_buttons(
    mut buttons: Query<(&Interaction, &AddComponentButton), (Changed<Interaction>, With<Button>)>,
    mut requests: MessageWriter<AddComponentNodeRequest>,
) {
    for (interaction, add_button) in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            requests.write(AddComponentNodeRequest {
                kind: add_button.kind.clone(),
            });
        }
    }
}

pub(crate) fn handle_inspector_select_buttons(
    mut buttons: Query<
        (&Interaction, &InspectorSelectButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut selected: ResMut<SelectedComponentContext>,
) {
    for (interaction, select_button) in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            selected.component_id = Some(select_button.component_id);
        }
    }
}

pub(crate) fn handle_inspector_remove_buttons(
    mut buttons: Query<
        (&Interaction, &InspectorRemoveButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut requests: MessageWriter<RemoveComponentNodeRequest>,
) {
    for (interaction, remove_button) in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            requests.write(RemoveComponentNodeRequest {
                component_id: remove_button.component_id,
            });
        }
    }
}

pub(crate) fn handle_inspector_adjust_buttons(
    mut buttons: Query<
        (&Interaction, &InspectorAdjustButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut scene: ResMut<EditorScene>,
    schemas: Res<ComponentSchemaRegistry>,
) {
    let Some(active_entity) = scene.get_active_entity_mut() else {
        return;
    };

    for (interaction, adjust) in buttons.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(component) = active_entity.get_component_mut(adjust.component_id) {
            schemas.apply_field_delta(component, &adjust.field_key, adjust.delta);
        }
    }
}

fn field_value_text(component: &crate::scene::EditorComponentInstance, field_key: &str) -> String {
    match &component.payload {
        EditorComponentPayload::EntityRoot(_) => "root".to_string(),
        EditorComponentPayload::Transform(value) => match field_key {
            "x" => format!("{:.2}", value.x),
            "y" => format!("{:.2}", value.y),
            "rotation_deg" => format!("{:.2}", value.rotation_deg),
            "scale_x" => format!("{:.2}", value.scale_x),
            "scale_y" => format!("{:.2}", value.scale_y),
            _ => "-".to_string(),
        },
        EditorComponentPayload::Sprite(value) => match field_key {
            "color_r" => format!("{:.2}", value.color_r),
            "color_g" => format!("{:.2}", value.color_g),
            "color_b" => format!("{:.2}", value.color_b),
            "color_a" => format!("{:.2}", value.color_a),
            _ => "-".to_string(),
        },
        EditorComponentPayload::Camera2D(value) => match field_key {
            "zoom" => format!("{:.2}", value.zoom),
            _ => "-".to_string(),
        },
        EditorComponentPayload::Unknown(_) => "unknown".to_string(),
    }
}

fn cycle_entity_id(scene: &EditorScene, current: Option<u64>, previous: bool) -> Option<u64> {
    let ids = scene.sorted_entity_ids();
    if ids.is_empty() {
        return None;
    }

    let current_index = current
        .and_then(|id| ids.iter().position(|entry| *entry == id))
        .unwrap_or(0);

    let next_index = if previous {
        if current_index == 0 {
            ids.len() - 1
        } else {
            current_index - 1
        }
    } else {
        (current_index + 1) % ids.len()
    };

    ids.get(next_index).copied()
}

fn unix_timestamp_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
