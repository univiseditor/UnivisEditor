use bevy::prelude::*;

use crate::{document::LiveGraphDocumentState, node_definition::NodeId};

#[derive(Message, Debug, Clone)]
pub enum GraphCommandRequest {
    SaveGraph,
    SaveGraphToPath { path: String },
    LoadGraph,
    LoadGraphFromPath { path: String, force_if_dirty: bool },
    DeleteSelectedNodes,
    SpawnNode { definition_id: NodeId, position: Vec2 },
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GraphOverlayState {
    pub active_surface: GraphOverlaySurface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphOverlaySurface {
    #[default]
    None,
    ContextMenu,
    CanvasIslandMenu,
    NodePopup,
}

pub struct GraphCommandsPlugin;

impl Plugin for GraphCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GraphCommandRequest>()
            .init_resource::<GraphOverlayState>()
            .init_resource::<LiveGraphDocumentState>();
    }
}
