use crate::components::{COMPONENT_KIND_ENTITY_ROOT, component_display_name};
use crate::context::*;
use crate::persistence::*;
use crate::scene::{EditorScene, SavedComponentLink, SavedGraphLayoutNode};
use crate::schemas::ComponentSchemaRegistry;
use crate::ui::*;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use univis_editor_core::prelude::*;
use univis_editor_ui::node_spawn::{
    spawn_component_mode_node_entity, spawn_missing_component_mode_node_entity,
};

#[derive(Component)]
struct LegacyHiddenInComponentMode;

#[derive(Resource, Default)]
struct LegacyModeStorage {
    was_hidden: bool,
    backed_up_connections: Vec<GraphLink>,
}

pub struct GameEditorPlugin;

impl Plugin for GameEditorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<crate::components::EEntityRoot>()
            .register_type::<crate::components::ETransform>()
            .register_type::<crate::components::ESprite>()
            .register_type::<crate::components::ECamera2D>()
            .init_resource::<EditorModeState>()
            .init_resource::<ActiveEntityContext>()
            .init_resource::<SelectedComponentContext>()
            .init_resource::<GameEditorRuntimeState>()
            .init_resource::<ComponentCompositionDiagnostics>()
            .init_resource::<EditorScene>()
            .init_resource::<ComponentSchemaRegistry>()
            .init_resource::<ScenePersistenceSettings>()
            .init_resource::<GameEditorStatusState>()
            .init_resource::<LegacyModeStorage>()
            .add_message::<SelectActiveEntityRequest>()
            .add_message::<AddComponentNodeRequest>()
            .add_message::<RemoveComponentNodeRequest>()
            .add_message::<SaveSceneRequest>()
            .add_message::<LoadSceneRequest>()
            .add_message::<SaveSceneToPathRequest>()
            .add_message::<LoadSceneFromPathRequest>()
            .add_message::<SetEditorModeRequest>()
            .add_systems(
                Startup,
                (
                    configure_component_schema_registry,
                    initialize_editor_scene_state,
                    setup_game_editor_ui,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                (
                    game_editor_shortcuts,
                    handle_save_scene_requests,
                    handle_load_scene_requests,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    set_game_editor_ui_visibility,
                    handle_top_bar_buttons,
                    handle_add_component_buttons,
                    handle_inspector_select_buttons,
                    handle_inspector_remove_buttons,
                    handle_inspector_adjust_buttons,
                    handle_set_editor_mode_requests,
                    handle_select_active_entity_requests,
                    handle_add_component_node_requests,
                    handle_remove_component_node_requests,
                    handle_component_mode_delete_shortcut,
                    rebuild_component_mode_canvas,
                    update_top_bar_labels,
                    rebuild_inspector_panel,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    sync_active_entity_context,
                    rebuild_canvas_links_from_scene,
                    sync_component_layout_from_canvas,
                    sync_component_links_from_canvas,
                    refresh_scene_dirty_state,
                    autosave_dirty_scene,
                    draw_game_editor_status_ui,
                )
                    .chain(),
            );
    }
}

fn configure_component_schema_registry(mut schemas: ResMut<ComponentSchemaRegistry>) {
    schemas.register_defaults();
}

fn initialize_editor_scene_state(
    mut scene: ResMut<EditorScene>,
    mut active_context: ResMut<ActiveEntityContext>,
    mut runtime: ResMut<GameEditorRuntimeState>,
    mut diagnostics: ResMut<ComponentCompositionDiagnostics>,
) {
    scene.ensure_active_entity();
    if let Some(active_entity_id) = scene.active_entity_id {
        let _ = scene.ensure_entity_root_component(active_entity_id);
    }

    active_context.active_entity_id = scene.active_entity_id;
    runtime.initialized = false;
    runtime.needs_rebaseline = true;
    runtime.canvas_needs_rebuild = true;
    runtime.links_need_rebuild = true;
    *diagnostics = ComponentCompositionDiagnostics::default();
}

