# Graph Core Execution Model

Status: Active supporting note. Kept alongside the cleanup roadmap because it defines ownership and execution-boundary rules that the roadmap only summarizes.

## Purpose

This note defines the execution boundary for `univis_graph_core` and is updated
as the runtime cleanup plan progresses.

The project should explicitly own two kinds of truth:

- `GraphDocument` as authored truth
- `ExecutableGraph` as execution truth

This separation is intentional. The authored model should stay simple,
persistent, and editor-friendly. The execution model should be derived,
directly linked, and optimized for control, readiness, and propagation.

## Current Baseline

Today, the project already has the main ingredients needed for the execution
model:

- `GraphDocument<Value, Prefab>` in `univis_graph_core`
- `GraphNodeRegistry<Value, PortDefinition<S>>` in `univis_graph_core`
- validation and topology analysis in `univis_graph_core`
- a Bevy-side live model built around:
  - `GraphConnection` entities
  - `AuthoredNodeInputs`
  - `GraphExecutableRuntimeState`
  - `GraphNode.values`
  - `NodeInputSignature`
  - `NodeOutputSignature`
  - `GraphRuntimeDiagnostics`

The old runtime projection resources:

- `GraphConnectivityIndex`
- `GraphResolvedInputs`

have already been removed. The next cleanup step is to reduce the remaining
adapter and cache layers around `ExecutableGraph`.

## Layer Definitions

### `GraphDocument`

`GraphDocument` is the persisted authored model.

It owns:

- node ids
- definition ids
- authored input values
- graph edges
- prefabs and subgraphs
- editor-facing view data such as selection and camera state

It does not own:

- direct runtime adjacency
- resolved inputs
- output buffers
- execution state
- dirty propagation
- execution scheduling

`GraphDocumentNode.inputs` are authored inputs only. They are not runtime
resolved values.

### `ExecutableGraph`

`ExecutableGraph<Value>` is the derived execution model.

It is built from:

- `&GraphDocument<Value, Prefab>`
- `&GraphNodeRegistry<Value, PortDefinition<S>>`

It owns:

- executable nodes
- direct incoming and outgoing relationships
- build-time readiness state
- execution-order seeds derived from topology
- execution-time buffers and state

It does not own:

- editor layout or view state
- selection
- prefabs or subgraphs as persistence concepts
- Bevy entities or ECS-only identifiers

`ExecutableGraph` should not expose `S` on its public surface. Schema details
remain a build and validation concern through the registry.

### `ExecutableNode`

`ExecutableNode<Value>` is the execution-time representation of one node.

It owns:

- `node_id`
- `definition_id`
- `authored_inputs`
- `resolved_inputs`
- `outputs`
- `NodeExecutionState`

It does not own:

- editor position
- selection state
- port styling
- Bevy handles or ECS components

## Ownership Matrix

| Concern | Owner |
| --- | --- |
| node id / definition id | `GraphDocumentNode` and `ExecutableNode` |
| authored inputs | `GraphDocumentNode` as persisted truth, mirrored in `ExecutableNode` for execution |
| resolved inputs | `ExecutableNode` |
| outputs | `ExecutableNode` |
| document edges | `GraphDocument` |
| direct links / adjacency | `ExecutableGraph` |
| execution state | `ExecutableNode` via `NodeExecutionState` |
| whole-graph correctness | `GraphValidationReport` |
| per-node blocked/degraded build outcome | build report `node_diagnostics` |

## Transitional Structure Classification

The remaining structures should be read using the following categories:

| Structure | Classification | Notes |
| --- | --- | --- |
| `GraphDocument` | truth | persisted authored graph |
| `AuthoredNodeInputs` | truth | ECS-side authored mirror for live editing |
| `ExecutableGraph` | truth | sole execution truth |
| `GraphExecutableRuntimeState` | adapter | ECS bridge to executable truth plus entity mapping |
| `GraphNode.values` | cache | projected resolved inputs and outputs for display or integration |
| `NodeInputSignature` | cache | local change tracking for authored input projection |
| `NodeOutputSignature` | cache | local change tracking for output projection |
| `GraphRuntimeDiagnostics` | adapter | presentation-oriented summary derived from executable truth |
| `GraphSceneOutputs` | adapter | world-facing sink summary derived from executable truth |
| `LiveGraphValidationState` | adapter | live validation cache and minimal Bevy glue over core validation |
| popup UI path | removed legacy | deleted in cleanup phase 3 |

Any new structure introduced during cleanup should explicitly fit one of these
categories. If it does not, it is likely duplicating responsibility.

## Compatibility Resources Remaining

The following ECS-side data still exist mainly for integration compatibility:

- `GraphExecutableRuntimeState`
- `GraphNode.values`
- `NodeInputSignature`
- `NodeOutputSignature`
- `GraphRuntimeDiagnostics`
- `GraphSceneOutputs`
- `LiveGraphValidationState`

These are acceptable only so long as they do not compete with the truth owned
by `GraphDocument`, `AuthoredNodeInputs`, or `ExecutableGraph`.

Two rules should remain explicit:

- adapters may summarize or project execution truth
- caches may mirror execution truth

but neither may become an independent decision-making source.

