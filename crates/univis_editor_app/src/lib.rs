//! UnivisEditor App facade

use bevy::prelude::*;
use univis_editor_core::node_registry::NodeRegistryPlugin;
use univis_editor_persistence::graph_persistence::GraphPersistencePlugin;
use univis_editor_runtime::NodeRuntimePlugin;
use univis_editor_ui::NodeUiPlugin;
use univis_editor_nodes_builtin as _;

#[cfg(feature = "game_editor")]
use univis_editor_game_editor::plugin::GameEditorPlugin;

pub mod prelude {
    pub use univis_editor_core::prelude::*;
    pub use univis_editor_persistence::prelude::*;
    pub use univis_editor_runtime::prelude::*;
    pub use univis_editor_ui::prelude::*;

    #[cfg(feature = "game_editor")]
    pub use univis_editor_game_editor::prelude::*;

    pub use crate::NodeGraphPlugin;
}

pub struct NodeGraphPlugin;

impl Plugin for NodeGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NodeRegistryPlugin)
            .add_plugins(NodeUiPlugin)
            .add_plugins(NodeRuntimePlugin)
            .add_plugins(GraphPersistencePlugin);

        #[cfg(feature = "game_editor")]
        {
            app.add_plugins(GameEditorPlugin);
        }
    }
}
