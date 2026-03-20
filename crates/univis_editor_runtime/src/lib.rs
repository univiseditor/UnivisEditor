mod connectivity;
mod diagnostics;
mod processing;
mod scene_outputs;
mod world_sync;

use bevy::prelude::*;

use self::connectivity::rebuild_connectivity_index_system;
use self::diagnostics::propagate_and_process_nodes_system;
use self::processing::initialize_node_defaults_system;
use self::scene_outputs::collect_scene_outputs_system;
use self::world_sync::{cleanup_orphaned_scene_roots_system, sync_scene_nodes_to_world_system};

pub use self::connectivity::{
    GraphConnectivityIndex, GraphExecutableRuntimeState, GraphResolvedInputs, NodeInputSignature,
    NodeOutputSignature,
};
pub use self::diagnostics::{
    GraphRuntimeDiagnostics, GraphRuntimeIssueSeverity, GraphRuntimeNodeIssue, GraphRuntimeTrace,
    GraphRuntimeTraceEntry, GraphRuntimeTraceSettings,
};
pub use self::scene_outputs::{GraphSceneOutputMode, GraphSceneOutputs, GraphSceneSinkOutput};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeRuntimeSystemSet {
    Prepare,
    Process,
    CollectOutputs,
    SyncWorld,
}

pub struct NodeRuntimePlugin;

impl Plugin for NodeRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphRuntimeDiagnostics>()
            .init_resource::<GraphRuntimeTraceSettings>()
            .init_resource::<GraphRuntimeTrace>()
            .init_resource::<connectivity::GraphExecutableRuntimeState>()
            .init_resource::<connectivity::GraphConnectivityIndex>()
            .init_resource::<connectivity::GraphResolvedInputs>()
            .init_resource::<GraphSceneOutputs>()
            .configure_sets(
                Update,
                (
                    NodeRuntimeSystemSet::Prepare,
                    NodeRuntimeSystemSet::Process,
                    NodeRuntimeSystemSet::CollectOutputs,
                    NodeRuntimeSystemSet::SyncWorld,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    initialize_node_defaults_system,
                    rebuild_connectivity_index_system,
                )
                    .chain()
                    .in_set(NodeRuntimeSystemSet::Prepare),
            )
            .add_systems(
                Update,
                propagate_and_process_nodes_system.in_set(NodeRuntimeSystemSet::Process),
            )
            .add_systems(
                Update,
                collect_scene_outputs_system.in_set(NodeRuntimeSystemSet::CollectOutputs),
            )
            .add_systems(
                Update,
                (
                    sync_scene_nodes_to_world_system,
                    cleanup_orphaned_scene_roots_system,
                )
                    .chain()
                    .in_set(NodeRuntimeSystemSet::SyncWorld),
            );
    }
}

pub mod prelude {
    pub use crate::{
        GraphConnectivityIndex, GraphExecutableRuntimeState, GraphResolvedInputs,
        GraphRuntimeDiagnostics, GraphRuntimeTrace, GraphRuntimeTraceEntry,
        GraphRuntimeTraceSettings, GraphSceneOutputMode, GraphSceneOutputs, GraphSceneSinkOutput,
        NodeInputSignature, NodeOutputSignature, NodeRuntimePlugin, NodeRuntimeSystemSet,
    };
}