In code, this now means:

- authored decisions should read `AuthoredNodeInputs`
- execution decisions should read `ExecutableGraph`
- ECS projection updates should prefer `GraphNode` projection helpers over
  treating `values` as authoritative state

For persistence specifically:

- save and autosave should reuse the live validation report when the saved
  document matches the live authored document
- persistence cleanup may clear UI resources when they exist, but should not
  require overlay or menu resources to be present in order to function

## Build Semantics

Build semantics are fixed as follows:

- validation happens before or during build
- build may produce a partial executable graph
- node diagnostics are the primary per-node truth for blocked or degraded build outcomes

The builder should be tolerant:

- it should build everything that can be built safely
- it should not collapse the whole graph on the first issue

The builder should therefore return a structured report rather than a bare
`Result`.

## Build Contract

The intended first build entry point is conceptually:

```rust
ExecutableGraph::build(document, registry) -> ExecutableGraphBuildReport<Value>
```

The build report should carry:

- `graph`
- `validation_report`
- `node_diagnostics`
- `is_partial`

It should not split build outcome into disconnected lists such as:

- `issues`
- `blocked_node_ids`

Instead, each node should have one primary diagnostic entry describing whether
it was:

- built cleanly
- built in a degraded way
- blocked
- omitted from the executable graph

and why.

## Node Diagnostics Model

`ExecutableNodeDiagnostic` is the primary per-node build result.

At minimum, it should make room for:

- a node id
- a build status
- one or more block or degradation reasons

Typical reasons include:

- missing definition
- invalid port reference
- type incompatibility
- requirement mismatch
- blocked topology or cycle involvement
- link omitted during tolerant build

This keeps the answer to "what was blocked and why?" in one place instead of
forcing higher layers to correlate multiple lists.

## Data Flow Model

The execution data flow is:

1. `authored_inputs` are seeded from document inputs.
2. Missing authored slots may be backfilled from port defaults where the build
   layer can materialize them into `Value`.
3. `resolved_inputs` are produced from an authored baseline plus graph-fed
   overrides.
4. `process(...)` reads `resolved_inputs`.
5. `process(...)` writes `outputs`.
6. Output changes may trigger dirty propagation to downstream nodes.

Two rules are strict:

- authored state is never overwritten by resolved graph flow
- outputs never write back into authored state

## Input Resolution Rule

The effective input model is:

- unconnected inputs keep their authored value
- connected inputs are overridden by upstream outputs when resolution succeeds
- if a connected input cannot be resolved, the node becomes blocked or degraded
  according to build or execution diagnostics

This means `resolved_inputs` are not a second authored buffer. They are the
current executable view of inputs.

## Execution State Model

`NodeExecutionState` is owned by `ExecutableNode`.

The first model should support at least:

- `enabled`
- `dirty`
- `blocked`
- `ready`
- `last_result`
- `last_run_revision`

Semantically:

- `enabled` answers whether the node is allowed to run
- `dirty` answers whether it needs recomputation
- `blocked` answers whether execution is currently prevented
- `ready` answers whether it can be scheduled now

Build diagnostics seed initial blocked state.
Execution logic may later refine readiness and dirtiness.

## Direct Links vs Document Edges

`GraphDocument.edges` remain the authored wiring description.

`ExecutableGraph` owns direct adjacency derived from those edges:

- incoming links by executable input
- outgoing links by executable output

Invalid links should not survive as opaque runtime edges. They should be
captured in diagnostics and either omitted or reflected as blocked state.

## Value Initialization Constraint

The execution layer needs a deterministic way to initialize:

- authored input buffers
- resolved input buffers
- output buffers

The first implementation should be explicit about this policy.

Two acceptable starting strategies are:

- require `Value: Clone + Default`
- or constrain the first build path to registries where port defaults can be
  materialized directly as `Value`

The important rule is that buffer initialization must be a defined build-time
policy, not an ad-hoc adapter detail.

## Relationship To The Current Bevy Runtime

Today, Bevy-side runtime code owns:

- live `GraphConnection` entities
- `GraphExecutableRuntimeState` as executable bridge state
- projected node input and output caches
- output signatures and change tracking

The target architecture is to move the pure parts of that model into
`ExecutableGraph`, while keeping Bevy responsible for:

- ECS entity projection
- world mutation
- rendering
- editor interaction
- system scheduling integration

In other words:

- `graph_core` should own execution semantics
- Bevy should own integration

## Cleanup Deletion Order

Cleanup should happen in this order:

1. remove projection resources that duplicate execution truth
2. thin runtime adapters until they only bridge ECS and `ExecutableGraph`
3. remove popup editing legacy and popup-only overlay state
4. simplify validation access so core validation is consumed directly
5. clarify all ECS caches and prevent them from acting as truth
6. tighten persistence and public API surface after the architectural shift

This order matters because deleting UI legacy before runtime truth converges can
hide architectural mistakes instead of removing them.

## Non-Goals Of This Model

This model does not make `graph_core` responsible for:

- Bevy ECS state
- world-space rendering
- editor drag behavior
- popup or menu interaction
- persistence file format changes

It only defines the boundary between authored graph truth and execution graph
truth.
