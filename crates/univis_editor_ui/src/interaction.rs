//! Graph interaction systems.
use crate::prelude::*;
use bevy::ecs::relationship::Relationship;
use bevy::ui::UiTargetCamera;
use bevy::{input::mouse::MouseWheel, platform::collections::HashSet, prelude::*};
use univis_ui::prelude::*;

fn find_graph_node_ancestor(
    start_entity: Entity,
    parents: &Query<&ChildOf>,
    node_markers: &Query<(), With<GraphNode>>,
) -> Option<Entity> {
    let mut current = start_entity;

    loop {
        if node_markers.get(current).is_ok() {
            return Some(current);
        }

        let Ok(parent) = parents.get(current) else {
            return None;
        };
        current = parent.get();
    }
}

fn interaction_is_pointer_active(interaction: &UInteraction) -> bool {
    matches!(
        *interaction,
        UInteraction::Pressed | UInteraction::Hovered | UInteraction::Clicked
    )
}

fn pointer_target_node(
    nodes_interaction: &Query<(Entity, &UInteraction), With<GraphNode>>,
    headers_interaction: &Query<(Entity, &UInteraction), With<Header>>,
    ports_interaction: &Query<(&UInteraction, &GraphPort)>,
    parents: &Query<&ChildOf>,
    node_markers: &Query<(), With<GraphNode>>,
) -> Option<Entity> {
    nodes_interaction
        .iter()
        .find_map(|(entity, interaction)| {
            interaction_is_pointer_active(interaction).then_some(entity)
        })
        .or_else(|| {
            headers_interaction
                .iter()
                .find_map(|(header_entity, interaction)| {
                    if !interaction_is_pointer_active(interaction) {
                        return None;
                    }

                    find_graph_node_ancestor(header_entity, parents, node_markers)
                })
        })
        .or_else(|| {
            ports_interaction.iter().find_map(|(interaction, port)| {
                if !interaction_is_pointer_active(interaction) {
                    return None;
                }

                (node_markers.get(port.node_entity).is_ok()).then_some(port.node_entity)
            })
        })
}

fn selection_additive_modifier(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft)
        || keys.pressed(KeyCode::ShiftRight)
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct BoxSelectionState {
    pub active: bool,
    pub additive: bool,
    pub start_screen_pos: Vec2,
    pub current_screen_pos: Vec2,
    pub suppress_click_selection: bool,
}

impl BoxSelectionState {
    pub fn clear(&mut self) {
        self.active = false;
        self.additive = false;
        self.start_screen_pos = Vec2::ZERO;
        self.current_screen_pos = Vec2::ZERO;
        self.suppress_click_selection = false;
    }

    fn min_screen_pos(&self) -> Vec2 {
        Vec2::new(
            self.start_screen_pos.x.min(self.current_screen_pos.x),
            self.start_screen_pos.y.min(self.current_screen_pos.y),
        )
    }

    fn max_screen_pos(&self) -> Vec2 {
        Vec2::new(
            self.start_screen_pos.x.max(self.current_screen_pos.x),
            self.start_screen_pos.y.max(self.current_screen_pos.y),
        )
    }

    fn drag_distance_sq(&self) -> f32 {
        self.start_screen_pos
            .distance_squared(self.current_screen_pos)
    }
}

#[derive(Component)]
pub struct BoxSelectionOverlay;

