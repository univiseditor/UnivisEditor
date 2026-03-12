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

#[derive(Debug, Clone)]
pub struct GraphLink {
    pub from_node: Entity,
    pub from_index: usize,
    pub to_node: Entity,
    pub to_index: usize,
    pub from_port: Entity,
    pub to_port: Entity,
}

#[derive(Resource, Default)]
pub struct WireConnectionState {
    pub dragging_from: Option<Entity>,
    pub node_from: Option<Entity>,
    pub index_from: Option<usize>,
    pub current_mouse_world_pos: Vec2,
    pub is_dragging: bool,
}

#[derive(Resource, Debug, Default)]
pub struct Connecting {
    pub connections: Vec<GraphLink>,
}
