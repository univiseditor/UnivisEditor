use serde::{Deserialize, Serialize};

use crate::{node_definition::NodeId, value::NodeValue};

pub const GRAPH_DOCUMENT_VERSION: u32 = 1;

fn default_graph_document_version() -> u32 {
    GRAPH_DOCUMENT_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphDocument {
    #[serde(default = "default_graph_document_version")]
    pub version: u32,
    #[serde(default)]
    pub nodes: Vec<GraphDocumentNode>,
    #[serde(default)]
    pub edges: Vec<GraphDocumentEdge>,
    #[serde(default)]
    pub view: GraphDocumentViewState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocumentNode {
    pub id: u64,
    pub definition_id: NodeId,
    pub position: [f32; 2],
    #[serde(default)]
    pub inputs: Vec<NodeValue>,
    pub input_count: usize,
    pub output_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocumentEdge {
    pub from_node_id: u64,
    pub from_index: usize,
    pub to_node_id: u64,
    pub to_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphDocumentViewState {
    pub camera: Option<GraphDocumentCameraState>,
    #[serde(default)]
    pub selected_node_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocumentCameraState {
    pub translation: [f32; 3],
    pub ortho_scale: f32,
}