fn sync_active_entity_context(
    scene: Res<EditorScene>,
    mut active_context: ResMut<ActiveEntityContext>,
) {
    if active_context.active_entity_id != scene.active_entity_id {
        active_context.active_entity_id = scene.active_entity_id;
    }
}

fn handle_set_editor_mode_requests(
    mut commands: Commands,
    mut requests: MessageReader<SetEditorModeRequest>,
    mut mode: ResMut<EditorModeState>,
    mut scene: ResMut<EditorScene>,
    mut selected_component: ResMut<SelectedComponentContext>,
    mut runtime: ResMut<GameEditorRuntimeState>,
    mut diagnostics: ResMut<ComponentCompositionDiagnostics>,
    mut graph: ResMut<Connecting>,
    mut storage: ResMut<LegacyModeStorage>,
    legacy_nodes: Query<Entity, (With<GraphNode>, Without<ComponentModeNode>)>,
    hidden_legacy_nodes: Query<Entity, With<LegacyHiddenInComponentMode>>,
    component_mode_nodes: Query<Entity, With<ComponentModeNode>>,
) {
    let mut target_mode = None;
    for request in requests.read() {
        target_mode = Some(request.mode);
    }

    let Some(target_mode) = target_mode else {
        return;
    };

    if target_mode == mode.mode {
        return;
    }

    match target_mode {
        EditorMode::ComponentMode => {
            if !storage.was_hidden {
                storage.backed_up_connections = graph.connections.clone();
                storage.was_hidden = true;
            }

            graph.connections.clear();

            for entity in legacy_nodes.iter() {
                commands
                    .entity(entity)
                    .insert((Visibility::Hidden, LegacyHiddenInComponentMode));
                commands.entity(entity).remove::<Selected>();
            }

            scene.ensure_active_entity();
            if let Some(active_entity_id) = scene.active_entity_id {
                let _ = scene.ensure_entity_root_component(active_entity_id);
            }

            mode.mode = EditorMode::ComponentMode;
            selected_component.component_id = None;
            runtime.open_confirm_until_secs = None;
            runtime.canvas_needs_rebuild = true;
            runtime.links_need_rebuild = true;
            *diagnostics = ComponentCompositionDiagnostics::default();
        }
        EditorMode::LegacyGraph => {
            for entity in component_mode_nodes.iter() {
                commands.entity(entity).despawn();
            }

            graph.connections.clear();
            if storage.was_hidden {
                graph.connections = storage.backed_up_connections.clone();
            }

            for entity in hidden_legacy_nodes.iter() {
                commands
                    .entity(entity)
                    .insert(Visibility::Visible)
                    .remove::<LegacyHiddenInComponentMode>();
            }

            storage.backed_up_connections.clear();
            storage.was_hidden = false;
            selected_component.component_id = None;
            runtime.open_confirm_until_secs = None;
            runtime.canvas_needs_rebuild = false;
            runtime.links_need_rebuild = false;
            mode.mode = EditorMode::LegacyGraph;
            *diagnostics = ComponentCompositionDiagnostics::default();
        }
    }
}

fn handle_select_active_entity_requests(
    mut requests: MessageReader<SelectActiveEntityRequest>,
    mut scene: ResMut<EditorScene>,
    mode: Res<EditorModeState>,
    mut selected_component: ResMut<SelectedComponentContext>,
    mut runtime: ResMut<GameEditorRuntimeState>,
) {
    if mode.mode != EditorMode::ComponentMode {
        return;
    }

    let mut changed = false;
    for request in requests.read() {
        let before = scene.active_entity_id;
        scene.set_active_entity(request.entity_id);
        changed |= before != scene.active_entity_id;
    }

    if changed {
        if let Some(active_entity_id) = scene.active_entity_id {
            let _ = scene.ensure_entity_root_component(active_entity_id);
        }
        selected_component.component_id = None;
        runtime.canvas_needs_rebuild = true;
        runtime.links_need_rebuild = true;
    }
}

