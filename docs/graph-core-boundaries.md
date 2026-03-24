# Graph Core Boundaries And API Status

Status: Phase 0 baseline note, updated through the Phase 7 docs-and-examples refresh.

This document defines:

- crate ownership across the graph stack
- where graph rules are allowed to live
- which `univis_graph_core` public APIs are intended for direct use
- which public APIs are advanced or still incomplete in practice

It exists to reduce drift before deeper refactors begin and to keep the intended public surface visible as the roadmap lands.

## Ownership Map

| Crate | Owns | Must Not Own |
|---|---|---|
| `univis_graph_core` | graph identities, topology helpers, schema contracts, processing contracts, pure registries, pure graph documents, validation rules and reports, executable graph model | Bevy ECS state, editor visuals, file-format policy, scene-specific presentation rules |
| `univis_node_graph` | Bevy-facing values and schema adapters, styled/editor-facing port metadata, Bevy node hooks, live ECS graph components, entity-to-document projection | a second source of truth for graph legality, duplicated core graph rules, long-term ownership of pure graph contracts |
| `univis_editor_runtime` | executable rebuild triggers, runtime processing loop, scene output extraction, world sync | authoring-law decisions, connection legality rules, save/load policy |
| `univis_editor_ui` | interaction, wire preview, menus, node spawning, canvas presentation, diagnostics display | a second validator, persistence rules, execution semantics |
| `univis_editor_persistence` | file format, migrations, save/load/apply orchestration, bridging between live state and documents | alternate connection legality rules, alternate document semantics |

## Boundary Rules

- If a rule decides whether a graph, node, edge, or port relationship is legal, it belongs in `univis_graph_core`.
- If a rule decides how legal graph state is shown or edited, it belongs in `univis_editor_ui`.
- If a rule decides how legal graph state is serialized or migrated, `univis_editor_persistence` consumes core semantics instead of redefining them.
- If a rule decides how graph state is compiled or propagated at runtime, `univis_editor_runtime` may derive cached state from core but should not redefine authoring law.
- UI wire preview and persistence apply/load should evaluate candidate edges through the shared `univis_graph_core::validation` connection helpers instead of carrying independent rule sets.
- `univis_node_graph` is an adapter layer. It may enrich core concepts with Bevy/editor concerns, but it should avoid mirroring pure core behavior when composition is enough.
- `univis_node_graph::NodeRegistry` may own adapter-facing menu/search/category indexes, but it should mirror pure definitions into `GraphNodeRegistry` instead of re-owning core graph semantics.

## `graph_core` Public API Classification

## Stable And Intended

These are the main APIs downstream crates are expected to depend on directly.

| Area | Symbols | Intended Use |
|---|---|---|
| Identity | `NodeId`, `NodeCategory`, `ConnectionPolicy` | stable graph identity and menu grouping contracts |
| Ports and schema | `PortSchema`, `PortDefinition`, `GraphSchema` | pure compatibility, requirements, and default-value contracts |
| Processing | `GraphNodeDefinition` / `NodeDefinition`, `ProcessContext`, `ProcessResult`, `ProcessValueAccess` | pure node-processing integration |
| Registry | `GraphNodeRegistry`, `ArcGraphNodeDefinition` | pure definition registration and lookup |
| Document model | `GraphDocument`, `GraphDocumentNode`, `GraphDocumentEdge`, `GraphDocumentPrefab`, `GraphDocumentSubgraph`, `GraphDocumentViewState`, `GraphDocumentCameraState`, `GraphDocumentSelectionBoundarySummary`, `GraphDocumentOperationError`, `GRAPH_DOCUMENT_VERSION` | saved authoring model and document mutation helpers; prefer `spawn_node` / `connect` / `insert_node` for normal authoring, while raw `GraphDocumentEdge` construction is mainly for persistence, migrations, diagnostics fixtures, and whole-document tooling |
| Connection rules | `GraphConnectionCandidate`, `GraphConnectionValidationOptions`, `GraphStructuralConnectionValidationContext`, `GraphSchemaConnectionValidationContext`, `validate_structural_connection_candidate`, `validate_schema_connection_candidate` | shared source of truth for edge legality across validation, UI wire acceptance, and persistence apply/load |
| Validation | `GraphValidationReport`, `GraphValidationIssue`, `GraphValidationIssueKind`, `validate_graph_document` | main validation entry point and report types |
| Topology helpers | `would_create_cycle`, `connected_input_mask` | lightweight graph-rule helpers reused by higher layers |

