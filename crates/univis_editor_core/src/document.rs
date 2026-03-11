use std::collections::{HashMap, HashSet};

use bevy::prelude::{Entity, Resource};
use serde::{Deserialize, Serialize};

use crate::{graph_validation::would_create_cycle, node_definition::NodeId, value::NodeValue};

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

#[derive(Resource, Debug, Clone, Default)]
pub struct LiveGraphDocumentState {
    pub document: GraphDocument,
    pub entity_to_node_id: HashMap<Entity, u64>,
    pub node_id_to_entity: HashMap<u64, Entity>,
}

impl LiveGraphDocumentState {
    pub fn clear(&mut self) {
        self.document = GraphDocument::default();
        self.entity_to_node_id.clear();
        self.node_id_to_entity.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphDocumentOperationError {
    DuplicateNodeId(u64),
    MissingSourceNode(u64),
    MissingTargetNode(u64),
    SelfConnection(u64),
    InvalidOutputPort {
        node_id: u64,
        index: usize,
        output_count: usize,
    },
    InvalidInputPort {
        node_id: u64,
        index: usize,
        input_count: usize,
    },
    InputAlreadyConnected {
        node_id: u64,
        index: usize,
    },
    DuplicateEdge,
    CycleDetected {
        from_node_id: u64,
        to_node_id: u64,
    },
}

impl std::fmt::Display for GraphDocumentOperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateNodeId(node_id) => write!(f, "node id {} already exists", node_id),
            Self::MissingSourceNode(node_id) => write!(f, "source node {} does not exist", node_id),
            Self::MissingTargetNode(node_id) => write!(f, "target node {} does not exist", node_id),
            Self::SelfConnection(node_id) => write!(f, "node {} cannot connect to itself", node_id),
            Self::InvalidOutputPort {
                node_id,
                index,
                output_count,
            } => write!(
                f,
                "node {} output {} is out of bounds ({} outputs)",
                node_id, index, output_count
            ),
            Self::InvalidInputPort {
                node_id,
                index,
                input_count,
            } => write!(
                f,
                "node {} input {} is out of bounds ({} inputs)",
                node_id, index, input_count
            ),
            Self::InputAlreadyConnected { node_id, index } => {
                write!(f, "node {} input {} is already connected", node_id, index)
            }
            Self::DuplicateEdge => write!(f, "edge already exists"),
            Self::CycleDetected {
                from_node_id,
                to_node_id,
            } => write!(
                f,
                "connecting node {} to node {} would create a cycle",
                from_node_id, to_node_id
            ),
        }
    }
}

impl std::error::Error for GraphDocumentOperationError {}

impl GraphDocument {
    pub fn next_node_id(&self) -> u64 {
        self.nodes.iter().map(|node| node.id).max().unwrap_or(0) + 1
    }

