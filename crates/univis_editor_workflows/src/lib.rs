//! Editor workflow systems that operate on graphs without moving editor UX concerns into Graph Core.

mod capture;
mod clipboard;
mod duplicate;
mod instantiate;
mod prefab_sync;
mod status;

use bevy::prelude::*;
use univis_editor_runtime::NodeRuntimeSystemSet;

use self::capture::{capture_prefab_from_selection_system, capture_subgraph_from_selection_system};
use self::clipboard::{copy_selected_nodes_to_clipboard_system, paste_nodes_from_clipboard_system};
use self::duplicate::duplicate_selected_nodes_system;
use self::instantiate::{insert_subgraph_instances_system, spawn_prefab_instance_nodes_system};
use self::prefab_sync::sync_prefab_instance_nodes_system;

pub struct GraphAssetWorkflowPlugin;

impl Plugin for GraphAssetWorkflowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            sync_prefab_instance_nodes_system.in_set(NodeRuntimeSystemSet::Prepare),
        )
        .add_systems(
            Update,
            (
                copy_selected_nodes_to_clipboard_system,
                paste_nodes_from_clipboard_system,
                duplicate_selected_nodes_system,
                capture_prefab_from_selection_system,
                capture_subgraph_from_selection_system,
                insert_subgraph_instances_system,
                spawn_prefab_instance_nodes_system,
            )
                .chain(),
        );
    }
}
