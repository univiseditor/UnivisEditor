use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use univis_editor_commands::GraphCommandsPlugin;
use univis_editor_persistence::graph_persistence::{
    ApplyGraphDocumentRequest, GraphPersistencePlugin, GraphPersistenceSettings,
    GraphPersistenceStatus, GraphPersistenceStatusSeverity,
};
use univis_editor_runtime::{GraphExecutableRuntimeState, NodeRuntimePlugin};
use univis_editor_ui::node_spawn::MissingNodePlaceholder;
use univis_editor_ui::prelude::{
    sync_live_graph_document_state, wire_drag_feedback_system, WireDragFeedback,
};
use univis_graph_core::prelude::{
    validate_graph_document, validate_structural_connection_candidate, GraphConnectionCandidate,
    GraphConnectionValidationOptions, GraphStructuralConnectionValidationContext,
    GraphValidationIssueKind,
};
use univis_node_graph::document::{
    GraphDocument, GraphDocumentEdge, GraphDocumentNode, LiveGraphDocumentState,
};
use univis_node_graph::graph_validation::LiveGraphValidationState;
use univis_node_graph::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_node_graph::node_registry::NodeRegistry;
use univis_node_graph::pin::{GraphConnection, WireConnectionState};
use univis_node_graph::prelude::{GraphNode, GraphPort, PortType};
use univis_node_graph::value::{NodeValue, ValueType};
use univis_ui::prelude::UInteraction;

struct ContractStringSourceNode;
struct ContractStringSinkNode;
struct ContractChainNode;
struct ContractMultiStringSinkNode;

impl NodeDefinition for ContractStringSourceNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/contract_string_source")
    }

    fn display_name(&self) -> &str {
        "Contract String Source"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        Vec::new()
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Output", ValueType::String)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        ctx.set(0, NodeValue::string("contract-source"));
        ProcessResult::Success
    }
}

impl NodeDefinition for ContractStringSinkNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/contract_string_sink")
    }

    fn display_name(&self) -> &str {
        "Contract String Sink"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::input_string("Input")]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        Vec::new()
    }

    fn process(&self, _ctx: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

impl NodeDefinition for ContractChainNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/contract_chain")
    }

    fn display_name(&self) -> &str {
        "Contract Chain"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::input_string("Input")]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Output", ValueType::String)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        if let Some(value) = ctx.inputs.first().cloned() {
            ctx.set(0, value);
        }
        ProcessResult::Success
    }
}

