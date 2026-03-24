#[path = "support/mod.rs"]
mod support;

use support::DemoValue;
use univis_graph_core::prelude::{
    GraphDocument, GraphDocumentCameraState, GraphDocumentNode, GraphDocumentPrefab, NodeId,
};

fn main() {
    let mut document = GraphDocument::<DemoValue, String>::default();

    document.upsert_prefab(GraphDocumentPrefab::new(
        "prefabs/hero",
        "Hero",
        "scene://hero".to_string(),
    ));
    document.upsert_prefab(GraphDocumentPrefab::new(
        "prefabs/hero",
        "Hero Updated",
        "scene://hero_v2".to_string(),
    ));
    document.upsert_prefab(GraphDocumentPrefab::new(
        "prefabs/enemy",
        "Enemy",
        "scene://enemy".to_string(),
    ));

    let left = document.spawn_node(NodeId::new("demo/input_number"), [0.0, 0.0], 1, 1);
    let right = document.spawn_node(NodeId::new("demo/input_number"), [0.0, 140.0], 1, 1);
    let add = document.spawn_node(NodeId::new("demo/add"), [260.0, 70.0], 2, 1);
    let format = document.spawn_node(NodeId::new("demo/format"), [520.0, 70.0], 1, 1);

    document.node_mut(left).expect("left node").inputs[0] = DemoValue::number(3.0);
    document.node_mut(right).expect("right node").inputs[0] = DemoValue::number(7.0);

    document.connect(left, 0, add, 0).expect("left -> add");
    document.connect(right, 0, add, 1).expect("right -> add");
    document.connect(add, 0, format, 0).expect("add -> format");

    let cycle_error = document.connect(add, 0, left, 0).unwrap_err();
    println!("cycle protection: {cycle_error}");

    document.set_camera(Some(GraphDocumentCameraState {
        translation: [120.0, -40.0, 1.0],
        ortho_scale: 0.75,
    }));
    document.set_selected_nodes([left, right, add]);

    let summary = document
        .selected_subgraph_boundary_summary()
        .expect("selection should produce a boundary summary");
    println!(
        "selection summary: nodes={} internal={} incoming={} outgoing={} omitted={}",
        summary.selected_node_count(),
        summary.internal_edge_count(),
        summary.incoming_edge_count(),
        summary.outgoing_edge_count(),
        summary.omitted_edge_count()
    );

    let capture_result = document.capture_selected_subgraph(
        "subgraphs/math_cluster".to_string(),
        "Math Cluster".to_string(),
    );
    println!("captured selected subgraph: {capture_result:?}");
    println!(
        "version={} prefabs={} subgraphs={}",
        document.version,
        document.prefab_count(),
        document.subgraph_count()
    );
    println!(
        "hero prefab root: {}",
        document
            .prefab("prefabs/hero")
            .expect("hero prefab should exist")
            .root
    );
    println!(
        "captured subgraph nodes: {}",
        document
            .subgraph("subgraphs/math_cluster")
            .expect("subgraph should exist")
            .document
            .nodes
            .len()
    );

    let mut merged = document
        .merged_with_subgraph_instance("subgraphs/math_cluster", [920.0, 120.0])
        .expect("captured subgraph should be instantiable");
    println!(
        "merged document nodes={} selected_instance_ids={:?}",
        merged.nodes.len(),
        merged.selected_node_ids()
    );

    let inserted_group = GraphDocumentNode {
        id: 999,
        definition_id: NodeId::new("demo/group"),
        position: [120.0, 320.0],
        inputs: vec![],
        input_count: 0,
        output_count: 0,
    };
    merged
        .insert_node(inserted_group.clone())
        .expect("group node should insert");
    let duplicate_insert_error = merged.insert_node(inserted_group).unwrap_err();
    println!("duplicate insert protection: {duplicate_insert_error}");

    let disconnected_specific = merged.disconnect_edge(right, 0, add, 1);
    let disconnected_input = merged.disconnect_input(format, 0);
    println!(
        "disconnect specific edge={disconnected_specific} disconnect input={disconnected_input}"
    );

    merged.select_single_node(add);
    let deleted_selected = merged.delete_selected_nodes();
    let deleted_one = merged.delete_node(left);
    let deleted_many = merged.delete_nodes([right]);
    merged.clear_selection();

    println!(
        "deleted selected={} deleted_one={} deleted_many={} remaining_nodes={}",
        deleted_selected,
        deleted_one,
        deleted_many,
        merged.nodes.len()
    );
}