fn handle_add_component_node_requests(
    mut requests: MessageReader<AddComponentNodeRequest>,
    mut scene: ResMut<EditorScene>,
    mode: Res<EditorModeState>,
    schemas: Res<ComponentSchemaRegistry>,
    mut selected_component: ResMut<SelectedComponentContext>,
    mut runtime: ResMut<GameEditorRuntimeState>,
) {
    if mode.mode != EditorMode::ComponentMode {
        return;
    }

    scene.ensure_active_entity();
    let Some(active_entity_id) = scene.active_entity_id else {
        return;
    };

    let Some(root_component_id) = scene.ensure_entity_root_component(active_entity_id) else {
        return;
    };

    for request in requests.read() {
        if request.kind == COMPONENT_KIND_ENTITY_ROOT {
            continue;
        }

        let Some(component) = schemas.create_default_component(&mut scene, &request.kind) else {
            warn!("Unknown component kind requested: {}", request.kind);
            continue;
        };

        let component_id = component.id;
        let Some(active_entity) = scene.get_active_entity_mut() else {
            continue;
        };

        for layout in &mut active_entity.graph_layout {
            layout.selected = false;
        }

        let index = active_entity.components.len() as f32;
        active_entity.components.push(component);
        active_entity.graph_layout.push(SavedGraphLayoutNode {
            component_id,
            position: [-260.0 + index * 42.0, 150.0 - index * 34.0],
            selected: true,
        });

        active_entity
            .composition_links
            .retain(|link| link.child_component_id != component_id);
        active_entity.composition_links.push(SavedComponentLink {
            child_component_id: component_id,
            parent_component_id: root_component_id,
        });

        selected_component.component_id = Some(component_id);
        runtime.canvas_needs_rebuild = true;
        runtime.links_need_rebuild = true;
    }
}

fn handle_remove_component_node_requests(
    mut requests: MessageReader<RemoveComponentNodeRequest>,
    mut scene: ResMut<EditorScene>,
    mode: Res<EditorModeState>,
    schemas: Res<ComponentSchemaRegistry>,
    mut selected_component: ResMut<SelectedComponentContext>,
    mut runtime: ResMut<GameEditorRuntimeState>,
) {
    if mode.mode != EditorMode::ComponentMode {
        return;
    }

    let mut removed_any = false;
    for request in requests.read() {
        let Some(active_entity) = scene.get_active_entity_mut() else {
            continue;
        };

        let is_root = active_entity
            .components
            .iter()
            .find(|component| component.id == request.component_id)
            .map(|component| {
                component.kind == COMPONENT_KIND_ENTITY_ROOT
                    || !schemas.is_deletable(&component.kind)
            })
            .unwrap_or(false);
        if is_root {
            continue;
        }

        let before = active_entity.components.len();
        active_entity
            .components
            .retain(|component| component.id != request.component_id);
        active_entity
            .graph_layout
            .retain(|layout| layout.component_id != request.component_id);
        active_entity.composition_links.retain(|link| {
            link.child_component_id != request.component_id
                && link.parent_component_id != request.component_id
        });

        if active_entity.components.len() != before {
            removed_any = true;
            if selected_component.component_id == Some(request.component_id) {
                selected_component.component_id = None;
            }
        }
    }

    if removed_any {
        runtime.canvas_needs_rebuild = true;
        runtime.links_need_rebuild = true;
    }
}

fn handle_component_mode_delete_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mode: Res<EditorModeState>,
    selected_component: Res<SelectedComponentContext>,
    mut remove_writer: MessageWriter<RemoveComponentNodeRequest>,
) {
    if mode.mode != EditorMode::ComponentMode {
        return;
    }

    if !(keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace)) {
        return;
    }

    if let Some(component_id) = selected_component.component_id {
        remove_writer.write(RemoveComponentNodeRequest { component_id });
    }
}

