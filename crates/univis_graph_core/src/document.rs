use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::identity::NodeId;
use crate::topology::would_create_cycle;

pub const GRAPH_DOCUMENT_VERSION: u32 = 1;

fn default_graph_document_version() -> u32 {
    GRAPH_DOCUMENT_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Value: Serialize, Prefab: Serialize",
    deserialize = "Value: Deserialize<'de> + Clone, Prefab: Deserialize<'de> + Clone"
))]
pub struct GraphDocument<Value, Prefab> {
    #[serde(default = "default_graph_document_version")]
    pub version: u32,
    #[serde(default)]
    pub nodes: Vec<GraphDocumentNode<Value>>,
    #[serde(default)]
    pub edges: Vec<GraphDocumentEdge>,
    #[serde(default)]
    pub prefabs: Vec<GraphDocumentPrefab<Prefab>>,
    #[serde(default)]
    pub subgraphs: Vec<GraphDocumentSubgraph<Value, Prefab>>,
    #[serde(default)]
    pub view: GraphDocumentViewState,
}

impl<Value, Prefab> Default for GraphDocument<Value, Prefab> {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Prefab: Serialize",
    deserialize = "Prefab: Deserialize<'de>"
))]
pub struct GraphDocumentPrefab<Prefab> {
    pub id: String,
    pub name: String,
    pub root: Prefab,
}

impl<Prefab> GraphDocumentPrefab<Prefab> {
    pub fn new(id: impl Into<String>, name: impl Into<String>, root: Prefab) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            root,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Value: Serialize, Prefab: Serialize",
    deserialize = "Value: Deserialize<'de> + Clone, Prefab: Deserialize<'de> + Clone"
))]
pub struct GraphDocumentSubgraph<Value, Prefab> {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub document: Box<GraphDocument<Value, Prefab>>,
}

