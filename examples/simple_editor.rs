use bevy::prelude::*;
use univis_editor_app::NodeGraphPlugin;
use univis_editor_ui::prelude::{
    GraphCamera, InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((NodeGraphPlugin, InfiniteGridPlugin))
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((Camera2d, GraphCamera));
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            scale: 50.,
            dot_fadeout_strength: 0.,
            z_axis_color: Color::NONE,
            x_axis_color: Color::NONE,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::Y, Vec3::Z)),
    ));
}
