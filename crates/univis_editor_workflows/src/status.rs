use bevy::prelude::*;
use univis_editor_persistence::graph_persistence::{
    ApplyGraphDocumentRequest, GraphPersistenceSettings, GraphPersistenceStatus,
    GraphPersistenceStatusMessage, GraphPersistenceStatusSeverity,
};
use univis_node_graph::document::{GraphDocument, GraphDocumentOperationError};

pub(super) struct WorkflowDocumentChange {
    pub document: GraphDocument,
    pub source_label: String,
    pub success_message: String,
}

#[derive(Debug, Clone)]
pub(super) enum WorkflowStatusFailure {
    Message {
        severity: GraphPersistenceStatusSeverity,
        text: String,
    },
    DocumentOperation {
        action: String,
        error: GraphDocumentOperationError,
    },
}

impl WorkflowStatusFailure {
    pub(super) fn warning(text: impl Into<String>) -> Self {
        Self::Message {
            severity: GraphPersistenceStatusSeverity::Warning,
            text: text.into(),
        }
    }

    pub(super) fn error(text: impl Into<String>) -> Self {
        Self::Message {
            severity: GraphPersistenceStatusSeverity::Error,
            text: text.into(),
        }
    }

    pub(super) fn document_operation(
        action: impl Into<String>,
        error: GraphDocumentOperationError,
    ) -> Self {
        Self::DocumentOperation {
            action: action.into(),
            error,
        }
    }

    fn severity(&self) -> GraphPersistenceStatusSeverity {
        match self {
            Self::Message { severity, .. } => *severity,
            Self::DocumentOperation { error, .. } => match error {
                GraphDocumentOperationError::EmptySelection
                | GraphDocumentOperationError::MissingSubgraph(_)
                | GraphDocumentOperationError::EmptyDocumentFragment { .. } => {
                    GraphPersistenceStatusSeverity::Warning
                }
                GraphDocumentOperationError::DuplicateNodeId(_)
                | GraphDocumentOperationError::MissingSourceNode(_)
                | GraphDocumentOperationError::MissingTargetNode(_)
                | GraphDocumentOperationError::SelfConnection(_)
                | GraphDocumentOperationError::InvalidOutputPort { .. }
                | GraphDocumentOperationError::InvalidInputPort { .. }
                | GraphDocumentOperationError::InputAlreadyConnected { .. }
                | GraphDocumentOperationError::DuplicateEdge
                | GraphDocumentOperationError::CycleDetected { .. } => {
                    GraphPersistenceStatusSeverity::Error
                }
            },
        }
    }

    fn text(&self) -> String {
        match self {
            Self::Message { text, .. } => text.clone(),
            Self::DocumentOperation { action, error } => {
                format!("Failed to {}: {}.", action, error)
            }
        }
    }
}

pub(super) fn set_asset_status(
    status: &mut GraphPersistenceStatus,
    severity: GraphPersistenceStatusSeverity,
    text: String,
    settings: &GraphPersistenceSettings,
    time: &Time,
) {
    status.active = Some(GraphPersistenceStatusMessage {
        text,
        severity,
        expires_at_secs: time.elapsed_secs_f64() + settings.status_duration_secs as f64,
    });
}

pub(super) fn publish_workflow_success(
    status: &mut GraphPersistenceStatus,
    text: impl Into<String>,
    settings: &GraphPersistenceSettings,
    time: &Time,
) {
    set_asset_status(
        status,
        GraphPersistenceStatusSeverity::Info,
        text.into(),
        settings,
        time,
    );
}

pub(super) fn publish_workflow_failure(
    status: &mut GraphPersistenceStatus,
    failure: WorkflowStatusFailure,
    settings: &GraphPersistenceSettings,
    time: &Time,
) {
    set_asset_status(status, failure.severity(), failure.text(), settings, time);
}

pub(super) fn apply_document_change(
    apply_writer: &mut MessageWriter<ApplyGraphDocumentRequest>,
    status: &mut GraphPersistenceStatus,
    settings: &GraphPersistenceSettings,
    time: &Time,
    change: WorkflowDocumentChange,
) {
    apply_writer.write(ApplyGraphDocumentRequest {
        document: change.document,
        source_label: change.source_label,
        track_for_undo: true,
        validation_report: None,
    });
    publish_workflow_success(status, change.success_message, settings, time);
}
