//! Editor settings shared by the graph UI.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::widgets::infinity_grid::InfiniteGridSettings;

#[derive(Component)]
pub struct GraphCamera;

#[derive(Debug, Clone, Copy)]
pub struct GridPaletteColors {
    pub x_axis_color: Color,
    pub z_axis_color: Color,
    pub minor_line_color: Color,
    pub major_line_color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GridDisplayMode {
    #[default]
    Lines,
    Dots,
}

impl GridDisplayMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Lines => Self::Dots,
            Self::Dots => Self::Lines,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Lines => "Lines",
            Self::Dots => "Points",
        }
    }

    pub fn as_shader_value(self) -> f32 {
        match self {
            Self::Lines => 0.0,
            Self::Dots => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GridColorPalette {
    #[default]
    Slate,
    Mono,
    Ocean,
    Ember,
    Forest,
}

impl GridColorPalette {
    pub fn next(self) -> Self {
        match self {
            Self::Slate => Self::Mono,
            Self::Mono => Self::Ocean,
            Self::Ocean => Self::Ember,
            Self::Ember => Self::Forest,
            Self::Forest => Self::Slate,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Slate => "Slate",
            Self::Mono => "Mono",
            Self::Ocean => "Ocean",
            Self::Ember => "Ember",
            Self::Forest => "Forest",
        }
    }

    pub fn colors(self) -> GridPaletteColors {
        match self {
            Self::Slate => GridPaletteColors {
                x_axis_color: Color::oklch(0.65, 0.24, 27.0),
                z_axis_color: Color::oklch(0.65, 0.19, 255.0),
                minor_line_color: Color::srgba(0.12, 0.13, 0.15, 0.82),
                major_line_color: Color::srgba(0.24, 0.26, 0.30, 0.92),
            },
            Self::Mono => GridPaletteColors {
                x_axis_color: Color::srgba(0.55, 0.55, 0.58, 0.92),
                z_axis_color: Color::srgba(0.42, 0.42, 0.46, 0.92),
                minor_line_color: Color::srgba(0.14, 0.14, 0.14, 0.84),
                major_line_color: Color::srgba(0.32, 0.32, 0.32, 0.94),
            },
            Self::Ocean => GridPaletteColors {
                x_axis_color: Color::srgba(0.20, 0.80, 0.88, 0.96),
                z_axis_color: Color::srgba(0.22, 0.45, 0.95, 0.96),
                minor_line_color: Color::srgba(0.05, 0.17, 0.20, 0.82),
                major_line_color: Color::srgba(0.10, 0.44, 0.52, 0.94),
            },
            Self::Ember => GridPaletteColors {
                x_axis_color: Color::srgba(0.98, 0.55, 0.20, 0.96),
                z_axis_color: Color::srgba(0.86, 0.24, 0.28, 0.96),
                minor_line_color: Color::srgba(0.18, 0.09, 0.08, 0.82),
                major_line_color: Color::srgba(0.42, 0.18, 0.12, 0.94),
            },
            Self::Forest => GridPaletteColors {
                x_axis_color: Color::srgba(0.62, 0.88, 0.30, 0.96),
                z_axis_color: Color::srgba(0.15, 0.74, 0.58, 0.96),
                minor_line_color: Color::srgba(0.08, 0.16, 0.11, 0.82),
                major_line_color: Color::srgba(0.18, 0.36, 0.24, 0.94),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WireStyle {
    #[default]
    Bezier,
    Straight,
    Stepped,
}

impl WireStyle {
    pub fn next(self) -> Self {
        match self {
            Self::Bezier => Self::Straight,
            Self::Straight => Self::Stepped,
            Self::Stepped => Self::Bezier,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Bezier => "Bezier",
            Self::Straight => "Straight",
            Self::Stepped => "Stepped",
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorSettings {
    pub default_node_width: f32,
    pub grid_size: f32,
    pub camera_speed: f32,
    pub zoom_speed: f32,
    pub grid_display_mode: GridDisplayMode,
    pub grid_color_palette: GridColorPalette,
    pub grid_point_size: f32,
    pub wire_style: WireStyle,
    pub wire_color_from_output: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            default_node_width: 200.0,
            grid_size: 20.0,
            camera_speed: 400.0,
            zoom_speed: 0.1,
            grid_display_mode: GridDisplayMode::Lines,
            grid_color_palette: GridColorPalette::Slate,
            grid_point_size: 1.0,
            wire_style: WireStyle::Bezier,
            wire_color_from_output: true,
        }
    }
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorSettings>()
            .add_systems(Update, sync_infinite_grid_settings);
    }
}

fn sync_infinite_grid_settings(
    settings: Res<EditorSettings>,
    mut grids: Query<&mut InfiniteGridSettings>,
) {
    let palette = settings.grid_color_palette.colors();
    for mut grid in grids.iter_mut() {
        if grid.display_mode != settings.grid_display_mode {
            grid.display_mode = settings.grid_display_mode;
        }
        if grid.x_axis_color != palette.x_axis_color {
            grid.x_axis_color = palette.x_axis_color;
        }
        if grid.z_axis_color != palette.z_axis_color {
            grid.z_axis_color = palette.z_axis_color;
        }
        if grid.minor_line_color != palette.minor_line_color {
            grid.minor_line_color = palette.minor_line_color;
        }
        if grid.major_line_color != palette.major_line_color {
            grid.major_line_color = palette.major_line_color;
        }
        if (grid.point_size - settings.grid_point_size).abs() > f32::EPSILON {
            grid.point_size = settings.grid_point_size;
        }
    }
}
