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

## Workspace Layout

The workspace is split into focused crates:

- `univis_node_graph`
  - graph types, node definitions, registry, values, commands, validation, document helpers
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
- `univis_editor_app`
  - facade crate that assembles the editor plugins

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
  - compare, branch, and, or, not
- scene nodes
  - transform, sprite, camera2d, text, name, merge entity, add child, scene

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
bash scripts/verify_workspace.sh app
```

The script forces sequential execution with `CARGO_BUILD_JOBS=1` unless you override it.

## Main Test Targets

Focused test targets currently maintained in the workspace:

- `cargo test -p univis_node_graph --test core_api`
- `cargo test -p univis_node_graph --test document_ops`
- `cargo test -p univis_node_graph --test graph_validation`
- `cargo test -p univis_editor_runtime --lib`
- `cargo test -p univis_editor_nodes_builtin --test input_nodes`
- `cargo test -p univis_editor_nodes_builtin --test math_nodes`
- `cargo test -p univis_editor_nodes_builtin --test logic_nodes`
- `cargo test -p univis_editor_nodes_builtin --test scene_nodes`
- `cargo test -p univis_editor_persistence --test persistence_defaults`
- `cargo test -p univis_editor_persistence --test persistence_format`

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

What is still evolving:

- editor polish
- deeper runtime smoke coverage
- CI automation
- further modularization of large editor and persistence modules

## Guiding Principle

If a feature helps author scene content, it is a good candidate for the graph.

If a feature exists to operate the editor itself, it should probably stay a normal system.
