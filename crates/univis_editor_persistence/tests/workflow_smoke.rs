use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use univis_editor_commands::{GraphCommandRequest, GraphCommandsPlugin};
use univis_editor_persistence::format::{parse_graph_document_payload, parse_graph_save_metadata};
use univis_editor_persistence::graph_persistence::{
    ApplyGraphDocumentRequest, GraphHistoryState, GraphPersistencePlugin, GraphPersistenceRuntimeState,
    GraphPersistenceSettings, GraphPersistenceStatus, GraphPersistenceStatusSeverity,
};
use univis_editor_ui::menu::ContextMenuState;
use univis_editor_ui::menu::execute_spawn_node_commands_system;
use univis_editor_ui::overlay::GraphOverlayState;
use univis_editor_ui::prelude::sync_live_graph_document_state;
use univis_node_graph::commands::GraphMutationTracker;
use univis_node_graph::document::{
    GraphDocument, GraphDocumentEdge, GraphDocumentNode, LiveGraphDocumentState,
};
use univis_node_graph::node_definition::{
    AuthoredNodeInputs, GraphNode, GraphPort, NodeCategory, NodeDefinition, NodeId, PortDefinition,
    PortType, ProcessContext, ProcessResult, Selected,
};
use univis_node_graph::node_registry::NodeRegistry;
use univis_node_graph::pin::{DragState, GraphConnection, WireConnectionState};
use univis_node_graph::value::{NodeValue, ValueType};

struct WorkflowValueNode;
struct WorkflowSinkNode;

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
        vec![
            PortDefinition::new("Value", ValueType::String)
                .with_default(NodeValue::string("default")),
        ]
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

fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<NodeRegistry>()
        .init_resource::<DragState>()
        .init_resource::<WireConnectionState>()
        .init_resource::<ContextMenuState>()
        .add_plugins(GraphCommandsPlugin)
        .add_plugins(GraphPersistencePlugin)
        .add_systems(Update, execute_spawn_node_commands_system)
        .add_systems(PostUpdate, sync_live_graph_document_state);

    {
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        registry.register(WorkflowValueNode);
        registry.register(WorkflowSinkNode);
    }

    {
        let mut settings = app.world_mut().resource_mut::<GraphPersistenceSettings>();
        settings.autosave_enabled = false;
        settings.file_path = unique_temp_path("default").to_string_lossy().into_owned();
        settings.backup_directory = unique_temp_path("backups").to_string_lossy().into_owned();
    }

    app.update();
    app
}

fn build_persistence_only_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<NodeRegistry>()
        .add_plugins(GraphCommandsPlugin)
        .add_plugins(GraphPersistencePlugin)
        .add_systems(PostUpdate, sync_live_graph_document_state);

    {
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        registry.register(WorkflowValueNode);
        registry.register(WorkflowSinkNode);
    }

    {
        let mut settings = app.world_mut().resource_mut::<GraphPersistenceSettings>();
        settings.autosave_enabled = false;
        settings.file_path = unique_temp_path("persistence_only")
            .to_string_lossy()
            .into_owned();
        settings.backup_directory = unique_temp_path("persistence_only_backups")
            .to_string_lossy()
            .into_owned();
    }

    app.update();
    app
}

