//! Wire interaction and rendering systems.
use crate::prelude::*;
use bevy::prelude::*;
use univis_ui::prelude::*;

use crate::editor::{EditorSettings, WireStyle};

const WIRE_SEGMENT_Z: f32 = 0.25;
const WIRE_SEGMENT_THICKNESS: f32 = 5.0;
const BEZIER_SEGMENT_COUNT: usize = 24;

#[derive(Component)]
pub struct WireVisualSegment;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum WireSystemsSet {
    Start,
    Update,
    Complete,
}

fn interaction_is_pointer_active(interaction: &UInteraction) -> bool {
    matches!(
        *interaction,
        UInteraction::Pressed | UInteraction::Hovered | UInteraction::Clicked
    )
}

/// Starts a new wire drag from the currently hovered output port.
pub fn wire_start_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut wire_state: ResMut<WireConnectionState>,
    ports: Query<(Entity, &UInteraction, &GraphPort)>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        for (entity, interaction, port) in ports.iter() {
            if interaction_is_pointer_active(interaction) && port.port_type == PortType::Output {
                wire_state.dragging_from = Some(entity);
                wire_state.node_from = Some(port.node_entity);
                wire_state.index_from = Some(port.index);
                wire_state.is_dragging = true;
                wire_state.current_mouse_world_pos = Vec2::ZERO;
                break;
            }
        }
    }
}

/// Updates the preview wire endpoint from the current cursor position.
pub fn wire_update_system(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    mut wire_state: ResMut<WireConnectionState>,
) {
    if !wire_state.is_dragging {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            wire_state.current_mouse_world_pos = world_pos;
        }
    }
}

/// Finalizes a wire drag when the hovered input port accepts the pending connection.
pub fn wire_complete_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut wire_state: ResMut<WireConnectionState>,
    registry: Res<NodeRegistry>,
    q_nodes: Query<&GraphNode>,
    q_connections: Query<&GraphConnection>,
    ports: Query<(Entity, &UInteraction, &GraphPort)>,
    mut mutations: ResMut<GraphMutationTracker>,
) {
    if !wire_state.is_dragging || !mouse_button.just_released(MouseButton::Left) {
        return;
    }

    if let (Some(from_port_entity), Some(from_index), Some(from_node)) = (
        wire_state.dragging_from,
        wire_state.index_from,
        wire_state.node_from,
    ) {
        let Ok((_, _, from_port_data)) = ports.get(from_port_entity) else {
            wire_state.clear();
            return;
        };
        let Ok(from_graph_node) = q_nodes.get(from_node) else {
            wire_state.clear();
            return;
        };
        let Some(from_definition) = registry.get(&from_graph_node.definition_id) else {
            wire_state.clear();
            return;
        };

        for (to_port_entity, interaction, port) in ports.iter() {
            if port.port_type != PortType::Input || port.node_entity == from_node {
                continue;
            }

            if !matches!(
                *interaction,
                UInteraction::Released
                    | UInteraction::Hovered
                    | UInteraction::Pressed
                    | UInteraction::Clicked
            ) {
                continue;
            }

            if !NodeValue::is_compatible(&from_port_data.value_type, &port.value_type) {
                warn!(
                    "Rejected link: incompatible types {} -> {}",
                    from_port_data.value_type.display_name(),
                    port.value_type.display_name()
                );
                break;
            }

            let Ok(to_graph_node) = q_nodes.get(port.node_entity) else {
                break;
            };
            let Some(to_definition) = registry.get(&to_graph_node.definition_id) else {
                break;
            };
            let to_inputs = to_definition.inputs();
            let Some(to_port_definition) = to_inputs.get(port.index) else {
                break;
            };
            let source_connected_inputs = connected_input_mask(
                from_graph_node.values.inputs.len(),
                q_connections
                    .iter()
                    .filter(|link| link.to_node == from_node)
                    .map(|link| link.to_index),
            );

            if !output_satisfies_requirement(
                &from_definition,
                from_index,
                &source_connected_inputs,
                to_port_definition.requirement.as_ref(),
            ) {
                let requirement = to_port_definition
                    .requirement
                    .as_ref()
                    .map(|requirement| requirement.label.as_str())
                    .unwrap_or("value");
                warn!(
                    "Rejected link: input '{}' on node {:?} requires '{}'",
                    to_port_definition.name, port.node_entity, requirement
                );
                break;
            }

            match to_port_definition.connection_policy {
                univis_node_graph::prelude::ConnectionPolicy::Single => {
                    if q_connections.iter().any(|link| link.to_port == to_port_entity) {
                        warn!(
                            "Rejected link: input port {} on node {:?} already connected",
                            port.index, port.node_entity
                        );
                        break;
                    }
                }
                univis_node_graph::prelude::ConnectionPolicy::Multiple => {
                    warn!(
                        "Rejected link: input '{}' on node {:?} declares a multiple-source policy, but the current runtime only supports one source per input.",
                        to_port_definition.name, port.node_entity
                    );
                    break;
                }
            }

            if would_create_cycle(
                q_connections.iter().map(|link| (link.from_node, link.to_node)),
                from_node,
                port.node_entity,
            ) {
                warn!(
                    "Rejected link: connecting node {:?} to node {:?} would create a cycle",
                    from_node, port.node_entity
                );
                break;
            }

            commands.spawn(GraphConnection {
                from_node,
                to_node: port.node_entity,
                from_index,
                to_index: port.index,
                from_port: from_port_entity,
                to_port: to_port_entity,
            });
            mutations.mark_changed();
            break;
        }
    }

    wire_state.clear();
}

