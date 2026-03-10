use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMode {
    LegacyGraph,
    ComponentMode,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct EditorModeState {
    pub mode: EditorMode,
}

impl Default for EditorModeState {
    fn default() -> Self {
        Self {
            mode: EditorMode::LegacyGraph,
        }
    }
}
