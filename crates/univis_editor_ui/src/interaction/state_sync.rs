use crate::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

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