fn rebuild_component_mode_canvas(
    mut commands: Commands,
    mut scene: ResMut<EditorScene>,
    mode: Res<EditorModeState>,
    schemas: Res<ComponentSchemaRegistry>,
    mut runtime: ResMut<GameEditorRuntimeState>,
    selected_component: Res<SelectedComponentContext>,
    mut graph: ResMut<Connecting>,
    existing_component_nodes: Query<Entity, With<ComponentModeNode>>,
) {
    if mode.mode != EditorMode::ComponentMode {
        return;
    }

    if !runtime.canvas_needs_rebuild {
        if existing_component_nodes.is_empty()
            && scene
                .get_active_entity()
                .map(|entity| !entity.components.is_empty())
                .unwrap_or(false)
        {
            runtime.canvas_needs_rebuild = true;
        } else {
            return;
        }
    }

    for entity in existing_component_nodes.iter() {
        commands.entity(entity).despawn();
    }
    graph.connections.clear();

    scene.ensure_active_entity();
    let Some(active_entity_id) = scene.active_entity_id else {
        runtime.canvas_needs_rebuild = false;
        runtime.links_need_rebuild = false;
        return;
    };

    let _ = scene.ensure_entity_root_component(active_entity_id);

    let Some(active_index) = scene
        .entities
        .iter()
        .position(|entity| entity.id == active_entity_id)
    else {
        runtime.canvas_needs_rebuild = false;
        runtime.links_need_rebuild = false;
        return;
    };

    let component_ids: HashSet<u64> = scene.entities[active_index]
        .components
        .iter()
        .map(|component| component.id)
        .collect();

    scene.entities[active_index]
        .graph_layout
        .retain(|layout| component_ids.contains(&layout.component_id));
    scene.entities[active_index]
        .composition_links
        .retain(|link| {
            component_ids.contains(&link.child_component_id)
                && component_ids.contains(&link.parent_component_id)
                && link.child_component_id != link.parent_component_id
        });

    let components_snapshot = scene.entities[active_index].components.clone();
    for (idx, component) in components_snapshot.iter().enumerate() {
        if scene.entities[active_index]
            .graph_layout
            .iter()
            .all(|layout| layout.component_id != component.id)
        {
            let index = idx as f32;
            scene.entities[active_index]
                .graph_layout
                .push(SavedGraphLayoutNode {
                    component_id: component.id,
                    position: [-260.0 + index * 42.0, 150.0 - index * 34.0],
                    selected: false,
                });
        }
    }

    let components = scene.entities[active_index].components.clone();
    let graph_layout = scene.entities[active_index].graph_layout.clone();

    for component in &components {
        let Some(layout) = graph_layout
            .iter()
            .find(|layout| layout.component_id == component.id)
        else {
            continue;
        };

        let position = Vec2::new(layout.position[0], layout.position[1]);
        let accepts_children = component.is_unknown() || schemas.can_have_children(&component.kind);
        let can_be_child = component.is_unknown() || schemas.can_be_child(&component.kind);

        let entity = if component.is_unknown() {
            spawn_missing_component_mode_node_entity(&mut commands, &component.kind, position)
        } else {
            spawn_component_mode_node_entity(
                &mut commands,
                &component.kind,
                &format!("{} #{}", component.display_name(), component.id),
                position,
                false,
                accepts_children,
                can_be_child,
            )
        };

        commands.entity(entity).insert(ComponentModeNode {
            editor_entity_id: active_entity_id,
            component_id: component.id,
        });

        if selected_component.component_id == Some(component.id) || layout.selected {
            commands.entity(entity).insert(Selected);
        }
    }

    runtime.canvas_needs_rebuild = false;
    runtime.links_need_rebuild = true;
}

