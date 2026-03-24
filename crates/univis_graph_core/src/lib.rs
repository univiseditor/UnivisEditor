pub mod document;
pub mod executable;
pub mod identity;
pub mod ports;
pub mod processing;
pub mod registry;
pub mod schema;
pub mod topology;
pub mod validation;

/// Default imports intended for ordinary downstream integrations.
///
/// More specialized executable, topology, and validation APIs remain available
/// from their defining modules instead of being pulled into the default import
/// surface.
pub mod prelude {
    pub use crate::document::{
        GraphDocument, GraphDocumentCameraState, GraphDocumentEdge, GraphDocumentNode,
        GraphDocumentOperationError, GraphDocumentPrefab, GraphDocumentSelectionBoundarySummary,
        GraphDocumentSubgraph, GraphDocumentViewState, GRAPH_DOCUMENT_VERSION,
    };
    pub use crate::executable::{
        ExecutableGraph, ExecutableNodeBuildStatus, ExecutableNodeRunOutcome,
        ExecutableNodeRunStatus,
    };
    pub use crate::identity::{ConnectionPolicy, NodeCategory, NodeId};
    pub use crate::ports::{PortDefinition, PortSchema};
    pub use crate::processing::{
        GraphNodeDefinition, NodeDefinition, ProcessContext, ProcessResult, ProcessValueAccess,
    };
    pub use crate::registry::{ArcGraphNodeDefinition, GraphNodeRegistry};
    pub use crate::schema::GraphSchema;
    pub use crate::topology::{connected_input_mask, would_create_cycle};
    pub use crate::validation::{
        validate_graph_document, validate_schema_connection_candidate,
        validate_structural_connection_candidate, GraphConnectionCandidate,
        GraphConnectionValidationError, GraphConnectionValidationOptions,
        GraphSchemaConnectionValidationContext, GraphStructuralConnectionValidationContext,
        GraphValidationIssue, GraphValidationIssueKind, GraphValidationReport,
    };
}
