use bevy::prelude::World;
use univis_node_graph::document::{
    GraphDocument, GraphDocumentNode, GraphDocumentPrefab, GraphDocumentSubgraph,
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
    let prefab = document.prefab("prefab_actor").expect("prefab should exist");
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

    assert!(document.capture_selected_subgraph(
        "subgraph_pair".to_string(),
        "Pair".to_string()
    ));

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
    live_document.document.connect(1, 0, 2, 0).expect("edge 1->2");
    live_document.document.connect(2, 0, 3, 0).expect("edge 2->3");
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
