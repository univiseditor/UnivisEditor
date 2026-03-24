use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use univis_editor_runtime::{
    GraphExecutableRuntimeState, GraphRuntimeDiagnostics, GraphRuntimeIssueSeverity,
};
use univis_node_graph::prelude::{
    AuthoredNodeInputs, ConnectionPolicy, GraphConnection, GraphNode, GraphPort, InputConnection,
    LiveGraphDocumentState, NodeRegistry, NodeValue, OutputConnections, PortType,
};
use univis_ui::prelude::*;

use crate::editor::GraphCamera;
use crate::node_spawn::PortLabel;
use crate::wire::WireDragFeedback;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiDiagnosticSeverity {
    #[default]
    Neutral,
    Active,
    Warning,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct PortDiagnosticInfo {
    pub severity: UiDiagnosticSeverity,
    pub status: String,
    pub detail: String,
    pub preview: Option<PortPreviewValue>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionDiagnosticInfo {
    pub severity: UiDiagnosticSeverity,
    pub detail: String,
    pub preview: Option<PortPreviewValue>,
}

#[derive(Debug, Clone, Default)]
pub struct PortPreviewValue {
    pub text: String,
    pub swatch: Option<Color>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphConnectionUiDiagnostics {
    pub ports: HashMap<Entity, PortDiagnosticInfo>,
    pub connections: HashMap<Entity, ConnectionDiagnosticInfo>,
    pub hovered_port: Option<Entity>,
    pub pinned_port: Option<Entity>,
    pub focused_port: Option<Entity>,
    pub focused_connection: Option<Entity>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphConnectionInspectorSummary {
    pub title: String,
    pub text: String,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphPortPreviewSummary {
    pub visible: bool,
    pub screen_position: Vec2,
    pub title: String,
    pub subtitle: String,
    pub value_text: String,
    pub detail: String,
    pub swatch: Option<Color>,
}

pub fn refresh_connection_ui_diagnostics_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    registry: Res<NodeRegistry>,
    live_document: Res<LiveGraphDocumentState>,
    executable_state: Res<GraphExecutableRuntimeState>,
    runtime_diagnostics: Res<GraphRuntimeDiagnostics>,
    wire_feedback: Res<WireDragFeedback>,
    q_nodes: Query<(Entity, &GraphNode, Option<&AuthoredNodeInputs>)>,
    q_ports: Query<(
        Entity,
        &GraphPort,
        Option<&InputConnection>,
        Option<&OutputConnections>,
        &UInteraction,
    )>,
    q_connections: Query<(Entity, &GraphConnection)>,
    mut diagnostics: ResMut<GraphConnectionUiDiagnostics>,
) {
    let hovered_port = q_ports.iter().find_map(|(entity, _, _, _, interaction)| {
        matches!(
            *interaction,
            UInteraction::Pressed | UInteraction::Hovered | UInteraction::Clicked
        )
        .then_some(entity)
    });

    if mouse_button.just_pressed(MouseButton::Left) {
        diagnostics.pinned_port = hovered_port;
    }

    if diagnostics
        .pinned_port
        .is_some_and(|entity| q_ports.get(entity).is_err())
    {
        diagnostics.pinned_port = None;
    }

    diagnostics.hovered_port = hovered_port;
    diagnostics.focused_port = hovered_port.or(diagnostics.pinned_port);

    let blocked_nodes = executable_state
        .blocked_entities()
        .into_iter()
        .collect::<HashSet<_>>();
    let mut issue_by_node = HashMap::<Entity, (UiDiagnosticSeverity, String)>::new();
    let mut missing_inputs = HashSet::<(Entity, usize)>::new();

    for issue in &runtime_diagnostics.node_issues {
        let severity = match issue.severity {
            GraphRuntimeIssueSeverity::Warning => UiDiagnosticSeverity::Warning,
            GraphRuntimeIssueSeverity::Error => UiDiagnosticSeverity::Error,
        };

        issue_by_node
            .entry(issue.node)
            .and_modify(|existing| {
                if severity_rank(severity) > severity_rank(existing.0) {
                    *existing = (severity, issue.message.clone());
                }
            })
            .or_insert_with(|| (severity, issue.message.clone()));

        if let Some(index) = parse_missing_input_index(&issue.message) {
            missing_inputs.insert((issue.node, index));
        }
    }

    let node_labels = q_nodes
        .iter()
        .map(|(entity, node, _)| {
            (
                entity,
                format_node_label(&registry, &live_document, entity, &node.definition_id),
            )
        })
        .collect::<HashMap<_, _>>();

    diagnostics.ports.clear();
    diagnostics.connections.clear();

    for (entity, port, input_connection, output_connections, _) in q_ports.iter() {
        let node_label = node_labels
            .get(&port.node_entity)
            .cloned()
            .unwrap_or_else(|| format!("Node#{:?}", port.node_entity));
        let blocked = blocked_nodes.contains(&port.node_entity);
        let issue = issue_by_node.get(&port.node_entity);

        let info = match port.port_type {
            PortType::Input => {
                let resolved_preview = executable_state
                    .node_for_entity(port.node_entity)
                    .and_then(|node| node.resolved_inputs().get(port.index))
                    .and_then(preview_if_present);
                let authored_preview = q_nodes
                    .get(port.node_entity)
                    .ok()
                    .and_then(|(_, _, authored)| authored)
                    .and_then(|authored| authored.values.get(port.index))
                    .and_then(preview_if_present);

                if wire_feedback.accepted_target == Some(entity) {
                    PortDiagnosticInfo {
                        severity: UiDiagnosticSeverity::Active,
                        status: "Accepts connection".to_string(),
                        detail: "Compatible input for the current wire drag.".to_string(),
                        preview: resolved_preview.or(authored_preview),
                    }
                } else if wire_feedback.hovered_target == Some(entity) {
                    PortDiagnosticInfo {
                        severity: UiDiagnosticSeverity::Error,
                        status: "Rejected".to_string(),
                        detail: wire_feedback.rejection_reason.clone().unwrap_or_else(|| {
                            "This input cannot accept the current wire.".to_string()
                        }),
                        preview: resolved_preview.or(authored_preview),
                    }
                } else if wire_feedback.valid_targets.contains(&entity) {
                    PortDiagnosticInfo {
                        severity: UiDiagnosticSeverity::Active,
                        status: "Valid target".to_string(),
                        detail: "Ready to accept the current wire drag.".to_string(),
                        preview: resolved_preview.or(authored_preview),
                    }
                } else if blocked {
                    PortDiagnosticInfo {
                        severity: UiDiagnosticSeverity::Warning,
                        status: "Blocked".to_string(),
                        detail: "Blocked by a cycle or dependency path.".to_string(),
                        preview: resolved_preview.or(authored_preview),
                    }
                } else if let Some(connection) =
                    input_connection.and_then(|connection| connection.connection_entity)
                {
                    let source_label = input_connection
                        .and_then(|connection| connection.source_node)
                        .and_then(|entity| node_labels.get(&entity).cloned())
                        .unwrap_or_else(|| "Unknown source".to_string());
                    PortDiagnosticInfo {
                        severity: issue
                            .map(|(severity, _)| *severity)
                            .unwrap_or(UiDiagnosticSeverity::Active),
                        status: "Connected".to_string(),
                        detail: format!("Resolved from {source_label}."),
                        preview: diagnostics
                            .connections
                            .get(&connection)
                            .and_then(|info| info.preview.clone())
                            .or(resolved_preview),
                    }
                } else if missing_inputs.contains(&(port.node_entity, port.index)) {
                    PortDiagnosticInfo {
                        severity: UiDiagnosticSeverity::Warning,
                        status: "Missing input".to_string(),
                        detail: format!("{node_label} is missing a required source here."),
                        preview: authored_preview,
                    }
                } else if let Some((severity, message)) = issue {
                    PortDiagnosticInfo {
                        severity: *severity,
                        status: "Node issue".to_string(),
                        detail: message.clone(),
                        preview: authored_preview,
                    }
                } else if let Some(preview) = authored_preview {
                    PortDiagnosticInfo {
                        severity: UiDiagnosticSeverity::Active,
                        status: "Authored".to_string(),
                        detail: "Using the node's authored value.".to_string(),
                        preview: Some(preview),
                    }
                } else {
                    PortDiagnosticInfo {
                        severity: UiDiagnosticSeverity::Neutral,
                        status: "Open".to_string(),
                        detail: "No source connected.".to_string(),
                        preview: None,
                    }
                }
            }
            PortType::Output => {
                let preview = executable_state
                    .node_for_entity(port.node_entity)
                    .and_then(|node| node.outputs().get(port.index))
                    .and_then(preview_if_present);
                let target_count = output_connections
                    .map(|connections| connections.targets.len())
                    .unwrap_or(0);

                if blocked {
                    PortDiagnosticInfo {
                        severity: UiDiagnosticSeverity::Warning,
                        status: "Blocked".to_string(),
                        detail: "This node is currently blocked.".to_string(),
                        preview,
                    }
                } else if let Some((severity, message)) = issue {
                    PortDiagnosticInfo {
                        severity: *severity,
                        status: "Node issue".to_string(),
                        detail: message.clone(),
                        preview,
                    }
                } else if target_count > 0 {
                    PortDiagnosticInfo {
                        severity: UiDiagnosticSeverity::Active,
                        status: "Feeding graph".to_string(),
                        detail: format!("{target_count} downstream target(s)."),
                        preview,
                    }
                } else {
                    PortDiagnosticInfo {
                        severity: UiDiagnosticSeverity::Neutral,
                        status: "Idle".to_string(),
                        detail: "No downstream targets yet.".to_string(),
                        preview,
                    }
                }
            }
        };

        diagnostics.ports.insert(entity, info);
    }

    for (entity, connection) in q_connections.iter() {
        let source_label = node_labels
            .get(&connection.from_node)
            .cloned()
            .unwrap_or_else(|| format!("Node#{:?}", connection.from_node));
        let target_label = node_labels
            .get(&connection.to_node)
            .cloned()
            .unwrap_or_else(|| format!("Node#{:?}", connection.to_node));
        let preview = executable_state
            .node_for_entity(connection.from_node)
            .and_then(|node| node.outputs().get(connection.from_index))
            .and_then(preview_if_present);

        let info = if blocked_nodes.contains(&connection.from_node)
            || blocked_nodes.contains(&connection.to_node)
        {
            ConnectionDiagnosticInfo {
                severity: UiDiagnosticSeverity::Warning,
                detail: format!("{source_label} -> {target_label} is blocked."),
                preview,
            }
        } else if let Some((severity, message)) = issue_by_node.get(&connection.to_node) {
            ConnectionDiagnosticInfo {
                severity: *severity,
                detail: format!("{source_label} -> {target_label}: {message}"),
                preview,
            }
        } else if preview.is_none() {
            ConnectionDiagnosticInfo {
                severity: UiDiagnosticSeverity::Warning,
                detail: format!("{source_label} -> {target_label}: source output is unresolved."),
                preview: None,
            }
        } else {
            ConnectionDiagnosticInfo {
                severity: UiDiagnosticSeverity::Active,
                detail: format!("{source_label} -> {target_label}"),
                preview,
            }
        };

        diagnostics.connections.insert(entity, info);
    }

    diagnostics.focused_connection = diagnostics
        .focused_port
        .and_then(|port_entity| q_ports.get(port_entity).ok())
        .and_then(
            |(_, port, input_connection, output_connections, _)| match port.port_type {
                PortType::Input => {
                    input_connection.and_then(|connection| connection.connection_entity)
                }
                PortType::Output => output_connections.and_then(|connections| {
                    if connections.targets.len() == 1 {
                        connections
                            .targets
                            .first()
                            .map(|target| target.connection_entity)
                    } else {
                        None
                    }
                }),
            },
        );
}

pub fn sync_port_diagnostic_visuals_system(
    diagnostics: Res<GraphConnectionUiDiagnostics>,
    wire_feedback: Res<WireDragFeedback>,
    mut q_ports: Query<(Entity, &GraphPort, &mut UNode, &mut UBorder)>,
    mut q_labels: Query<(&PortLabel, &mut UTextLabel)>,
) {
    if !diagnostics.is_changed() && !wire_feedback.is_changed() {
        return;
    }

    for (entity, port, mut node, mut border) in q_ports.iter_mut() {
        let info = diagnostics.ports.get(&entity).cloned().unwrap_or_default();
        let focused = diagnostics.focused_port == Some(entity);
        let mut palette = palette_for_port(port.value_type.port_color(), info.severity, focused);

        if wire_feedback.valid_targets.contains(&entity) {
            palette.size += 2.0;
            palette.border_width += 0.75;
            palette.border = Color::srgba(0.62, 0.96, 1.0, 0.98);
            palette.label = Color::srgba(0.90, 0.98, 1.0, 1.0);
        } else if wire_feedback.hovered_target == Some(entity)
            && wire_feedback.rejection_reason.is_some()
        {
            palette.border = Color::srgba(0.98, 0.52, 0.52, 0.98);
            palette.fill = Color::srgba(0.62, 0.14, 0.14, 0.96);
            palette.label = Color::srgba(1.0, 0.86, 0.86, 1.0);
            palette.border_width += 0.75;
        }

        node.width = UVal::Px(palette.size);
        node.height = UVal::Px(palette.size);
        node.background_color = palette.fill;
        node.border_radius = UCornerRadius::all(palette.size * 0.5);
        border.color = palette.border;
        border.width = palette.border_width;
        border.radius = UCornerRadius::all(palette.size * 0.5);

        for (label, mut text) in q_labels.iter_mut() {
            if label.port_entity == entity {
                text.color = palette.label;
            }
        }
    }
}

pub fn sync_connection_inspector_summary_system(
    diagnostics: Res<GraphConnectionUiDiagnostics>,
    registry: Res<NodeRegistry>,
    live_document: Res<LiveGraphDocumentState>,
    q_nodes: Query<&GraphNode>,
    q_ports: Query<(
        Entity,
        &GraphPort,
        Option<&InputConnection>,
        Option<&OutputConnections>,
    )>,
    mut summary: ResMut<GraphConnectionInspectorSummary>,
) {
    if !diagnostics.is_changed() && !registry.is_changed() && !live_document.is_changed() {
        return;
    }

    let Some(port_entity) = diagnostics.focused_port else {
        summary.title = "Connection Inspector".to_string();
        summary.text = "Hover a port or click one to pin its connection details.".to_string();
        return;
    };

    let Ok((_, port, input_connection, output_connections)) = q_ports.get(port_entity) else {
        summary.title = "Connection Inspector".to_string();
        summary.text = "Focused port is no longer available.".to_string();
        return;
    };

    let node_label = q_nodes
        .get(port.node_entity)
        .ok()
        .map(|node| {
            format_node_label(
                &registry,
                &live_document,
                port.node_entity,
                &node.definition_id,
            )
        })
        .unwrap_or_else(|| format!("Node#{:?}", port.node_entity));
    let port_kind = match port.port_type {
        PortType::Input => "Input",
        PortType::Output => "Output",
    };
    let port_status = diagnostics
        .ports
        .get(&port_entity)
        .cloned()
        .unwrap_or_default();

    let mut lines = vec![
        format!("{port_kind} {} on {node_label}", port.index + 1),
        format!("Type: {}", port.value_type.display_name()),
        format!("Status: {}", port_status.status),
        port_status.detail,
    ];

    if port.port_type == PortType::Input {
        let policy_label = q_nodes
            .get(port.node_entity)
            .ok()
            .and_then(|node| registry.get(&node.definition_id))
            .and_then(|definition| definition.inputs().get(port.index).cloned())
            .map(|definition| match definition.connection_policy() {
                ConnectionPolicy::Single => "single source".to_string(),
                ConnectionPolicy::Multiple => {
                    "multiple sources (currently unsupported)".to_string()
                }
            });
        if let Some(policy_label) = policy_label {
            lines.push(format!("Policy: {policy_label}"));
        }
    }

    if let Some(preview) = port_status.preview {
        lines.push(format!("Value: {}", preview.text));
    }

    match port.port_type {
        PortType::Input => {
            if let Some(connection_entity) =
                input_connection.and_then(|connection| connection.connection_entity)
            {
                if let Some(info) = diagnostics.connections.get(&connection_entity) {
                    lines.push(format!("Wire: {}", info.detail));
                    if let Some(preview) = &info.preview {
                        lines.push(format!("Wire value: {}", preview.text));
                    }
                }
            }
        }
        PortType::Output => {
            if let Some(connections) = output_connections {
                if connections.targets.is_empty() {
                    lines.push("Targets: none".to_string());
                } else {
                    let mut target_labels = connections
                        .targets
                        .iter()
                        .take(4)
                        .filter_map(|target| {
                            q_nodes.get(target.target_node).ok().map(|node| {
                                format_node_label(
                                    &registry,
                                    &live_document,
                                    target.target_node,
                                    &node.definition_id,
                                )
                            })
                        })
                        .collect::<Vec<_>>();
                    if connections.targets.len() > 4 {
                        target_labels.push(format!("+{}", connections.targets.len() - 4));
                    }
                    lines.push(format!("Targets: {}", target_labels.join(", ")));
                }
            }
        }
    }

    summary.title = "Connection Inspector".to_string();
    summary.text = lines.join("\n");
}

pub fn sync_port_preview_summary_system(
    diagnostics: Res<GraphConnectionUiDiagnostics>,
    registry: Res<NodeRegistry>,
    live_document: Res<LiveGraphDocumentState>,
    windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    q_nodes: Query<&GraphNode>,
    q_ports: Query<(Entity, &GraphPort, &GlobalTransform)>,
    mut summary: ResMut<GraphPortPreviewSummary>,
) {
    if !diagnostics.is_changed() && !registry.is_changed() && !live_document.is_changed() {
        return;
    }

    let Some(port_entity) = diagnostics.focused_port else {
        *summary = GraphPortPreviewSummary::default();
        return;
    };

    let Ok((_, port, transform)) = q_ports.get(port_entity) else {
        *summary = GraphPortPreviewSummary::default();
        return;
    };
    let Ok((camera, camera_transform)) = q_camera.single() else {
        *summary = GraphPortPreviewSummary::default();
        return;
    };
    let Ok(window) = windows.single() else {
        *summary = GraphPortPreviewSummary::default();
        return;
    };
    let Ok(screen_pos) = camera.world_to_viewport(camera_transform, transform.translation()) else {
        *summary = GraphPortPreviewSummary::default();
        return;
    };

    let node_label = q_nodes
        .get(port.node_entity)
        .ok()
        .map(|node| {
            format_node_label(
                &registry,
                &live_document,
                port.node_entity,
                &node.definition_id,
            )
        })
        .unwrap_or_else(|| format!("Node#{:?}", port.node_entity));
    let port_kind = match port.port_type {
        PortType::Input => "Input",
        PortType::Output => "Output",
    };
    let status = diagnostics
        .ports
        .get(&port_entity)
        .cloned()
        .unwrap_or_default();
    let preview = status.preview.clone();
    let max_left = (window.width() - 280.0).max(8.0);
    let max_top = (window.height() - 160.0).max(8.0);
    let screen_position = Vec2::new(
        (screen_pos.x + 18.0).clamp(8.0, max_left),
        (window.height() - screen_pos.y + 14.0).clamp(8.0, max_top),
    );

    *summary = GraphPortPreviewSummary {
        visible: true,
        screen_position,
        title: format!("{port_kind} {} on {node_label}", port.index + 1),
        subtitle: format!("{} · {}", port.value_type.display_name(), status.status),
        value_text: preview
            .as_ref()
            .map(|preview| preview.text.clone())
            .unwrap_or_else(|| "No resolved value yet.".to_string()),
        detail: status.detail,
        swatch: preview.and_then(|preview| preview.swatch),
    };
}

fn parse_missing_input_index(message: &str) -> Option<usize> {
    message
        .strip_prefix("Missing input: ")
        .and_then(|index| index.parse::<usize>().ok())
}

fn format_node_label(
    registry: &NodeRegistry,
    live_document: &LiveGraphDocumentState,
    entity: Entity,
    definition_id: &univis_node_graph::prelude::NodeId,
) -> String {
    let display_name = registry
        .get(definition_id)
        .map(|definition| definition.display_name().to_string())
        .unwrap_or_else(|| definition_id.as_str().to_string());

    if let Some(node_id) = live_document.node_id_for_entity(entity) {
        format!("{display_name}#{node_id}")
    } else {
        display_name
    }
}

fn preview_if_present(value: &NodeValue) -> Option<PortPreviewValue> {
    (!value.is_none()).then(|| PortPreviewValue {
        text: format_node_value_preview(value),
        swatch: match value {
            NodeValue::Color(color) => Some(*color),
            _ => None,
        },
    })
}

fn format_node_value_preview(value: &NodeValue) -> String {
    match value {
        NodeValue::Float(number) => format!("{number:.2}"),
        NodeValue::Int(number) => number.to_string(),
        NodeValue::Bool(value) => value.to_string(),
        NodeValue::String(text) => format!("\"{}\"", truncate(text, 24)),
        NodeValue::Vec2(value) => format!("vec2({:.2}, {:.2})", value.x, value.y),
        NodeValue::Vec3(value) => format!("vec3({:.2}, {:.2}, {:.2})", value.x, value.y, value.z),
        NodeValue::Vec4(value) => {
            format!(
                "vec4({:.2}, {:.2}, {:.2}, {:.2})",
                value.x, value.y, value.z, value.w
            )
        }
        NodeValue::Color(color) => {
            let srgba = color.to_srgba();
            format!(
                "rgba({:.2}, {:.2}, {:.2}, {:.2})",
                srgba.red, srgba.green, srgba.blue, srgba.alpha
            )
        }
        NodeValue::Entity(entity) => format!(
            "Entity(children: {}, components: {})",
            entity.children.len(),
            entity.components.len()
        ),
        NodeValue::TaggedData { tag, .. } => format!("tag:{tag}"),
        NodeValue::None => "None".to_string(),
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn severity_rank(severity: UiDiagnosticSeverity) -> u8 {
    match severity {
        UiDiagnosticSeverity::Neutral => 0,
        UiDiagnosticSeverity::Active => 1,
        UiDiagnosticSeverity::Warning => 2,
        UiDiagnosticSeverity::Error => 3,
    }
}

#[derive(Clone, Copy)]
struct PortVisualPalette {
    fill: Color,
    border: Color,
    label: Color,
    size: f32,
    border_width: f32,
}

fn palette_for_port(
    base: Color,
    severity: UiDiagnosticSeverity,
    focused: bool,
) -> PortVisualPalette {
    let base = base.to_srgba();
    let fill = match severity {
        UiDiagnosticSeverity::Neutral => {
            Color::srgba(base.red * 0.55, base.green * 0.55, base.blue * 0.55, 0.78)
        }
        UiDiagnosticSeverity::Active => Color::srgba(base.red, base.green, base.blue, 0.96),
        UiDiagnosticSeverity::Warning => Color::srgba(0.96, 0.63, 0.24, 0.96),
        UiDiagnosticSeverity::Error => Color::srgba(0.95, 0.28, 0.28, 0.98),
    };
    let border = if focused {
        Color::srgba(0.72, 0.94, 1.0, 0.98)
    } else {
        match severity {
            UiDiagnosticSeverity::Neutral => Color::srgba(1.0, 1.0, 1.0, 0.12),
            UiDiagnosticSeverity::Active => Color::srgba(1.0, 1.0, 1.0, 0.35),
            UiDiagnosticSeverity::Warning => Color::srgba(1.0, 0.86, 0.34, 0.95),
            UiDiagnosticSeverity::Error => Color::srgba(1.0, 0.62, 0.62, 0.98),
        }
    };
    let label = if focused {
        Color::srgba(0.92, 0.98, 1.0, 1.0)
    } else {
        match severity {
            UiDiagnosticSeverity::Neutral => Color::srgba(0.68, 0.68, 0.72, 1.0),
            UiDiagnosticSeverity::Active => Color::WHITE,
            UiDiagnosticSeverity::Warning => Color::srgba(1.0, 0.86, 0.5, 1.0),
            UiDiagnosticSeverity::Error => Color::srgba(1.0, 0.72, 0.72, 1.0),
        }
    };

    PortVisualPalette {
        fill,
        border,
        label,
        size: if focused { 15.0 } else { 12.0 },
        border_width: if focused { 2.0 } else { 1.0 },
    }
}