pub fn sanitize_graph_editor_state(
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut connect: ResMut<Connecting>,
    mut drag_state: ResMut<DragState>,
    mut wire_state: ResMut<WireConnectionState>,
    mut popup: ResMut<NodePopupState>,
    mut overlay: ResMut<GraphOverlayState>,
    q_nodes: Query<(), With<GraphNode>>,
    q_ports: Query<&GraphPort>,
) {
    let node_exists = |entity: Entity| q_nodes.get(entity).is_ok();
    let port_matches = |port_entity: Entity,
                        expected_node: Entity,
                        expected_type: PortType,
                        expected_index: usize| {
        q_ports.get(port_entity).is_ok_and(|port| {
            port.node_entity == expected_node
                && port.port_type == expected_type
                && port.index == expected_index
        })
    };

    connect.connections.retain(|link| {
        node_exists(link.from_node)
            && node_exists(link.to_node)
            && port_matches(
                link.from_port,
                link.from_node,
                PortType::Output,
                link.from_index,
            )
            && port_matches(link.to_port, link.to_node, PortType::Input, link.to_index)
    });

    live_document.retain_existing_entities(|entity| q_nodes.get(entity).is_ok());

    if drag_state
        .active_entity
        .is_some_and(|entity| !node_exists(entity))
    {
        drag_state.clear();
    }

    if wire_state
        .node_from
        .is_some_and(|entity| !node_exists(entity))
        || wire_state
            .dragging_from
            .is_some_and(|entity| q_ports.get(entity).is_err())
    {
        wire_state.clear();
    }

    if popup.open_for.is_some_and(|entity| !node_exists(entity)) {
        popup.open_for = None;
    }

    if overlay.active_surface == GraphOverlaySurface::NodePopup && popup.open_for.is_none() {
        overlay.active_surface = GraphOverlaySurface::None;
    }
}

pub fn box_selection_input_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    overlay: Res<GraphOverlayState>,
    mut box_selection: ResMut<BoxSelectionState>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    nodes_interaction: Query<(Entity, &UInteraction), With<GraphNode>>,
    headers_interaction: Query<(Entity, &UInteraction), With<Header>>,
    ports_interaction: Query<(&UInteraction, &GraphPort)>,
    parents: Query<&ChildOf>,
    node_markers: Query<(), With<GraphNode>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    q_node_transforms: Query<(Entity, &GlobalTransform), With<GraphNode>>,
    selected_nodes: Query<Entity, With<Selected>>,
) {
    if overlay.active_surface != GraphOverlaySurface::None && !box_selection.active {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let cursor_pos = window.cursor_position();

    if mouse_button.just_pressed(MouseButton::Left)
        && overlay.active_surface == GraphOverlaySurface::None
    {
        let clicked_node = pointer_target_node(
            &nodes_interaction,
            &headers_interaction,
            &ports_interaction,
            &parents,
            &node_markers,
        );

        if clicked_node.is_none() {
            let Some(cursor_pos) = cursor_pos else {
                return;
            };

            box_selection.active = true;
            box_selection.additive = selection_additive_modifier(&keys);
            box_selection.start_screen_pos = cursor_pos;
            box_selection.current_screen_pos = cursor_pos;
            box_selection.suppress_click_selection = true;

            if !box_selection.additive {
                live_document.clear_selected_entities();
                for entity in selected_nodes.iter() {
                    commands.entity(entity).try_remove::<Selected>();
                }
            }
        }
    }

    if !box_selection.active {
        return;
    }

    if let Some(cursor_pos) = cursor_pos {
        box_selection.current_screen_pos = cursor_pos;
    }

    if !mouse_button.just_released(MouseButton::Left) {
        return;
    }

    if box_selection.drag_distance_sq() >= 16.0 {
        let Ok((camera, camera_transform)) = q_camera.single() else {
            box_selection.clear();
            return;
        };

        let min = box_selection.min_screen_pos();
        let max = box_selection.max_screen_pos();
        let mut next_selection: HashSet<Entity> = if box_selection.additive {
            selected_nodes.iter().collect()
        } else {
            HashSet::default()
        };

        for (entity, global_transform) in q_node_transforms.iter() {
            let Ok(screen_pos) =
                camera.world_to_viewport(camera_transform, global_transform.translation())
            else {
                continue;
            };
            if screen_pos.x >= min.x
                && screen_pos.x <= max.x
                && screen_pos.y >= min.y
                && screen_pos.y <= max.y
            {
                next_selection.insert(entity);
            }
        }

        for entity in selected_nodes.iter() {
            if !next_selection.contains(&entity) {
                commands.entity(entity).try_remove::<Selected>();
            }
        }
        for entity in &next_selection {
            commands.entity(*entity).try_insert(Selected);
        }
        live_document.set_selected_entities(next_selection.iter().copied());
    }

    box_selection.active = false;
    box_selection.additive = false;
    box_selection.start_screen_pos = Vec2::ZERO;
    box_selection.current_screen_pos = Vec2::ZERO;
    box_selection.suppress_click_selection = true;
}

pub fn sync_box_selection_overlay(
    mut commands: Commands,
    box_selection: Res<BoxSelectionState>,
    q_graph_camera: Query<Entity, With<GraphCamera>>,
    mut overlays: Query<(Entity, &mut Node, Option<&UiTargetCamera>), With<BoxSelectionOverlay>>,
) {
    if overlays.is_empty() {
        let mut overlay_commands = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(0.0),
                height: Val::Px(0.0),
                border: UiRect::all(Val::Px(1.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.32, 0.64, 1.0, 0.12)),
            BorderColor::all(Color::srgba(0.55, 0.8, 1.0, 0.95)),
            ZIndex(1500),
            BoxSelectionOverlay,
        ));

        if let Some(graph_camera) = q_graph_camera.iter().next() {
            overlay_commands.insert(UiTargetCamera(graph_camera));
        }
    }

    let Some(graph_camera) = q_graph_camera.iter().next() else {
        return;
    };

    for (entity, mut node, target_camera) in overlays.iter_mut() {
        if target_camera.map(|target| target.entity()) != Some(graph_camera) {
            commands
                .entity(entity)
                .try_insert(UiTargetCamera(graph_camera));
        }

        if box_selection.active && box_selection.drag_distance_sq() >= 4.0 {
            let min = box_selection.min_screen_pos();
            let max = box_selection.max_screen_pos();
            node.display = Display::Flex;
            node.left = Val::Px(min.x);
            node.top = Val::Px(min.y);
            node.width = Val::Px((max.x - min.x).max(1.0));
            node.height = Val::Px((max.y - min.y).max(1.0));
        } else {
            node.display = Display::None;
        }
    }
}

