use bevy::prelude::*;

pub use univis_graph_core::prelude::{
    GraphTopologyAnalysis, GraphValidationIssue, GraphValidationIssueKind, GraphValidationReport,
    analyze_graph_topology, connected_input_mask, validate_graph_document_structure,
    would_create_cycle,
};

use crate::{
    document::{GraphDocument, LiveGraphDocumentState},
    node_registry::NodeRegistry,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct LiveGraphValidationState {
    pub report: GraphValidationReport,
}

pub fn refresh_live_graph_validation_state_system(
    live_document: Res<LiveGraphDocumentState>,
    registry: Res<NodeRegistry>,
    mut validation_state: ResMut<LiveGraphValidationState>,
) {
    if !live_document.is_changed() && !registry.is_changed() {
        return;
    }

    validation_state.report = validate_graph_document_report(&live_document.document, &registry);
}

pub fn validate_graph_document_report(
    document: &GraphDocument,
    registry: &NodeRegistry,
) -> GraphValidationReport {
    univis_graph_core::prelude::validate_graph_document(document, registry.core_registry())
}

pub fn validate_graph_document(
    document: &GraphDocument,
    registry: &NodeRegistry,
) -> Vec<GraphValidationIssue> {
    validate_graph_document_report(document, registry).issues
}
