# Graph Core Validation Roadmap

## Summary

The next `univis_graph_core` milestone should introduce a reusable validation layer that owns graph-document correctness rules instead of leaving them inside the Bevy adapter.

Today, the project already has:

- generic document and topology primitives in `univis_graph_core`
- generic schema and registry contracts in `univis_graph_core`
- an adapter-specific validator in `crates/univis_node_graph/src/graph_validation.rs`

The goal of this roadmap is to move that validation logic downward into the reusable kernel, then let the adapter consume it instead of maintaining a parallel validation path.

## Why This Work Matters

This is the highest-leverage `graph_core` improvement available right now because it would give the project one shared source of truth for:

- editor diagnostics
- runtime preflight checks
- persistence sanity checks
- future graph compilation and execution planning

It also improves the architectural boundary: `graph_core` should own graph correctness, while adapters should own rendering, ECS projection, and value semantics.

## Current Starting Point

The core already has most of the inputs needed for validation:

- `GraphDocument<Value, Prefab>` in `crates/univis_graph_core/src/document.rs`
- `GraphNodeRegistry<Value, Port>` in `crates/univis_graph_core/src/registry.rs`
- `GraphSchema` and `PortSchema` in `crates/univis_graph_core/src/schema.rs` and `crates/univis_graph_core/src/ports.rs`
- topology helpers in `crates/univis_graph_core/src/topology.rs`

The missing piece is a reusable report model and a validator that combines those building blocks.

## Scope

This roadmap covers:

- creating `univis_graph_core::validation`
- moving adapter-independent validation rules from `univis_node_graph`
- defining reusable issue/report types
- making the adapter call into the core validator
- adding pure-core tests for the new behavior

This roadmap does not cover:

- runtime execution compilation
- editor UI presentation changes
- persistence format changes
- node processing refactors unrelated to validation

## Target API Shape

The first version should expose a small, stable API such as:

```rust
pub enum GraphValidationIssueKind { ... }

pub struct GraphValidationIssue {
    pub kind: GraphValidationIssueKind,
    pub edge_index: Option<usize>,
    pub node_ids: Vec<u64>,
    pub message: String,
}

pub struct GraphValidationReport {
    pub issues: Vec<GraphValidationIssue>,
    pub topology: GraphTopologyAnalysis<u64>,
}

pub fn validate_graph_document<S, Value, Prefab>(
    document: &GraphDocument<Value, Prefab>,
    registry: &GraphNodeRegistry<Value, PortDefinition<S>>,
) -> GraphValidationReport
where
    S: GraphSchema;
```

The exact type signatures may shift, but the important outcome is:

- one reusable report object
- one reusable validation entry point
- no adapter-specific correctness rules left behind unless they are genuinely adapter-only

## Phase 1: Establish Core Validation Types

Create `crates/univis_graph_core/src/validation.rs` and define:

- `GraphValidationIssueKind`
- `GraphValidationIssue`
- `GraphValidationReport`

The report should support both:

- issue-by-issue inspection
- quick status checks such as `is_valid()` or `has_errors()`

Exit criteria:

- the new module is exported from `crates/univis_graph_core/src/lib.rs`
- the report type is generic enough for both tests and adapter integration

## Phase 2: Move Structural Validation Into The Core

Port the adapter-independent rules from `crates/univis_node_graph/src/graph_validation.rs` into `graph_core`.

This includes:

- duplicate node ids
- missing node definitions
- missing source and target nodes
- self-connections
- invalid input and output indexes
- multiple connections to single-input ports
- cycle or blocked-path reporting

This phase should depend only on:

- `GraphDocument`
- `GraphNodeRegistry`
- `GraphTopologyAnalysis`
- core port connection-policy data

Exit criteria:

- pure-core tests cover all structural issue kinds
- `univis_node_graph` no longer owns duplicate copies of these rules

## Phase 3: Move Schema-Aware Validation Into The Core

Once structural validation is in place, move schema-driven checks into `graph_core`.

This includes:

- port type compatibility through `PortSchema` / `S::ports_compatible`
- requirement matching through `GraphSchema::requirement_satisfied`
- requirement labels through `GraphSchema::requirement_label`

Because `graph_core` already uses `PortDefinition<S>`, the validator should work directly with:

- output `type_tag`
- input `type_tag`
- input `requirement`
- input `connection_policy`

One design question must be resolved in this phase:

- whether requirement-token derivation belongs in a new core-facing node-definition hook
- or whether the adapter should pass a lightweight callback/context into the validator

Recommended direction:

- introduce a small core-facing hook for output requirement tokens so validation stays reusable and does not depend on adapter logic

Exit criteria:

- type and requirement errors are reported by `graph_core`
- `univis_node_graph::graph_validation` becomes a thin wrapper or is removed entirely

## Phase 4: Adopt The Core Validator Upstream

After the validator is stable, switch upstream crates to rely on it.

Adoption targets:

- `univis_node_graph`
- runtime diagnostics setup
- editor diagnostics summaries
- persistence preflight or load warnings where appropriate

The main goal is not to change UI yet, but to ensure all higher layers consume the same core report.

Exit criteria:

- the adapter no longer carries a forked validation implementation
- diagnostics and runtime checks are sourced from one shared report

## Phase 5: Add Confidence Tests

Add focused tests at the pure-core level for:

- duplicate nodes
- missing definitions
- invalid edges
- incompatible types
- unsatisfied requirements
- cycles and blocked nodes
- mixed valid and invalid edge sets

Also add one adapter-side regression test to confirm:

- `univis_node_graph` still produces the same validation outcomes after delegating to the core

Exit criteria:

- the core validator can be evolved without relying on UI-level smoke coverage alone

## Suggested Work Breakdown

The implementation sequence should be:

1. Add `validation.rs` types and exports.
2. Port structural validation logic.
3. Add pure-core tests for structural issues.
4. Introduce the minimal hook needed for requirement-token validation.
5. Port schema-aware checks.
6. Replace adapter-side validation with a wrapper or re-export.
7. Update runtime and editor consumers to use the shared report.

## Risks And Decisions

The biggest design risk is requirement validation.

Structural checks are already clearly core-owned, but requirement-token derivation currently depends on node-definition behavior that may still be expressed through adapter assumptions.

Before implementation starts, we should explicitly decide:

- whether `GraphNodeDefinition` in `graph_core` gains a small requirement-token method
- or whether validation accepts an injected callback for output requirement evaluation

The roadmap recommends the first option because it keeps validation deterministic and portable.

## Done Criteria

This roadmap is complete when:

- `univis_graph_core` exports a reusable validation module
- `univis_node_graph` no longer owns its own full validator
- editor/runtime/persistence consumers can all rely on the same validation report
- pure-core tests cover both structural and schema-aware failures

At that point, the project will be ready for the next logical core step:

- a reusable graph compilation or execution-plan layer built on top of validated documents
