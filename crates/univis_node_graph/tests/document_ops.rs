use univis_node_graph::document::{
    GRAPH_DOCUMENT_VERSION, GraphDocument, GraphDocumentCameraState, GraphDocumentNode,
    GraphDocumentOperationError,
};
use univis_node_graph::node_definition::NodeId;
use univis_node_graph::value::NodeValue;

fn node(
    id: u64,
    definition_id: &str,
    input_count: usize,
    output_count: usize,
) -> GraphDocumentNode {
    GraphDocumentNode {
        id,
        definition_id: NodeId::new(definition_id),
        position: [id as f32, 0.0],
        inputs: vec![NodeValue::None; input_count],
        input_count,
        output_count,
    }
}

#[test]
fn default_document_uses_current_schema_version() {
    assert_eq!(GraphDocument::default().version, GRAPH_DOCUMENT_VERSION);
}

#[test]
fn insert_node_rejects_duplicate_ids() {
    let mut document = GraphDocument::default();

    document
        .insert_node(node(1, "tests/source", 0, 1))
        .expect("initial insert should succeed");

    let result = document.insert_node(node(1, "tests/duplicate", 1, 0));
    assert_eq!(result, Err(GraphDocumentOperationError::DuplicateNodeId(1)));
}

#[test]
fn connect_and_disconnect_edges_enforce_graph_rules() {
    let mut document = GraphDocument::default();
    document
        .insert_node(node(1, "tests/a", 1, 1))
        .expect("node 1");
    document
        .insert_node(node(2, "tests/b", 1, 1))
        .expect("node 2");
    document
        .insert_node(node(3, "tests/c", 1, 1))
        .expect("node 3");

    document.connect(1, 0, 2, 0).expect("valid edge");
    assert_eq!(document.edges.len(), 1);

    let duplicate_input = document.connect(1, 0, 2, 0);
    assert_eq!(
        duplicate_input,
        Err(GraphDocumentOperationError::InputAlreadyConnected {
            node_id: 2,
            index: 0,
        })
    );

    let invalid_output = document.connect(1, 9, 3, 0);
    assert_eq!(
        invalid_output,
        Err(GraphDocumentOperationError::InvalidOutputPort {
            node_id: 1,
            index: 9,
            output_count: 1,
        })
    );

    assert!(document.disconnect_input(2, 0));
    assert!(!document.disconnect_input(2, 0));
    assert!(document.edges.is_empty());
}

#[test]
fn connect_rejects_cycles_and_delete_selected_removes_nodes_edges_and_selection() {
    let mut document = GraphDocument::default();
    document
        .insert_node(node(1, "tests/a", 1, 1))
        .expect("node 1");
    document
        .insert_node(node(2, "tests/b", 1, 1))
        .expect("node 2");
    document
        .insert_node(node(3, "tests/c", 1, 1))
        .expect("node 3");

    document.connect(1, 0, 2, 0).expect("edge 1->2");
    document.connect(2, 0, 3, 0).expect("edge 2->3");

    let cycle = document.connect(3, 0, 1, 0);
    assert_eq!(
        cycle,
        Err(GraphDocumentOperationError::CycleDetected {
            from_node_id: 3,
            to_node_id: 1,
        })
    );

    document.set_selected_nodes([2, 3]);
    assert_eq!(document.selected_node_ids(), &[2, 3]);
    assert_eq!(document.delete_selected_nodes(), 2);
    assert_eq!(document.nodes.len(), 1);
    assert!(document.edges.is_empty());
    assert!(document.selected_node_ids().is_empty());
}

#[test]
fn spawn_node_and_camera_state_round_trip_through_document_helpers() {
    let mut document = GraphDocument::default();
    let node_id = document.spawn_node(NodeId::new("tests/spawned"), [4.0, 8.0], 2, 1);

    assert_eq!(node_id, 1);
    assert_eq!(
        document.node(node_id).expect("spawned node").position,
        [4.0, 8.0]
    );

    document.set_camera(Some(GraphDocumentCameraState {
        translation: [1.0, 2.0, 3.0],
        ortho_scale: 0.75,
    }));

    assert_eq!(
        document.view.camera.as_ref().expect("camera").translation,
        [1.0, 2.0, 3.0]
    );
    assert_eq!(
        document.view.camera.as_ref().expect("camera").ortho_scale,
        0.75
    );
}