/// Selects the node under the pointer and mirrors that selection into the live document.
pub fn selection_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut box_selection: ResMut<BoxSelectionState>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    nodes_interaction: Query<(Entity, &UInteraction), With<GraphNode>>,
    headers_interaction: Query<(Entity, &UInteraction), With<Header>>,
    ports_interaction: Query<(&UInteraction, &GraphPort)>,
    parents: Query<&ChildOf>,
    node_markers: Query<(), With<GraphNode>>,
    selected_nodes: Query<Entity, With<Selected>>,
) {
    if box_selection.suppress_click_selection {
        box_selection.suppress_click_selection = false;
        return;
    }

    if mouse_button.just_pressed(MouseButton::Left) {
        let clicked_node = pointer_target_node(
            &nodes_interaction,
            &headers_interaction,
            &ports_interaction,
            &parents,
            &node_markers,
        );
        let additive = selection_additive_modifier(&keys);

        if let Some(target_entity) = clicked_node {
            if additive {
                let mut next_selection: HashSet<Entity> = selected_nodes.iter().collect();
                if !next_selection.insert(target_entity) {
                    next_selection.remove(&target_entity);
                    commands.entity(target_entity).try_remove::<Selected>();
                } else {
                    commands.entity(target_entity).try_insert(Selected);
                }

                for entity in selected_nodes.iter() {
                    if !next_selection.contains(&entity) {
                        commands.entity(entity).try_remove::<Selected>();
                    }
                }

                live_document.set_selected_entities(next_selection.iter().copied());
            } else {
                live_document.select_single_entity(target_entity);
                for entity in selected_nodes.iter() {
                    if entity != target_entity {
                        commands.entity(entity).try_remove::<Selected>();
                    }
                }
                commands.entity(target_entity).try_insert(Selected);
            }
        } else if !additive {
            live_document.clear_selected_entities();
            for entity in selected_nodes.iter() {
                commands.entity(entity).try_remove::<Selected>();
            }
        }
    }
}

