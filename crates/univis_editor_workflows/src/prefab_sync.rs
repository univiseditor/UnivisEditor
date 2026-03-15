use bevy::prelude::*;
use std::any::Any;
use univis_node_graph::{
    prelude::{GraphNode, LiveGraphDocumentState},
    value::NodeValue,
};

pub(super) fn sync_prefab_instance_nodes_system(
    live_document: Res<LiveGraphDocumentState>,
    mut q_nodes: Query<&mut GraphNode>,
) {
    // Prefab instance nodes resolve saved graph assets into runtime-ready entity values before processing.
    for mut node in q_nodes.iter_mut() {
        if node.definition_id.as_str() != "scene/prefab_instance" {
            continue;
        }

        let prefab_id = node
            .values
            .inputs
            .first()
            .and_then(NodeValue::as_string)
            .map(str::trim)
            .unwrap_or("");
        let root = live_document
            .document
            .prefabs
            .iter()
            .find(|prefab| prefab.id == prefab_id)
            .map(|prefab| prefab.root.clone());

        node.custom_data = root.map(|root| Box::new(root) as Box<dyn Any + Send + Sync>);
    }
}
