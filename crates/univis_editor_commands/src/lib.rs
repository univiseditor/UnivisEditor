//! Editor-facing command messages that drive graph UX, persistence, and workflow actions.

use bevy::prelude::*;
use univis_node_graph::{
    commands::GraphMutationTracker,
    document::{GraphDocument, LiveGraphDocumentState},
    graph_validation::LiveGraphValidationState,
    node_definition::NodeId,
};

#[derive(Debug, Clone, Default)]
pub struct GraphClipboardSnapshot {
    pub document: GraphDocument,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphClipboardState {
    pub snapshot: Option<GraphClipboardSnapshot>,
}

impl GraphClipboardState {
    pub fn has_contents(&self) -> bool {
        self.snapshot
            .as_ref()
            .is_some_and(|snapshot| !snapshot.document.nodes.is_empty())
    }
}

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
    CopySelectedNodes,
    PasteNodes {
        position: Vec2,
    },
    DuplicateSelectedNodes,
    FrameSelectedNodes,
    CapturePrefabFromSelection,
    CaptureSubgraphFromSelection,
    UpdatePrefabFromSelection {
        prefab_id: String,
    },
    UpdateSubgraphFromSelection {
        subgraph_id: String,
    },
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
            .init_resource::<GraphClipboardState>()
            .init_resource::<GraphMutationTracker>()
            .init_resource::<LiveGraphValidationState>()
            .init_resource::<LiveGraphDocumentState>();
    }
}

pub mod prelude {
    pub use crate::{
        GraphClipboardSnapshot, GraphClipboardState, GraphCommandRequest, GraphCommandsPlugin,
    };
}
