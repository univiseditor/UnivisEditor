use bevy::prelude::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use univis_editor_commands::{GraphCommandRequest, GraphCommandsPlugin};
use univis_editor_persistence::graph_persistence::{
    GraphPersistencePlugin, GraphPersistenceSettings, GraphPersistenceStatus,
};
use univis_editor_ui::menu::{execute_spawn_node_commands_system, ContextMenuState};
use univis_editor_ui::node_popup::NodePopupState;
use univis_editor_ui::prelude::sync_live_graph_document_state;
use univis_editor_workflows::GraphAssetWorkflowPlugin;
use univis_node_graph::commands::GraphMutationTracker;
use univis_node_graph::document::LiveGraphDocumentState;
use univis_node_graph::node_definition::{
    GraphNode, GraphPort, NodeCategory, NodeDefinition, NodeId, PortDefinition, PortType,
    ProcessContext, ProcessResult, Selected,
};
use univis_node_graph::node_registry::NodeRegistry;
use univis_node_graph::pin::{DragState, GraphConnection, WireConnectionState};
use univis_node_graph::value::{NodeValue, ValueType};
use univis_scene::EntityValue;

struct WorkflowValueNode;
struct WorkflowSinkNode;
struct WorkflowEntityNode;
struct WorkflowPrefabInstanceNode;

impl NodeDefinition for WorkflowValueNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/workflow_value")
    }

    fn display_name(&self) -> &str {
        "Workflow Value"
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
        if let Some(value) = ctx.inputs.first().cloned() {
            ctx.set(0, value);
        }
        ProcessResult::Success
    }
}

impl NodeDefinition for WorkflowSinkNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/workflow_sink")
    }

    fn display_name(&self) -> &str {
        "Workflow Sink"
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

impl NodeDefinition for WorkflowEntityNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/workflow_entity")
    }

    fn display_name(&self) -> &str {
        "Workflow Entity"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        Vec::new()
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_entity("Entity")]
    }

    fn process(&self, _ctx: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

impl NodeDefinition for WorkflowPrefabInstanceNode {
    fn id(&self) -> NodeId {
        NodeId::new("scene/prefab_instance")
    }

    fn display_name(&self) -> &str {
        "Prefab Instance"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Scene")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::input_string("Prefab Id")]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_entity("Entity")]
    }

    fn process(&self, _ctx: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<NodeRegistry>()
        .init_resource::<DragState>()
        .init_resource::<WireConnectionState>()
        .init_resource::<ContextMenuState>()
        .init_resource::<NodePopupState>()
        .add_plugins(GraphCommandsPlugin)
        .add_plugins(GraphPersistencePlugin)
        .add_plugins(GraphAssetWorkflowPlugin)
        .add_systems(Update, execute_spawn_node_commands_system)
        .add_systems(PostUpdate, sync_live_graph_document_state);

    {
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        registry.register(WorkflowValueNode);
        registry.register(WorkflowSinkNode);
        registry.register(WorkflowEntityNode);
        registry.register(WorkflowPrefabInstanceNode);
    }

    {
        let mut settings = app.world_mut().resource_mut::<GraphPersistenceSettings>();
        settings.autosave_enabled = false;
        settings.file_path = unique_temp_path("workflow_assets").to_string_lossy().into_owned();
        settings.backup_directory = unique_temp_path("workflow_assets_backups")
            .to_string_lossy()
            .into_owned();
    }

    app.update();
    app
}

fn unique_temp_path(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be available for test paths")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "univis_{}_{}_{}.json",
        label,
        std::process::id(),
        stamp
    ))
}

fn update_frames(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

fn spawn_node(app: &mut App, definition_id: &str, position: Vec2) -> Entity {
    app.world_mut()
        .write_message(GraphCommandRequest::SpawnNode {
            definition_id: NodeId::new(definition_id),
            position,
        })
        .expect("spawn request should enqueue");
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(Entity, &GraphNode, &Transform)>();
    query
        .iter(world)
        .find_map(|(entity, node, transform)| {
            (node.definition_id.as_str() == definition_id
                && transform.translation.truncate() == position)
                .then_some(entity)
        })
        .expect("spawned node should exist")
}

fn connect_nodes(app: &mut App, from_node: Entity, to_node: Entity) {
    let (from_port, to_port) = {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &GraphPort)>();
        let mut from_port = None;
        let mut to_port = None;

        for (entity, port) in query.iter(world) {
            if port.node_entity == from_node
                && port.port_type == PortType::Output
                && port.index == 0
            {
                from_port = Some(entity);
            }
            if port.node_entity == to_node && port.port_type == PortType::Input && port.index == 0 {
                to_port = Some(entity);
            }
        }

        (
            from_port.expect("source output port"),
            to_port.expect("target input port"),
        )
    };

    {
        app.world_mut().spawn(GraphConnection {
            from_node,
            from_index: 0,
            to_node,
            to_index: 0,
            from_port,
            to_port,
        });
    }
    app.world_mut()
        .resource_mut::<GraphMutationTracker>()
        .mark_changed();
    app.update();
}

fn set_output_entity(app: &mut App, entity: Entity, name: &str) {
    let root = EntityValue::named(name);
    {
        let world = app.world_mut();
        let mut entity_ref = world.entity_mut(entity);
        let mut node = entity_ref
            .get_mut::<GraphNode>()
            .expect("graph node should exist");
        node.values.outputs[0] = NodeValue::entity(root);
    }
}

