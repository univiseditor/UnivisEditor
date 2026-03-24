# Runtime Consolidation And Legacy Cleanup Roadmap

## Goal

Move the project from:

- a mixed architecture where `ExecutableGraph` is present but transitional runtime and UI layers still exist

to:

- a single execution model centered on `ExecutableGraph`
- a thinner runtime adapter layer
- a cleaner editor and persistence stack with old paths removed

The intended outcome is that authored truth stays in `GraphDocument` and
`AuthoredNodeInputs`, execution truth stays in `ExecutableGraph`, and any ECS
projection exists only as a display or integration cache.

## Decisions Locked Before Cleanup

- `ExecutableGraph` is the only execution truth.
- `GraphDocument` and `AuthoredNodeInputs` remain the authored truth.
- `GraphNode.values` in ECS are cache or projection data, not authoritative execution state.
- `GraphConnectivityIndex` and `GraphResolvedInputs` are transitional compatibility resources and should be removed.
- Popup-based node editing is retired; inline node editing is the only editor path.
- Validation should be consumed from `graph_core` or `LiveGraphValidationState`, not reimplemented in adapters.
- Cleanup should prefer deletion and direct usage over adding new wrappers.

## Success Signals

- No runtime system needs `GraphConnectivityIndex`.
- No runtime or UI system needs `GraphResolvedInputs`.
- `node_popup.rs` and `NodePopupState` no longer exist.
- Runtime diagnostics and scene outputs read from `ExecutableGraph` directly.
- Validation flows through `GraphValidationReport` and executable node diagnostics without parallel legacy paths.
- The public surface of runtime, UI, and node-graph crates is smaller and clearer than before cleanup.

## Phase 0: Lock The Cleanup Model

- [x] List the remaining transitional structures and mark each one as:
  - [x] truth
  - [x] cache
  - [x] adapter
  - [x] legacy
- [x] Update `docs/graph-core-execution-model.md` where the cleanup changes ownership language
- [x] Make explicit which ECS resources still exist only for compatibility
- [x] Define the deletion order so cleanup can happen without breaking the editor

## Phase 1: Remove Runtime Projection Resources

- [x] Migrate consumers away from:
  - [x] `GraphConnectivityIndex`
  - [x] `GraphResolvedInputs`
- [x] Move the following readers to `GraphExecutableRuntimeState.graph` directly:
  - [x] scene output collection
  - [x] connection diagnostics
  - [x] runtime-facing tests that still inspect projected resolved inputs
- [x] Add any missing read helpers on `ExecutableGraph` or `GraphExecutableRuntimeState`
- [x] Delete:
  - [x] `GraphConnectivityIndex`
  - [x] `GraphResolvedInputs`
  - [x] `project_runtime_resources`

## Phase 2: Thin The Runtime Adapter Layer

- [x] Keep `GraphExecutableRuntimeState` focused on:
  - [x] `ExecutableGraph`
  - [x] entity to node-id mapping
  - [x] node-id to entity mapping
- [x] Remove duplicated runtime knowledge that already exists inside `ExecutableGraph`
- [x] Keep `GraphRuntimeDiagnostics` only as a presentation resource, not as a competing source of truth
- [x] Audit runtime systems for duplicated:
  - [x] adjacency logic
  - [x] execution-order logic
  - [x] blocked-state logic
- [x] Delete any remaining duplicated logic after moving or reusing the core version

## Phase 3: Remove Popup Editing Legacy

- [x] Delete `crates/univis_editor_ui/src/node_popup.rs`
- [x] Remove `NodePopupState` from:
  - [x] `NodeUiPlugin`
  - [x] selection systems
  - [x] state-sync systems
  - [x] persistence cleanup state
  - [x] workflow and persistence tests
- [x] Remove any popup-only overlay state or surface flags that no longer have a user-facing role
- [x] Ensure settings actions now map to:
  - [x] inline editing
  - [x] inline section expand or collapse
  - [x] or nothing, if the control is obsolete

## Phase 4: Simplify Validation Access

- [x] Move consumers to `GraphValidationReport` and executable node diagnostics directly where practical
- [x] Keep only the live validation resource and the minimum adapter glue still needed in `node_graph`
- [x] Remove compatibility helpers that only rewrap core validation results
- [x] Audit persistence, UI, and tests for old `Vec<GraphValidationIssue>` style access
- [x] Prefer one validation truth path across:
  - [x] editor
  - [x] persistence
  - [x] runtime

## Phase 5: Clarify ECS Cache Boundaries

- [x] Make it explicit in code and naming that `GraphNode.values` are projection data
- [x] Audit systems that read `GraphNode.values` for decisions
- [x] Move decision-making reads to:
  - [x] `AuthoredNodeInputs`
  - [x] `ExecutableGraph`
- [x] Keep ECS projections only where needed for:
  - [x] rendering
  - [x] widgets
  - [x] debug or inspector output
- [x] Rename helpers or fields if needed to reduce ambiguity

## Phase 6: Persistence And Apply Cleanup

- [x] Remove popup-specific cleanup assumptions from persistence flows
- [x] Reuse document signatures and validation reports from the already unified sources only
- [x] Audit load or apply branches that still exist only for transitional compatibility
- [x] Keep legacy save-file migration only where it still serves real old payload support
- [x] Delete branches that became obsolete after the executable-runtime unification

## Phase 7: Tighten Public Surface And Delete Dead Code

- [x] Remove unused exports, wrappers, and helper functions
- [x] Prune prelude exports that no longer represent supported architecture
- [x] Delete outdated documentation references to removed systems
- [x] Rewrite or remove tests that only exist for deleted compatibility layers
- [x] Keep the surviving public API intentionally small

## Phase 8: Regression Coverage For The New Shape

- [x] Add targeted tests for:
  - [x] runtime reading execution truth directly from `ExecutableGraph`
  - [x] scene outputs without `GraphResolvedInputs`
  - [x] UI diagnostics without `GraphConnectivityIndex`
  - [x] editor startup and persistence without popup resources
  - [x] authored input change flowing through execution to UI or world state
- [x] Keep tests focused on architectural guarantees, not only smoke behavior

## Documentation Maintenance

- [x] Keep `docs/roadmap.ar.md` and `docs/roadmap.md` aligned in order and meaning
- [x] Update `changelog.md` as each cleanup phase lands
- [x] Audit supplementary documents and archive only what this roadmap fully supersedes
