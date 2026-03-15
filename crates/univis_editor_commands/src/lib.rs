//! Editor-facing command messages that drive graph UX, persistence, and workflow actions.

use bevy::prelude::*;
use univis_node_graph::{
    commands::GraphMutationTracker, document::LiveGraphDocumentState, node_definition::NodeId,
};

#[derive(Message, Debug, Clone)]
pub enum GraphCommandRequest {
    SaveGraph,
    SaveGraphToPath {
        path: String,
    },
    LoadGraph,
    LoadGraphFromPath {
        path: String,
        force_if_dirty: bool,
    },
    DeleteSelectedNodes,
    DuplicateSelectedNodes,
    FrameSelectedNodes,
    CapturePrefabFromSelection,
    CaptureSubgraphFromSelection,
    InsertSubgraph {
        subgraph_id: String,
        position: Vec2,
    },
    SpawnPrefabNode {
        prefab_id: String,
        position: Vec2,
    },
    UndoGraphChange,
    RedoGraphChange,
    SpawnNode {
        definition_id: NodeId,
        position: Vec2,
    },
}

pub struct GraphCommandsPlugin;

impl Plugin for GraphCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GraphCommandRequest>()
            .init_resource::<GraphMutationTracker>()
            .init_resource::<LiveGraphDocumentState>();
    }
}

pub mod prelude {
    pub use crate::{GraphCommandRequest, GraphCommandsPlugin};
}