fn unique_temp_path(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for tests")
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
        .expect("spawn message should enqueue");
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(Entity, &GraphNode)>();
    query
        .iter(world)
        .find_map(|(entity, node)| (node.definition_id.as_str() == definition_id).then_some(entity))
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

fn set_string_input(app: &mut App, entity: Entity, input_index: usize, value: &str) {
    {
        let world = app.world_mut();
        let mut entity_ref = world.entity_mut(entity);
        entity_ref
            .get_mut::<GraphNode>()
            .expect("graph node should exist")
            .values
            .inputs[input_index] = NodeValue::string(value);
        entity_ref
            .get_mut::<AuthoredNodeInputs>()
            .expect("authored inputs should exist")
            .values[input_index] = NodeValue::string(value);
    }
    app.world_mut()
        .resource_mut::<GraphMutationTracker>()
        .mark_changed();
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

fn legacy_raw_document() -> GraphDocument {
    GraphDocument {
        version: 1,
        nodes: vec![
            GraphDocumentNode {
                id: 1,
                definition_id: NodeId::new("tests/workflow_value"),
                position: [32.0, 48.0],
                inputs: vec![NodeValue::string("legacy-value")],
                input_count: 1,
                output_count: 1,
            },
            GraphDocumentNode {
                id: 2,
                definition_id: NodeId::new("tests/workflow_sink"),
                position: [220.0, 48.0],
                inputs: vec![NodeValue::None],
                input_count: 1,
                output_count: 0,
            },
        ],
        edges: vec![GraphDocumentEdge {
            from_node_id: 1,
            from_index: 0,
            to_node_id: 2,
            to_index: 0,
        }],
        ..GraphDocument::default()
    }
}

#[test]
fn smoke_save_and_load_round_trip_restores_graph_shape_and_values() {
    let mut app = build_test_app();
    let save_path = unique_temp_path("workflow_roundtrip");

    let value_node = spawn_node(&mut app, "tests/workflow_value", Vec2::new(32.0, 48.0));
    let sink_node = spawn_node(&mut app, "tests/workflow_sink", Vec2::new(220.0, 48.0));
    set_string_input(&mut app, value_node, 0, "smoke-value");
    connect_nodes(&mut app, value_node, sink_node);
    app.world_mut().entity_mut(sink_node).insert(Selected);

    app.world_mut()
        .write_message(GraphCommandRequest::SaveGraphToPath {
            path: save_path.to_string_lossy().into_owned(),
        })
        .expect("save request should enqueue");
    app.update();

    let content = fs::read_to_string(&save_path).expect("saved graph file");
    let parsed = parse_graph_document_payload(&content).expect("saved payload should parse");
    assert_eq!(parsed.document.nodes.len(), 2);
    assert_eq!(parsed.document.edges.len(), 1);
    assert_eq!(parsed.document.view.selected_node_ids.len(), 1);

    spawn_node(&mut app, "tests/workflow_sink", Vec2::new(420.0, 48.0));
    assert_eq!(node_count(&mut app), 3);

    app.world_mut()
        .write_message(GraphCommandRequest::LoadGraphFromPath {
            path: save_path.to_string_lossy().into_owned(),
            force_if_dirty: true,
        })
        .expect("load request should enqueue");
    update_frames(&mut app, 2);

    assert_eq!(node_count(&mut app), 2);
    assert_eq!(edge_count(&mut app), 1);

    let live_document = &app.world().resource::<LiveGraphDocumentState>().document;
    assert_eq!(live_document.nodes.len(), 2);
    assert_eq!(live_document.edges.len(), 1);
    assert_eq!(live_document.selected_node_ids().len(), 1);

    let restored_value_node = live_document
        .nodes
        .iter()
        .find(|node| node.definition_id.as_str() == "tests/workflow_value")
        .expect("restored value node");
    assert_eq!(
        restored_value_node.inputs.first(),
        Some(&NodeValue::string("smoke-value"))
    );

    let _ = fs::remove_file(save_path);
}

#[test]
fn smoke_apply_graph_document_works_without_optional_ui_state_resources() {
    let mut app = build_persistence_only_app();

    let document = GraphDocument {
        nodes: vec![GraphDocumentNode {
            id: 1,
            definition_id: NodeId::new("tests/workflow_value"),
            position: [32.0, 48.0],
            inputs: vec![NodeValue::string("from-apply")],
            input_count: 1,
            output_count: 1,
        }],
        ..GraphDocument::default()
    };

    app.world_mut()
        .write_message(ApplyGraphDocumentRequest {
            document,
            source_label: "apply without ui state".to_string(),
            track_for_undo: false,
            validation_report: None,
        })
        .expect("apply request should enqueue");
    update_frames(&mut app, 2);

    assert_eq!(node_count(&mut app), 1);
    assert!(!app.world().contains_resource::<ContextMenuState>());
    assert!(!app.world().contains_resource::<GraphOverlayState>());
    assert!(!app.world().contains_resource::<DragState>());
    assert!(!app.world().contains_resource::<WireConnectionState>());

    let live_document = &app.world().resource::<LiveGraphDocumentState>().document;
    assert_eq!(live_document.nodes.len(), 1);
    assert_eq!(
        live_document.nodes[0].inputs.first(),
        Some(&NodeValue::string("from-apply"))
    );
}

#[test]
fn smoke_save_preserves_created_at_and_refreshes_updated_at() {
    let mut app = build_test_app();
    let save_path = unique_temp_path("workflow_metadata");

    let value_node = spawn_node(&mut app, "tests/workflow_value", Vec2::new(32.0, 48.0));
    let sink_node = spawn_node(&mut app, "tests/workflow_sink", Vec2::new(220.0, 48.0));
    set_string_input(&mut app, value_node, 0, "first-save");
    connect_nodes(&mut app, value_node, sink_node);

    app.world_mut()
        .write_message(GraphCommandRequest::SaveGraphToPath {
            path: save_path.to_string_lossy().into_owned(),
        })
        .expect("first save request should enqueue");
    app.update();

    let first_meta = parse_graph_save_metadata(
        &fs::read_to_string(&save_path).expect("first saved graph file should exist"),
    )
    .expect("first save metadata should parse")
    .expect("first save should contain metadata");

    thread::sleep(Duration::from_millis(2));

    set_string_input(&mut app, value_node, 0, "second-save");
    app.world_mut()
        .write_message(GraphCommandRequest::SaveGraphToPath {
            path: save_path.to_string_lossy().into_owned(),
        })
        .expect("second save request should enqueue");
    app.update();

    let second_meta = parse_graph_save_metadata(
        &fs::read_to_string(&save_path).expect("second saved graph file should exist"),
    )
    .expect("second save metadata should parse")
    .expect("second save should contain metadata");

    assert_eq!(first_meta.created_at, second_meta.created_at);
    assert_ne!(first_meta.updated_at, second_meta.updated_at);

    let _ = fs::remove_file(save_path);
}

#[test]
fn smoke_load_marks_migrated_legacy_graph_dirty_until_resave() {
    let mut app = build_test_app();
    let save_path = unique_temp_path("workflow_legacy_load");
    let legacy_payload =
        serde_json::to_string(&legacy_raw_document()).expect("legacy raw graph should serialize");
    fs::write(&save_path, legacy_payload).expect("legacy raw graph should be written");

    app.world_mut()
        .write_message(GraphCommandRequest::LoadGraphFromPath {
            path: save_path.to_string_lossy().into_owned(),
            force_if_dirty: true,
        })
        .expect("load request should enqueue");
    update_frames(&mut app, 2);

    assert_eq!(node_count(&mut app), 2);
    assert_eq!(edge_count(&mut app), 1);

    let runtime = app.world().resource::<GraphPersistenceRuntimeState>();
    assert!(runtime.dirty);
    assert!(runtime.requires_resave_after_migration);

    let status = app.world().resource::<GraphPersistenceStatus>();
    let active = status.active.as_ref().expect("status message should exist");
    assert_eq!(active.severity, GraphPersistenceStatusSeverity::Warning);
    assert!(
        active
            .text
            .contains("Save the graph to rewrite it in the current format.")
    );

    let _ = fs::remove_file(save_path);
}

#[test]
fn smoke_undo_and_redo_restore_spawn_and_connection_steps() {
    let mut app = build_test_app();

    let value_node = spawn_node(&mut app, "tests/workflow_value", Vec2::new(16.0, 24.0));
    let sink_node = spawn_node(&mut app, "tests/workflow_sink", Vec2::new(180.0, 24.0));
    connect_nodes(&mut app, value_node, sink_node);

    assert_eq!(node_count(&mut app), 2);
    assert_eq!(edge_count(&mut app), 1);
    assert!(!app.world().resource::<GraphHistoryState>().past.is_empty());

    app.world_mut()
        .write_message(GraphCommandRequest::UndoGraphChange)
        .expect("undo request should enqueue");
    update_frames(&mut app, 2);

    assert_eq!(node_count(&mut app), 2);
    assert_eq!(edge_count(&mut app), 0);

    app.world_mut()
        .write_message(GraphCommandRequest::RedoGraphChange)
        .expect("redo request should enqueue");
    update_frames(&mut app, 2);

    assert_eq!(node_count(&mut app), 2);
    assert_eq!(edge_count(&mut app), 1);
}