impl NodeDefinition for ContractMultiStringSinkNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/contract_multi_sink")
    }

    fn display_name(&self) -> &str {
        "Contract Multi Sink"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::input_string("Input").allow_multiple_connections()]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        Vec::new()
    }

    fn process(&self, _ctx: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

fn register_contract_nodes(registry: &mut NodeRegistry) {
    registry.register(ContractStringSourceNode);
    registry.register(ContractStringSinkNode);
    registry.register(ContractChainNode);
    registry.register(ContractMultiStringSinkNode);
}

fn node(
    id: u64,
    definition_id: &str,
    input_count: usize,
    output_count: usize,
) -> GraphDocumentNode {
    GraphDocumentNode {
        id,
        definition_id: NodeId::new(definition_id),
        position: [id as f32 * 32.0, id as f32 * 12.0],
        inputs: vec![NodeValue::None; input_count],
        input_count,
        output_count,
    }
}

fn cycle_document() -> GraphDocument {
    GraphDocument {
        nodes: vec![
            node(1, "tests/contract_chain", 1, 1),
            node(2, "tests/contract_chain", 1, 1),
        ],
        edges: vec![
            GraphDocumentEdge {
                from_node_id: 1,
                from_index: 0,
                to_node_id: 2,
                to_index: 0,
            },
            GraphDocumentEdge {
                from_node_id: 2,
                from_index: 0,
                to_node_id: 1,
                to_index: 0,
            },
        ],
        ..GraphDocument::default()
    }
}

fn single_input_conflict_document() -> GraphDocument {
    GraphDocument {
        nodes: vec![
            node(1, "tests/contract_string_source", 0, 1),
            node(2, "tests/contract_string_source", 0, 1),
            node(3, "tests/contract_string_sink", 1, 0),
        ],
        edges: vec![
            GraphDocumentEdge {
                from_node_id: 1,
                from_index: 0,
                to_node_id: 3,
                to_index: 0,
            },
            GraphDocumentEdge {
                from_node_id: 2,
                from_index: 0,
                to_node_id: 3,
                to_index: 0,
            },
        ],
        ..GraphDocument::default()
    }
}

fn unsupported_multiple_policy_document() -> GraphDocument {
    GraphDocument {
        nodes: vec![
            node(1, "tests/contract_string_source", 0, 1),
            node(2, "tests/contract_string_source", 0, 1),
            node(3, "tests/contract_multi_sink", 1, 0),
        ],
        edges: vec![
            GraphDocumentEdge {
                from_node_id: 1,
                from_index: 0,
                to_node_id: 3,
                to_index: 0,
            },
            GraphDocumentEdge {
                from_node_id: 2,
                from_index: 0,
                to_node_id: 3,
                to_index: 0,
            },
        ],
        ..GraphDocument::default()
    }
}

fn invalid_input_port_document() -> GraphDocument {
    GraphDocument {
        nodes: vec![
            node(1, "tests/contract_string_source", 0, 1),
            node(2, "tests/contract_string_sink", 1, 0),
        ],
        edges: vec![GraphDocumentEdge {
            from_node_id: 1,
            from_index: 0,
            to_node_id: 2,
            to_index: 1,
        }],
        ..GraphDocument::default()
    }
}

fn missing_definition_document() -> GraphDocument {
    GraphDocument {
        nodes: vec![node(1, "tests/missing_definition", 1, 1)],
        ..GraphDocument::default()
    }
}

fn core_issue_kinds(
    document: &GraphDocument,
    registry: &NodeRegistry,
) -> Vec<GraphValidationIssueKind> {
    validate_graph_document(document, registry.core_registry())
        .issues
        .into_iter()
        .map(|issue| issue.kind)
        .collect()
}

fn assert_has_issue(issue_kinds: &[GraphValidationIssueKind], expected: GraphValidationIssueKind) {
    assert!(
        issue_kinds.iter().any(|kind| *kind == expected),
        "expected issue {:?}, got {:?}",
        expected,
        issue_kinds
    );
}

fn build_ui_feedback_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<NodeRegistry>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<WireConnectionState>()
        .init_resource::<WireDragFeedback>()
        .add_systems(Update, wire_drag_feedback_system);

    {
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        register_contract_nodes(&mut registry);
    }

    app
}

fn spawn_output_port(world: &mut World, node_entity: Entity) -> Entity {
    world
        .spawn((
            GraphPort {
                node_entity,
                port_type: PortType::Output,
                index: 0,
                value_type: ValueType::String,
            },
            UInteraction::default(),
        ))
        .id()
}

fn spawn_hovered_input_port(world: &mut World, node_entity: Entity) -> Entity {
    world
        .spawn((
            GraphPort {
                node_entity,
                port_type: PortType::Input,
                index: 0,
                value_type: ValueType::String,
            },
            UInteraction::Hovered,
        ))
        .id()
}

fn start_wire_drag(app: &mut App, from_port: Entity, from_node: Entity) {
    let mut wire_state = app.world_mut().resource_mut::<WireConnectionState>();
    wire_state.dragging_from = Some(from_port);
    wire_state.node_from = Some(from_node);
    wire_state.index_from = Some(0);
    wire_state.is_dragging = true;
}

