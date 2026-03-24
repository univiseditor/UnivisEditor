use bevy::prelude::World;
use univis_node_graph::document::{
    build_graph_document_from_snapshots, GraphDocument, GraphDocumentBuildIssue,
    GraphDocumentEdgeSnapshot, GraphDocumentNode, GraphDocumentNodeSnapshot,
    GraphDocumentOperationError, GraphDocumentPrefab, GraphDocumentSubgraph,
    LiveGraphDocumentState,
};
use univis_node_graph::node_definition::NodeId;
use univis_node_graph::value::NodeValue;
use univis_scene::EntityValue;

fn node(
    id: u64,
    definition_id: &str,
    position: [f32; 2],
    input_count: usize,
    output_count: usize,
) -> GraphDocumentNode {
    GraphDocumentNode {
        id,
        definition_id: NodeId::new(definition_id),
        position,
        inputs: vec![NodeValue::None; input_count],
        input_count,
        output_count,
    }
}

#[test]
fn upsert_prefab_replaces_existing_prefab_by_id() {
    let mut document = GraphDocument::default();
    document.upsert_prefab(GraphDocumentPrefab {
        id: "prefab_actor".to_string(),
        name: "Actor".to_string(),
        root: EntityValue::named("Actor"),
    });
    document.upsert_prefab(GraphDocumentPrefab {
        id: "prefab_actor".to_string(),
        name: "Updated Actor".to_string(),
        root: EntityValue::named("Updated Actor"),
    });

    assert_eq!(document.prefab_count(), 1);
    let prefab = document
        .prefab("prefab_actor")
        .expect("prefab should exist");
    assert_eq!(prefab.name, "Updated Actor");
    assert_eq!(prefab.root.name.as_deref(), Some("Updated Actor"));
}

#[test]
fn capture_selected_subgraph_normalizes_positions_and_keeps_internal_edges() {
    let mut document = GraphDocument::default();
    document
        .insert_node(node(1, "tests/source", [100.0, 220.0], 0, 1))
        .expect("node 1");
    document
        .insert_node(node(2, "tests/target", [280.0, 300.0], 1, 1))
        .expect("node 2");
    document
        .insert_node(node(3, "tests/external", [520.0, 360.0], 1, 0))
        .expect("node 3");
    document.connect(1, 0, 2, 0).expect("edge 1->2");
    document.connect(2, 0, 3, 0).expect("edge 2->3");
    document.upsert_prefab(GraphDocumentPrefab {
        id: "prefab_actor".to_string(),
        name: "Actor".to_string(),
        root: EntityValue::named("Actor"),
    });
    document.set_selected_nodes([1, 2]);

    document
        .capture_selected_subgraph("subgraph_pair".to_string(), "Pair".to_string())
        .expect("subgraph capture should succeed");

    let subgraph = document
        .subgraph("subgraph_pair")
        .expect("subgraph should exist");
    assert_eq!(subgraph.document.nodes.len(), 2);
    assert_eq!(subgraph.document.edges.len(), 1);
    assert_eq!(subgraph.document.prefabs.len(), 1);
    assert!(subgraph.document.subgraphs.is_empty());

    let first = subgraph
        .document
        .node(1)
        .expect("first node should be preserved");
    let second = subgraph
        .document
        .node(2)
        .expect("second node should be preserved");
    assert_eq!(first.position, [0.0, 0.0]);
    assert_eq!(second.position, [180.0, 80.0]);
    assert_eq!(subgraph.document.edges[0].from_node_id, 1);
    assert_eq!(subgraph.document.edges[0].to_node_id, 2);
}

#[test]
fn selected_subgraph_boundary_summary_tracks_internal_and_omitted_edges() {
    let mut document = GraphDocument::default();
    document
        .insert_node(node(1, "tests/source", [100.0, 220.0], 1, 1))
        .expect("node 1");
    document
        .insert_node(node(2, "tests/middle", [280.0, 300.0], 1, 1))
        .expect("node 2");
    document
        .insert_node(node(3, "tests/outside_in", [40.0, 180.0], 0, 1))
        .expect("node 3");
    document
        .insert_node(node(4, "tests/outside_out", [520.0, 360.0], 1, 0))
        .expect("node 4");
    document.connect(1, 0, 2, 0).expect("edge 1->2");
    document.connect(3, 0, 1, 0).expect("edge 3->1");
    document.connect(2, 0, 4, 0).expect("edge 2->4");
    document.set_selected_nodes([1, 2]);

    let summary = document
        .selected_subgraph_boundary_summary()
        .expect("summary should exist");

    assert_eq!(summary.selected_node_ids, vec![1, 2]);
    assert_eq!(summary.internal_edge_count(), 1);
    assert_eq!(summary.incoming_edge_count(), 1);
    assert_eq!(summary.outgoing_edge_count(), 1);
    assert_eq!(summary.omitted_edge_count(), 2);
    assert_eq!(summary.internal_edges[0].from_node_id, 1);
    assert_eq!(summary.internal_edges[0].to_node_id, 2);
    assert_eq!(summary.incoming_edges[0].from_node_id, 3);
    assert_eq!(summary.incoming_edges[0].to_node_id, 1);
    assert_eq!(summary.outgoing_edges[0].from_node_id, 2);
    assert_eq!(summary.outgoing_edges[0].to_node_id, 4);
}

