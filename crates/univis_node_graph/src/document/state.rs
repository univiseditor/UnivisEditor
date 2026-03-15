use bevy::prelude::Entity;

use super::{
    build_graph_document_from_snapshots, GraphDocument, GraphDocumentCameraState,
    GraphDocumentEdgeSnapshot, GraphDocumentNodeSnapshot, LiveGraphDocumentState,
};

impl LiveGraphDocumentState {
    pub fn clear(&mut self) {
        self.document = GraphDocument::default();
        self.entity_to_node_id.clear();
        self.node_id_to_entity.clear();
    }

    pub fn rebuild_from_snapshots<I, J>(
        &mut self,
        node_snapshots: I,
        edge_snapshots: J,
        camera: Option<GraphDocumentCameraState>,
    ) where
        I: IntoIterator<Item = GraphDocumentNodeSnapshot>,
        J: IntoIterator<Item = GraphDocumentEdgeSnapshot>,
    {
        let build = build_graph_document_from_snapshots(
            node_snapshots,
            edge_snapshots,
            camera,
            Some(&self.entity_to_node_id),
        );

        let mut document = build.document;
        document.prefabs = self.document.prefabs.clone();
        document.subgraphs = self.document.subgraphs.clone();

        self.document = document;
        self.entity_to_node_id = build.entity_to_node_id;
        self.node_id_to_entity = build.node_id_to_entity;
    }

    pub fn node_id_for_entity(&self, entity: Entity) -> Option<u64> {
        self.entity_to_node_id.get(&entity).copied()
    }

    pub fn entity_for_node_id(&self, node_id: u64) -> Option<Entity> {
        self.node_id_to_entity.get(&node_id).copied()
    }

    pub fn selected_entities(&self) -> Vec<Entity> {
        self.document
            .selected_node_ids()
            .iter()
            .filter_map(|node_id| self.entity_for_node_id(*node_id))
            .collect()
    }

    pub fn select_single_entity(&mut self, entity: Entity) -> bool {
        let Some(node_id) = self.node_id_for_entity(entity) else {
            return false;
        };
        self.document.select_single_node(node_id);
        true
    }

    pub fn clear_selected_entities(&mut self) {
        self.document.clear_selection();
    }

    pub fn set_selected_entities<I>(&mut self, entities: I)
    where
        I: IntoIterator<Item = Entity>,
    {
        let node_ids: Vec<u64> = entities
            .into_iter()
            .filter_map(|entity| self.node_id_for_entity(entity))
            .collect();
        self.document.set_selected_nodes(node_ids);
    }

    pub fn delete_selected_entities(&mut self) -> Vec<Entity> {
        let selected_entities = self.selected_entities();
        self.document.delete_selected_nodes();
        selected_entities
    }

    pub fn disconnect_input_for_entity(&mut self, entity: Entity, input_index: usize) -> bool {
        let Some(node_id) = self.node_id_for_entity(entity) else {
            return false;
        };
        self.document.disconnect_input(node_id, input_index)
    }

    pub fn retain_existing_entities<F>(&mut self, mut exists: F) -> bool
    where
        F: FnMut(Entity) -> bool,
    {
        let original_mapping_count = self.entity_to_node_id.len();
        let original_selected_count = self.document.view.selected_node_ids.len();
        let original_node_count = self.document.nodes.len();
        let original_edge_count = self.document.edges.len();

        self.entity_to_node_id.retain(|entity, _| exists(*entity));
        self.node_id_to_entity
            .retain(|_, entity| self.entity_to_node_id.contains_key(entity));
        self.document
            .nodes
            .retain(|node| self.node_id_to_entity.contains_key(&node.id));
        self.document.edges.retain(|edge| {
            self.node_id_to_entity.contains_key(&edge.from_node_id)
                && self.node_id_to_entity.contains_key(&edge.to_node_id)
        });
        self.document
            .view
            .selected_node_ids
            .retain(|node_id| self.node_id_to_entity.contains_key(node_id));

        original_mapping_count != self.entity_to_node_id.len()
            || original_selected_count != self.document.view.selected_node_ids.len()
            || original_node_count != self.document.nodes.len()
            || original_edge_count != self.document.edges.len()
    }
}
