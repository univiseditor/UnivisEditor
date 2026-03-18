pub use univis_graph_core::prelude::{
    GraphTopologyAnalysis, GraphValidationIssue, GraphValidationIssueKind, analyze_graph_topology,
    connected_input_mask, validate_graph_document_structure, would_create_cycle,
};

use crate::{document::GraphDocument, node_registry::NodeRegistry};

pub fn validate_graph_document(
    document: &GraphDocument,
    registry: &NodeRegistry,
) -> Vec<GraphValidationIssue> {
    univis_graph_core::prelude::validate_graph_document(document, registry.core_registry()).issues
}