#[test]
fn merged_with_subgraph_instance_offsets_nodes_and_selects_inserted_nodes() {
    let mut document = GraphDocument::default();
    document
        .insert_node(node(5, "tests/root", [32.0, 48.0], 0, 1))
        .expect("existing node");
    document.upsert_subgraph(GraphDocumentSubgraph {
        id: "subgraph_pair".to_string(),
        name: "Pair".to_string(),
        document: Box::new(GraphDocument {
            nodes: vec![
                node(1, "tests/source", [0.0, 0.0], 0, 1),
                node(2, "tests/target", [160.0, 40.0], 1, 0),
            ],
            edges: vec![univis_node_graph::document::GraphDocumentEdge {
                from_node_id: 1,
                from_index: 0,
                to_node_id: 2,
                to_index: 0,
            }],
            prefabs: vec![GraphDocumentPrefab {
                id: "prefab_actor".to_string(),
                name: "Actor".to_string(),
                root: EntityValue::named("Actor"),
            }],
            ..GraphDocument::default()
        }),
    });

    let merged = document
        .merged_with_subgraph_instance("subgraph_pair", [420.0, 260.0])
        .expect("subgraph should merge");

    assert_eq!(merged.nodes.len(), 3);
    assert_eq!(merged.edges.len(), 1);
    assert_eq!(merged.prefab_count(), 1);
    assert_eq!(merged.selected_node_ids().len(), 2);

    let selected_positions = merged
        .selected_node_ids()
        .iter()
        .filter_map(|node_id| merged.node(*node_id))
        .map(|node| node.position)
        .collect::<Vec<_>>();
    assert!(selected_positions.contains(&[420.0, 260.0]));
    assert!(selected_positions.contains(&[580.0, 300.0]));
    assert!(document.selected_node_ids().is_empty());
}

#[test]
fn capture_and_reinstantiate_subgraph_round_trips_internal_shape() {
    let mut document = GraphDocument::default();
    document
        .insert_node(node(1, "tests/source", [100.0, 220.0], 0, 1))
        .expect("node 1");
    document
        .insert_node(node(2, "tests/middle", [260.0, 300.0], 1, 1))
        .expect("node 2");
    document
        .insert_node(node(3, "tests/target", [420.0, 360.0], 1, 0))
        .expect("node 3");
    document.connect(1, 0, 2, 0).expect("edge 1->2");
    document.connect(2, 0, 3, 0).expect("edge 2->3");
    document.set_selected_nodes([1, 2, 3]);

    document
        .capture_selected_subgraph("subgraph_round_trip".to_string(), "Round Trip".to_string())
        .expect("subgraph capture should succeed");

    let merged = document
        .merged_with_subgraph_instance("subgraph_round_trip", [640.0, 120.0])
        .expect("captured subgraph should merge");

    assert_eq!(merged.prefab_count(), document.prefab_count());
    assert_eq!(merged.selected_node_ids().len(), 3);

    let selected_nodes = merged
        .selected_node_ids()
        .iter()
        .filter_map(|node_id| merged.node(*node_id))
        .collect::<Vec<_>>();
    assert_eq!(selected_nodes.len(), 3);

    let selected_positions = selected_nodes
        .iter()
        .map(|node| node.position)
        .collect::<Vec<_>>();
    assert!(selected_positions.contains(&[640.0, 120.0]));
    assert!(selected_positions.contains(&[800.0, 200.0]));
    assert!(selected_positions.contains(&[960.0, 260.0]));

    let selected_set = merged
        .selected_node_ids()
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    let internal_edges = merged
        .edges
        .iter()
        .filter(|edge| {
            selected_set.contains(&edge.from_node_id) && selected_set.contains(&edge.to_node_id)
        })
        .count();
    assert_eq!(internal_edges, 2);
}

#[test]
fn capture_and_instantiate_report_result_errors_for_empty_or_missing_subgraphs() {
    let mut document = GraphDocument::default();

    assert_eq!(
        document.capture_selected_subgraph("empty".to_string(), "Empty".to_string()),
        Err(GraphDocumentOperationError::EmptySelection)
    );

    assert!(matches!(
        document.merged_with_subgraph_instance("missing", [0.0, 0.0]),
        Err(GraphDocumentOperationError::MissingSubgraph(subgraph_id))
            if subgraph_id == "missing"
    ));

    document.upsert_subgraph(GraphDocumentSubgraph {
        id: "empty_subgraph".to_string(),
        name: "Empty Subgraph".to_string(),
        document: Box::new(GraphDocument::default()),
    });

    assert!(matches!(
        document.merged_with_subgraph_instance("empty_subgraph", [0.0, 0.0]),
        Err(GraphDocumentOperationError::EmptyDocumentFragment { context })
            if context == "subgraph 'empty_subgraph'"
    ));
}

