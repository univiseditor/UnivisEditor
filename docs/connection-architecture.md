# Connection Architecture Report

## Summary

UnivisEditor now uses a Blender-like live connection model for graph wiring.

Instead of keeping all live links in a central `Connecting(Vec<...>)` resource, each wire is represented as its own `GraphConnection` entity. The runtime compiles those links into adjacency data and propagates values through dependency order.

This gives the project a clearer split between:

- visual wiring in the editor
- authored graph data stored in documents
- runtime-resolved values used during node execution

## Why This Change Was Needed

The previous live model relied on a shared connection list as the main source of truth. That approach worked for small graphs, but it made several things harder over time:

- every system had to scan or reinterpret the same central connection state
- large graphs paid more global work than necessary
- live editor wiring and runtime dependency flow were too tightly coupled
- it was harder to separate authored values from values resolved through connections

The new model moves the project closer to how mature node editors behave: links are first-class graph objects, while execution uses a compiled dependency view.

## What Changed

### 1. Live Links Are First-Class Entities

Each live wire is now stored as a `GraphConnection` entity with:

- source node and output index
- target node and input index
- source port entity
- target port entity

This means the visual wire you see in the editor corresponds to an actual live graph object rather than a derived view over a central vector.

### 2. Persistence Still Saves Document Edges

The saved graph format did not need a noisy migration.

`GraphDocument` still stores connections as `edges`, but load/apply paths now rebuild live `GraphConnection` entities in memory. Save/capture paths perform the inverse and rebuild document edges from live connection entities.

That keeps file compatibility stable while improving the runtime/editor model internally.

### 3. Runtime Uses Compiled Connectivity

The runtime now builds a `GraphConnectivityIndex` with:

- incoming sources by node input
- outgoing targets by node output
- ordered nodes
- blocked nodes

This index is rebuilt only when graph structure changes. Node processing then uses the compiled adjacency instead of scanning every connection for every node.

### 4. Authored Inputs And Resolved Inputs Are Separate

The editor now keeps authored input values in `AuthoredNodeInputs`.

This solves an important architectural problem:

- authored inputs are the values the user typed or saved
- resolved inputs are the values that actually arrive at runtime after connection propagation

Without this separation, persistence and popup editing could accidentally serialize transient runtime values instead of user-authored values.

## Benefits

### Better Scaling

As the graph grows, the runtime no longer needs broad connection scans just to answer simple dependency questions. The compiled adjacency index keeps evaluation targeted.

### Cleaner Boundaries

The graph document remains the saved authoring model.
The editor owns live wire entities and interaction.
The runtime owns dependency compilation and propagation.

This keeps crate responsibilities aligned with the intended architecture.

### More Reliable Data Flow

Because the runtime tracks direct incoming and outgoing relationships, data propagation is easier to reason about and easier to test.

This also makes later features more realistic:

- stronger diagnostics
- more precise dirty propagation
- larger graphs
- better prefab/subgraph workflows
- future optimization work

## Important Bug Fixes That Landed With This Work

While shipping the new connection model, two real propagation bugs were uncovered and fixed.

### Authored Input Synchronization

Some paths were still reading or writing `GraphNode.values.inputs` directly after the authored/resolved split. That caused wires to exist visually while runtime data did not consistently reflect user-authored inputs.

The fix was to:

- initialize authored inputs at spawn time
- restore authored inputs on load/apply
- make popup editing update authored inputs explicitly
- keep runtime-resolved values separate

### Custom-Body Source Nodes

Widget-driven source nodes such as `Number`, `Boolean`, and `Text` update outputs through `sync_visual`.

Those nodes were not always causing downstream propagation after the connection refactor, because:

- visual sync was not being polled reliably enough
- runtime dirty tracking did not treat externally updated outputs as meaningful changes

The fix was to:

- poll visual-sync nodes every frame
- avoid writing no-op output changes
- treat external output changes as dirty so downstream nodes reprocess

## Tradeoffs

This change increases internal structure:

- there are more ECS entities overall because each wire is now its own entity
- the runtime owns extra cached state
- persistence has to bridge between live connection entities and document edges

That added complexity is intentional. It buys much better long-term maintainability than keeping all live wiring in a single mutable list.

## Current Value To The Project

This feature is important because it changes the graph from a visually connected editor into a more robust execution model.

It improves:

- architectural clarity
- runtime scalability
- correctness of propagated values
- testability of graph behavior
- readiness for larger authoring workflows

In short: the editor is now much closer to a real graph-native scene tool, not just a canvas that draws wires.

## Verified Paths

The following paths were explicitly validated while landing this work:

- runtime value propagation through `GraphConnection`
- save and load round-trips with preserved values and edges
- undo and redo over spawned nodes and connections
- prefab and subgraph workflow smoke checks
- custom-body source propagation through polled visual sync

## Related Files

- `crates/univis_node_graph/src/pin.rs`
- `crates/univis_node_graph/src/node_registry.rs`
- `crates/univis_editor_runtime/src/connectivity.rs`
- `crates/univis_editor_runtime/src/diagnostics.rs`
- `crates/univis_editor_persistence/src/graph_persistence/io.rs`
- `crates/univis_editor_persistence/src/graph_persistence/apply.rs`
- `crates/univis_editor_ui/src/wire.rs`
- `crates/univis_editor_ui/src/node_popup.rs`
