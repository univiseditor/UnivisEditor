use bevy::prelude::*;
use univis_editor_ui::prelude::{
    wire_complete_system, wire_drag_feedback_system, GraphConnectionUiDiagnostics,
    WireDragFeedback,
};
use univis_node_graph::commands::GraphMutationTracker;
use univis_node_graph::pin::{GraphConnection, WireConnectionState};
use univis_node_graph::prelude::{
    GraphNode, GraphPort, InputConnection, NodeCategory, NodeDefinition, NodeId, NodeRegistry,
    OutputConnections, PortDefinition, PortType, ProcessContext, ProcessResult,
};
use univis_node_graph::value::ValueType;
use univis_ui::prelude::UInteraction;

struct WireSourceNode;
struct WireSinkNode;
struct WireChainNode;

impl NodeDefinition for WireSourceNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/wire_source")
    }

    fn display_name(&self) -> &str {
        "Wire Source"
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

    fn process(&self, _ctx: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

impl NodeDefinition for WireSinkNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/wire_sink")
    }

    fn display_name(&self) -> &str {
        "Wire Sink"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Input", ValueType::String)]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        Vec::new()
    }

    fn process(&self, _ctx: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

impl NodeDefinition for WireChainNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/wire_chain")
    }

    fn display_name(&self) -> &str {
        "Wire Chain"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Input", ValueType::String)]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Output", ValueType::String)]
    }

    fn process(&self, _ctx: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<NodeRegistry>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<WireConnectionState>()
        .init_resource::<WireDragFeedback>()
        .init_resource::<GraphConnectionUiDiagnostics>()
        .init_resource::<GraphMutationTracker>()
        .add_systems(Update, (wire_drag_feedback_system, wire_complete_system).chain());

    {
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        registry.register(WireSourceNode);
        registry.register(WireSinkNode);
        registry.register(WireChainNode);
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
            OutputConnections::default(),
            UInteraction::default(),
        ))
        .id()
}

fn spawn_input_port(world: &mut World, node_entity: Entity, hovered: bool) -> Entity {
    world
        .spawn((
            GraphPort {
                node_entity,
                port_type: PortType::Input,
                index: 0,
                value_type: ValueType::String,
            },
            InputConnection::default(),
            if hovered {
                UInteraction::Hovered
            } else {
                UInteraction::default()
            },
        ))
        .id()
}

