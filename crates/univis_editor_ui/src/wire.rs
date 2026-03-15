//! Wire interaction and rendering systems.
use std::collections::HashSet;

use crate::prelude::*;
use bevy::prelude::*;
use univis_ui::prelude::*;

use crate::editor::{EditorSettings, WireStyle};

const WIRE_SEGMENT_Z: f32 = 0.25;
const WIRE_SEGMENT_THICKNESS: f32 = 5.0;
const BEZIER_SEGMENT_COUNT: usize = 24;

#[derive(Component)]
pub struct WireVisualSegment;

#[derive(Resource, Debug, Clone, Default)]
pub struct WireDragFeedback {
    pub source_port: Option<Entity>,
    pub hovered_target: Option<Entity>,
    pub accepted_target: Option<Entity>,
    pub valid_targets: HashSet<Entity>,
    pub rejection_reason: Option<String>,
}

impl WireDragFeedback {
    pub fn clear(&mut self) {
        self.source_port = None;
        self.hovered_target = None;
        self.accepted_target = None;
        self.valid_targets.clear();
        self.rejection_reason = None;
    }

    pub fn is_active(&self) -> bool {
        self.source_port.is_some()
    }
}

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
    mut wire_feedback: ResMut<WireDragFeedback>,
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
                wire_feedback.clear();
                wire_feedback.source_port = Some(entity);
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
pub fn wire_drag_feedback_system(
    wire_state: Res<WireConnectionState>,
    registry: Res<NodeRegistry>,
    q_nodes: Query<&GraphNode>,
    q_connections: Query<&GraphConnection>,
    ports: Query<(Entity, &UInteraction, &GraphPort)>,
    mut wire_feedback: ResMut<WireDragFeedback>,
) {
    if !wire_state.is_dragging {
        if wire_feedback.is_active() {
            wire_feedback.clear();
        }
        return;
    }

    let (Some(from_port_entity), Some(from_index), Some(from_node)) = (
        wire_state.dragging_from,
        wire_state.index_from,
        wire_state.node_from,
    ) else {
        wire_feedback.clear();
        return;
    };

    let Ok((_, _, from_port_data)) = ports.get(from_port_entity) else {
        wire_feedback.clear();
        return;
    };
    let Ok(from_graph_node) = q_nodes.get(from_node) else {
        wire_feedback.clear();
        return;
    };
    let Some(from_definition) = registry.get(&from_graph_node.definition_id) else {
        wire_feedback.clear();
        return;
    };
    let source_connected_inputs = connected_input_mask(
        from_graph_node.values.inputs.len(),
        q_connections
            .iter()
            .filter(|link| link.to_node == from_node)
            .map(|link| link.to_index),
    );

    let mut next_feedback = WireDragFeedback {
        source_port: Some(from_port_entity),
        ..Default::default()
    };

    for (to_port_entity, interaction, port) in ports.iter() {
        if port.port_type != PortType::Input || port.node_entity == from_node {
            continue;
        }

        let evaluation = evaluate_wire_target(
            &registry,
            &q_nodes,
            &q_connections,
            from_port_entity,
            from_index,
            from_node,
            from_port_data,
            from_graph_node,
            &from_definition,
            &source_connected_inputs,
            to_port_entity,
            port,
        );

        if evaluation.is_ok() {
            next_feedback.valid_targets.insert(to_port_entity);
        }

        if !interaction_is_pointer_active(interaction) {
            continue;
        }

        match evaluation {
            Ok(()) => {
                next_feedback.accepted_target = Some(to_port_entity);
            }
            Err(reason) => {
                next_feedback.hovered_target = Some(to_port_entity);
                next_feedback.rejection_reason = Some(reason);
            }
        }
    }

    *wire_feedback = next_feedback;
}

pub fn wire_complete_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut wire_state: ResMut<WireConnectionState>,
    mut wire_feedback: ResMut<WireDragFeedback>,
    mut diagnostics: ResMut<GraphConnectionUiDiagnostics>,
    ports: Query<(Entity, &GraphPort)>,
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
        if let Some(to_port_entity) = wire_feedback.accepted_target {
            let Ok((_, port)) = ports.get(to_port_entity) else {
                wire_state.clear();
                wire_feedback.clear();
                return;
            };
            commands.spawn(GraphConnection {
                from_node,
                to_node: port.node_entity,
                from_index,
                to_index: port.index,
                from_port: from_port_entity,
                to_port: to_port_entity,
            });
            mutations.mark_changed();
            diagnostics.pinned_port = Some(to_port_entity);
            diagnostics.focused_port = Some(to_port_entity);
        } else if let (Some(target_port), Some(reason)) = (
            wire_feedback.hovered_target,
            wire_feedback.rejection_reason.as_deref(),
        ) {
            warn!("Rejected link: {}", reason);
            diagnostics.pinned_port = Some(target_port);
            diagnostics.focused_port = Some(target_port);
        }
    }

    wire_state.clear();
    wire_feedback.clear();
}

