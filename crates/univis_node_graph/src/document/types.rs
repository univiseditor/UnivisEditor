use std::collections::HashMap;

use bevy::prelude::{Entity, Resource};
pub use univis_graph_core::document::{
    GRAPH_DOCUMENT_VERSION, GraphDocumentCameraState, GraphDocumentEdge,
    GraphDocumentOperationError, GraphDocumentSelectionBoundarySummary, GraphDocumentViewState,
};
use univis_graph_core::document::{
    GraphDocument as CoreGraphDocument, GraphDocumentNode as CoreGraphDocumentNode,
    GraphDocumentPrefab as CoreGraphDocumentPrefab,
    GraphDocumentSubgraph as CoreGraphDocumentSubgraph,
};
use univis_scene::EntityValue;

use crate::{node_definition::NodeId, value::NodeValue};

pub type GraphDocument = CoreGraphDocument<NodeValue, EntityValue>;
pub type GraphDocumentPrefab = CoreGraphDocumentPrefab<EntityValue>;
pub type GraphDocumentSubgraph = CoreGraphDocumentSubgraph<NodeValue, EntityValue>;
pub type GraphDocumentNode = CoreGraphDocumentNode<NodeValue>;

pub fn graph_document_signature(document: &GraphDocument) -> Result<String, String> {
    serde_json::to_string(document)
        .map_err(|err| format!("signature serialization failed: {}", err))
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphDocumentBuildIssue {
    MissingSourceNodeMapping {
        from_entity: Entity,
        to_entity: Entity,
        from_index: usize,
        to_index: usize,
    },
    MissingTargetNodeMapping {
        from_entity: Entity,
        to_entity: Entity,
        from_index: usize,
        to_index: usize,
    },
    RejectedEdge {
        edge: GraphDocumentEdge,
        error: GraphDocumentOperationError,
    },
}

impl std::fmt::Display for GraphDocumentBuildIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSourceNodeMapping {
                from_entity,
                to_entity,
                from_index,
                to_index,
            } => write!(
                f,
                "edge {}:{} -> {}:{} references a source entity that is missing from the node snapshot mapping",
                from_entity.index(),
                from_index,
                to_entity.index(),
                to_index
            ),
            Self::MissingTargetNodeMapping {
                from_entity,
                to_entity,
                from_index,
                to_index,
            } => write!(
                f,
                "edge {}:{} -> {}:{} references a target entity that is missing from the node snapshot mapping",
                from_entity.index(),
                from_index,
                to_entity.index(),
                to_index
            ),
            Self::RejectedEdge { edge, error } => write!(
                f,
                "edge {}:{} -> {}:{} was rejected while rebuilding the live document: {}",
                edge.from_node_id, edge.from_index, edge.to_node_id, edge.to_index, error
            ),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GraphDocumentBuildResult {
    pub document: GraphDocument,
    pub entity_to_node_id: HashMap<Entity, u64>,
    pub node_id_to_entity: HashMap<u64, Entity>,
    pub issues: Vec<GraphDocumentBuildIssue>,
}
