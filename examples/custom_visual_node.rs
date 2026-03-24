use bevy::prelude::*;
use univis_editor_app::NodeGraphPlugin;
use univis_editor_ui::prelude::{
    GraphCamera, InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings,
};
use univis_node_graph::register_node;
use univis_node_graph::{
    node_definition::{
        GraphNode, NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext,
        ProcessResult,
    },
    value::NodeValue,
};
use univis_ui::prelude::*;

#[derive(Component)]
struct HeatValueLabel {
    node_entity: Entity,
}

#[derive(Component)]
struct HeatBarFill {
    node_entity: Entity,
}

#[derive(Component)]
struct HeatPanel {
    node_entity: Entity,
}

pub struct HeatPreviewNode;

impl NodeDefinition for HeatPreviewNode {
    fn id(&self) -> NodeId {
        NodeId::new("example/visual_heat")
    }

    fn display_name(&self) -> &str {
        "Visual Heat"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Examples/Visual")
    }

    fn description(&self) -> Option<&str> {
        Some("Custom node appearance with dynamic body updates")
    }

    fn color(&self) -> Color {
        Color::srgb(0.14, 0.22, 0.33)
    }

    fn title_color(&self) -> Color {
        Color::srgb(0.92, 0.97, 1.0)
    }

    fn body_width(&self) -> f32 {
        235.0
    }

    fn body_height(&self) -> f32 {
        170.0
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("Temperature C")
                .with_default(NodeValue::float(22.0))
                .editable_inline()
                .with_ui_step(1.0)
                .with_ui_range(-20.0, 100.0),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Out C")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let temp = ctx.get_float_or(0, 22.0);
        ctx.set_float(0, temp);
        ProcessResult::Success
    }

    fn has_custom_body(&self) -> bool {
        true
    }

    fn build_body(&self, body: &mut ChildSpawnerCommands, node_entity: Entity) {
        body.spawn((
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(105.0),
                padding: USides::all(10.0),
                background_color: Color::srgb(0.1, 0.15, 0.22),
                border_radius: UCornerRadius::all(10.0),
                ..default()
            },
            ULayout {
                flex_direction: UFlexDirection::Column,
                gap: 8.0,
                ..default()
            },
            HeatPanel { node_entity },
        ))
        .with_children(|panel| {
            panel.spawn((
                HeatValueLabel { node_entity },
                UTextLabel {
                    text: "22.0 C".to_string(),
                    font_size: 13.0,
                    color: Color::srgb(0.95, 0.98, 1.0),
                    autosize: false,
                    ..default()
                },
            ));

            panel
                .spawn((
                    UNode {
                        width: UVal::Percent(1.0),
                        height: UVal::Px(16.0),
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.25),
                        border_radius: UCornerRadius::all(8.0),
                        ..default()
                    },
                    ULayout {
                        justify_content: UJustifyContent::Start,
                        align_items: UAlignItems::Stretch,
                        ..default()
                    },
                ))
                .with_children(|track| {
                    track.spawn((
                        HeatBarFill { node_entity },
                        UNode {
                            width: UVal::Percent(0.0),
                            height: UVal::Percent(1.0),
                            background_color: Color::srgb(0.18, 0.45, 0.98),
                            border_radius: UCornerRadius::all(8.0),
                            ..default()
                        },
                    ));
                });

            panel.spawn(UTextLabel {
                text: "Edit inline on the node".to_string(),
                font_size: 10.0,
                color: Color::srgb(0.62, 0.75, 0.95),
                autosize: false,
                ..default()
            });
        });
    }
    fn sync_visual(&self, world: &mut World, node_entity: Entity) {
        let temp = world
            .get::<GraphNode>(node_entity)
            .and_then(|node| {
                node.output_projection(0)
                    .or_else(|| node.input_projection(0))
                    .and_then(NodeValue::as_float)
            })
            .unwrap_or(22.0) as f32;

        let normalized = ((temp + 20.0) / 120.0).clamp(0.0, 1.0);
        let heat_color = Color::srgb(
            0.2 + normalized * 0.75,
            0.25 + normalized * 0.2,
            1.0 - normalized * 0.8,
        );

        let mut text_query = world.query::<(&HeatValueLabel, &mut UTextLabel)>();
        for (marker, mut label) in text_query.iter_mut(world) {
            if marker.node_entity == node_entity {
                label.text = format!("{temp:.1} C");
            }
        }

        let mut bar_query = world.query::<(&HeatBarFill, &mut UNode)>();
        for (marker, mut fill) in bar_query.iter_mut(world) {
            if marker.node_entity == node_entity {
                fill.width = UVal::Percent(normalized.clamp(0.04, 1.0));
                fill.background_color = heat_color;
            }
        }

        let mut panel_query = world.query::<(&HeatPanel, &mut UNode)>();
        for (marker, mut panel) in panel_query.iter_mut(world) {
            if marker.node_entity == node_entity {
                panel.background_color = Color::srgba(
                    0.06 + normalized * 0.18,
                    0.09 + normalized * 0.08,
                    0.16 + normalized * 0.04,
                    1.0,
                );
            }
        }
    }
}

register_node!(HeatPreviewNode);

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
            scale: 50.0,
            dot_fadeout_strength: 0.0,
            z_axis_color: Color::NONE,
            x_axis_color: Color::NONE,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::Y, Vec3::Z)),
    ));
}