pub fn wire_visuals_system(
    mut commands: Commands,
    q_links: Query<(Entity, &GraphConnection)>,
    q_changed_links: Query<Entity, Or<(Added<GraphConnection>, Changed<GraphConnection>)>>,
    mut removed_links: RemovedComponents<GraphConnection>,
    wire_state: Res<WireConnectionState>,
    connection_diagnostics: Res<GraphConnectionUiDiagnostics>,
    settings: Res<EditorSettings>,
    port_transforms: Query<(&GlobalTransform, &GraphPort)>,
    changed_ports: Query<
        (),
        (
            With<GraphPort>,
            Or<(Changed<GlobalTransform>, Changed<GraphPort>)>,
        ),
    >,
    existing_visuals: Query<Entity, With<WireVisualSegment>>,
) {
    // Refresh only when graph links, preview drag state, or visible port transforms actually change.
    let should_refresh = !q_changed_links.is_empty()
        || removed_links.read().next().is_some()
        || wire_state.is_changed()
        || connection_diagnostics.is_changed()
        || settings.is_changed()
        || !changed_ports.is_empty();
    if !should_refresh {
        return;
    }

    if q_links.is_empty() && !wire_state.is_dragging && existing_visuals.is_empty() {
        return;
    }

    for entity in existing_visuals.iter() {
        commands.entity(entity).try_despawn();
    }

    for (connection_entity, link) in q_links.iter() {
        let Ok((start_transform, from_port)) = port_transforms.get(link.from_port) else {
            continue;
        };
        let Ok((end_transform, _)) = port_transforms.get(link.to_port) else {
            continue;
        };
        let connection_info = connection_diagnostics.connections.get(&connection_entity);
        let focused = connection_diagnostics.focused_connection == Some(connection_entity);

        spawn_wire_segments(
            &mut commands,
            start_transform.translation().truncate(),
            end_transform.translation().truncate(),
            resolved_wire_color(&settings, from_port, connection_info, focused),
            resolved_wire_thickness(connection_info, focused),
            settings.wire_style,
        );
    }

    if !wire_state.is_dragging {
        return;
    }

    let Some(from_entity) = wire_state.dragging_from else {
        return;
    };
    let Ok((start_transform, from_port)) = port_transforms.get(from_entity) else {
        return;
    };

    spawn_wire_segments(
        &mut commands,
        start_transform.translation().truncate(),
        wire_state.current_mouse_world_pos,
        preview_wire_color(&settings, from_port),
        WIRE_SEGMENT_THICKNESS,
        settings.wire_style,
    );
}