fn ui_rejection_reason_for_single_input_conflict() -> String {
    let mut app = build_ui_feedback_app();
    let source_a = app
        .world_mut()
        .spawn(GraphNode::new(
            NodeId::new("tests/contract_string_source"),
            0,
            1,
        ))
        .id();
    let source_b = app
        .world_mut()
        .spawn(GraphNode::new(
            NodeId::new("tests/contract_string_source"),
            0,
            1,
        ))
        .id();
    let sink = app
        .world_mut()
        .spawn(GraphNode::new(
            NodeId::new("tests/contract_string_sink"),
            1,
            0,
        ))
        .id();
    let source_a_port = spawn_output_port(app.world_mut(), source_a);
    let source_b_port = spawn_output_port(app.world_mut(), source_b);
    let sink_port = spawn_hovered_input_port(app.world_mut(), sink);

    app.world_mut().spawn(GraphConnection {
        from_node: source_a,
        from_index: 0,
        to_node: sink,
        to_index: 0,
        from_port: source_a_port,
        to_port: sink_port,
    });

    start_wire_drag(&mut app, source_b_port, source_b);
    app.update();

    app.world()
        .resource::<WireDragFeedback>()
        .rejection_reason
        .clone()
        .expect("single-input conflict should be rejected")
}

fn ui_rejection_reason_for_cycle() -> String {
    let mut app = build_ui_feedback_app();
    let node_a = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/contract_chain"), 1, 1))
        .id();
    let node_b = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/contract_chain"), 1, 1))
        .id();
    let a_output = spawn_output_port(app.world_mut(), node_a);
    let a_input = spawn_hovered_input_port(app.world_mut(), node_a);
    let b_output = spawn_output_port(app.world_mut(), node_b);
    let b_input = spawn_hovered_input_port(app.world_mut(), node_b);

    app.world_mut().spawn(GraphConnection {
        from_node: node_a,
        from_index: 0,
        to_node: node_b,
        to_index: 0,
        from_port: a_output,
        to_port: b_input,
    });

    start_wire_drag(&mut app, b_output, node_b);
    let _ = a_input;
    app.update();

    app.world()
        .resource::<WireDragFeedback>()
        .rejection_reason
        .clone()
        .expect("cycle should be rejected")
}

fn ui_rejection_reason_for_unsupported_multiple_policy() -> String {
    let mut app = build_ui_feedback_app();
    let source = app
        .world_mut()
        .spawn(GraphNode::new(
            NodeId::new("tests/contract_string_source"),
            0,
            1,
        ))
        .id();
    let sink = app
        .world_mut()
        .spawn(GraphNode::new(
            NodeId::new("tests/contract_multi_sink"),
            1,
            0,
        ))
        .id();
    let source_port = spawn_output_port(app.world_mut(), source);
    let _sink_port = spawn_hovered_input_port(app.world_mut(), sink);

    start_wire_drag(&mut app, source_port, source);
    app.update();

    app.world()
        .resource::<WireDragFeedback>()
        .rejection_reason
        .clone()
        .expect("unsupported multi-input policy should be rejected")
}

fn structural_error_message(
    candidate: GraphConnectionCandidate<Entity>,
    existing_edges: &[GraphConnectionCandidate<Entity>],
    target_accepts_multiple_connections: bool,
) -> String {
    validate_structural_connection_candidate(
        GraphStructuralConnectionValidationContext {
            candidate,
            source_output_count: 1,
            target_input_count: 1,
            source_output_available: true,
            target_input_available: true,
            target_accepts_multiple_connections,
        },
        existing_edges.iter().copied(),
        GraphConnectionValidationOptions::live_connection_rules(),
    )
    .map(|_| "connection unexpectedly accepted".to_string())
    .unwrap_err()
    .to_string()
}

fn build_persistence_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<NodeRegistry>()
        .add_plugins(GraphCommandsPlugin)
        .add_plugins(GraphPersistencePlugin)
        .add_systems(PostUpdate, sync_live_graph_document_state);

    {
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        register_contract_nodes(&mut registry);
    }

    {
        let mut settings = app.world_mut().resource_mut::<GraphPersistenceSettings>();
        settings.autosave_enabled = false;
        settings.file_path = unique_temp_path("contract_parity")
            .to_string_lossy()
            .into_owned();
        settings.backup_directory = unique_temp_path("contract_parity_backups")
            .to_string_lossy()
            .into_owned();
    }

    app.update();
    app
}

fn apply_document(app: &mut App, document: GraphDocument, label: &str) {
    app.world_mut()
        .write_message(ApplyGraphDocumentRequest {
            document,
            source_label: label.to_string(),
            track_for_undo: false,
            validation_report: None,
        })
        .expect("apply request should enqueue");
    update_frames(app, 2);
}