fn rebuild_canvas_links_from_scene(
    mode: Res<EditorModeState>,
    scene: Res<EditorScene>,
    mut runtime: ResMut<GameEditorRuntimeState>,
    mut diagnostics: ResMut<ComponentCompositionDiagnostics>,
    mut graph: ResMut<Connecting>,
    component_nodes: Query<(Entity, &ComponentModeNode)>,
    ports: Query<(Entity, &GraphPort)>,
) {
    if mode.mode != EditorMode::ComponentMode || !runtime.links_need_rebuild {
        return;
    }

    let Some(active_entity_id) = scene.active_entity_id else {
        graph.connections.clear();
        runtime.links_need_rebuild = false;
        return;
    };

    let Some(active_entity) = scene.get_active_entity() else {
        graph.connections.clear();
        runtime.links_need_rebuild = false;
        return;
    };

    let mut component_to_node: HashMap<u64, Entity> = HashMap::new();
    let mut node_to_component: HashMap<Entity, u64> = HashMap::new();
    for (node_entity, marker) in component_nodes.iter() {
        if marker.editor_entity_id != active_entity_id {
            continue;
        }
        component_to_node.insert(marker.component_id, node_entity);
        node_to_component.insert(node_entity, marker.component_id);
    }

    let mut input_ports: HashMap<u64, Entity> = HashMap::new();
    let mut output_ports: HashMap<u64, Entity> = HashMap::new();
    for (port_entity, port) in ports.iter() {
        let Some(component_id) = node_to_component.get(&port.node_entity).copied() else {
            continue;
        };

        match port.port_type {
            PortType::Input if port.index == 0 => {
                input_ports.insert(component_id, port_entity);
            }
            PortType::Output if port.index == 0 => {
                output_ports.insert(component_id, port_entity);
            }
            _ => {}
        }
    }

    let mut skipped = 0usize;
    let mut rebuilt_links = Vec::new();
    for link in &active_entity.composition_links {
        let Some(from_node) = component_to_node.get(&link.child_component_id).copied() else {
            skipped += 1;
            continue;
        };
        let Some(to_node) = component_to_node.get(&link.parent_component_id).copied() else {
            skipped += 1;
            continue;
        };
        let Some(from_port) = output_ports.get(&link.child_component_id).copied() else {
            skipped += 1;
            continue;
        };
        let Some(to_port) = input_ports.get(&link.parent_component_id).copied() else {
            skipped += 1;
            continue;
        };

        rebuilt_links.push(GraphLink {
            from_node,
            from_index: 0,
            to_node,
            to_index: 0,
            from_port,
            to_port,
        });
    }

    graph.connections = rebuilt_links;
    runtime.links_need_rebuild = false;

    diagnostics.invalid_link_count = skipped;
    if skipped > 0 {
        diagnostics.warnings.push(format!(
            "{} composition link(s) skipped while rebuilding canvas.",
            skipped
        ));
        diagnostics.warnings.truncate(3);
    }
}

fn sync_component_layout_from_canvas(
    mode: Res<EditorModeState>,
    mut scene: ResMut<EditorScene>,
    mut selected_component: ResMut<SelectedComponentContext>,
    nodes: Query<(&ComponentModeNode, &Transform, Option<&Selected>)>,
) {
    if mode.mode != EditorMode::ComponentMode {
        return;
    }

    let Some(active_entity_id) = scene.active_entity_id else {
        selected_component.component_id = None;
        return;
    };

    let Some(active_entity) = scene.get_active_entity_mut() else {
        selected_component.component_id = None;
        return;
    };

    let mut selected_id_from_canvas = None;

    for (marker, transform, selected) in nodes.iter() {
        if marker.editor_entity_id != active_entity_id {
            continue;
        }

        if selected.is_some() && selected_id_from_canvas.is_none() {
            selected_id_from_canvas = Some(marker.component_id);
        }

        if let Some(layout) = active_entity
            .graph_layout
            .iter_mut()
            .find(|layout| layout.component_id == marker.component_id)
        {
            layout.position = [transform.translation.x, transform.translation.y];
            layout.selected = selected.is_some();
        } else {
            active_entity.graph_layout.push(SavedGraphLayoutNode {
                component_id: marker.component_id,
                position: [transform.translation.x, transform.translation.y],
                selected: selected.is_some(),
            });
        }
    }

    if selected_component.component_id != selected_id_from_canvas {
        selected_component.component_id = selected_id_from_canvas;
    }

    if let Some(component_id) = selected_component.component_id {
        if active_entity
            .components
            .iter()
            .all(|component| component.id != component_id)
        {
            selected_component.component_id = None;
        }
    }
}

