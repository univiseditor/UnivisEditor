//! UnivisEditor App facade

mod island;

use bevy::prelude::*;
use island::CanvasIslandPlugin;
use univis_node_graph::{commands::GraphCommandsPlugin, node_registry::NodeRegistryPlugin};
use univis_editor_persistence::graph_persistence::GraphPersistencePlugin;
use univis_editor_runtime::NodeRuntimePlugin;
use univis_editor_ui::NodeUiPlugin;
use univis_editor_nodes_builtin as _;

pub mod prelude {
    pub use univis_node_graph::prelude::*;
    pub use univis_editor_persistence::prelude::*;
    pub use univis_editor_runtime::prelude::*;
    pub use univis_editor_ui::prelude::*;

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
            .add_plugins(CanvasIslandPlugin);
    }
}
