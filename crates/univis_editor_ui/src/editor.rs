//! إعدادات المحرر والمكونات الإضافية

use bevy::prelude::*;

/// مكون للكاميرا المستخدمة في المحرر
#[derive(Component)]
pub struct GraphCamera;

/// إعدادات المحرر
#[derive(Resource, Debug, Clone)]
pub struct EditorSettings {
    /// عرض العقدة الافتراضي
    pub default_node_width: f32,
    /// مسافة الشبكة
    pub grid_size: f32,
    /// سرعة الكاميرا
    pub camera_speed: f32,
    /// سرعة التكبير
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

/// Plugin لإعدادات المحرر
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorSettings>();
    }
}
