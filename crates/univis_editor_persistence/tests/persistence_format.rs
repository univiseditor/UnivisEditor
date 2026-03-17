use serde_json::json;
use univis_editor_persistence::format::{
    graph_document_signature, parse_graph_document_payload, parse_graph_save_metadata,
    prepare_graph_document_write, serialize_graph_document,
};
use univis_node_graph::document::{GraphDocument, GraphDocumentEdge, GraphDocumentNode};
use univis_node_graph::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};
use univis_node_graph::node_registry::NodeRegistry;
use univis_node_graph::value::{NodeValue, ValueType};

struct TestNode;

impl NodeDefinition for TestNode {
    fn id(&self) -> NodeId {
        NodeId::new("tests/value")
    }

    fn display_name(&self) -> &str {
        "Value"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Tests")
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::new("Input", ValueType::Any)]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_any("Output")]
    }

    fn process(&self, _context: &mut ProcessContext) -> ProcessResult {
        ProcessResult::Success
    }
}

fn sample_document(definition_id: &str) -> GraphDocument {
    GraphDocument {
        nodes: vec![
            GraphDocumentNode {
                id: 1,
                definition_id: NodeId::new(definition_id),
                position: [10.0, 20.0],
                inputs: vec![NodeValue::string("hello")],
                input_count: 1,
                output_count: 1,
            },
            GraphDocumentNode {
                id: 2,
                definition_id: NodeId::new(definition_id),
                position: [40.0, 60.0],
                inputs: vec![NodeValue::None],
                input_count: 1,
                output_count: 1,
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
fn prepare_graph_document_write_generates_payload_signature_and_validation_count() {
    let mut registry = NodeRegistry::new();
    registry.register(TestNode);

    let document = sample_document("tests/value");
    let prepared = prepare_graph_document_write(document.clone(), true, &registry)
        .expect("document should serialize");
    let payload: serde_json::Value =
        serde_json::from_str(&prepared.payload).expect("save payload should be valid JSON");

    assert!(prepared.payload.contains('\n'));
    assert_eq!(
        prepared.signature,
        graph_document_signature(&document).unwrap()
    );
    assert_eq!(prepared.validation_issue_count, 0);
    assert_eq!(prepared.document.nodes.len(), document.nodes.len());
    assert_eq!(prepared.document.edges.len(), document.edges.len());
    assert_eq!(
        prepared.document.nodes[0].definition_id.as_str(),
        "tests/value"
    );
    assert_eq!(payload["format"], "univis.graph");
    assert_eq!(payload["format_version"], 1);
    assert!(payload["meta"]["created_at"].is_string());
    assert!(payload["meta"]["updated_at"].is_string());
    assert_eq!(
        payload["document"]["nodes"][0]["authored_inputs"][0]["kind"],
        "string"
    );
}

#[test]
fn format_helpers_round_trip_documents_and_support_compact_json() {
    let document = sample_document("tests/value");
    let compact = serialize_graph_document(&document, false).expect("compact JSON");
    let payload: serde_json::Value =
        serde_json::from_str(&compact).expect("compact payload should be valid JSON");
    assert!(!compact.contains('\n'));
    assert_eq!(payload["format"], "univis.graph");
    assert!(payload["meta"]["created_at"].is_string());
    assert!(payload["meta"]["updated_at"].is_string());

    let parsed = parse_graph_document_payload(&compact).expect("valid payload");
    assert_eq!(parsed.document.nodes.len(), document.nodes.len());
    assert_eq!(parsed.document.edges.len(), document.edges.len());
    assert_eq!(parsed.document.nodes[0].position, [10.0, 20.0]);
    assert!(parsed.migration_note.is_none());
}

#[test]
fn parse_graph_document_payload_migrates_v0_documents() {
    let legacy = json!({
        "nodes": [
            {
                "id": 7,
                "definition_id": "tests/value",
                "position": [1.0, 2.0],
                "inputs": [NodeValue::string("legacy")],
                "input_count": 1,
                "output_count": 0
            }
        ],
        "links": [],
        "ui": {
            "selected_node_ids": [7]
        }
    })
    .to_string();

    let parsed = parse_graph_document_payload(&legacy).expect("legacy payload should migrate");
    assert_eq!(parsed.document.version, 1);
    assert_eq!(parsed.document.nodes.len(), 1);
    assert_eq!(parsed.document.nodes[0].id, 7);
    assert_eq!(parsed.document.nodes[0].input_count, 1);
    assert_eq!(parsed.document.view.selected_node_ids, vec![7]);
    assert_eq!(
        parsed.migration_note.as_deref(),
        Some("Migrated legacy graph document schema from v0 into save-file v1.")
    );
}

#[test]
fn prepare_graph_document_write_reports_validation_issues_for_unknown_nodes() {
    let registry = NodeRegistry::new();
    let document = sample_document("tests/missing");

    let prepared = prepare_graph_document_write(document, true, &registry)
        .expect("serialization should still succeed");
    assert_eq!(prepared.validation_issue_count, 2);
}

#[test]
fn parse_graph_document_payload_migrates_legacy_raw_graph_documents() {
    let legacy = serde_json::to_string(&sample_document("tests/value"))
        .expect("legacy raw graph document should serialize");

    let parsed =
        parse_graph_document_payload(&legacy).expect("legacy raw payload should migrate cleanly");
    assert_eq!(parsed.document.nodes.len(), 2);
    assert_eq!(parsed.document.edges.len(), 1);
    assert_eq!(
        parsed.migration_note.as_deref(),
        Some("Migrated legacy raw graph document into save-file v1.")
    );
}

#[test]
fn parse_graph_save_metadata_reads_envelope_timestamps() {
    let payload = serialize_graph_document(&sample_document("tests/value"), false)
        .expect("graph should serialize with metadata");

    let meta = parse_graph_save_metadata(&payload)
        .expect("metadata parse should succeed")
        .expect("save payload should contain metadata");
    assert!(meta
        .created_at
        .as_deref()
        .is_some_and(|value| !value.is_empty()));
    assert!(meta
        .updated_at
        .as_deref()
        .is_some_and(|value| !value.is_empty()));
}