fn select_nodes(app: &mut App, entities: &[Entity]) {
    let selected_set = entities.iter().copied().collect::<std::collections::HashSet<_>>();
    let existing = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<(Entity, Option<&Selected>), With<GraphNode>>();
        query
            .iter(world)
            .map(|(entity, selected)| (entity, selected.is_some()))
            .collect::<Vec<_>>()
    };

    for (entity, is_selected) in existing {
        let mut entity_ref = app.world_mut().entity_mut(entity);
        if selected_set.contains(&entity) {
            entity_ref.insert(Selected);
        } else if is_selected {
            entity_ref.remove::<Selected>();
        }
    }

    app.update();
}

fn node_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<&GraphNode>();
    query.iter(world).count()
}

fn edge_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<&GraphConnection>();
    query.iter(world).count()
}

#[test]
fn smoke_duplicate_selected_nodes_applies_snapshot_and_offsets_selection() {
    let mut app = build_test_app();
    let value_node = spawn_node(&mut app, "tests/workflow_value", Vec2::new(16.0, 24.0));
    let sink_node = spawn_node(&mut app, "tests/workflow_sink", Vec2::new(196.0, 24.0));
    connect_nodes(&mut app, value_node, sink_node);
    select_nodes(&mut app, &[value_node, sink_node]);

    app.world_mut()
        .write_message(GraphCommandRequest::DuplicateSelectedNodes)
        .expect("duplicate request should enqueue");
    update_frames(&mut app, 3);

    assert_eq!(node_count(&mut app), 4);
    assert_eq!(edge_count(&mut app), 2);

    let live_document = &app.world().resource::<LiveGraphDocumentState>().document;
    let mut selected_positions = live_document
        .selected_node_ids()
        .iter()
        .filter_map(|node_id| live_document.node(*node_id))
        .map(|node| node.position)
        .collect::<Vec<_>>();
    selected_positions.sort_by(|a, b| {
        a[0].partial_cmp(&b[0])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    assert_eq!(selected_positions.len(), 2);
    assert_eq!(selected_positions[0], [64.0, -24.0]);
    assert_eq!(selected_positions[1], [244.0, -24.0]);
}

#[test]
fn smoke_capture_prefab_and_subgraph_then_reinsert_latest_assets() {
    let mut app = build_test_app();

    let entity_node = spawn_node(&mut app, "tests/workflow_entity", Vec2::new(40.0, 40.0));
    set_output_entity(&mut app, entity_node, "Hero");
    select_nodes(&mut app, &[entity_node]);
    app.world_mut()
        .write_message(GraphCommandRequest::CapturePrefabFromSelection)
        .expect("prefab capture request should enqueue");
    app.update();

    let prefab_id = {
        let live_document = &app.world().resource::<LiveGraphDocumentState>().document;
        let prefab = live_document.prefabs.last().expect("prefab should be captured");
        assert_eq!(prefab.root.name.as_deref(), Some("Hero"));
        prefab.id.clone()
    };

    let upstream_node = spawn_node(&mut app, "tests/workflow_value", Vec2::new(-40.0, 180.0));
    let value_node = spawn_node(&mut app, "tests/workflow_value", Vec2::new(120.0, 180.0));
    let sink_node = spawn_node(&mut app, "tests/workflow_sink", Vec2::new(280.0, 180.0));
    connect_nodes(&mut app, upstream_node, value_node);
    connect_nodes(&mut app, value_node, sink_node);
    select_nodes(&mut app, &[value_node, sink_node]);
    app.world_mut()
        .write_message(GraphCommandRequest::CaptureSubgraphFromSelection)
        .expect("subgraph capture request should enqueue");
    app.update();

    {
        let live_document = &app.world().resource::<LiveGraphDocumentState>().document;
        let subgraph = live_document
            .subgraphs
            .last()
            .expect("subgraph should be captured");
        assert_eq!(subgraph.document.nodes.len(), 2);
        assert_eq!(subgraph.document.edges.len(), 1);
    }
    {
        let status = app.world().resource::<GraphPersistenceStatus>();
        let text = status
            .active
            .as_ref()
            .map(|status| status.text.clone())
            .unwrap_or_default();
        assert!(text.contains("1 internal wire"));
        assert!(text.contains("1 incoming"));
        assert!(text.contains("0 outgoing"));
    }

    app.world_mut()
        .write_message(GraphCommandRequest::InsertSubgraph {
            subgraph_id: String::new(),
            position: Vec2::new(420.0, 260.0),
        })
        .expect("insert subgraph request should enqueue");
    update_frames(&mut app, 3);

    assert_eq!(node_count(&mut app), 6);
    let live_document = &app.world().resource::<LiveGraphDocumentState>().document;
    assert_eq!(live_document.selected_node_ids().len(), 2);

    app.world_mut()
        .write_message(GraphCommandRequest::SpawnPrefabNode {
            prefab_id: String::new(),
            position: Vec2::new(540.0, 72.0),
        })
        .expect("spawn prefab request should enqueue");
    update_frames(&mut app, 2);

    let world = app.world_mut();
    let mut query = world.query::<&GraphNode>();
    let prefab_instance = query
        .iter(world)
        .find(|node| node.definition_id.as_str() == "scene/prefab_instance")
        .expect("prefab instance node should exist");
    assert_eq!(
        prefab_instance.values.inputs.first(),
        Some(&NodeValue::string(prefab_id))
    );
}