/// Styles node borders to reflect drag and selection state.
pub fn node_highlight_system(
    drag_state: Res<DragState>,
    mut nodes: Query<(Entity, &mut UBorder, Option<&Selected>), With<GraphNode>>,
) {
    for (entity, mut border, selected) in nodes.iter_mut() {
        if Some(entity) == drag_state.active_entity {
            border.color = Color::WHITE;
            border.width = 2.0;
        } else if selected.is_some() {
            border.color = Color::srgb(1.0, 0.5, 0.0);
            border.width = 2.0;
        } else {
            border.color = Color::NONE;
            border.width = 1.0;
        }
    }
}

/// Deletes the current selection and removes attached wires from the live graph.
pub fn delete_node_system(
    mut commands: Commands,
    activation: Option<Res<GraphEditingUiActivation>>,
    mut command_requests: MessageReader<GraphCommandRequest>,
    mut delete_requests: MessageReader<DeleteSelectedNodesRequest>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut connect: ResMut<Connecting>,
    mut drag_state: ResMut<DragState>,
    mut wire_state: ResMut<WireConnectionState>,
    mut popup: ResMut<NodePopupState>,
    mut overlay: ResMut<GraphOverlayState>,
    mut mutations: ResMut<GraphMutationTracker>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        command_requests.clear();
        delete_requests.clear();
        return;
    }

    let delete_requested = delete_requests.read().next().is_some()
        || command_requests
            .read()
            .any(|command| matches!(command, GraphCommandRequest::DeleteSelectedNodes));
    if !delete_requested {
        return;
    }

    let deleted_entities = live_document.delete_selected_entities();
    if deleted_entities.is_empty() {
        return;
    }

    let deleted_set: HashSet<Entity> = deleted_entities.iter().copied().collect();
    for entity in deleted_entities {
        commands.entity(entity).try_despawn();
    }

    connect.connections.retain(|link| {
        !deleted_set.contains(&link.from_node)
            && !deleted_set.contains(&link.to_node)
            && !deleted_set.contains(&link.from_port)
            && !deleted_set.contains(&link.to_port)
    });

    if drag_state
        .active_entity
        .is_some_and(|entity| deleted_set.contains(&entity))
    {
        drag_state.clear();
    }

    if wire_state
        .node_from
        .is_some_and(|entity| deleted_set.contains(&entity))
    {
        wire_state.clear();
    }

    if popup
        .open_for
        .is_some_and(|entity| deleted_set.contains(&entity))
    {
        popup.open_for = None;
        if overlay.active_surface == GraphOverlaySurface::NodePopup {
            overlay.active_surface = GraphOverlaySurface::None;
        }
    }

    mutations.mark_changed();
}

pub fn request_delete_selected_nodes(
    keys: Res<ButtonInput<KeyCode>>,
    activation: Option<Res<GraphEditingUiActivation>>,
    mut command_writer: MessageWriter<GraphCommandRequest>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        return;
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        command_writer.write(GraphCommandRequest::DeleteSelectedNodes);
    }
}

pub fn request_graph_workflow_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    overlay: Res<GraphOverlayState>,
    activation: Option<Res<GraphEditingUiActivation>>,
    mut command_writer: MessageWriter<GraphCommandRequest>,
) {
    if !graph_editing_enabled(activation.as_deref())
        || overlay.active_surface != GraphOverlaySurface::None
    {
        return;
    }

    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyD) {
        command_writer.write(GraphCommandRequest::DuplicateSelectedNodes);
    }

    if keys.just_pressed(KeyCode::KeyF) {
        command_writer.write(GraphCommandRequest::FrameSelectedNodes);
    }
}

fn graph_editing_enabled(activation: Option<&GraphEditingUiActivation>) -> bool {
    activation
        .map(|activation| activation.enabled)
        .unwrap_or(true)
}

