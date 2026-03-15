//! Basic pin, drag, and connection state types.
use bevy::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub enum Pin {
    Output,
    Input,
}

#[derive(Resource, Default)]
pub struct DragState {
    pub active_entity: Option<Entity>,
    pub initial_mouse_pos: Vec2,
    pub initial_node_pos: Vec2,
    pub last_mouse_pos: Vec2,
}

impl DragState {
    pub fn clear(&mut self) {
        self.active_entity = None;
        self.initial_mouse_pos = Vec2::ZERO;
        self.initial_node_pos = Vec2::ZERO;
        self.last_mouse_pos = Vec2::ZERO;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphConnection {
    pub from_node: Entity,
    pub from_index: usize,
    pub to_node: Entity,
    pub to_index: usize,
    pub from_port: Entity,
    pub to_port: Entity,
}

impl GraphConnection {
    pub fn references_node(&self, entity: Entity) -> bool {
        self.from_node == entity || self.to_node == entity
    }

    pub fn references_port(&self, entity: Entity) -> bool {
        self.from_port == entity || self.to_port == entity
    }
}

#[derive(Resource, Default)]
pub struct WireConnectionState {
    pub dragging_from: Option<Entity>,
    pub node_from: Option<Entity>,
    pub index_from: Option<usize>,
    pub current_mouse_world_pos: Vec2,
    pub is_dragging: bool,
}

impl WireConnectionState {
    pub fn clear(&mut self) {
        self.dragging_from = None;
        self.node_from = None;
        self.index_from = None;
        self.current_mouse_world_pos = Vec2::ZERO;
        self.is_dragging = false;
    }

    pub fn references_node(&self, entity: Entity) -> bool {
        self.node_from == Some(entity)
    }
}
