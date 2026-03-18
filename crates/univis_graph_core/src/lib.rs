pub mod document;
pub mod identity;
pub mod ports;
pub mod processing;
pub mod registry;
pub mod schema;
pub mod topology;
pub mod validation;

pub mod prelude {
    pub use crate::document::{
        GRAPH_DOCUMENT_VERSION, GraphDocument, GraphDocumentCameraState, GraphDocumentEdge,
        GraphDocumentNode, GraphDocumentOperationError, GraphDocumentPrefab,
        GraphDocumentSelectionBoundarySummary, GraphDocumentSubgraph, GraphDocumentViewState,
    };
    pub use crate::identity::{ConnectionPolicy, NodeCategory, NodeId};
    pub use crate::ports::{PortDefinition, PortSchema};
    pub use crate::processing::{
        GraphNodeDefinition, NodeDefinition, ProcessContext, ProcessResult, ProcessValueAccess,
    };
    pub use crate::registry::{ArcGraphNodeDefinition, GraphNodeRegistry};
    pub use crate::schema::GraphSchema;
    pub use crate::topology::{
        GraphTopologyAnalysis, analyze_graph_topology, connected_input_mask, would_create_cycle,
    };
    pub use crate::validation::{
        GraphValidationIssue, GraphValidationIssueKind, GraphValidationReport,
        validate_graph_document, validate_graph_document_structure,
    };
}
