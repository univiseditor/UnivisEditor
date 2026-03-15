//! Graph interaction systems split by editing responsibility.
use crate::prelude::*;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use univis_ui::prelude::*;

mod box_selection;
mod camera;
mod drag;
mod selection;
mod state_sync;
mod workflow_shortcuts;

pub use box_selection::*;
pub use camera::*;
pub use drag::*;
pub use selection::*;
pub use state_sync::*;
pub use workflow_shortcuts::*;

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

fn graph_editing_enabled(activation: Option<&GraphEditingUiActivation>) -> bool {
    activation
        .map(|activation| activation.enabled)
        .unwrap_or(true)
}