impl<Value, Prefab> Default for GraphDocumentSubgraph<Value, Prefab> {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            document: Box::new(GraphDocument::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Value: Serialize",
    deserialize = "Value: Deserialize<'de> + Clone"
))]
pub struct GraphDocumentNode<Value> {
    pub id: u64,
    pub definition_id: NodeId,
    pub position: [f32; 2],
    #[serde(default)]
    pub inputs: Vec<Value>,
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

impl<Value, Prefab> GraphDocument<Value, Prefab>
where
    Value: Clone + Default,
    Prefab: Clone,
{
    pub fn prefab_count(&self) -> usize {
        self.prefabs.len()
    }

    pub fn subgraph_count(&self) -> usize {
        self.subgraphs.len()
    }

    pub fn prefab(&self, id: &str) -> Option<&GraphDocumentPrefab<Prefab>> {
        self.prefabs.iter().find(|prefab| prefab.id == id)
    }

    pub fn subgraph(&self, id: &str) -> Option<&GraphDocumentSubgraph<Value, Prefab>> {
        self.subgraphs.iter().find(|subgraph| subgraph.id == id)
    }

    pub fn upsert_prefab(&mut self, prefab: GraphDocumentPrefab<Prefab>) {
        if let Some(existing) = self
            .prefabs
            .iter_mut()
            .find(|existing| existing.id == prefab.id)
        {
            *existing = prefab;
        } else {
            self.prefabs.push(prefab);
        }
    }

    pub fn upsert_subgraph(&mut self, subgraph: GraphDocumentSubgraph<Value, Prefab>) {
        if let Some(existing) = self
            .subgraphs
            .iter_mut()
            .find(|existing| existing.id == subgraph.id)
        {
            *existing = subgraph;
        } else {
            self.subgraphs.push(subgraph);
        }
    }

    pub fn next_node_id(&self) -> u64 {
        self.nodes.iter().map(|node| node.id).max().unwrap_or(0) + 1
    }

    pub fn node(&self, id: u64) -> Option<&GraphDocumentNode<Value>> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn node_mut(&mut self, id: u64) -> Option<&mut GraphDocumentNode<Value>> {
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
            inputs: vec![Value::default(); input_count],
            input_count,
            output_count,
        });
        node_id
    }

    pub fn insert_node(
        &mut self,
        node: GraphDocumentNode<Value>,
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
        self.view
            .selected_node_ids
            .retain(|selected| *selected != node_id);
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
            self.edges
                .iter()
                .map(|edge| (edge.from_node_id, edge.to_node_id)),
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

    pub fn selected_node_ids(&self) -> &[u64] {
        &self.view.selected_node_ids
    }

    pub fn delete_selected_nodes(&mut self) -> usize {
        self.delete_nodes(self.view.selected_node_ids.clone())
    }

    pub fn set_camera(&mut self, camera: Option<GraphDocumentCameraState>) {
        self.view.camera = camera;
    }

    pub fn selected_subgraph_boundary_summary(&self) -> Option<GraphDocumentSelectionBoundarySummary> {
        self.subgraph_boundary_summary(self.selected_node_ids().iter().copied())
    }

    pub fn subgraph_boundary_summary<I>(
        &self,
        node_ids: I,
    ) -> Option<GraphDocumentSelectionBoundarySummary>
    where
        I: IntoIterator<Item = u64>,
    {
        let selected = node_ids.into_iter().collect::<HashSet<_>>();
        if selected.is_empty() {
            return None;
        }

        let mut selected_node_ids = self
            .nodes
            .iter()
            .filter_map(|node| selected.contains(&node.id).then_some(node.id))
            .collect::<Vec<_>>();
        if selected_node_ids.is_empty() {
            return None;
        }
        selected_node_ids.sort_unstable();

        let mut summary = GraphDocumentSelectionBoundarySummary {
            selected_node_ids,
            ..Default::default()
        };

        for edge in &self.edges {
            let from_selected = selected.contains(&edge.from_node_id);
            let to_selected = selected.contains(&edge.to_node_id);
            match (from_selected, to_selected) {
                (true, true) => summary.internal_edges.push(edge.clone()),
                (false, true) => summary.incoming_edges.push(edge.clone()),
                (true, false) => summary.outgoing_edges.push(edge.clone()),
                (false, false) => {}
            }
        }

        Some(summary)
    }

    pub fn capture_selected_subgraph(&mut self, id: String, name: String) -> bool {
        let Some(boundary) = self.selected_subgraph_boundary_summary() else {
            return false;
        };
        let selected = boundary
            .selected_node_ids
            .iter()
            .copied()
            .collect::<HashSet<u64>>();

        let selected_nodes: Vec<GraphDocumentNode<Value>> = self
            .nodes
            .iter()
            .filter(|node| selected.contains(&node.id))
            .cloned()
            .collect();
        if selected_nodes.is_empty() {
            return false;
        }

        let min_x = selected_nodes
            .iter()
            .map(|node| node.position[0])
            .fold(f32::INFINITY, f32::min);
        let min_y = selected_nodes
            .iter()
            .map(|node| node.position[1])
            .fold(f32::INFINITY, f32::min);

        let nodes = selected_nodes
            .into_iter()
            .map(|mut node| {
                node.position[0] -= min_x;
                node.position[1] -= min_y;
                node
            })
            .collect();
        let edges = boundary.internal_edges;

        self.upsert_subgraph(GraphDocumentSubgraph {
            id,
            name,
            document: Box::new(GraphDocument {
                version: GRAPH_DOCUMENT_VERSION,
                nodes,
                edges,
                prefabs: self.prefabs.clone(),
                subgraphs: Vec::new(),
                view: GraphDocumentViewState::default(),
            }),
        });
        true
    }

    pub fn merged_with_subgraph_instance(
        &self,
        subgraph_id: &str,
        origin: [f32; 2],
    ) -> Option<GraphDocument<Value, Prefab>> {
        let subgraph = self.subgraph(subgraph_id)?;
        if subgraph.document.nodes.is_empty() {
            return None;
        }

        let min_x = subgraph
            .document
            .nodes
            .iter()
            .map(|node| node.position[0])
            .fold(f32::INFINITY, f32::min);
        let min_y = subgraph
            .document
            .nodes
            .iter()
            .map(|node| node.position[1])
            .fold(f32::INFINITY, f32::min);

        let mut merged = self.clone();
        let mut next_node_id = merged.next_node_id();
        let mut selected_node_ids = Vec::new();
        let mut node_id_map = HashMap::new();

        for node in &subgraph.document.nodes {
            let new_id = next_node_id;
            next_node_id += 1;
            node_id_map.insert(node.id, new_id);

            let mut cloned = node.clone();
            cloned.id = new_id;
            cloned.position[0] = origin[0] + (cloned.position[0] - min_x);
            cloned.position[1] = origin[1] + (cloned.position[1] - min_y);
            selected_node_ids.push(new_id);
            merged.nodes.push(cloned);
        }

        for edge in &subgraph.document.edges {
            let Some(from_node_id) = node_id_map.get(&edge.from_node_id).copied() else {
                continue;
            };
            let Some(to_node_id) = node_id_map.get(&edge.to_node_id).copied() else {
                continue;
            };

            merged.edges.push(GraphDocumentEdge {
                from_node_id,
                from_index: edge.from_index,
                to_node_id,
                to_index: edge.to_index,
            });
        }

        for prefab in &subgraph.document.prefabs {
            merged.upsert_prefab(prefab.clone());
        }
        for nested_subgraph in &subgraph.document.subgraphs {
            merged.upsert_subgraph(nested_subgraph.clone());
        }

        merged.set_selected_nodes(selected_node_ids);
        Some(merged)
    }
}
