use bevy::prelude::*;
use univis_editor_runtime::{GraphResolvedInputs, NodeRuntimePlugin};
use univis_node_graph::prelude::{
    AuthoredNodeInputs, GraphConnection, GraphNode, NodeCategory, NodeDefinition, NodeId,
    NodeRegistry, NodeRegistryPlugin,
    PortDefinition, ProcessContext, ProcessResult,
};
use univis_node_graph::value::{NodeValue, ValueType};

struct RuntimeSourceNode;
struct RuntimeSinkNode;
struct RuntimeVisualSourceNode;

#[derive(Component)]
struct TestVisualValue(String);

impl NodeDefinition for RuntimeSourceNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/runtime_source")
    }

    fn display_name(&self) -> &str {
        "Runtime Source"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Value", ValueType::String)
            .with_default(NodeValue::string("default"))]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Output", ValueType::String)]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        ctx.set(0, ctx.inputs.first().cloned().unwrap_or(NodeValue::None));
        ProcessResult::Success
    }
}

impl NodeDefinition for RuntimeSinkNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/runtime_sink")
    }

    fn display_name(&self) -> &str {
        "Runtime Sink"
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

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        ctx.set(0, ctx.inputs.first().cloned().unwrap_or(NodeValue::None));
        ProcessResult::Success
    }
}

impl NodeDefinition for RuntimeVisualSourceNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/runtime_visual_source")
    }

    fn display_name(&self) -> &str {
        "Runtime Visual Source"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Output", ValueType::String)]
    }

    fn process(&self, _ctx: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }

    fn has_custom_body(&self) -> bool {
        true
    }

    fn sync_visual(&self, world: &mut World, node_entity: Entity) {
        let Some(value) = world.get::<TestVisualValue>(node_entity) else {
            return;
        };
        let next_value = NodeValue::string(&value.0);
        let Some(mut node) = world.get_mut::<GraphNode>(node_entity) else {
            return;
        };
        if node.values.outputs.first() != Some(&next_value) {
            node.values.outputs[0] = next_value;
        }
    }
}

#[test]
fn runtime_propagates_values_through_graph_connections() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<NodeRegistry>()
        .add_plugins(NodeRuntimePlugin);

    {
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        registry.register(RuntimeSourceNode);
        registry.register(RuntimeSinkNode);
        registry.register(RuntimeVisualSourceNode);
    }

    let source = app.world_mut().spawn(GraphNode::new(
        NodeId::new("tests/runtime_source"),
        1,
        1,
    ))
    .id();
    let sink = app.world_mut().spawn(GraphNode::new(
        NodeId::new("tests/runtime_sink"),
        1,
        1,
    ))
    .id();

    app.world_mut().spawn(GraphConnection {
        from_node: source,
        from_index: 0,
        to_node: sink,
        to_index: 0,
        from_port: Entity::PLACEHOLDER,
        to_port: Entity::PLACEHOLDER,
    });

    app.update();

    {
        let world = app.world_mut();
        let mut source_ref = world.entity_mut(source);
        source_ref
            .get_mut::<GraphNode>()
            .expect("source node")
            .values
            .inputs[0] = NodeValue::string("hello");
        source_ref
            .get_mut::<AuthoredNodeInputs>()
            .expect("authored inputs")
            .values[0] = NodeValue::string("hello");
    }

    app.update();
    app.update();

    let world = app.world_mut();
    let sink_node = world.entity(sink).get::<GraphNode>().expect("sink node");
    assert_eq!(sink_node.values.outputs.first(), Some(&NodeValue::string("hello")));

    let resolved = world
        .resource::<GraphResolvedInputs>()
        .by_node
        .get(&sink)
        .cloned()
        .expect("resolved inputs for sink");
    assert_eq!(resolved.first(), Some(&NodeValue::string("hello")));
}

#[test]
fn runtime_propagates_external_output_changes_through_graph_connections() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<NodeRegistry>()
        .add_plugins(NodeRuntimePlugin);

    {
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        registry.register(RuntimeVisualSourceNode);
        registry.register(RuntimeSinkNode);
    }

    let source = app
        .world_mut()
        .spawn(GraphNode::new(
            NodeId::new("tests/runtime_visual_source"),
            0,
            1,
        ))
        .id();
    let sink = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/runtime_sink"), 1, 1))
        .id();

    app.world_mut().spawn(GraphConnection {
        from_node: source,
        from_index: 0,
        to_node: sink,
        to_index: 0,
        from_port: Entity::PLACEHOLDER,
        to_port: Entity::PLACEHOLDER,
    });

    app.update();

    app.world_mut()
        .entity_mut(source)
        .get_mut::<GraphNode>()
        .expect("visual source node")
        .values
        .outputs[0] = NodeValue::string("from-visual");

    app.update();

    let world = app.world_mut();
    let sink_node = world.entity(sink).get::<GraphNode>().expect("sink node");
    assert_eq!(
        sink_node.values.outputs.first(),
        Some(&NodeValue::string("from-visual"))
    );
}

#[test]
fn runtime_propagates_polled_visual_changes_through_graph_connections() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<NodeRegistry>()
        .add_plugins(NodeRegistryPlugin)
        .add_plugins(NodeRuntimePlugin);

    {
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        registry.register(RuntimeVisualSourceNode);
        registry.register(RuntimeSinkNode);
    }

    let source = app
        .world_mut()
        .spawn((
            GraphNode::new(NodeId::new("tests/runtime_visual_source"), 0, 1),
            TestVisualValue("initial".to_string()),
        ))
        .id();
    let sink = app
        .world_mut()
        .spawn(GraphNode::new(NodeId::new("tests/runtime_sink"), 1, 1))
        .id();

    app.world_mut().spawn(GraphConnection {
        from_node: source,
        from_index: 0,
        to_node: sink,
        to_index: 0,
        from_port: Entity::PLACEHOLDER,
        to_port: Entity::PLACEHOLDER,
    });

    app.update();
    app.update();

    app.world_mut()
        .entity_mut(source)
        .get_mut::<TestVisualValue>()
        .expect("visual value component")
        .0 = "from-widget".to_string();

    app.update();
    app.update();

    let world = app.world_mut();
    let sink_node = world.entity(sink).get::<GraphNode>().expect("sink node");
    assert_eq!(
        sink_node.values.outputs.first(),
        Some(&NodeValue::string("from-widget"))
    );
}