fn build_runtime_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<NodeRegistry>()
        .add_plugins(NodeRuntimePlugin);

    {
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        register_contract_nodes(&mut registry);
    }

    app
}

fn spawn_runtime_node(
    app: &mut App,
    definition_id: &str,
    input_count: usize,
    output_count: usize,
) -> Entity {
    app.world_mut()
        .spawn(GraphNode::new(
            NodeId::new(definition_id),
            input_count,
            output_count,
        ))
        .id()
}

fn spawn_runtime_connection(app: &mut App, from_node: Entity, to_node: Entity) {
    app.world_mut().spawn(GraphConnection {
        from_node,
        from_index: 0,
        to_node,
        to_index: 0,
        from_port: Entity::PLACEHOLDER,
        to_port: Entity::PLACEHOLDER,
    });
}

fn update_frames(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

fn unique_temp_path(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be available")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "univis_contract_{}_{}_{}.json",
        label,
        std::process::id(),
        stamp
    ))
}

fn active_status(app: &App) -> (&GraphPersistenceStatusSeverity, &str) {
    let status = app.world().resource::<GraphPersistenceStatus>();
    let active = status
        .active
        .as_ref()
        .expect("persistence status should be set");
    (&active.severity, active.text.as_str())
}

#[test]
fn contract_cycle_case_stays_aligned_across_validation_ui_persistence_and_runtime() {
    let mut registry = NodeRegistry::new();
    register_contract_nodes(&mut registry);

    let issue_kinds = core_issue_kinds(&cycle_document(), &registry);
    assert_has_issue(&issue_kinds, GraphValidationIssueKind::CycleDetected);

    let node_a = Entity::from_raw_u32(1).expect("entity index 1 should be valid");
    let node_b = Entity::from_raw_u32(2).expect("entity index 2 should be valid");
    let expected_ui_error = structural_error_message(
        GraphConnectionCandidate {
            from_node_id: node_b,
            from_index: 0,
            to_node_id: node_a,
            to_index: 0,
        },
        &[GraphConnectionCandidate {
            from_node_id: node_a,
            from_index: 0,
            to_node_id: node_b,
            to_index: 0,
        }],
        false,
    );
    assert_eq!(ui_rejection_reason_for_cycle(), expected_ui_error);

    let mut persistence_app = build_persistence_app();
    apply_document(&mut persistence_app, cycle_document(), "contract cycle");
    assert_eq!(
        persistence_app
            .world()
            .resource::<LiveGraphDocumentState>()
            .document
            .edges
            .len(),
        1
    );
    let persistence_issues = persistence_app
        .world()
        .resource::<LiveGraphValidationState>()
        .report
        .issues
        .iter()
        .map(|issue| issue.kind)
        .collect::<Vec<_>>();
    assert_has_issue(&persistence_issues, GraphValidationIssueKind::CycleDetected);
    let (severity, text) = active_status(&persistence_app);
    assert_eq!(*severity, GraphPersistenceStatusSeverity::Warning);
    assert!(text.contains("1 skipped link"));

    let mut runtime_app = build_runtime_app();
    let node_a = spawn_runtime_node(&mut runtime_app, "tests/contract_chain", 1, 1);
    let node_b = spawn_runtime_node(&mut runtime_app, "tests/contract_chain", 1, 1);
    spawn_runtime_connection(&mut runtime_app, node_a, node_b);
    spawn_runtime_connection(&mut runtime_app, node_b, node_a);
    update_frames(&mut runtime_app, 2);

    let runtime_state = runtime_app
        .world()
        .resource::<GraphExecutableRuntimeState>();
    let runtime_issue_kinds = runtime_state
        .graph
        .validation_report()
        .issues
        .iter()
        .map(|issue| issue.kind)
        .collect::<Vec<_>>();
    assert_has_issue(
        &runtime_issue_kinds,
        GraphValidationIssueKind::CycleDetected,
    );
    let blocked = runtime_state.blocked_entities();
    assert!(blocked.contains(&node_a));
    assert!(blocked.contains(&node_b));
}

