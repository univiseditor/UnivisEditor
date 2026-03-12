//! Editor settings shared by the graph UI.
use bevy::prelude::*;

#[derive(Component)]
pub struct GraphCamera;

#[derive(Resource, Debug, Clone)]
pub struct EditorSettings {
    pub default_node_width: f32,
    pub grid_size: f32,
    pub camera_speed: f32,
    pub zoom_speed: f32,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            default_node_width: 200.0,
            grid_size: 20.0,
            camera_speed: 400.0,
            zoom_speed: 0.1,
        }
    }
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorSettings>();
    }
}
