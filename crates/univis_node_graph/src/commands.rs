use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct GraphMutationTracker {
    pub capture_requested: bool,
}

impl GraphMutationTracker {
    pub fn mark_changed(&mut self) {
        self.capture_requested = true;
    }
}
