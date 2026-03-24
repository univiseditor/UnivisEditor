use crate::internal_prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use univis_ui::prelude::*;

use super::{
    BoxSelectionState, graph_editing_enabled, interaction_is_pointer_active, pointer_target_node,
    selection_additive_modifier,
};

/// Selects the node under the pointer and mirrors that selection into the live document.
pub fn selection_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    overlay: Res<GraphOverlayState>,
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

    if overlay.active_surface != GraphOverlaySurface::None {
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

/// Deletes the current selection and removes attached wires from the live graph.
pub fn delete_node_system(
    mut commands: Commands,
    activation: Option<Res<GraphEditingUiActivation>>,
    mut command_requests: MessageReader<GraphCommandRequest>,
    mut delete_requests: MessageReader<DeleteSelectedNodesRequest>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut drag_state: ResMut<DragState>,
    mut wire_state: ResMut<WireConnectionState>,
    q_connections: Query<(Entity, &GraphConnection)>,
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

    for (connection_entity, link) in q_connections.iter() {
        if deleted_set.contains(&link.from_node)
            || deleted_set.contains(&link.to_node)
            || deleted_set.contains(&link.from_port)
            || deleted_set.contains(&link.to_port)
        {
            commands.entity(connection_entity).try_despawn();
        }
    }

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

pub fn disconnect_wire_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    mut commands: Commands,
    q_connections: Query<(Entity, &GraphConnection)>,
    ports: Query<(Entity, &UInteraction, &GraphPort)>,
    mut mutations: ResMut<GraphMutationTracker>,
) {
    if mouse_button.just_pressed(MouseButton::Right) {
        for (port_entity, interaction, port_data) in ports.iter() {
            if interaction_is_pointer_active(interaction) && port_data.port_type == PortType::Input
            {
                if live_document.disconnect_input_for_entity(port_data.node_entity, port_data.index)
                {
                    for (connection_entity, link) in q_connections.iter() {
                        if link.to_port == port_entity {
                            commands.entity(connection_entity).try_despawn();
                        }
                    }
                    mutations.mark_changed();
                }
            }
        }
    }
}
