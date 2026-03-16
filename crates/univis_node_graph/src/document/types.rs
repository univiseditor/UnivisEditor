use std::collections::HashMap;

use bevy::prelude::{Entity, Resource};
pub use univis_graph_core::document::{
    GRAPH_DOCUMENT_VERSION, GraphDocumentCameraState, GraphDocumentEdge,
    GraphDocumentOperationError, GraphDocumentSelectionBoundarySummary,
    GraphDocumentViewState,
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
