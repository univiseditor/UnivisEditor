use serde::Deserialize;
use serde_json::Value as JsonValue;
use univis_node_graph::document::{
    GraphDocument, GraphDocumentEdge, GraphDocumentNode, GRAPH_DOCUMENT_VERSION,
};
use univis_node_graph::node_definition::NodeId;
use univis_node_graph::value::NodeValue;

use crate::format::{
    parse_graph_save_file_value, GraphDocumentV1, GraphSaveFileV1, GraphSaveMetaV1,
    GRAPH_SAVE_FILE_FORMAT, GRAPH_SAVE_FILE_VERSION,
};

#[derive(Debug, Clone)]
pub(crate) struct ParsedGraphSaveFile {
    pub save_file: GraphSaveFileV1,
    pub migration_note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GraphSaveFileV0 {
    #[serde(default)]
    nodes: Vec<SavedNodeV0>,
    #[serde(default)]
    links: Vec<GraphDocumentEdge>,
    #[serde(default)]
    ui: univis_node_graph::document::GraphDocumentViewState,
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

pub(crate) fn parse_save_file_payload(value: JsonValue) -> Result<ParsedGraphSaveFile, String> {
    if crate::format::is_graph_save_file_envelope(&value) {
        return Ok(ParsedGraphSaveFile {
            save_file: parse_graph_save_file_value(value)?,
            migration_note: None,
        });
    }

    migrate_legacy_payload(value)
}

fn migrate_legacy_payload(value: JsonValue) -> Result<ParsedGraphSaveFile, String> {
    let version = value
        .get("version")
        .and_then(|field| field.as_u64())
        .map(|version| version as u32)
        .unwrap_or(0);

    match version {
        0 => {
            let legacy: GraphSaveFileV0 = serde_json::from_value(value)
                .map_err(|err| format!("invalid legacy schema v0 payload: {}", err))?;

            Ok(ParsedGraphSaveFile {
                save_file: migrate_legacy_v0_to_save_file(legacy),
                migration_note: Some(
                    "Migrated legacy graph document schema from v0 into save-file v1.".to_string(),
                ),
            })
        }
        GRAPH_DOCUMENT_VERSION => {
            let current: GraphDocument = serde_json::from_value(value)
                .map_err(|err| format!("invalid legacy raw graph document payload: {}", err))?;

            Ok(ParsedGraphSaveFile {
                save_file: GraphSaveFileV1::from_document(&current),
                migration_note: Some(
                    "Migrated legacy raw graph document into save-file v1.".to_string(),
                ),
            })
        }
        other => Err(format!(
            "unsupported legacy schema version {} (latest supported {})",
            other, GRAPH_DOCUMENT_VERSION
        )),
    }
}

fn migrate_legacy_v0_to_save_file(legacy: GraphSaveFileV0) -> GraphSaveFileV1 {
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

    let document = GraphDocument {
        version: GRAPH_DOCUMENT_VERSION,
        nodes,
        edges: legacy.links,
        view: legacy.ui,
        ..GraphDocument::default()
    };

    GraphSaveFileV1 {
        format: GRAPH_SAVE_FILE_FORMAT.to_string(),
        format_version: GRAPH_SAVE_FILE_VERSION,
        generator: None,
        meta: GraphSaveMetaV1::default(),
        document: GraphDocumentV1::from_document(&document),
    }
}
