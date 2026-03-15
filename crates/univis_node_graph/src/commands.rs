use bevy::prelude::*;

use crate::{document::LiveGraphDocumentState, node_definition::NodeId};

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

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct GraphMutationTracker {
    pub capture_requested: bool,
}

impl GraphMutationTracker {
    pub fn mark_changed(&mut self) {
        self.capture_requested = true;
    }
}

pub struct GraphCommandsPlugin;

impl Plugin for GraphCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GraphCommandRequest>()
            .init_resource::<GraphMutationTracker>()
            .init_resource::<LiveGraphDocumentState>();
    }
}
