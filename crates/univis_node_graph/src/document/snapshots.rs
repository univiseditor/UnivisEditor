use std::collections::HashMap;

use bevy::prelude::Entity;

use super::{
    GraphDocument, GraphDocumentBuildResult, GraphDocumentCameraState, GraphDocumentEdge,
    GraphDocumentEdgeSnapshot, GraphDocumentNode, GraphDocumentNodeSnapshot,
    GRAPH_DOCUMENT_VERSION,
};

pub fn build_graph_document_from_snapshots<I, J>(
    node_snapshots: I,
    edge_snapshots: J,
    camera: Option<GraphDocumentCameraState>,
    existing_ids: Option<&HashMap<Entity, u64>>,
) -> GraphDocumentBuildResult
where
    I: IntoIterator<Item = GraphDocumentNodeSnapshot>,
    J: IntoIterator<Item = GraphDocumentEdgeSnapshot>,
{
    // Reuse known ids when possible so undo/load flows keep stable node identities across rebuilds.
    let mut nodes: Vec<_> = node_snapshots.into_iter().collect();
    nodes.sort_by_key(|node| node.entity.index());

    let mut entity_to_node_id = HashMap::new();
    let mut node_id_to_entity = HashMap::new();
    let mut selected_node_ids = Vec::new();
    let mut next_node_id = existing_ids
        .and_then(|ids| ids.values().copied().max())
        .unwrap_or(0)
        + 1;
    let mut document = GraphDocument {
        version: GRAPH_DOCUMENT_VERSION,
        ..GraphDocument::default()
    };

    for node in nodes {
        let node_id = existing_ids
            .and_then(|ids| ids.get(&node.entity).copied())
            .unwrap_or_else(|| {
                let current = next_node_id;
                next_node_id += 1;
                current
            });

        entity_to_node_id.insert(node.entity, node_id);
        node_id_to_entity.insert(node_id, node.entity);

        if node.selected {
            selected_node_ids.push(node_id);
        }

        document
            .insert_node(GraphDocumentNode {
                id: node_id,
                definition_id: node.definition_id,
                position: node.position,
                inputs: node.inputs,
                input_count: node.input_count,
                output_count: node.output_count,
            })
            .expect("build_graph_document_from_snapshots assigns unique node ids");
    }

    for edge in edge_snapshots {
        let Some(from_node_id) = entity_to_node_id.get(&edge.from_entity).copied() else {
            continue;
        };
        let Some(to_node_id) = entity_to_node_id.get(&edge.to_entity).copied() else {
            continue;
        };

        document.edges.push(GraphDocumentEdge {
            from_node_id,
            from_index: edge.from_index,
            to_node_id,
            to_index: edge.to_index,
        });
    }

    document.set_camera(camera);
    document.set_selected_nodes(selected_node_ids);

    GraphDocumentBuildResult {
        document,
        entity_to_node_id,
        node_id_to_entity,
    }
}