fn preview_wire_color(settings: &EditorSettings, from_port: &GraphPort) -> Color {
    if settings.wire_color_from_output {
        from_port.value_type.port_color()
    } else {
        Color::srgba(1.0, 0.92, 0.35, 0.85)
    }
}

fn resolved_wire_color(
    settings: &EditorSettings,
    from_port: &GraphPort,
    connection_info: Option<&ConnectionDiagnosticInfo>,
    focused: bool,
) -> Color {
    if focused {
        return Color::srgba(0.74, 0.95, 1.0, 0.98);
    }

    match connection_info.map(|info| info.severity) {
        Some(UiDiagnosticSeverity::Warning) => Color::srgba(0.98, 0.68, 0.25, 0.92),
        Some(UiDiagnosticSeverity::Error) => Color::srgba(0.97, 0.32, 0.32, 0.95),
        _ => {
            if settings.wire_color_from_output {
                from_port.value_type.port_color()
            } else {
                Color::WHITE
            }
        }
    }
}

fn resolved_wire_thickness(
    connection_info: Option<&ConnectionDiagnosticInfo>,
    focused: bool,
) -> f32 {
    if focused {
        WIRE_SEGMENT_THICKNESS + 2.5
    } else {
        match connection_info.map(|info| info.severity) {
            Some(UiDiagnosticSeverity::Warning) => WIRE_SEGMENT_THICKNESS + 1.0,
            Some(UiDiagnosticSeverity::Error) => WIRE_SEGMENT_THICKNESS + 1.5,
            _ => WIRE_SEGMENT_THICKNESS,
        }
    }
}

fn spawn_wire_segments(
    commands: &mut Commands,
    start: Vec2,
    end: Vec2,
    color: Color,
    thickness: f32,
    style: WireStyle,
) {
    // The wire mesh is rebuilt as short sprites so styles can stay lightweight and z-ordered under nodes.
    let points = wire_points(start, end, style);
    for pair in points.windows(2) {
        let from = pair[0];
        let to = pair[1];
        let delta = to - from;
        let length = delta.length();
        if length <= f32::EPSILON {
            continue;
        }

        let center = (from + to) * 0.5;
        let rotation = Quat::from_rotation_z(delta.y.atan2(delta.x));

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(length, thickness)),
                ..default()
            },
            Transform {
                translation: center.extend(WIRE_SEGMENT_Z),
                rotation,
                ..default()
            },
            Pickable::IGNORE,
            WireVisualSegment,
        ));
    }
}

fn wire_points(start: Vec2, end: Vec2, style: WireStyle) -> Vec<Vec2> {
    match style {
        WireStyle::Bezier => bezier_points(start, end),
        WireStyle::Straight => vec![start, end],
        WireStyle::Stepped => stepped_points(start, end),
    }
}

fn bezier_points(start: Vec2, end: Vec2) -> Vec<Vec2> {
    let dist = (end.x - start.x).abs().max(50.0);
    let control_offset = dist * 0.5;
    let cp1 = start + Vec2::new(control_offset, 0.0);
    let cp2 = end - Vec2::new(control_offset, 0.0);
    let bezier = CubicBezier::new([[start, cp1, cp2, end]]);

    bezier
        .to_curve()
        .map(|curve| curve.iter_positions(BEZIER_SEGMENT_COUNT).collect())
        .unwrap_or_else(|_| vec![start, end])
}

fn stepped_points(start: Vec2, end: Vec2) -> Vec<Vec2> {
    let mid_x = start.x + ((end.x - start.x) * 0.5);
    vec![
        start,
        Vec2::new(mid_x, start.y),
        Vec2::new(mid_x, end.y),
        end,
    ]
}
