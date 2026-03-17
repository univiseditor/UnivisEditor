use bevy::prelude::*;
use univis_node_graph::prelude::GraphNode;
use univis_scene::{EntitySpawnOptions, spawn_scene_document_recursive};

use crate::scene_outputs::{GraphSceneOutputMode, GraphSceneOutputs};

#[derive(Component, Clone, Default)]
pub(crate) struct SceneWorldDisplayState {
    root: Option<Entity>,
    last_signature: Option<String>,
}

#[derive(Component)]
pub(crate) struct SceneWorldRoot {
    node_entity: Entity,
}

pub(super) fn sync_scene_nodes_to_world_system(
    mut commands: Commands,
    scene_outputs: Res<GraphSceneOutputs>,
    q_nodes: Query<(Entity, Option<&SceneWorldDisplayState>), With<GraphNode>>,
) {
    // Rebuild each sink root only when its scene signature changes, to keep world sync predictable.
    for (node_entity, state) in q_nodes.iter() {
        let next_output = scene_outputs.sinks.iter().find(|sink| {
            sink.node_entity == node_entity && sink.mode == GraphSceneOutputMode::World
        });

        if next_output.is_none() {
            if let Some(state) = state.cloned() {
                if let Some(root) = state.root {
                    commands.entity(root).try_despawn();
                }
                commands
                    .entity(node_entity)
                    .try_remove::<SceneWorldDisplayState>();
            }
            continue;
        }

        let next_output = next_output.expect("checked above");
        let next_scene = next_output.scene.as_ref();
        let next_signature = next_output.signature.clone();
        let mut next_state = state.cloned().unwrap_or_default();
        let unchanged = next_state.last_signature == next_signature
            && if next_scene.is_some() {
                next_state.root.is_some()
            } else {
                next_state.root.is_none()
            };

        if unchanged {
            continue;
        }

        if let Some(root) = next_state.root.take() {
            commands.entity(root).try_despawn();
        }

        if let Some(scene) = next_scene.as_ref() {
            let spawn_options = EntitySpawnOptions::default();
            let root = commands
                .spawn((
                    SceneWorldRoot { node_entity },
                    Name::new("Scene Display Root"),
                    Transform::default(),
                    Visibility::Visible,
                ))
                .id();
            commands.entity(root).with_children(|parent| {
                spawn_scene_document_recursive(parent, scene, &spawn_options);
            });
            next_state.root = Some(root);
        }

        next_state.last_signature = next_signature;
        commands.entity(node_entity).try_insert(next_state);
    }
}

pub(super) fn cleanup_orphaned_scene_roots_system(
    mut commands: Commands,
    q_roots: Query<(Entity, &SceneWorldRoot)>,
    q_nodes: Query<(), With<GraphNode>>,
) {
    for (root, marker) in q_roots.iter() {
        if q_nodes.get(marker.node_entity).is_err() {
            commands.entity(root).try_despawn();
        }
    }
}