#[test]
fn contract_single_input_case_stays_aligned_across_validation_ui_persistence_and_runtime() {
    let mut registry = NodeRegistry::new();
    register_contract_nodes(&mut registry);

    let issue_kinds = core_issue_kinds(&single_input_conflict_document(), &registry);
    assert_has_issue(
        &issue_kinds,
        GraphValidationIssueKind::InputAlreadyConnected,
    );

    let source_a = Entity::from_raw_u32(1).expect("entity index 1 should be valid");
    let source_b = Entity::from_raw_u32(2).expect("entity index 2 should be valid");
    let sink = Entity::from_raw_u32(3).expect("entity index 3 should be valid");
    let expected_ui_error = structural_error_message(
        GraphConnectionCandidate {
            from_node_id: source_b,
            from_index: 0,
            to_node_id: sink,
            to_index: 0,
        },
        &[GraphConnectionCandidate {
            from_node_id: source_a,
            from_index: 0,
            to_node_id: sink,
            to_index: 0,
        }],
        false,
    );
    assert_eq!(
        ui_rejection_reason_for_single_input_conflict(),
        expected_ui_error
    );

    let mut persistence_app = build_persistence_app();
    apply_document(
        &mut persistence_app,
        single_input_conflict_document(),
        "contract single-input conflict",
    );
    assert_eq!(
        persistence_app
            .world()
            .resource::<LiveGraphDocumentState>()
            .document
            .edges
            .len(),
        1
    );
    let persistence_issues = persistence_app
        .world()
        .resource::<LiveGraphValidationState>()
        .report
        .issues
        .iter()
        .map(|issue| issue.kind)
        .collect::<Vec<_>>();
    assert_has_issue(
        &persistence_issues,
        GraphValidationIssueKind::InputAlreadyConnected,
    );
    let (severity, text) = active_status(&persistence_app);
    assert_eq!(*severity, GraphPersistenceStatusSeverity::Warning);
    assert!(text.contains("1 skipped link"));

    let mut runtime_app = build_runtime_app();
    let source_a = spawn_runtime_node(&mut runtime_app, "tests/contract_string_source", 0, 1);
    let source_b = spawn_runtime_node(&mut runtime_app, "tests/contract_string_source", 0, 1);
    let sink = spawn_runtime_node(&mut runtime_app, "tests/contract_string_sink", 1, 0);
    spawn_runtime_connection(&mut runtime_app, source_a, sink);
    spawn_runtime_connection(&mut runtime_app, source_b, sink);
    update_frames(&mut runtime_app, 2);

    let runtime_state = runtime_app
        .world()
        .resource::<GraphExecutableRuntimeState>();
    let runtime_issue_kinds = runtime_state
        .graph
        .validation_report()
        .issues
        .iter()
        .map(|issue| issue.kind)
        .collect::<Vec<_>>();
    assert_has_issue(
        &runtime_issue_kinds,
        GraphValidationIssueKind::InputAlreadyConnected,
    );
}