fn sync_component_links_from_canvas(
    mode: Res<EditorModeState>,
    mut scene: ResMut<EditorScene>,
    schemas: Res<ComponentSchemaRegistry>,
    mut graph: ResMut<Connecting>,
    mut diagnostics: ResMut<ComponentCompositionDiagnostics>,
    component_nodes: Query<(Entity, &ComponentModeNode)>,
    ports: Query<(Entity, &GraphPort)>,
) {
    if mode.mode != EditorMode::ComponentMode {
        return;
    }

    let Some(active_entity_id) = scene.active_entity_id else {
        diagnostics.orphan_component_ids.clear();
        diagnostics.invalid_link_count = 0;
        diagnostics.warnings.clear();
        return;
    };

    let Some(active_entity) = scene.get_active_entity_mut() else {
        diagnostics.orphan_component_ids.clear();
        diagnostics.invalid_link_count = 0;
        diagnostics.warnings.clear();
        return;
    };

    let Some(root_component_id) = active_entity.root_component_id() else {
        diagnostics.orphan_component_ids.clear();
        diagnostics.invalid_link_count = 0;
        diagnostics.warnings =
            vec!["Missing root component; composition validation skipped.".to_string()];
        return;
    };

    let mut node_to_component: HashMap<Entity, u64> = HashMap::new();
    let mut component_to_node: HashMap<u64, Entity> = HashMap::new();
    for (node_entity, marker) in component_nodes.iter() {
        if marker.editor_entity_id != active_entity_id {
            continue;
        }
        node_to_component.insert(node_entity, marker.component_id);
        component_to_node.insert(marker.component_id, node_entity);
    }

    let mut input_ports: HashMap<u64, Entity> = HashMap::new();
    let mut output_ports: HashMap<u64, Entity> = HashMap::new();
    for (port_entity, port) in ports.iter() {
        let Some(component_id) = node_to_component.get(&port.node_entity).copied() else {
            continue;
        };

        match port.port_type {
            PortType::Input if port.index == 0 => {
                input_ports.insert(component_id, port_entity);
            }
            PortType::Output if port.index == 0 => {
                output_ports.insert(component_id, port_entity);
            }
            _ => {}
        }
    }

    let component_kind_by_id: HashMap<u64, String> = active_entity
        .components
        .iter()
        .map(|component| (component.id, component.kind.clone()))
        .collect();

    let existing_links = graph.connections.clone();
    let mut parent_by_child: HashMap<u64, (u64, usize)> = HashMap::new();
    let mut invalid_reasons = Vec::new();

    for (order, link) in existing_links.iter().enumerate() {
        let Some(child_component_id) = node_to_component.get(&link.from_node).copied() else {
            invalid_reasons.push("Link source is outside active entity.".to_string());
            continue;
        };
        let Some(parent_component_id) = node_to_component.get(&link.to_node).copied() else {
            invalid_reasons.push("Link target is outside active entity.".to_string());
            continue;
        };

        if child_component_id == root_component_id {
            invalid_reasons.push("Root component cannot be attached as child.".to_string());
            continue;
        }

        if child_component_id == parent_component_id {
            invalid_reasons.push("Self-link is not allowed.".to_string());
            continue;
        }

        let Some(child_kind) = component_kind_by_id.get(&child_component_id) else {
            invalid_reasons.push("Child component not found in entity.".to_string());
            continue;
        };
        let Some(parent_kind) = component_kind_by_id.get(&parent_component_id) else {
            invalid_reasons.push("Parent component not found in entity.".to_string());
            continue;
        };

        if !schemas.can_have_children(parent_kind) {
            invalid_reasons.push(format!(
                "Component '{}' cannot accept children.",
                component_display_name(parent_kind)
            ));
            continue;
        }

        if !schemas.can_attach_to_parent(child_kind, parent_kind) {
            invalid_reasons.push(format!(
                "Cannot attach '{}' to '{}'.",
                component_display_name(child_kind),
                component_display_name(parent_kind)
            ));
            continue;
        }

        let previous = parent_by_child.remove(&child_component_id);
        if creates_cycle(child_component_id, parent_component_id, &parent_by_child) {
            if let Some(previous_entry) = previous {
                parent_by_child.insert(child_component_id, previous_entry);
            }
            invalid_reasons.push("Link creates a cycle and was rejected.".to_string());
            continue;
        }

        parent_by_child.insert(child_component_id, (parent_component_id, order));
    }

    let mut ordered_links: Vec<(u64, u64, usize)> = parent_by_child
        .iter()
        .map(|(child, (parent, order))| (*child, *parent, *order))
        .collect();
    ordered_links.sort_by_key(|(_, _, order)| *order);

    active_entity.composition_links = ordered_links
        .iter()
        .map(
            |(child_component_id, parent_component_id, _)| SavedComponentLink {
                child_component_id: *child_component_id,
                parent_component_id: *parent_component_id,
            },
        )
        .collect();

    let mut normalized_graph_links = Vec::new();
    for (child_component_id, parent_component_id, _) in ordered_links {
        let Some(from_node) = component_to_node.get(&child_component_id).copied() else {
            invalid_reasons.push("Missing child node while normalizing links.".to_string());
            continue;
        };
        let Some(to_node) = component_to_node.get(&parent_component_id).copied() else {
            invalid_reasons.push("Missing parent node while normalizing links.".to_string());
            continue;
        };
        let Some(from_port) = output_ports.get(&child_component_id).copied() else {
            invalid_reasons.push("Missing child output port while normalizing links.".to_string());
            continue;
        };
        let Some(to_port) = input_ports.get(&parent_component_id).copied() else {
            invalid_reasons.push("Missing parent input port while normalizing links.".to_string());
            continue;
        };

        normalized_graph_links.push(GraphLink {
            from_node,
            from_index: 0,
            to_node,
            to_index: 0,
            from_port,
            to_port,
        });
    }

    graph.connections = normalized_graph_links;

    let parent_map: HashMap<u64, u64> = active_entity
        .composition_links
        .iter()
        .map(|link| (link.child_component_id, link.parent_component_id))
        .collect();

    let mut orphan_component_ids = HashSet::new();
    for component in &active_entity.components {
        if component.id == root_component_id {
            continue;
        }

        if !reaches_root(component.id, root_component_id, &parent_map) {
            orphan_component_ids.insert(component.id);
        }
    }

    diagnostics.orphan_component_ids = orphan_component_ids;
    diagnostics.invalid_link_count = invalid_reasons.len();
    diagnostics.warnings = invalid_reasons.into_iter().take(4).collect();
}