## Advanced Public

These are valid public APIs, but they are more specialized and should not be the default entry points for most upper-layer code.

| Area | Symbols | Notes |
|---|---|---|
| Validation internals | `validate_graph_document_structure` | useful for tooling or layered validation, but not the main app path |
| Topology analysis | `GraphTopologyAnalysis`, `analyze_graph_topology` | intended for specialized topology inspection and tooling |
| Executable graph surface | `ExecutableGraph`, `ExecutableGraphBuildReport`, `ExecutableNode`, `ExecutableNodeRunOutcome`, `ExecutableNodeRunStatus`, `ExecutableNodeBuildStatus`, `ExecutableInputResolutionState`, `ExecutableNodeBlockReason`, `ExecutableNodeDiagnostic` | intended for runtime and deeper diagnostics, but not needed by every graph consumer; `ExecutableGraphBuildReport` is meant to be consumed through its summary accessors rather than direct field ownership |

## Public But Internal-In-Practice

These are currently public, but most workspace crates should avoid depending on them directly unless there is a strong reason.

| Symbols | Why They Are Internal-In-Practice |
|---|---|
| `ExecutableNode` | most high-level code should query through `ExecutableGraph` instead of owning node internals |

## Internalized In Phase 4

These no longer belong to the workspace-wide public contract.

| Symbols | Phase 4 Decision |
|---|---|
| `ExecutableDirectLinks` | narrowed to crate-private because it is only low-level execution adjacency bookkeeping |
| `ExecutablePortRef` | narrowed to crate-private because it is only used by execution adjacency internals |
| `NodeExecutionState` | narrowed to crate-private because downstream crates only need higher-level readiness / dirty helpers |

## Prelude Guidance

- `univis_graph_core::prelude` now aims to cover ordinary integrations only.
- Specialized executable, topology, and layered-validation APIs stay available from their defining modules instead of being pulled into every downstream import set.
- Downstream crates should prefer adding targeted module imports when they truly need advanced surfaces.

## Experimental Or Explicitly Unsupported In Practice

These are exposed in a way that currently overstates end-to-end support.

| Area | Current Status |
|---|---|
| `ConnectionPolicy::Multiple` | still modeled in core for future design work, but current workspace flows treat it as explicitly unsupported rather than partially supported |
| `PortDefinition::allow_multiple_connections` | future-facing declaration only; using it leads to unsupported validation and interaction results today |
| Multi-source executable input behavior | adjacency can still store multiple incoming links internally, but runtime semantics are intentionally not exposed as a supported product behavior |

## Downstream Usage Guidance

- Prefer the stable/intended areas for new integrations.
- Use advanced APIs when building tooling, deeper diagnostics, or runtime-specific orchestration.
- Avoid introducing new dependencies on public-but-internal-in-practice APIs unless Phase 4 explicitly keeps them public.
- Treat `ConnectionPolicy::Multiple` as explicitly unsupported in current workspace flows until a future roadmap phase implements full fan-in semantics.

## Phase 0 Outputs

This note is part of the Phase 0 baseline together with:

- [`roadmap.md`](./roadmap.md)
- [`roadmap.ar.md`](./roadmap.ar.md)
- [`graph-core-analysis.ar.md`](./graph-core-analysis.ar.md)
- [`connection-architecture.md`](./connection-architecture.md)
