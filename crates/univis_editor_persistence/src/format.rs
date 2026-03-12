use serde::Deserialize;
use serde_json::Value;
use univis_node_graph::document::{
    GRAPH_DOCUMENT_VERSION, GraphDocument, GraphDocumentNode, GraphDocumentViewState,
};
use univis_node_graph::graph_validation::validate_graph_document;
use univis_node_graph::node_definition::NodeId;
use univis_node_graph::node_registry::NodeRegistry;
use univis_node_graph::value::NodeValue;

#[derive(Debug, Clone)]
pub struct ParsedGraphDocument {
    pub document: GraphDocument,
    pub migration_note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedGraphWrite {
    pub document: GraphDocument,
    pub payload: String,
    pub signature: String,
    pub validation_issue_count: usize,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GraphSaveFileV0 {
    #[serde(default)]
    nodes: Vec<SavedNodeV0>,
    #[serde(default)]
    links: Vec<univis_node_graph::document::GraphDocumentEdge>,
    #[serde(default)]
    ui: GraphDocumentViewState,
}

#[derive(Debug, Clone, Deserialize)]
struct SavedNodeV0 {
    id: u64,
    definition_id: NodeId,
    position: [f32; 2],
    #[serde(default)]
    inputs: Vec<NodeValue>,
    #[serde(default)]
    input_count: Option<usize>,
    #[serde(default)]
    output_count: Option<usize>,
}

pub fn parse_graph_document_payload(content: &str) -> Result<ParsedGraphDocument, String> {
    let value: Value =
        serde_json::from_str(content).map_err(|err| format!("invalid JSON: {}", err))?;

    let version = value
        .get("version")
        .and_then(|field| field.as_u64())
        .map(|version| version as u32)
        .unwrap_or(0);

    match version {
        0 => {
            let legacy: GraphSaveFileV0 = serde_json::from_value(value)
                .map_err(|err| format!("invalid schema v0 payload: {}", err))?;

            let nodes = legacy
                .nodes
                .into_iter()
                .map(|node| {
                    let inputs = node.inputs;
                    GraphDocumentNode {
                        id: node.id,
                        definition_id: node.definition_id,
                        position: node.position,
                        input_count: node.input_count.unwrap_or(inputs.len()),
                        output_count: node.output_count.unwrap_or(0),
                        inputs,
                    }
                })
                .collect();

            Ok(ParsedGraphDocument {
                document: GraphDocument {
                    version: GRAPH_DOCUMENT_VERSION,
                    nodes,
                    edges: legacy.links,
                    view: legacy.ui,
                },
                migration_note: Some("Migrated graph document schema from v0 to v1.".to_string()),
            })
        }
        GRAPH_DOCUMENT_VERSION => {
            let current: GraphDocument = serde_json::from_value(value)
                .map_err(|err| format!("invalid schema v1 payload: {}", err))?;
            Ok(ParsedGraphDocument {
                document: current,
                migration_note: None,
            })
        }
        other => Err(format!(
            "unsupported schema version {} (latest supported {})",
            other, GRAPH_DOCUMENT_VERSION
        )),
    }
}

pub fn serialize_graph_document(
    document: &GraphDocument,
    pretty_json: bool,
) -> Result<String, String> {
    if pretty_json {
        serde_json::to_string_pretty(document)
            .map_err(|err| format!("cannot serialize JSON payload: {}", err))
    } else {
        serde_json::to_string(document).map_err(|err| format!("cannot serialize JSON payload: {}", err))
    }
}

pub fn graph_document_signature(document: &GraphDocument) -> Result<String, String> {
    serde_json::to_string(document)
        .map_err(|err| format!("signature serialization failed: {}", err))
}

pub fn prepare_graph_document_write(
    document: GraphDocument,
    pretty_json: bool,
    registry: &NodeRegistry,
) -> Result<PreparedGraphWrite, String> {
    let payload = serialize_graph_document(&document, pretty_json)?;
    let signature = graph_document_signature(&document)?;
    let validation_issue_count = validate_graph_document(&document, registry).len();

    Ok(PreparedGraphWrite {
        document,
        payload,
        signature,
        validation_issue_count,
    })
}