fn creates_cycle(
    child_component_id: u64,
    parent_component_id: u64,
    parent_by_child: &HashMap<u64, (u64, usize)>,
) -> bool {
    let mut cursor = parent_component_id;
    let mut visited = HashSet::new();

    while let Some((next_parent, _)) = parent_by_child.get(&cursor) {
        if *next_parent == child_component_id {
            return true;
        }

        if !visited.insert(*next_parent) {
            return true;
        }

        cursor = *next_parent;
    }

    false
}

fn reaches_root(component_id: u64, root_component_id: u64, parent_map: &HashMap<u64, u64>) -> bool {
    let mut cursor = component_id;
    let mut visited = HashSet::new();

    while let Some(parent_id) = parent_map.get(&cursor).copied() {
        if parent_id == root_component_id {
            return true;
        }

        if !visited.insert(parent_id) {
            return false;
        }

        cursor = parent_id;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_cycle_when_parent_chain_returns_to_child() {
        let mut map = HashMap::new();
        map.insert(2, (1, 0));
        map.insert(3, (2, 1));

        assert!(creates_cycle(1, 3, &map));
        assert!(!creates_cycle(4, 3, &map));
    }

    #[test]
    fn reaches_root_checks_parent_chain() {
        let mut parent_map = HashMap::new();
        parent_map.insert(11, 10);
        parent_map.insert(12, 11);

        assert!(reaches_root(12, 10, &parent_map));
        assert!(!reaches_root(13, 10, &parent_map));
    }
}
