//! المكونات الأساسية للعُقد والمنافذ

use bevy::prelude::*;

// علامة لتمييز المنافذ القديمة (للتوافق)
#[derive(Clone, PartialEq, Eq)]
pub enum Pin {
    Output,
    Input,
}

// لتخزين حالة السحب
#[derive(Resource, Default)]
pub struct DragState {
    pub active_entity: Option<Entity>,
    pub initial_mouse_pos: Vec2,
    pub initial_node_pos: Vec2,
    pub last_mouse_pos: Vec2,
}

// تعريف وصلة بين منفذين (للخطوط)
#[derive(Debug, Clone)]
pub struct GraphLink {
    pub from_node: Entity,
    pub from_index: usize,
    pub to_node: Entity,
    pub to_index: usize,
    pub from_port: Entity,
    pub to_port: Entity,
}

/// حالة الوصلة أثناء السحب
#[derive(Resource, Default)]
pub struct WireConnectionState {
    pub dragging_from: Option<Entity>,
    pub node_from: Option<Entity>,
    pub index_from: Option<usize>,
    pub current_mouse_world_pos: Vec2,
    pub is_dragging: bool,
}

/// قائمة الوصلات
#[derive(Resource, Debug, Default)]
pub struct Connecting {
    pub connections: Vec<GraphLink>,
}