/// Clears transient input values for ports that are no longer connected by a wire.
pub fn reset_inputs(mut q_nodes: Query<(Entity, &mut GraphNode)>, graph: Res<Connecting>) {
    let mut connected_pins: HashSet<(Entity, usize)> = HashSet::new();

    for link in &graph.connections {
        connected_pins.insert((link.to_node, link.to_index));
    }

    for (entity, mut node) in q_nodes.iter_mut() {
        for (index, input) in node.values.inputs.iter_mut().enumerate() {
            if !connected_pins.contains(&(entity, index)) {
                *input = NodeValue::None;
            }
        }
    }
}

/// Rebuilds the live graph document snapshot from the current editor world.
pub fn sync_live_graph_document_state(
    graph: Res<Connecting>,
    q_nodes: Query<(Entity, &GraphNode, &Transform, Option<&Selected>)>,
    q_camera: Query<(&Transform, &Projection), With<GraphCamera>>,
    mut live_document: ResMut<LiveGraphDocumentState>,
) {
    let mut nodes_data = Vec::new();
    for (entity, node, transform, selected) in q_nodes.iter() {
        nodes_data.push(GraphDocumentNodeSnapshot {
            entity,
            definition_id: node.definition_id.clone(),
            position: [transform.translation.x, transform.translation.y],
            inputs: node.values.inputs.clone(),
            input_count: node.values.inputs.len(),
            output_count: node.values.outputs.len(),
            selected: selected.is_some(),
        });
    }

    let camera = q_camera
        .iter()
        .next()
        .map(|(transform, projection)| GraphDocumentCameraState {
            translation: [
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ],
            ortho_scale: match projection {
                Projection::Orthographic(ortho) => ortho.scale,
                _ => 1.0,
            },
        });

    let edge_snapshots = graph
        .connections
        .iter()
        .map(|link| GraphDocumentEdgeSnapshot {
            from_entity: link.from_node,
            from_index: link.from_index,
            to_entity: link.to_node,
            to_index: link.to_index,
        });

    live_document.rebuild_from_snapshots(nodes_data, edge_snapshots, camera);
}

pub fn disconnect_wire_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut connect: ResMut<Connecting>,
    ports: Query<(Entity, &UInteraction, &GraphPort)>,
    mut mutations: ResMut<GraphMutationTracker>,
) {
    if mouse_button.just_pressed(MouseButton::Right) {
        for (port_entity, interaction, port_data) in ports.iter() {
            if interaction_is_pointer_active(interaction) && port_data.port_type == PortType::Input
            {
                if live_document.disconnect_input_for_entity(port_data.node_entity, port_data.index)
                {
                    connect
                        .connections
                        .retain(|link| link.to_port != port_entity);
                    mutations.mark_changed();
                }
            }
        }
    }
}

pub fn frame_selected_nodes_system(
    activation: Option<Res<GraphEditingUiActivation>>,
    mut command_requests: MessageReader<GraphCommandRequest>,
    live_document: Res<LiveGraphDocumentState>,
    windows: Query<&Window>,
    mut q_camera: Query<(&mut Transform, &mut Projection), With<GraphCamera>>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        command_requests.clear();
        return;
    }

    if !command_requests
        .read()
        .any(|command| matches!(command, GraphCommandRequest::FrameSelectedNodes))
    {
        return;
    }

    let selected_nodes = live_document
        .document
        .selected_node_ids()
        .iter()
        .filter_map(|node_id| live_document.document.node(*node_id))
        .collect::<Vec<_>>();
    if selected_nodes.is_empty() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut camera_transform, mut projection)) = q_camera.single_mut() else {
        return;
    };

    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for node in selected_nodes {
        let position = Vec2::new(node.position[0], node.position[1]);
        min = min.min(position);
        max = max.max(position);
    }

    let node_extent = Vec2::new(300.0, 160.0);
    min -= node_extent * 0.5;
    max += node_extent * 0.5;

    let center = (min + max) * 0.5;
    let framed_size = (max - min) + Vec2::new(220.0, 180.0);

    camera_transform.translation.x = center.x;
    camera_transform.translation.y = center.y;

    if let Projection::Orthographic(ref mut ortho) = *projection {
        let width_scale = framed_size.x / (window.width() * 0.72).max(1.0);
        let height_scale = framed_size.y / (window.height() * 0.72).max(1.0);
        ortho.scale = width_scale.max(height_scale).max(0.45).clamp(0.2, 5.0);
    }
}