#[test]
fn contract_multiple_input_policy_stays_aligned_across_validation_ui_persistence_and_runtime() {
    let mut registry = NodeRegistry::new();
    register_contract_nodes(&mut registry);

    let issue_kinds = core_issue_kinds(&unsupported_multiple_policy_document(), &registry);
    assert_has_issue(
        &issue_kinds,
        GraphValidationIssueKind::UnsupportedConnectionPolicy,
    );
    assert!(issue_kinds
        .iter()
        .all(|kind| *kind != GraphValidationIssueKind::InputAlreadyConnected));

    let source = Entity::from_raw_u32(1).expect("entity index 1 should be valid");
    let sink = Entity::from_raw_u32(2).expect("entity index 2 should be valid");
    let expected_ui_error = structural_error_message(
        GraphConnectionCandidate {
            from_node_id: source,
            from_index: 0,
            to_node_id: sink,
            to_index: 0,
        },
        &[],
        true,
    );
    assert_eq!(
        ui_rejection_reason_for_unsupported_multiple_policy(),
        expected_ui_error
    );

    let mut persistence_app = build_persistence_app();
    apply_document(
        &mut persistence_app,
        unsupported_multiple_policy_document(),
        "contract unsupported multiple policy",
    );
    assert_eq!(
        persistence_app
            .world()
            .resource::<LiveGraphDocumentState>()
            .document
            .edges
            .len(),
        0
    );
    let persistence_issues = persistence_app
        .world()
        .resource::<LiveGraphValidationState>()
        .report
        .issues
        .iter()
        .map(|issue| issue.kind)
        .collect::<Vec<_>>();
    assert_has_issue(
        &persistence_issues,
        GraphValidationIssueKind::UnsupportedConnectionPolicy,
    );
    let (severity, text) = active_status(&persistence_app);
    assert_eq!(*severity, GraphPersistenceStatusSeverity::Warning);
    assert!(text.contains("skipped link"));

    let mut runtime_app = build_runtime_app();
    let source_a = spawn_runtime_node(&mut runtime_app, "tests/contract_string_source", 0, 1);
    let source_b = spawn_runtime_node(&mut runtime_app, "tests/contract_string_source", 0, 1);
    let sink = spawn_runtime_node(&mut runtime_app, "tests/contract_multi_sink", 1, 0);
    spawn_runtime_connection(&mut runtime_app, source_a, sink);
    spawn_runtime_connection(&mut runtime_app, source_b, sink);
    update_frames(&mut runtime_app, 2);

    let runtime_state = runtime_app
        .world()
        .resource::<GraphExecutableRuntimeState>();
    let runtime_issue_kinds = runtime_state
        .graph
        .validation_report()
        .issues
        .iter()
        .map(|issue| issue.kind)
        .collect::<Vec<_>>();
    assert_has_issue(
        &runtime_issue_kinds,
        GraphValidationIssueKind::UnsupportedConnectionPolicy,
    );
}

#[test]
fn contract_missing_definitions_and_invalid_ports_surface_without_silent_apply_failures() {
    let mut registry = NodeRegistry::new();
    register_contract_nodes(&mut registry);

    let missing_issue_kinds = core_issue_kinds(&missing_definition_document(), &registry);
    assert_has_issue(
        &missing_issue_kinds,
        GraphValidationIssueKind::MissingNodeDefinition,
    );

    let mut missing_app = build_persistence_app();
    apply_document(
        &mut missing_app,
        missing_definition_document(),
        "contract missing definition",
    );
    let missing_live_issues = missing_app
        .world()
        .resource::<LiveGraphValidationState>()
        .report
        .issues
        .iter()
        .map(|issue| issue.kind)
        .collect::<Vec<_>>();
    assert_has_issue(
        &missing_live_issues,
        GraphValidationIssueKind::MissingNodeDefinition,
    );
    let placeholder_count = {
        let world = missing_app.world_mut();
        let mut query = world.query::<&MissingNodePlaceholder>();
        query.iter(world).count()
    };
    assert_eq!(placeholder_count, 1);
    let (missing_severity, missing_text) = active_status(&missing_app);
    assert_eq!(*missing_severity, GraphPersistenceStatusSeverity::Warning);
    assert!(missing_text.contains("1 placeholder node"));

    let invalid_issue_kinds = core_issue_kinds(&invalid_input_port_document(), &registry);
    assert_has_issue(
        &invalid_issue_kinds,
        GraphValidationIssueKind::InvalidInputPort,
    );

    let mut invalid_app = build_persistence_app();
    apply_document(
        &mut invalid_app,
        invalid_input_port_document(),
        "contract invalid input port",
    );
    let invalid_live_issues = invalid_app
        .world()
        .resource::<LiveGraphValidationState>()
        .report
        .issues
        .iter()
        .map(|issue| issue.kind)
        .collect::<Vec<_>>();
    assert_has_issue(
        &invalid_live_issues,
        GraphValidationIssueKind::InvalidInputPort,
    );
    assert_eq!(
        invalid_app
            .world()
            .resource::<LiveGraphDocumentState>()
            .document
            .edges
            .len(),
        0
    );
    let (invalid_severity, invalid_text) = active_status(&invalid_app);
    assert_eq!(*invalid_severity, GraphPersistenceStatusSeverity::Warning);
    assert!(invalid_text.contains("1 skipped link"));
}