#[test]
fn retain_existing_entities_prunes_dead_mappings_nodes_edges_and_selection() {
    let mut world = World::new();
    let entity_a = world.spawn_empty().id();
    let entity_b = world.spawn_empty().id();
    let entity_c = world.spawn_empty().id();

    let mut live_document = LiveGraphDocumentState::default();
    live_document.document.nodes = vec![
        node(1, "tests/a", [0.0, 0.0], 0, 1),
        node(2, "tests/b", [120.0, 0.0], 1, 1),
        node(3, "tests/c", [240.0, 0.0], 1, 0),
    ];
    live_document
        .document
        .connect(1, 0, 2, 0)
        .expect("edge 1->2");
    live_document
        .document
        .connect(2, 0, 3, 0)
        .expect("edge 2->3");
    live_document.document.set_selected_nodes([2, 3]);
    live_document.entity_to_node_id.insert(entity_a, 1);
    live_document.entity_to_node_id.insert(entity_b, 2);
    live_document.entity_to_node_id.insert(entity_c, 3);
    live_document.node_id_to_entity.insert(1, entity_a);
    live_document.node_id_to_entity.insert(2, entity_b);
    live_document.node_id_to_entity.insert(3, entity_c);

    assert!(live_document.retain_existing_entities(|entity| entity != entity_b));
    assert_eq!(live_document.document.nodes.len(), 2);
    assert_eq!(live_document.document.edges.len(), 0);
    assert_eq!(live_document.document.selected_node_ids(), &[3]);
    assert!(live_document.node_id_for_entity(entity_b).is_none());
    assert!(live_document.entity_for_node_id(2).is_none());
}

#[test]
fn snapshot_build_reuses_document_connect_and_reports_rejected_edges() {
    let mut world = World::new();
    let source = world.spawn_empty().id();
    let middle = world.spawn_empty().id();
    let target = world.spawn_empty().id();

    let build = build_graph_document_from_snapshots(
        [
            GraphDocumentNodeSnapshot {
                entity: source,
                definition_id: NodeId::new("tests/source"),
                position: [0.0, 0.0],
                inputs: vec![],
                input_count: 0,
                output_count: 1,
                selected: false,
            },
            GraphDocumentNodeSnapshot {
                entity: middle,
                definition_id: NodeId::new("tests/middle"),
                position: [160.0, 0.0],
                inputs: vec![NodeValue::None],
                input_count: 1,
                output_count: 1,
                selected: false,
            },
            GraphDocumentNodeSnapshot {
                entity: target,
                definition_id: NodeId::new("tests/target"),
                position: [320.0, 0.0],
                inputs: vec![NodeValue::None],
                input_count: 1,
                output_count: 0,
                selected: false,
            },
        ],
        [
            GraphDocumentEdgeSnapshot {
                from_entity: source,
                from_index: 0,
                to_entity: middle,
                to_index: 0,
            },
            GraphDocumentEdgeSnapshot {
                from_entity: middle,
                from_index: 0,
                to_entity: source,
                to_index: 0,
            },
            GraphDocumentEdgeSnapshot {
                from_entity: middle,
                from_index: 0,
                to_entity: target,
                to_index: 0,
            },
        ],
        None,
        None,
    );

    assert_eq!(build.document.edges.len(), 2);
    assert_eq!(build.issues.len(), 1);
    assert!(matches!(
        &build.issues[0],
        GraphDocumentBuildIssue::RejectedEdge { error, .. }
            if *error
                == GraphDocumentOperationError::CycleDetected {
                    from_node_id: 2,
                    to_node_id: 1,
                }
    ));
}

#[test]
fn snapshot_build_reports_missing_node_mappings_for_stale_edges() {
    let mut world = World::new();
    let source = world.spawn_empty().id();
    let missing = world.spawn_empty().id();

    let build = build_graph_document_from_snapshots(
        [GraphDocumentNodeSnapshot {
            entity: source,
            definition_id: NodeId::new("tests/source"),
            position: [0.0, 0.0],
            inputs: vec![],
            input_count: 0,
            output_count: 1,
            selected: false,
        }],
        [GraphDocumentEdgeSnapshot {
            from_entity: source,
            from_index: 0,
            to_entity: missing,
            to_index: 0,
        }],
        None,
        None,
    );

    assert!(build.document.edges.is_empty());
    assert_eq!(build.issues.len(), 1);
    assert!(matches!(
        &build.issues[0],
        GraphDocumentBuildIssue::MissingTargetNodeMapping {
            from_entity,
            to_entity,
            from_index,
            to_index,
        } if *from_entity == source
            && *to_entity == missing
            && *from_index == 0
            && *to_index == 0
    ));
}
