# Graph Core Execution Model

## Purpose

This note defines the execution boundary for `univis_graph_core` before
implementation starts.

The project should explicitly own two kinds of truth:

- `GraphDocument` as authored truth
- `ExecutableGraph` as execution truth

This separation is intentional. The authored model should stay simple,
persistent, and editor-friendly. The execution model should be derived,
directly linked, and optimized for control, readiness, and propagation.

## Current Baseline

Today, the project already has the main ingredients needed to define the target
execution model:

- `GraphDocument<Value, Prefab>` in `univis_graph_core`
- `GraphNodeRegistry<Value, PortDefinition<S>>` in `univis_graph_core`
- validation and topology analysis in `univis_graph_core`
- a Bevy-side live model built around:
  - `GraphConnection` entities
  - `GraphConnectivityIndex`
  - `GraphResolvedInputs`
  - `AuthoredNodeInputs`
  - `NodeInputSignature`
  - `NodeOutputSignature`

The next step is to move the pure execution semantics downward into
`graph_core`, while leaving Bevy responsible only for ECS projection, world
mutation, rendering, and editor interaction.

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
- compiled adjacency in `GraphConnectivityIndex`
- resolved input caches
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

## Non-Goals Of This Model

This model does not make `graph_core` responsible for:

- Bevy ECS state
- world-space rendering
- editor drag behavior
- popup or menu interaction
- persistence file format changes

It only defines the boundary between authored graph truth and execution graph
truth.
