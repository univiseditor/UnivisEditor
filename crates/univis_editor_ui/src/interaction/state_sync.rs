use crate::prelude::*;
use bevy::prelude::*;

pub fn sanitize_graph_editor_state(
    mut commands: Commands,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut drag_state: ResMut<DragState>,
    mut wire_state: ResMut<WireConnectionState>,
    mut popup: ResMut<NodePopupState>,
    mut overlay: ResMut<GraphOverlayState>,
    q_nodes: Query<(), With<GraphNode>>,
    q_connections: Query<(Entity, &GraphConnection)>,
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

    for (entity, link) in q_connections.iter() {
        let is_valid = node_exists(link.from_node)
            && node_exists(link.to_node)
            && port_matches(
                link.from_port,
                link.from_node,
                PortType::Output,
                link.from_index,
            )
            && port_matches(link.to_port, link.to_node, PortType::Input, link.to_index);

        if !is_valid {
            commands.entity(entity).try_despawn();
        }
    }

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

/// Rebuilds per-port connection caches whenever live connection entities change.
pub fn sync_port_connection_caches_system(
    q_all_connections: Query<(Entity, &GraphConnection)>,
    q_changed_connections: Query<Entity, Or<(Added<GraphConnection>, Changed<GraphConnection>)>>,
    q_added_ports: Query<Entity, Added<GraphPort>>,
    mut removed_connections: RemovedComponents<GraphConnection>,
    mut q_inputs: Query<&mut InputConnection>,
    mut q_outputs: Query<&mut OutputConnections>,
) {
    let should_refresh = !q_changed_connections.is_empty()
        || !q_added_ports.is_empty()
        || removed_connections.read().next().is_some();
    if !should_refresh {
        return;
    }

    for mut input in q_inputs.iter_mut() {
        *input = InputConnection::default();
    }

    for mut output in q_outputs.iter_mut() {
        output.targets.clear();
    }

    for (connection_entity, link) in q_all_connections.iter() {
        if let Ok(mut input) = q_inputs.get_mut(link.to_port) {
            input.source_node = Some(link.from_node);
            input.source_port_index = Some(link.from_index);
            input.source_port = Some(link.from_port);
            input.connection_entity = Some(connection_entity);
        }

        if let Ok(mut output) = q_outputs.get_mut(link.from_port) {
            output.targets.push(OutputTarget {
                target_node: link.to_node,
                target_port_index: link.to_index,
                target_port: link.to_port,
                connection_entity,
            });
        }
    }
}

/// Rebuilds the live graph document snapshot from the current editor world.
pub fn sync_live_graph_document_state(
    q_nodes: Query<(
        Entity,
        &GraphNode,
        Option<&AuthoredNodeInputs>,
        &Transform,
        Option<&Selected>,
    )>,
    q_changed_nodes: Query<
        Entity,
        (
            With<GraphNode>,
            Or<(
                Added<GraphNode>,
                Added<AuthoredNodeInputs>,
                Changed<AuthoredNodeInputs>,
                Changed<Transform>,
                Added<Selected>,
            )>,
        ),
    >,
    q_connections: Query<&GraphConnection>,
    q_changed_connections: Query<Entity, Or<(Added<GraphConnection>, Changed<GraphConnection>)>>,
    q_camera: Query<(&Transform, &Projection), With<GraphCamera>>,
    q_changed_camera: Query<
        Entity,
        (
            With<GraphCamera>,
            Or<(Added<GraphCamera>, Changed<Transform>, Changed<Projection>)>,
        ),
    >,
    mut removed_nodes: RemovedComponents<GraphNode>,
    mut removed_connections: RemovedComponents<GraphConnection>,
    mut removed_selected: RemovedComponents<Selected>,
    mut removed_cameras: RemovedComponents<GraphCamera>,
    mut live_document: ResMut<LiveGraphDocumentState>,
) {
    let should_refresh = !q_changed_nodes.is_empty()
        || !q_changed_connections.is_empty()
        || !q_changed_camera.is_empty()
        || removed_nodes.read().next().is_some()
        || removed_connections.read().next().is_some()
        || removed_selected.read().next().is_some()
        || removed_cameras.read().next().is_some();
    if !should_refresh {
        return;
    }

    let mut nodes_data = Vec::new();
    for (entity, node, authored_inputs, transform, selected) in q_nodes.iter() {
        nodes_data.push(GraphDocumentNodeSnapshot {
            entity,
            definition_id: node.definition_id.clone(),
            position: [transform.translation.x, transform.translation.y],
            inputs: authored_inputs
                .map(|inputs| inputs.values.clone())
                .unwrap_or_else(|| node.values.inputs.clone()),
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

    let mut live_connections = q_connections.iter().copied().collect::<Vec<_>>();
    live_connections.sort_by_key(|link| {
        (
            link.from_node.index(),
            link.from_index,
            link.to_node.index(),
            link.to_index,
        )
    });

    let edge_snapshots = live_connections
        .into_iter()
        .map(|link| GraphDocumentEdgeSnapshot {
            from_entity: link.from_node,
            from_index: link.from_index,
            to_entity: link.to_node,
            to_index: link.to_index,
        });

    live_document.rebuild_from_snapshots(nodes_data, edge_snapshots, camera);
}
