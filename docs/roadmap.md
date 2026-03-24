# Code Quality Roadmap For `graph_core`

This roadmap is based on [`graph-core-analysis.ar.md`](./graph-core-analysis.ar.md) and focuses on raising code quality, architectural consistency, and long-term maintainability across the graph stack.

## North Star

Raise the current overall quality from roughly `7/10` to `9.5/10+` by:

- [ ] removing major architectural duplication
- [ ] making `graph_core` the single source of truth for graph rules
- [ ] resolving partially supported features instead of leaving them half-exposed
- [x] tightening the public API to match intended usage
- [ ] reducing drift between core, UI, persistence, and runtime

## Priority Outcomes

- [x] One connection-rule implementation shared by validation, UI, and persistence.
- [x] A clear end-to-end decision on `ConnectionPolicy::Multiple`.
- [x] A thinner `univis_node_graph` adapter with less mirroring of core types.
- [x] A smaller, more intentional `graph_core` public surface.
- [x] Fewer silent failures in document operations.

## Phase 0: Baseline And Ownership

- [x] Document crate boundaries and ownership.
- [x] Classify public API into stable and intended, public but internal in practice, or experimental and incomplete.
- [x] Align docs around the quality plan.

## Phase 1: Single Source Of Truth For Connection Rules

- [x] Extract shared connection-evaluation helpers into `graph_core`.
- [x] Reuse the same logic from validation, wire preview / acceptance, and persistence apply / load.
- [x] Standardize rejection reasons so upper layers stop inventing their own rule sets.

## Phase 2: Resolve `ConnectionPolicy::Multiple`

Current state: designed, partially modeled, not truly supported end-to-end.

Decision gate:

- [ ] Fully implement it across document, validation, UI, persistence, and runtime.
- [x] Or temporarily reduce / hide the API until it is truly supported.

Recommended approach:

- [x] first make the public truth match the real implementation
- [ ] then add full support only if the product actually needs multi-source inputs

## Phase 3: Reduce Mirroring In `univis_node_graph`

- [x] Refactor `PortDefinition` toward composition instead of near-duplication.
- [x] Reduce overlap between core node-definition contracts and Bevy-facing contracts.
- [x] Simplify registry ownership so the adapter layer forwards less duplicated behavior.

## Phase 4: Tighten `graph_core` Public API

- [x] Audit everything exported through `prelude`.
- [x] Move low-level items to narrower visibility when possible.
- [x] Keep public only what downstream crates are expected to depend on directly.

## Phase 5: Improve Error Flow

- [x] Stop ignoring `GraphDocumentOperationError` in workflow-heavy paths.
- [x] Standardize mutation results and user-facing failure handling.
- [x] Remove silent document-operation failures where possible.

## Phase 6: Add Contract Tests Across Layers

- [x] Add parity tests for validation behavior, UI connection acceptance, persistence apply rules, and runtime execution assumptions.
- [x] Focus on cross-layer consistency, not only isolated unit behavior.

## Phase 7: Update Examples And Docs

- [x] Make examples reflect intended stable usage.
- [x] Remove or reduce examples that depend on APIs likely to be narrowed.
- [x] Update workspace docs to reflect the final architecture.

## Phase 8: Harden Live Document Rebuilds

- [x] Stop rebuilding live `GraphDocument` snapshots by pushing raw edges directly.
- [x] Reuse `insert_node` and `connect` invariants during snapshot rebuilds in `univis_node_graph`.
- [x] Surface rejected or stale live edges as structured build issues instead of letting them disappear silently.

## Suggested Execution Order

- [x] Baseline and ownership
- [x] Shared connection rules
- [x] `ConnectionPolicy::Multiple` decision
- [x] Error-flow improvements
- [x] Adapter-layer de-duplication
- [x] Public API tightening
- [x] Contract tests
- [x] Docs and examples refresh
- [x] Live document rebuild hardening

## Highest-Leverage Work

If only three things are done first, they should be:

- [x] unify connection rules
- [x] resolve `ConnectionPolicy::Multiple`
- [x] reduce `univis_node_graph` mirroring

Those three changes will produce the biggest jump in quality, consistency, and maintainability.
