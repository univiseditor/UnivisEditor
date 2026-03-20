# Graph Core Execution Roadmap

## Goal

Move `graph_core` from:

- an authoring and validation kernel

to:

- the authoring truth of the graph
- and the execution truth of the graph

The intended outcome is that `GraphDocument` remains the authored truth, while
`ExecutableGraph` becomes the execution truth.

## Decisions Locked Before Coding

- `ExecutableGraph` should not expose `S` as part of its public surface.
- Schema concerns stay inside:
  - `GraphNodeRegistry`
  - `PortDefinition`
  - `build` and `validation` paths
- Building from a document should be tolerant:
  - build as much as possible
  - do not collapse the whole graph on the first issue
- Build output should not be a simple `Result`.
- Execution APIs start as internal and experimental, not stable public APIs.

## Build Semantics

- validation happens before or during build
- build may produce a partial executable graph
- node diagnostics are the primary per-node truth for blocked/degraded build outcomes

## Phase 0: Lock The Execution Model

- [x] Create: `docs/graph-core-execution-model.md`
- [x] Define the boundary between:
  - [x] `GraphDocument`
  - [x] `ExecutableGraph`
  - [x] `ExecutableNode`
- [x] Define ownership of:
  - [x] `authored_inputs`
  - [x] `resolved_inputs`
  - [x] `outputs`
- [x] Define ownership of:
  - [x] `edges`
  - [x] `direct links`
  - [x] `execution state`
- [x] Make explicit that:
  - [x] `GraphDocument` is not the runtime graph
  - [x] `ExecutableGraph` is the derived execution structure

## Phase 1: Introduce The Executable Layer

- [x] Create: `crates/univis_graph_core/src/executable.rs`
- [x] Define:
  - [x] `ExecutableGraph<Value>`
  - [x] `ExecutableNode<Value>`
  - [x] `NodeExecutionState`
- [x] Add direct relationship storage:
  - [x] incoming links
  - [x] outgoing links
- [x] Add core node data:
  - [x] `node_id`
  - [x] `definition_id`
  - [x] `authored_inputs`
  - [x] `resolved_inputs`
  - [x] `outputs`
- [x] Add core execution state:
  - [x] `enabled`
  - [x] `dirty`
  - [x] `blocked`
  - [x] `ready`
  - [x] `last_result`
  - [x] `last_run_revision`

## Phase 2: Build From `GraphDocument`

- [x] Define an explicit build API such as:
  - [x] `ExecutableGraph::build(document, registry)`
- [x] Pass:
  - [x] `&GraphDocument`
  - [x] `&GraphNodeRegistry`
- [x] Define:
  - [x] `ExecutableGraphBuildReport<Value>`
  - [x] `ExecutableNodeDiagnostic`
  - [x] `ExecutableNodeBuildStatus`
  - [x] `ExecutableNodeBlockReason`

### Target Build Report Shape

- [x] `ExecutableGraphBuildReport` should contain:
  - [x] `graph`
  - [x] `validation_report`
  - [x] `node_diagnostics`
  - [x] `is_partial`
- [x] The report should not split state into separate:
  - [x] `issues`
  - [x] `blocked_node_ids`
- [x] Instead, each node should expose:
  - [x] an explicit status
  - [x] explicit block or degradation reasons

### Build Responsibilities

- [x] Convert:
  - [x] `GraphDocumentNode -> ExecutableNode`
  - [x] `GraphDocumentEdge -> direct links`
- [x] Link ports using definitions from `registry`
- [x] Initialize:
  - [x] `authored_inputs`
  - [x] initial `resolved_inputs` buffers
  - [x] initial `outputs` buffers
- [x] Record the following in the build report:
  - [x] missing definitions
  - [x] invalid ports
  - [x] cycles / blocked topology
  - [x] type incompatibility
  - [x] requirement mismatch

### Build Rule

- [x] Building is tolerant:
  - [x] it does not fail globally on the first issue
  - [x] it reports what was built
  - [x] and reports what was blocked and why

## Phase 3: Lock Data Separation

- [x] Inside `ExecutableNode`:
  - [x] keep `authored_inputs` distinct
  - [x] keep `resolved_inputs` distinct
  - [x] keep `outputs` distinct
- [x] Define an explicit flow:
  - [x] how `authored_inputs` become `resolved_inputs`
- [x] Prevent:
  - [x] authored and resolved data from being mixed
- [x] Guarantee:
  - [x] outputs never mutate authored state
- [x] Define:
  - [x] ready condition
  - [x] blocked condition

## Phase 4: Internal Experimental Execution API

> This API is internal and unstable at this stage.

- [x] Add APIs inside `ExecutableGraph`:
  - [x] `enable_node(node_id)`
  - [x] `disable_node(node_id)`
  - [x] `mark_dirty(node_id)`
  - [x] `set_authored_input(node_id, index, value)`
  - [x] `resolve_inputs(node_id)`
  - [x] `run_node(node_id)`
  - [x] `run_ready_nodes()`
  - [x] `run_from(node_id)`
- [x] Add read helpers:
  - [x] `get_outputs(node_id)`
  - [x] `get_upstream(node_id)`
  - [x] `get_downstream(node_id)`

## Phase 5: Dirty Propagation

- [ ] Define the dirty propagation model
- [ ] When authored input changes or external mutation happens:
  - [ ] mark the node dirty
- [ ] Propagate dirty state to:
  - [ ] downstream nodes
- [ ] Execute:
  - [ ] ready nodes only
- [ ] After execution:
  - [ ] compare previous and new outputs
- [ ] If outputs did not change:
  - [ ] stop propagation
- [ ] If outputs changed:
  - [ ] continue propagation

## Phase 6: Move Execution Logic Downward

- [ ] Move:
  - [ ] adjacency logic
  - [ ] propagation
  - [ ] ready / blocked logic
  - [ ] scheduling
  - [ ] execution traversal
- [ ] Keep outside core:
  - [ ] Bevy ECS
  - [ ] world mutation
  - [ ] rendering
  - [ ] UI
- [ ] Make higher runtime layers:
  - [ ] adapters over core

## Phase 7: Integrate Validation With Execution

- [ ] Make `validation_report` part of build readiness
- [ ] Link:
  - [ ] blocked node diagnostics ← validation causes
  - [ ] topology ← execution-order seed
- [ ] Let the core answer:
  - [ ] can this graph execute?
  - [ ] which nodes are blocked?
  - [ ] why are they blocked?

## Phase 8: Performance Model

- [ ] Create: `docs/graph-core-performance-model.md`
- [ ] Define:
  - [ ] what is stored permanently in `ExecutableGraph`
  - [ ] what is rebuilt only when structure changes
  - [ ] what marks a node dirty
  - [ ] when outputs are considered changed
  - [ ] the cost of core operations
- [ ] Decide:
  - [ ] when to rebuild
  - [ ] when to rerun execution

## Phase 9: Execution Tests

- [ ] Test:
  - [ ] build from document
  - [ ] direct linking
  - [ ] node diagnostics
  - [ ] dirty propagation
  - [ ] disabled nodes
  - [ ] blocked nodes
  - [ ] partial execution
  - [ ] unchanged-output short-circuit

## Documentation Maintenance

- [ ] Keep `docs/roadmap.ar.md` and `docs/roadmap.md` aligned in order and meaning
- [ ] Keep the locked decisions at the top of the roadmap updated as implementation evolves