pub fn node_drag_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    q_camera: Query<&Projection, With<GraphCamera>>,
    mut drag_state: ResMut<DragState>,
    headers: Query<(Entity, &UInteraction), With<Header>>,
    parents: Query<&ChildOf>,
    node_markers: Query<(), With<GraphNode>>,
    mut node_transforms: Query<&mut Transform, With<GraphNode>>,
    mut mutations: ResMut<GraphMutationTracker>,
) {
    let window = if let Ok(w) = windows.single() {
        w
    } else {
        return;
    };
    let current_mouse_pos = if let Some(pos) = window.cursor_position() {
        pos
    } else {
        return;
    };

    let zoom_factor = {
        let Ok(projection) = q_camera.single() else {
            return;
        };
        match projection {
            Projection::Orthographic(ortho) => ortho.scale,
            _ => 1.0,
        }
    };

    if mouse_button.just_pressed(MouseButton::Left) {
        let dragged_entity = headers.iter().find_map(|(header_entity, interaction)| {
            if !interaction_is_pointer_active(interaction) {
                return None;
            }

            find_graph_node_ancestor(header_entity, &parents, &node_markers)
        });

        if let Some(entity) = dragged_entity {
            drag_state.active_entity = Some(entity);
            drag_state.last_mouse_pos = current_mouse_pos;
        }
    }

    if mouse_button.pressed(MouseButton::Left) {
        if let Some(entity) = drag_state.active_entity {
            if let Ok(mut trans) = node_transforms.get_mut(entity) {
                let delta_screen = current_mouse_pos - drag_state.last_mouse_pos;
                let delta_world = delta_screen * zoom_factor;
                trans.translation.x = trans.translation.x + delta_world.x;
                trans.translation.y = trans.translation.y - delta_world.y;
                drag_state.last_mouse_pos = current_mouse_pos;
            } else {
                drag_state.active_entity = None;
            }
        }
    }

    if mouse_button.just_released(MouseButton::Left) {
        if drag_state.active_entity.is_some() {
            mutations.mark_changed();
        }
        drag_state.active_entity = None;
    }
}

pub fn camera_controller(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scroll_evr: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    mut query: Query<
        (&mut Transform, &mut Projection, &Camera, &GlobalTransform),
        With<GraphCamera>,
    >,
) {
    let zoom_speed = 0.1;
    let camera_speed = 400.0;

    let mut scroll_amount = 0.0;
    for ev in scroll_evr.read() {
        scroll_amount += ev.y;
    }

    let Ok((mut transform, mut projection, camera, cam_global_transform)) = query.single_mut()
    else {
        return;
    };

    let Ok(window) = windows.single() else { return };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    if let Projection::Orthographic(ref mut ortho) = *projection {
        let mouse_world_pos = camera
            .viewport_to_world_2d(cam_global_transform, cursor_position)
            .unwrap_or(Vec2::ZERO);

        let scale_before = ortho.scale;

        if scroll_amount != 0.0 {
            ortho.scale *= 1.0 - (scroll_amount * zoom_speed * 0.1);
            ortho.scale = ortho.scale.clamp(0.2, 5.0);
        }

        let scale_after = ortho.scale;

        let cam_pos_vec = transform.translation.truncate();
        let vector_to_mouse = mouse_world_pos - cam_pos_vec;
        let ratio = scale_after / scale_before;
        transform.translation =
            (mouse_world_pos - vector_to_mouse * ratio).extend(transform.translation.z);

        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            transform.translation +=
                direction.normalize() * camera_speed * scale_after * time.delta_secs();
        }
    }
}

pub fn draggable_value_system(_query: Query<Entity>) {}
