use std::collections::HashMap;

use bevy::prelude::{Entity, Resource};
use serde::{Deserialize, Serialize};
use univis_scene::EntityValue;

use crate::{node_definition::NodeId, value::NodeValue};

pub const GRAPH_DOCUMENT_VERSION: u32 = 1;

fn default_graph_document_version() -> u32 {
    GRAPH_DOCUMENT_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocument {
    #[serde(default = "default_graph_document_version")]
    pub version: u32,
    #[serde(default)]
    pub nodes: Vec<GraphDocumentNode>,
    #[serde(default)]
    pub edges: Vec<GraphDocumentEdge>,
    #[serde(default)]
    pub prefabs: Vec<GraphDocumentPrefab>,
    #[serde(default)]
    pub subgraphs: Vec<GraphDocumentSubgraph>,
    #[serde(default)]
    pub view: GraphDocumentViewState,
}

impl Default for GraphDocument {
    fn default() -> Self {
        Self {
            version: GRAPH_DOCUMENT_VERSION,
            nodes: Vec::new(),
            edges: Vec::new(),
            prefabs: Vec::new(),
            subgraphs: Vec::new(),
            view: GraphDocumentViewState::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphDocumentPrefab {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub root: EntityValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocumentSubgraph {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub document: Box<GraphDocument>,
}

impl Default for GraphDocumentSubgraph {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            document: Box::new(GraphDocument::default()),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphDocumentEdge {
    pub from_node_id: u64,
    pub from_index: usize,
    pub to_node_id: u64,
    pub to_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GraphDocumentSelectionBoundarySummary {
    pub selected_node_ids: Vec<u64>,
    pub internal_edges: Vec<GraphDocumentEdge>,
    pub incoming_edges: Vec<GraphDocumentEdge>,
    pub outgoing_edges: Vec<GraphDocumentEdge>,
}

impl GraphDocumentSelectionBoundarySummary {
    pub fn selected_node_count(&self) -> usize {
        self.selected_node_ids.len()
    }

    pub fn internal_edge_count(&self) -> usize {
        self.internal_edges.len()
    }

    pub fn incoming_edge_count(&self) -> usize {
        self.incoming_edges.len()
    }

    pub fn outgoing_edge_count(&self) -> usize {
        self.outgoing_edges.len()
    }

    pub fn omitted_edge_count(&self) -> usize {
        self.incoming_edges.len() + self.outgoing_edges.len()
    }
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

#[derive(Resource, Debug, Clone, Default)]
pub struct LiveGraphDocumentState {
    pub document: GraphDocument,
    pub entity_to_node_id: HashMap<Entity, u64>,
    pub node_id_to_entity: HashMap<u64, Entity>,
}

#[derive(Debug, Clone)]
pub struct GraphDocumentNodeSnapshot {
    pub entity: Entity,
    pub definition_id: NodeId,
    pub position: [f32; 2],
    pub inputs: Vec<NodeValue>,
    pub input_count: usize,
    pub output_count: usize,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct GraphDocumentEdgeSnapshot {
    pub from_entity: Entity,
    pub from_index: usize,
    pub to_entity: Entity,
    pub to_index: usize,
}

#[derive(Debug, Clone, Default)]
pub struct GraphDocumentBuildResult {
    pub document: GraphDocument,
    pub entity_to_node_id: HashMap<Entity, u64>,
    pub node_id_to_entity: HashMap<u64, Entity>,
}
