//! UnivisEditor app orchestration for editor UX surfaces and plugin assembly.

mod editor_settings_persistence;
mod island;
mod panels;

use bevy::prelude::*;
use editor_settings_persistence::EditorSettingsPersistencePlugin;
use island::CanvasIslandPlugin;
use panels::FloatingPanelsPlugin;
use univis_editor_commands::GraphCommandsPlugin;
use univis_editor_nodes_builtin as _;
use univis_editor_persistence::graph_persistence::GraphPersistencePlugin;
use univis_editor_runtime::NodeRuntimePlugin;
use univis_editor_ui::NodeUiPlugin;
use univis_editor_workflows::GraphAssetWorkflowPlugin;
use univis_node_graph::node_registry::NodeRegistryPlugin;

pub mod prelude {
    pub use crate::NodeGraphPlugin;
}

pub struct NodeGraphPlugin;

impl Plugin for NodeGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NodeRegistryPlugin)
            .add_plugins(GraphCommandsPlugin)
            .add_plugins(NodeUiPlugin)
            .add_plugins(NodeRuntimePlugin)
            .add_plugins(GraphPersistencePlugin)
            .add_plugins(CanvasIslandPlugin)
            .add_plugins(GraphAssetWorkflowPlugin)
            .add_plugins(FloatingPanelsPlugin)
            .add_plugins(EditorSettingsPersistencePlugin);
    }
}
