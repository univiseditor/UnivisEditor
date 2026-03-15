# Contributing

This workspace is organized around a clear authoring/runtime split:

- `crates/univis_node_graph`: graph document model, validation, registry, generic node values, and mutation tracking.
- `crates/univis_editor_commands`: editor-facing command messages for spawn/save/load/history/workflow actions.
- `crates/univis_scene`: Scene IR types plus helpers that materialize scene data into Bevy entities.
- `crates/univis_editor_runtime`: graph execution, diagnostics, scene-output collection, and world sync.
- `crates/univis_editor_ui`: graph camera, interaction systems, wires, menus, popup editing, and editor settings.
- `crates/univis_editor_persistence`: save/load, history, autosave, graph apply flows, and persistence status.
- `crates/univis_editor_nodes_builtin`: built-in input, math, logic, and scene node definitions.
- `crates/univis_editor_workflows`: duplicate, prefab, subgraph, and other editor-facing workflow systems.
- `crates/univis_editor_app`: application shell, panels, island layout, and plugin orchestration.

## Product Axes

The active product should stay inside three axes:

- `Graph Core`: graph types, node definitions, values, validation, and document operations.
- `Scene Authoring`: scene data, scene nodes, prefabs, subgraphs, and scene materialization.
- `Editor UX`: canvas interaction, persistence, history, diagnostics, settings, and editor workflow actions.

When a change does not clearly strengthen one of these axes, it should usually be moved, simplified, or deferred.

The main boundary to preserve is:

- authoring logic inside graph-facing data and node processing
- editor operation outside the graph in commands, UI, persistence, or workflow crates

## Adding A Node

1. Implement `NodeDefinition` in the most appropriate crate.
2. Register the node in the crate that owns that family, usually `univis_editor_nodes_builtin`.
3. Add crate-local tests near the node family instead of only relying on workspace-level coverage.
4. If the node emits or consumes scene data, prefer `univis_scene::EntityValue` / `SceneDocument` instead of editor-specific data.
5. If the change needs runtime handling, keep execution logic in `univis_editor_runtime` or `univis_editor_workflows`, not in `univis_editor_app`.
6. If you need a new editor action, prefer adding it to `univis_editor_commands` rather than pushing it into `univis_node_graph`.

## Verification

Run only the slice you changed when possible:

```bash
bash scripts/verify_workspace.sh core
bash scripts/verify_workspace.sh builtin
bash scripts/verify_workspace.sh persistence
bash scripts/verify_workspace.sh workflows
bash scripts/verify_workspace.sh app
```

For full local maintenance checks:

```bash
bash scripts/verify_workspace.sh all
bash scripts/verify_workspace.sh fmt
bash scripts/verify_workspace.sh clippy-core
bash scripts/verify_workspace.sh clippy-editor
```

The verification script runs sequentially with `CARGO_BUILD_JOBS=1` by default, which keeps the workflow usable on lower-memory machines.