fn spawn_input_port_at(
    world: &mut World,
    node_entity: Entity,
    hovered: bool,
    position: Vec2,
) -> Entity {
    world
        .spawn((
            GraphPort {
                node_entity,
                port_type: PortType::Input,
                index: 0,
                value_type: ValueType::String,
            },
            InputConnection::default(),
            GlobalTransform::from(Transform::from_translation(position.extend(0.0))),
            if hovered {
                UInteraction::Hovered
            } else {
                UInteraction::default()
            },
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

fn start_wire_drag_at(app: &mut App, from_port: Entity, from_node: Entity, cursor_world: Vec2) {
    let mut wire_state = app.world_mut().resource_mut::<WireConnectionState>();
    wire_state.dragging_from = Some(from_port);
    wire_state.node_from = Some(from_node);
    wire_state.index_from = Some(0);
    wire_state.current_mouse_world_pos = cursor_world;
    wire_state.is_dragging = true;
}

#[test]
fn wire_feedback_accepts_valid_hovered_input_and_pins_it_on_connect() {
    let mut app = build_test_app();

    let source = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_source"), 0, 1))
        .id();
    let sink = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_sink"), 1, 0))
        .id();
    let source_port = spawn_output_port(app.world_mut(), source);
    let sink_port = spawn_input_port(app.world_mut(), sink, true);

    start_wire_drag(&mut app, source_port, source);
    app.update();

    {
        let feedback = app.world().resource::<WireDragFeedback>();
        assert_eq!(feedback.accepted_target, Some(sink_port));
        assert!(feedback.valid_targets.contains(&sink_port));
    }

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let world = app.world_mut();
    let mut connections = world.query::<&GraphConnection>();
    assert_eq!(connections.iter(world).count(), 1);
    assert_eq!(
        world
            .resource::<GraphConnectionUiDiagnostics>()
            .pinned_port,
        Some(sink_port)
    );
}

#[test]
fn wire_complete_keeps_last_accepted_target_when_hover_drops_on_release() {
    let mut app = build_test_app();

    let source = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_source"), 0, 1))
        .id();
    let sink = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_sink"), 1, 0))
        .id();
    let source_port = spawn_output_port(app.world_mut(), source);
    let sink_port = spawn_input_port(app.world_mut(), sink, true);

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    start_wire_drag(&mut app, source_port, source);
    app.update();

    app.world_mut()
        .entity_mut(sink_port)
        .insert(UInteraction::default());

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let world = app.world_mut();
    let mut connections = world.query::<&GraphConnection>();
    assert_eq!(connections.iter(world).count(), 1);
}

#[test]
fn wire_feedback_rejects_hovered_input_that_is_already_connected() {
    let mut app = build_test_app();

    let source = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_source"), 0, 1))
        .id();
    let other_source = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_source"), 0, 1))
        .id();
    let sink = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_sink"), 1, 0))
        .id();
    let source_port = spawn_output_port(app.world_mut(), source);
    let other_source_port = spawn_output_port(app.world_mut(), other_source);
    let sink_port = spawn_input_port(app.world_mut(), sink, true);

    app.world_mut().spawn(GraphConnection {
        from_node: other_source,
        from_index: 0,
        to_node: sink,
        to_index: 0,
        from_port: other_source_port,
        to_port: sink_port,
    });

    start_wire_drag(&mut app, source_port, source);
    app.update();

    let feedback = app.world().resource::<WireDragFeedback>();
    assert_eq!(feedback.accepted_target, None);
    assert_eq!(feedback.hovered_target, Some(sink_port));
    assert!(
        feedback
            .rejection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("already connected"))
    );
}

#[test]
fn wire_feedback_rejects_cycle_creating_hovered_input() {
    let mut app = build_test_app();

    let node_a = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_chain"), 1, 1))
        .id();
    let node_b = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_chain"), 1, 1))
        .id();
    let a_output = spawn_output_port(app.world_mut(), node_a);
    let a_input = spawn_input_port(app.world_mut(), node_a, true);
    let b_output = spawn_output_port(app.world_mut(), node_b);
    let b_input = spawn_input_port(app.world_mut(), node_b, false);

    app.world_mut().spawn(GraphConnection {
        from_node: node_a,
        from_index: 0,
        to_node: node_b,
        to_index: 0,
        from_port: a_output,
        to_port: b_input,
    });

    start_wire_drag(&mut app, b_output, node_b);
    app.update();

    let feedback = app.world().resource::<WireDragFeedback>();
    assert_eq!(feedback.accepted_target, None);
    assert_eq!(feedback.hovered_target, Some(a_input));
    assert!(
        feedback
            .rejection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("cycle"))
    );
}

#[test]
fn wire_feedback_accepts_nearby_input_without_ui_hover_state() {
    let mut app = build_test_app();

    let source = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_source"), 0, 1))
        .id();
    let sink = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/wire_sink"), 1, 0))
        .id();
    let source_port = spawn_output_port(app.world_mut(), source);
    let sink_port = spawn_input_port_at(app.world_mut(), sink, false, Vec2::new(128.0, 64.0));

    start_wire_drag_at(&mut app, source_port, source, Vec2::new(130.0, 65.0));
    app.update();

    let feedback = app.world().resource::<WireDragFeedback>();
    assert_eq!(feedback.accepted_target, Some(sink_port));
    assert!(feedback.valid_targets.contains(&sink_port));
}
