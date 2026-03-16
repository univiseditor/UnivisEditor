# Roadmap

Arabic version: [docs/roadmap.ar.md](/home/abdellah/Desktop/Univis/UnivisEditor/docs/roadmap.ar.md).

## Purpose

This roadmap keeps UnivisEditor focused on becoming a strong graph-native scene editor for Bevy.

The active product remains constrained to three axes:

- `Graph Core`
- `Scene Authoring`
- `Editor UX`

Work that does not clearly strengthen one of those axes should usually be deferred, simplified, or moved out of the active layer that owns it.

## Current Position

The project already has the right architectural direction:

- a reusable engine-independent graph core
- a Bevy adapter layer for live graph state and editor/runtime integration
- a scene-facing value model
- a canvas-first editor with persistence, workflows, and diagnostics

The next step is not broadening scope. The next step is turning that foundation into a stable, polished, and extensible authoring workflow.

## Roadmap Principles

- stabilize file compatibility before adding broad new surface area
- polish one strong scene-authoring workflow before expanding node families aggressively
- keep engine-independent graph logic in `univis_graph_core`
- keep Bevy-facing live state and adapter value semantics in `univis_node_graph`
- treat diagnostics, validation, and testing as product features, not cleanup work

## Phase 1: Stabilize The Foundation

### Goals

- make the current architecture predictable and safe to build on
- reduce migration risk while graph/document boundaries continue to evolve
- lock down the editor/runtime/persistence loop for everyday use

### Main Work

- add explicit graph document versioning and migration entry points
- define compatibility expectations for saved graphs, subgraphs, and prefabs
- expand regression coverage around live wiring, persistence round-trips, undo/redo, and runtime propagation
- improve runtime diagnostics for blocked nodes, invalid links, and schema mismatch paths
- pin `univis_ui` to a verified commit hash after the current `dev` branch validation period

### Done When

- older saved graphs load through a documented migration path
- live wire behavior, save/load, and graph execution have stable regression coverage
- dependency updates are reproducible across machines
- `bash scripts/verify_workspace.sh all` is the normal confidence path before merging

## Phase 2: Polish One Hero Authoring Workflow

### Goals

- make the editor feel complete for a focused 2D scene-authoring flow
- improve confidence that the product is useful before adding much more breadth

### Main Work

- polish the built-in `scene` family around sprite, text, transform, hierarchy, grouping, and scene output
- make subgraph and prefab capture/edit/apply flows predictable and easy to inspect
- improve popup editing, selection feedback, connection feedback, and graph diagnostics UX
- surface clearer authoring warnings before save and during graph execution
- tighten the example set so the intended workflow is obvious from first run

### Done When

- a small 2D scene can be authored, saved, reopened, edited, and replayed without confusing edge cases
- prefab/subgraph workflows are reliable enough to use as normal composition tools
- the main shipped example demonstrates the intended product story without extra explanation

## Phase 3: Make Extension A First-Class Feature

### Goals

- let other developers extend the graph without reading large parts of the workspace internals
- turn the new graph-core split into an actual adoption advantage

### Main Work

- publish a minimal custom-node template and a documented registration path
- keep `univis_graph_core` examples Bevy-free and focused on reusable contracts
- clarify which extension points belong in pure core, Bevy adapter, runtime, and editor crates
- add smoke coverage for external-style node registration and custom schema usage
- document stable extension patterns for custom values, ports, and visual node behavior

### Done When

- a contributor can add a custom node or schema by following docs and examples
- extension docs match the real crate boundaries
- custom integration paths are covered by at least one maintained verification target

## Phase 4: Scale Runtime And Diagnostics

### Goals

- keep larger graphs understandable and performant
- make failures visible before users have to inspect internals

### Main Work

- improve dirty propagation and avoid unnecessary full-graph work where practical
- add profiling-style diagnostics for graph execution order, blocked nodes, and expensive nodes
- expose clearer summaries for subgraph boundaries, omitted links, and scene-output issues
- keep runtime/editor sync costs visible as graphs grow
- expand validation to catch more structural and semantic problems before execution

### Done When

- large graphs remain debuggable without stepping through internal ECS state
- diagnostics can explain why a node did not run or why a scene output is incomplete
- runtime work is targeted enough that larger authoring graphs stay practical

## Phase 5: Release Readiness

### Goals

- prepare the project to be adopted, tested, and iterated on with less friction

### Main Work

- define a release process for crate versions, graph format changes, and dependency updates
- ship a curated example set that covers the main product story and extension story
- tighten `README`, contribution docs, and architecture docs around the current product
- decide which APIs and file-format behaviors are experimental versus expected to remain stable
- add a lightweight public milestone checklist for pre-release validation

### Done When

- the project can cut a tagged release with a known verification path
- docs tell contributors where to extend the system and where not to
- users can understand current capabilities, limitations, and upgrade expectations

## Out Of Scope For Now

The following items should stay deferred unless they clearly support the phases above:

- turning editor infrastructure such as save/open or overlay state into nodes
- widening the product into a general-purpose node editor outside scene authoring
- embedding large viewport-style experiences inside node bodies
- adding many new node families before the current scene workflow is stable

## Suggested Execution Order

1. finish Phase 1 before broadening scope
2. treat Phase 2 as the main product milestone
3. use Phase 3 and Phase 4 to turn the architecture into leverage
4. use Phase 5 to prepare for external adoption and releases

## Near-Term Recommended Priority

If only a few items can be tackled immediately, the highest-value sequence is:

1. graph document versioning and migrations
2. pinning `univis_ui` to a verified revision
3. hero 2D scene-authoring workflow polish
4. extension template and documentation
5. runtime profiling and diagnostics
