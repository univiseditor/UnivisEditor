# UnivisEditor

UnivisEditor is a graph-native scene editor for Bevy.

The project is built around one core idea: scenes should be authored as composable node networks. The graph is responsible for scene content and scene composition. The editor, persistence layer, validation, and runtime orchestration remain regular systems around that graph.

## What The Project Is

UnivisEditor is not trying to turn every internal editor behavior into a node.

Its current direction is:

- nodes describe scene data and scene composition
- the graph produces pure authoring data
- runtime systems materialize that data into Bevy entities
- editor systems handle UI, persistence, validation, commands, and workflow

In practice, that means nodes are used for things like values, transforms, sprites, text, cameras, entity merging, child composition, and final scene output.

## Current Scope

The repository currently provides:

- a canvas-first node editor built on Bevy
- a node graph core with registry, port definitions, validation, and document operations
- a pure scene data model centered on `EntityValue`
- built-in input, math, logic, and scene nodes
- runtime systems that evaluate graphs and sync scene output into the world
- graph save/load and autosave support
- crate-local tests and a sequential verification script for lower-spec machines

This is best described today as:

`A graph-native scene editor where scenes are authored as composable node networks and materialized into runtime entities.`

## Product Axes

The active product is intentionally constrained to three axes only:

- `Graph Core`
  - node definitions, registry, ports, values, graph documents, validation, and structural graph operations
- `Scene Authoring`
  - scene-facing value composition, built-in scene nodes, prefabs, subgraphs, scene outputs, and runtime scene materialization
- `Editor UX`
  - canvas interaction, menus, wires, grid, settings, persistence, history, diagnostics, and workflow shortcuts

Anything that does not clearly improve one of these axes is either moved out of the active layer that owns it, simplified, or deferred.

Current examples of deferred or intentionally excluded scope:

- turning editor internals such as save/open, autosave timers, or overlay focus into nodes
- embedding full render viewports directly inside nodes
- widening the product into a general-purpose node editor outside scene authoring

## Architectural Model

The project is organized around four layers:

1. Graph authoring
   - users build a graph from nodes and edges
2. Pure scene data
   - the graph resolves into scene-facing values such as `EntityValue`
3. Runtime materialization
   - systems turn graph output into Bevy ECS entities
4. Editor infrastructure
   - UI, persistence, validation, commands, overlay state, and editor workflow

This boundary is intentional. Scene authoring belongs in the graph. Editor infrastructure does not.

The rule used across the workspace is:

- authoring logic lives inside graph-facing data and node processing
- editor operation lives outside the graph as UX, persistence, and workflow systems

## Workspace Layout

The workspace is split into focused crates:

- `univis_node_graph`
  - graph types, node definitions, registry, values, validation, and document helpers
- `univis_editor_commands`
  - editor-facing command messages for spawn, save/load, history, duplicate, frame, and prefab/subgraph workflows
- `univis_scene`
  - pure scene data types such as `EntityValue` and component payloads
- `univis_editor_runtime`
  - graph execution and scene-to-world synchronization
- `univis_editor_ui`
  - editor canvas, interaction systems, node spawning, popup editing, wire UI
- `univis_editor_persistence`
  - save/load, autosave, serialization helpers, and graph format handling
- `univis_editor_nodes_builtin`
  - built-in input, math, logic, and scene nodes
- `univis_editor_workflows`
  - duplicate, prefab, subgraph, and other editor workflow systems that operate around the graph
- `univis_editor_app`
  - thin facade crate that assembles the editor plugins

You can read the same layout through the three product axes:

- `Graph Core`
  - `univis_node_graph`
- `Scene Authoring`
  - `univis_scene`, `univis_editor_runtime`, `univis_editor_nodes_builtin`
- `Editor UX`
  - `univis_editor_commands`, `univis_editor_ui`, `univis_editor_persistence`, `univis_editor_workflows`, `univis_editor_app`

## Scene Authoring Model

The graph is intended to cover:

- input values
- graph-side math and logic
- entity construction
- transform and component application
- composition by merge and parent-child relationships
- final scene sinks

The graph is not intended to cover:

- save/open commands
- editor camera behavior
- context-menu state
- autosave timers
- overlay focus management
- validation scheduling

Those remain normal systems because they are editor infrastructure, not authored scene content.

## Built-In Node Families

Current built-in nodes include:

- input nodes
  - number, integer, boolean, text, vec2, vec3, color
- math nodes
  - add, subtract, multiply, divide, clamp, lerp, min, max, abs, sin, cos
