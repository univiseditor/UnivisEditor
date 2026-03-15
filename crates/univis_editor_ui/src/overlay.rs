use bevy::prelude::*;

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
