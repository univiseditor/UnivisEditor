use bevy::prelude::*;
use univis_graph_core::prelude::GraphValidationReport;

use crate::{
    document::{GraphDocument, LiveGraphDocumentState, graph_document_signature},
    node_registry::NodeRegistry,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct LiveGraphValidationState {
    pub report: GraphValidationReport,
    pub document_signature: Option<String>,
}

impl LiveGraphValidationState {
    pub fn set_report_for_document(
        &mut self,
        document: &GraphDocument,
        report: GraphValidationReport,
    ) {
        self.document_signature = graph_document_signature(document).ok();
        self.report = report;
    }
}

pub fn refresh_live_graph_validation_state_system(
    live_document: Res<LiveGraphDocumentState>,
    registry: Res<NodeRegistry>,
    mut validation_state: ResMut<LiveGraphValidationState>,
) {
    if !live_document.is_changed() && !registry.is_changed() {
        return;
    }

    let current_signature = graph_document_signature(&live_document.document).ok();
    if !registry.is_changed()
        && current_signature.is_some()
        && current_signature == validation_state.document_signature
    {
        return;
    }

    validation_state.report =
        univis_graph_core::prelude::validate_graph_document(&live_document.document, registry.core_registry());
    validation_state.document_signature = current_signature;
}