- logic nodes
  - compare, branch, and, or, not, reroute, note
- scene nodes
  - transform, name, visibility, anchor, prefab instance, sprite, text, camera2d, scale, rotation, z-order, merge entity, add child, group, scene

## Quick Start

Run the shipped example:

```bash
cargo run -p univis_editor_app --example simple_editor
```

Or add the facade plugin to your own Bevy app:

```rust
use bevy::prelude::*;
use univis_editor_app::{NodeGraphPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NodeGraphPlugin)
        .run();
}
```

The example at [examples/simple_editor.rs](/home/abdellah/Desktop/Univis/UnivisEditor/examples/simple_editor.rs) also adds the infinite grid plugin and a graph camera setup.

## Verification

The repository now uses crate-local tests and a sequential verification script so the workspace can be checked without running everything at once.

Run the full maintained sequence:

```bash
bash scripts/verify_workspace.sh all
```

Run only one area:

```bash
bash scripts/verify_workspace.sh core
bash scripts/verify_workspace.sh builtin
bash scripts/verify_workspace.sh persistence
bash scripts/verify_workspace.sh workflows
bash scripts/verify_workspace.sh app
bash scripts/verify_workspace.sh fmt
bash scripts/verify_workspace.sh clippy-core
bash scripts/verify_workspace.sh clippy-editor
```

The script forces sequential execution with `CARGO_BUILD_JOBS=1` unless you override it.

GitHub Actions runs the same script through `.github/workflows/verify-workspace.yml`, split into `core`, `builtin`, `persistence`, `workflows`, `app`, `fmt`, `clippy-core`, and `clippy-editor` jobs instead of maintaining a separate CI command list.

## Main Test Targets

Focused test targets currently maintained in the workspace:

- `cargo test -p univis_node_graph --test core_api`
- `cargo test -p univis_node_graph --test document_ops`
- `cargo test -p univis_node_graph --test document_workflows`
- `cargo test -p univis_node_graph --test graph_validation`
- `cargo test -p univis_editor_runtime --lib`
- `cargo test -p univis_editor_nodes_builtin --test input_nodes`
- `cargo test -p univis_editor_nodes_builtin --test math_nodes`
- `cargo test -p univis_editor_nodes_builtin --test logic_nodes`
- `cargo test -p univis_editor_nodes_builtin --test scene_nodes`
- `cargo test -p univis_editor_persistence --test persistence_defaults`
- `cargo test -p univis_editor_persistence --test persistence_format`
- `cargo test -p univis_editor_persistence --test workflow_smoke`
- `cargo test -p univis_editor_ui --test editor_smoke`
- `cargo test -p univis_editor_workflows --test workflow_assets_smoke`

For contribution guidelines and crate boundaries, see [CONTRIBUTING.md](/home/abdellah/Desktop/Univis/UnivisEditor/CONTRIBUTING.md).

## Extending The Graph

Custom nodes are added by implementing `NodeDefinition` and registering them in the node registry.

Relevant examples:

- [examples/custom_nodes.rs](/home/abdellah/Desktop/Univis/UnivisEditor/examples/custom_nodes.rs)
- [examples/custom_visual_node.rs](/home/abdellah/Desktop/Univis/UnivisEditor/examples/custom_visual_node.rs)
- [examples/custom_scene_node.rs](/home/abdellah/Desktop/Univis/UnivisEditor/examples/custom_scene_node.rs)

The core extension points live in:

- [node_definition.rs](/home/abdellah/Desktop/Univis/UnivisEditor/crates/univis_node_graph/src/node_definition.rs)
- [node_registry.rs](/home/abdellah/Desktop/Univis/UnivisEditor/crates/univis_node_graph/src/node_registry.rs)
- [document.rs](/home/abdellah/Desktop/Univis/UnivisEditor/crates/univis_node_graph/src/document.rs)

## Project Status

The project is already beyond a throwaway prototype, but it is still an evolving editor architecture rather than a finished product.

What is already clear:

- the scene-authoring direction is intentional
- the workspace boundaries are real and useful
- the graph model is the center of the product
- the product is being actively constrained to `Graph Core`, `Scene Authoring`, and `Editor UX`

What is still evolving:

- editor polish
- deeper runtime smoke coverage
- broader CI coverage beyond the staged Linux verification workflow
- further modularization of large editor and persistence modules
- continued pruning of features that do not clearly strengthen one of the three product axes

## Guiding Principle

If a feature helps author scene content, it is a good candidate for the graph.

If a feature exists to operate the editor itself, it should probably stay a normal system.