    pub fn node(&self, id: u64) -> Option<&GraphDocumentNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn node_mut(&mut self, id: u64) -> Option<&mut GraphDocumentNode> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }

    pub fn spawn_node(
        &mut self,
        definition_id: NodeId,
        position: [f32; 2],
        input_count: usize,
        output_count: usize,
    ) -> u64 {
        let node_id = self.next_node_id();
        self.nodes.push(GraphDocumentNode {
            id: node_id,
            definition_id,
            position,
            inputs: vec![NodeValue::None; input_count],
            input_count,
            output_count,
        });
        node_id
    }

    pub fn insert_node(
        &mut self,
        node: GraphDocumentNode,
    ) -> Result<(), GraphDocumentOperationError> {
        if self.node(node.id).is_some() {
            return Err(GraphDocumentOperationError::DuplicateNodeId(node.id));
        }
        self.nodes.push(node);
        Ok(())
    }

    pub fn delete_node(&mut self, node_id: u64) -> bool {
        let original_len = self.nodes.len();
        self.nodes.retain(|node| node.id != node_id);
        if self.nodes.len() == original_len {
            return false;
        }

        self.edges
            .retain(|edge| edge.from_node_id != node_id && edge.to_node_id != node_id);
        self.view.selected_node_ids.retain(|selected| *selected != node_id);
        true
    }

    pub fn delete_nodes<I>(&mut self, node_ids: I) -> usize
    where
        I: IntoIterator<Item = u64>,
    {
        let ids: HashSet<u64> = node_ids.into_iter().collect();
        if ids.is_empty() {
            return 0;
        }

        let original_len = self.nodes.len();
        self.nodes.retain(|node| !ids.contains(&node.id));
        self.edges
            .retain(|edge| !ids.contains(&edge.from_node_id) && !ids.contains(&edge.to_node_id));
        self.view
            .selected_node_ids
            .retain(|selected| !ids.contains(selected));
        original_len.saturating_sub(self.nodes.len())
    }

    pub fn connect(
        &mut self,
        from_node_id: u64,
        from_index: usize,
        to_node_id: u64,
        to_index: usize,
    ) -> Result<(), GraphDocumentOperationError> {
        let from_node = self
            .node(from_node_id)
            .ok_or(GraphDocumentOperationError::MissingSourceNode(from_node_id))?;
        let to_node = self
            .node(to_node_id)
            .ok_or(GraphDocumentOperationError::MissingTargetNode(to_node_id))?;

        if from_node_id == to_node_id {
            return Err(GraphDocumentOperationError::SelfConnection(from_node_id));
        }

        if from_index >= from_node.output_count {
            return Err(GraphDocumentOperationError::InvalidOutputPort {
                node_id: from_node_id,
                index: from_index,
                output_count: from_node.output_count,
            });
        }

        if to_index >= to_node.input_count {
            return Err(GraphDocumentOperationError::InvalidInputPort {
                node_id: to_node_id,
                index: to_index,
                input_count: to_node.input_count,
            });
        }

        if self
            .edges
            .iter()
            .any(|edge| edge.to_node_id == to_node_id && edge.to_index == to_index)
        {
            return Err(GraphDocumentOperationError::InputAlreadyConnected {
                node_id: to_node_id,
                index: to_index,
            });
        }

        if self.edges.iter().any(|edge| {
            edge.from_node_id == from_node_id
                && edge.from_index == from_index
                && edge.to_node_id == to_node_id
                && edge.to_index == to_index
        }) {
            return Err(GraphDocumentOperationError::DuplicateEdge);
        }

        if would_create_cycle(
            self.edges.iter().map(|edge| (edge.from_node_id, edge.to_node_id)),
            from_node_id,
            to_node_id,
        ) {
            return Err(GraphDocumentOperationError::CycleDetected {
                from_node_id,
                to_node_id,
            });
        }

        self.edges.push(GraphDocumentEdge {
            from_node_id,
            from_index,
            to_node_id,
            to_index,
        });

        Ok(())
    }

    pub fn disconnect_input(&mut self, to_node_id: u64, to_index: usize) -> bool {
        let original_len = self.edges.len();
        self.edges
            .retain(|edge| !(edge.to_node_id == to_node_id && edge.to_index == to_index));
        original_len != self.edges.len()
    }

    pub fn disconnect_edge(
        &mut self,
        from_node_id: u64,
        from_index: usize,
        to_node_id: u64,
        to_index: usize,
    ) -> bool {
        let original_len = self.edges.len();
        self.edges.retain(|edge| {
            !(edge.from_node_id == from_node_id
                && edge.from_index == from_index
                && edge.to_node_id == to_node_id
                && edge.to_index == to_index)
        });
        original_len != self.edges.len()
    }

    pub fn set_selected_nodes<I>(&mut self, node_ids: I)
    where
        I: IntoIterator<Item = u64>,
    {
        self.view.selected_node_ids = node_ids.into_iter().collect();
    }

    pub fn select_single_node(&mut self, node_id: u64) {
        self.view.selected_node_ids.clear();
        self.view.selected_node_ids.push(node_id);
    }

    pub fn clear_selection(&mut self) {
        self.view.selected_node_ids.clear();
    }

    pub fn set_camera(&mut self, camera: Option<GraphDocumentCameraState>) {
        self.view.camera = camera;
    }
}