pub fn wire_visuals_system(
    mut commands: Commands,
    q_links: Query<(Entity, &GraphConnection)>,
    q_changed_links: Query<Entity, Or<(Added<GraphConnection>, Changed<GraphConnection>)>>,
    mut removed_links: RemovedComponents<GraphConnection>,
    wire_state: Res<WireConnectionState>,
    wire_feedback: Res<WireDragFeedback>,
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
        || wire_feedback.is_changed()
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
        preview_wire_color(&settings, from_port, &wire_feedback),
        preview_wire_thickness(&wire_feedback),
        settings.wire_style,
    );
}

fn preview_wire_color(
    settings: &EditorSettings,
    from_port: &GraphPort,
    wire_feedback: &WireDragFeedback,
) -> Color {
    if wire_feedback.accepted_target.is_some() {
        return Color::srgba(0.52, 0.96, 0.96, 0.96);
    }
    if wire_feedback.hovered_target.is_some() && wire_feedback.rejection_reason.is_some() {
        return Color::srgba(0.97, 0.34, 0.34, 0.94);
    }
    if settings.wire_color_from_output {
        from_port.value_type.port_color()
    } else {
        Color::srgba(1.0, 0.92, 0.35, 0.85)
    }
}

fn preview_wire_thickness(wire_feedback: &WireDragFeedback) -> f32 {
    if wire_feedback.accepted_target.is_some() {
        WIRE_SEGMENT_THICKNESS + 1.5
    } else if wire_feedback.hovered_target.is_some() && wire_feedback.rejection_reason.is_some() {
        WIRE_SEGMENT_THICKNESS + 0.75
    } else {
        WIRE_SEGMENT_THICKNESS
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

fn evaluate_wire_target(
    registry: &NodeRegistry,
    q_nodes: &Query<&GraphNode>,
    q_connections: &Query<&GraphConnection>,
    from_port_entity: Entity,
    from_index: usize,
    from_node: Entity,
    from_port_data: &GraphPort,
    from_graph_node: &GraphNode,
    from_definition: &ArcNodeDefinition,
    source_connected_inputs: &[bool],
    to_port_entity: Entity,
    port: &GraphPort,
) -> Result<(), String> {
    let _ = from_port_entity;

    if !NodeValue::is_compatible(&from_port_data.value_type, &port.value_type) {
        return Err(format!(
            "Incompatible types: {} -> {}",
            from_port_data.value_type.display_name(),
            port.value_type.display_name()
        ));
    }

    let to_graph_node = q_nodes
        .get(port.node_entity)
        .map_err(|_| "Target node is no longer available.".to_string())?;
    let to_definition = registry
        .get(&to_graph_node.definition_id)
        .ok_or_else(|| "Target node definition is missing.".to_string())?;
    let to_inputs = to_definition.inputs();
    let to_port_definition = to_inputs
        .get(port.index)
        .ok_or_else(|| "Target input definition is missing.".to_string())?;

    if !output_satisfies_requirement(
        from_definition,
        from_index,
        source_connected_inputs,
        to_port_definition.requirement.as_ref(),
    ) {
        let requirement = to_port_definition
            .requirement
            .as_ref()
            .map(|requirement| requirement.label.as_str())
            .unwrap_or("value");
        return Err(format!(
            "Input '{}' requires '{}'.",
            to_port_definition.name, requirement
        ));
    }

    match to_port_definition.connection_policy {
        univis_node_graph::prelude::ConnectionPolicy::Single => {
            if q_connections.iter().any(|link| link.to_port == to_port_entity) {
                return Err(format!(
                    "Input '{}' is already connected.",
                    to_port_definition.name
                ));
            }
        }
        univis_node_graph::prelude::ConnectionPolicy::Multiple => {
            return Err(format!(
                "Input '{}' declares a multiple-source policy, but runtime fan-in is not enabled yet.",
                to_port_definition.name
            ));
        }
    }

    if would_create_cycle(
        q_connections.iter().map(|link| (link.from_node, link.to_node)),
        from_node,
        port.node_entity,
    ) {
        return Err("This link would create a cycle.".to_string());
    }

    let _ = from_graph_node;
    Ok(())
}
