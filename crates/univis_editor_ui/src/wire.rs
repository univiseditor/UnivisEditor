//! أنظمة الوصلات والأسلاك

use crate::prelude::*;
use bevy::prelude::*;
use univis_ui::prelude::*;

// تعريف Sets لترتيب تنفيذ الأنظمة
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

pub fn wire_start_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut wire_state: ResMut<WireConnectionState>,
    ports: Query<(Entity, &UInteraction, &GraphPort)>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        for (entity, interaction, port) in ports.iter() {
            if interaction_is_pointer_active(interaction) {
                // نتأكد أننا نبدأ من منفذ خروج (Output) فقط
                if port.port_type == PortType::Output {
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
}

/// نظام تحديث موقع الفأرة
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

/// نظام إنهاء الوصلة
pub fn wire_complete_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut wire_state: ResMut<WireConnectionState>,
    mut connect: ResMut<Connecting>,
    registry: Res<NodeRegistry>,
    q_nodes: Query<&GraphNode>,
    ports: Query<(Entity, &UInteraction, &GraphPort)>,
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
            wire_state.dragging_from = None;
            wire_state.node_from = None;
            wire_state.index_from = None;
            wire_state.is_dragging = false;
            return;
        };
        let Ok(from_graph_node) = q_nodes.get(from_node) else {
            wire_state.dragging_from = None;
            wire_state.node_from = None;
            wire_state.index_from = None;
            wire_state.is_dragging = false;
            return;
        };
        let Some(from_definition) = registry.get(&from_graph_node.definition_id) else {
            wire_state.dragging_from = None;
            wire_state.node_from = None;
            wire_state.index_from = None;
            wire_state.is_dragging = false;
            return;
        };

        for (to_port_entity, interaction, port) in ports.iter() {
            if port.port_type != PortType::Input {
                continue;
            }

            if port.node_entity == from_node {
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
                connect
                    .connections
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
                    to_port_definition.name,
                    port.node_entity,
                    requirement
                );
                break;
            }

            if connect
                .connections
                .iter()
                .any(|link| link.to_port == to_port_entity)
            {
                warn!(
                    "Rejected link: input port {} on node {:?} already connected",
                    port.index, port.node_entity
                );
                break;
            }

            if would_create_cycle(
                connect
                    .connections
                    .iter()
                    .map(|link| (link.from_node, link.to_node)),
                from_node,
                port.node_entity,
            ) {
                warn!(
                    "Rejected link: connecting node {:?} to node {:?} would create a cycle",
                    from_node, port.node_entity
                );
                break;
            }

            connect.connections.push(GraphLink {
                from_node,
                to_node: port.node_entity,
                from_index,
                to_index: port.index,
                from_port: from_port_entity,
                to_port: to_port_entity,
            });
            break;
        }
    }

    wire_state.dragging_from = None;
    wire_state.node_from = None;
    wire_state.index_from = None;
    wire_state.is_dragging = false;
}

/// رسم الخط المؤقت أثناء السحب
pub fn wire_preview_system(
    mut gizmos: Gizmos,
    wire_state: ResMut<WireConnectionState>,
    port_transforms: Query<&GlobalTransform, With<GraphPort>>,
) {
    if !wire_state.is_dragging {
        return;
    }

    if let Some(from_entity) = wire_state.dragging_from {
        if let Ok(start_transform) = port_transforms.get(from_entity) {
            let start = start_transform.translation().truncate();
            let end = wire_state.current_mouse_world_pos;
            let z = start_transform.translation().z;

            draw_bezier_wire(&mut gizmos, start, end, z, Color::srgba(1.0, 1.0, 0.0, 0.8));
        }
    }
}

/// دالة رسم خط Bezier
fn draw_bezier_wire(gizmos: &mut Gizmos, start: Vec2, end: Vec2, z: f32, color: Color) {
    let dist = (end.x - start.x).abs().max(50.0);
    let control_offset = dist * 0.5;

    let cp1 = start + Vec2::new(control_offset, 0.0);
    let cp2 = end - Vec2::new(control_offset, 0.0);

    let bezier = CubicBezier::new([[start, cp1, cp2, end]]);

    if let Ok(curve) = bezier.to_curve() {
        let points = curve.iter_positions(30).map(|p| Vec3::new(p.x, p.y, z));
        gizmos.linestrip(points, color);
    }
}

/// نظام رسم جميع الوصلات المكتملة
pub fn wire_render_system(
    mut gizmos: Gizmos,
    links: Res<Connecting>,
    port_transforms: Query<&GlobalTransform, With<GraphPort>>,
) {
    for link in &links.connections {
        let start_transform = port_transforms.get(link.from_port);
        let end_transform = port_transforms.get(link.to_port);

        if let (Ok(start), Ok(end)) = (start_transform, end_transform) {
            let start_pos = start.translation().truncate();
            let end_pos = end.translation().truncate();
            let z = start.translation().z;

            draw_bezier_wire(
                &mut gizmos,
                start_pos,
                end_pos,
                z,
                Color::srgb(1.0, 1.0, 1.0),
            );
        }
    }
}
