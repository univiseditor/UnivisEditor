# Changelog

## 2026-03-20

- Moved the remaining runtime execution path onto `ExecutableGraph` by introducing a `GraphExecutableRuntimeState` adapter resource in `univis_editor_runtime`, rebuilding executable state from the authored graph snapshot, and projecting compatibility resources such as connectivity and resolved-input caches from the core execution graph instead of maintaining a second execution engine in Bevy systems.
- Added core-side synchronization hooks on `ExecutableGraph` for whole-node authored-input replacement, external output synchronization, downstream dirtiness propagation, and runtime `custom_data` handoff so higher layers can keep ECS-facing widget and visual mutations while delegating actual scheduling and propagation rules to `univis_graph_core`.
- Reworked runtime node processing and diagnostics so `univis_editor_runtime` now feeds authored changes, visual output mutations, and custom runtime payloads into the executable graph, runs `run_ready_nodes(...)` from core, then projects resolved inputs, outputs, blocked state, trace entries, and runtime issues back into ECS and existing editor-facing resources.
- Completed `Phase 6` of the graph-core execution roadmap in both Arabic and English by making the Bevy runtime an adapter over core execution rather than the owner of separate adjacency, propagation, readiness, scheduling, and traversal logic.
- Folded validation directly into `ExecutableGraph` state by storing the build-time `validation_report`, node-level diagnostics, execution order seeded from topology, and partial-build state inside the executable graph instead of leaving them only on the external build report.
- Added core-facing readiness and blocked-state helpers such as `is_build_ready`, `can_execute`, `blocked_node_ids`, and `blocked_node_diagnostics`, so higher layers can now ask execution viability questions directly of `graph_core` instead of reconstructing them from separate validation outputs.
- Switched runtime projection code to consume execution order and node diagnostics from the executable graph itself, keeping `validation` and `execution` aligned around one in-memory source of truth.
- Completed `Phase 7` of the graph-core execution roadmap by linking build diagnostics to execution readiness and making blocked-node answers come directly from the core executable model.
- Added a dedicated `graph-core-performance-model` note that locks the intended rebuild vs rerun policy, the permanent execution-state footprint of `ExecutableGraph`, dirty triggers, output-change semantics, and the rule that higher runtime layers should project from one long-lived executable graph instead of owning a second execution cache.
- Rounded out executable-graph coverage with focused core tests for build-from-document direct links and execution order, disabled-node skipping, partial execution when some nodes are omitted from the build, and the already-landed dirty-propagation and blocked-node cases, then marked `Phase 8` and `Phase 9` complete in both roadmap documents.

## 2026-03-19

- Moved graph-document validation deeper into `univis_graph_core` by introducing a reusable `validation` module with shared issue/report types, structural document checks, schema-aware type compatibility, requirement validation, and topology-aware reporting in one core-level pipeline.
- Extended the core `GraphNodeDefinition` contract with an `output_requirement_token` hook so requirement matching no longer depends on Bevy-side adapter logic and can stay reusable across graph adapters.
- Reduced `univis_node_graph::graph_validation` to a thin adapter layer over the core validator, added an explicit `validate_graph_document_report` entry point, and kept the older issue-list helper for compatibility while the workspace migrates.
- Started adopting the shared validation report in higher layers by switching persistence paths to consume report counts directly, logging blocked-topology information during load, and teaching the floating diagnostics panel to read `blocked` and `ordered` topology state from a shared live validation report instead of recomputing validation locally.
- Added a centralized `LiveGraphValidationState` resource that refreshes after live-document snapshot sync, giving editor surfaces a single in-memory validation source of truth for current graph diagnostics.
- Stopped the live-document snapshot and live validation path from doing unconditional no-op work every `PostUpdate` by gating graph-document rebuilds on actual node, authored-input, selection, connection, and camera changes, then skipping live validation refreshes unless the live document or registry changed.
- Tightened the prefab-instance spawn workflow so it updates `AuthoredNodeInputs` alongside the live `GraphNode` input buffer, keeping the stricter live-document sync gating compatible with editor-authored state and save/load expectations.
- Split the persistence write-preparation path so save and autosave can reuse a precomputed validation issue count when the built graph document still matches the current live document, avoiding an extra validation pass during ordinary writes while keeping a safe fallback when signatures differ.
- Upgraded persistence history and apply/load plumbing from raw documents to richer graph snapshots that carry validation metadata, letting `undo`, `redo`, load, and graph-apply flows preserve validation state alongside the document instead of recomputing only a fresh issue count each time.
- Extended `LiveGraphValidationState` with a document signature plus explicit seeding helpers so persistence can inject already-known validation reports during apply/load/undo/redo paths, then aligned history snapshot capture to run after live validation refreshes so stored snapshots and live diagnostics stay in sync.
- Simplified persistence snapshot state by removing duplicated cached validation issue counts from history and pending-apply state, making `GraphValidationReport` the single source of truth there and deriving counts only when user-facing warnings need them.
- Moved `graph_document_signature` into `univis_node_graph::document`, then updated live validation and persistence history/dirty tracking to consume the same shared document-signature helper instead of leaving signature logic split between graph state and save-file formatting code.
- Clarified persistence naming around document identity by renaming saved/history signature fields to `document_signature` / `last_saved_document_signature` / `last_document_signature`, and removed the temporary `graph_document_signature` re-export from `univis_editor_persistence::format` so save-file helpers no longer expose document-layer utilities.
- Added a dedicated `graph core execution` roadmap in Arabic and English plus a `graph-core-execution-model` note, locking the authored-vs-executable boundary, build semantics, and the staged execution plan before implementation started.
- Introduced a first `ExecutableGraph` layer inside `univis_graph_core`, including executable nodes, direct links, node execution state, build reports, node diagnostics, and a tolerant `ExecutableGraph::build(document, registry)` path that reuses core validation output instead of inventing a separate build-time truth.
- Locked execution-time data separation in the new core layer by keeping `authored_inputs`, `resolved_inputs`, and `outputs` distinct, adding explicit input-resolution states and reseeding rules, and defining initial `ready` / `blocked` semantics inside `ExecutableNode`.
- Added the first internal experimental execution API to `ExecutableGraph`, covering node enable/disable and dirtiness controls, authored-input mutation, input resolution, direct node execution, ready-node execution, graph traversal from a starting node, output reads, adjacency queries, and per-node custom runtime data.

## 2026-03-18

- Added an `Assets` surface to `CanvasIsland` so saved prefabs and subgraphs can be browsed directly in the editor, with per-asset `Spawn`, `Insert`, and `Update from Selection` actions instead of relying only on latest-asset shortcuts.
- Extended the prefab/subgraph capture workflow and command layer to support in-place updates of existing assets by id, preserving asset identity while refreshing the stored graph selection and adding smoke coverage for the update path.
- Refreshed graph node presentation with richer headers and a more informative spawn menu, including `univis_ui` icon usage, category-aware fallback icons, item descriptions, and cleaner menu rows for node discovery.
- Started phasing out popup-driven parameter editing in favor of Blender-style inline node controls by embedding editable input widgets directly inside nodes, aligning simple value controls with their ports, and introducing a collapsible in-node `Transform` section for denser scene-node settings.
- Tightened the built-in input node bodies (`Number`, `Integer`, `Boolean`, and `Text`) so their value widgets sit closer to the output side and better match the new inline-editing direction.
- Reworked right-click behavior in the graph so the first context menu now behaves more like Blender, opening with clipboard and selection actions (`Paste`, `Copy`, `Duplicate`, `Delete`, `Frame Selected`) instead of jumping straight into the node browser, while still preserving right-click disconnect on connected input ports.
- Added an editor-side graph clipboard flow for selected nodes, including explicit `CopySelectedNodes` / `PasteNodes` commands, cursor-position paste placement, and `Ctrl+C` / `Ctrl+V` shortcuts, with node spawning moved behind an explicit secondary `Add Node...` step from the context menu.
- Continued polishing the Blender-style context menu by turning `Add Node...` into a category-first browsing flow, then revealing category-specific node flyouts on hover so node discovery stays structured instead of dumping the full registry at once.
- Refined the new graph menu presentation with stronger per-category and per-node color accents, clearer hover and active states, and dedicated scroll support for the category and node flyouts in addition to the main context menu.

## 2026-03-17

- Replaced direct `GraphDocument` JSON writes with a versioned save-file envelope in `univis_editor_persistence`, introducing explicit DTOs for authored inputs, edges, scene values, and view state while keeping load-time migration support for older raw graph payloads.
- Split legacy save migration into an explicit `univis_editor_persistence::migrations` pipeline so save-file parsing, v0 upgrades, and raw-document upgrades no longer stay embedded inside the serialization module.
- Added `docs/save-file-format.md` to define the active `univis.graph` save contract, persisted authored-state boundaries, and the expected legacy migration path into save-file `v1`.
- Started writing `meta.created_at` and `meta.updated_at` into save-file `v1`, preserving the original creation timestamp when overwriting an existing envelope-based graph file while refreshing the update timestamp on each save.
- Added workflow smoke coverage for repeated saves to the same graph path so persistence now checks that `created_at` stays stable while `updated_at` refreshes across overwrite saves.
- Marked migrated legacy loads as needing resave until the user writes the graph back in `save-file v1`, and added warning-level load feedback plus smoke coverage for that upgraded-load path.

## 2026-03-16

- Fixed live wire creation in the editor by making drag-time target detection fall back to world-space port proximity and by preserving the last accepted target through the mouse-release frame, which restores new valid connections, rejected-target feedback, and the expected data flow for links created after loading a graph.
- Restored backward-compatible access to live graph ECS types through `univis_node_graph::node_definition`, documented the `Default` expectation for custom schema values, extended staged app verification to build every shipped editor example, and rewired the Bevy-side `NodeRegistry` to delegate ordering/search/category bookkeeping through the pure `GraphNodeRegistry` so adapter and core registration logic no longer drift separately.
- Moved the live ECS component set (`GraphNode`, ports, authored inputs, selection/drag markers, and related adapter state) out of `node_definition.rs` into a dedicated `live_graph` module, making the Bevy adapter boundary clearer and keeping pure node-definition concerns separate from live world state.
- Added `docs/custom-graph-schema.md` plus a pure `univis_graph_core/examples/minimal_schema.rs` example to show how to define custom values, tags, schema rules, registry usage, documents, and processing without depending on Bevy.
- Extended the staged core verification path to build the new pure-core example so the reusable-kernel story is checked alongside the library crates.
- Added a pure generic `GraphNodeRegistry` plus `ArcGraphNodeDefinition` to `univis_graph_core`, giving the reusable kernel its own registration/lookup layer without pulling in Bevy-facing visual hooks or adapter values.
- Clarified the node-definition boundary by renaming the engine-independent contract to `GraphNodeDefinition` in `univis_graph_core` and the adapter contract to `BevyNodeDefinition` in `univis_node_graph`, while keeping compatibility aliases so the workspace can migrate incrementally.
- Explicitly kept `NodeValue` and `ValueType` owned by `univis_node_graph`, documenting that adapter/runtime value semantics stay outside the reusable core instead of drifting back into the generic kernel.
- Moved the pure generic `GraphDocument` model and its structural operations into `univis_graph_core`, then reduced `univis_node_graph::document` to typed aliases plus live Bevy projection/state so the reusable kernel now owns document semantics while the adapter owns entity mapping and snapshot sync.
- Switched `univis_node_graph` validation to use the adapter-owned `NodeGraphSchema` compatibility hook instead of calling `ValueType` rules directly, so port compatibility now flows through schema contracts that other graph adapters can replace.
- Added an explicit `GraphSchema` contract to `univis_graph_core` for validation-level compatibility and requirement matching, then implemented it in the Bevy adapter so schema rules are now centralized instead of being split between port types and ad-hoc validation helpers.
- Removed the stale `univis_node_graph::document::operations` module after moving pure graph document behavior into `univis_graph_core`, leaving the adapter side focused on live state and snapshot projection only.

## 2026-03-15

- Started extracting a reusable engine-independent graph kernel by adding a new `univis_graph_core` crate for node identities, connection policies, and generic topology helpers, then re-exporting those pieces through `univis_node_graph` so the workspace can migrate incrementally without breaking editor/runtime callers.
- Moved the base processing contract into `univis_graph_core` by introducing a reusable generic `ProcessContext`, `ProcessResult`, `ProcessValueAccess`, and engine-independent `NodeDefinition<Value, Port>` trait, while keeping `univis_node_graph` as the Bevy adapter that layers visual hooks and ECS-specific behavior on top.
- Moved `ValueType` plus a color-free `PortRequirement` / `PortDefinition<Value>` schema into `univis_graph_core`, then taught `univis_node_graph` to adapt styled ports back into the pure core shape so the reusable kernel now owns graph semantics while Bevy-facing color and runtime-value behavior stays in the adapter layer.
- Corrected the kernel boundary by making `univis_graph_core::PortDefinition` generic over a `PortSchema` trait and moving `ValueType` ownership back into `univis_node_graph`, so the core now defines only reusable port structure while the Bevy adapter owns compatibility rules, styled requirements, and concrete value semantics.
- Preserved node selection while context menus or other overlay surfaces are open by teaching click-selection to ignore non-canvas overlays, so selection now clears only from direct empty-canvas interaction instead of ordinary menu usage.
- Added connection-focused wire coverage for valid links, occupied-input rejection, and cycle rejection, and tied the new `wire_feedback` target into `verify_workspace.sh` so wiring regressions are checked alongside other workflow smoke tests.
- Polished editor wiring UX by introducing shared drag-time acceptance/rejection evaluation, live valid-target highlighting, rejection feedback on hovered inputs, preview-wire color/thickness feedback, and automatic inspector focus on the input that just accepted or rejected a drag.
- Reduced runtime/editor overhead by marking only nodes that actually need `sync_visual` polling, reusing connectivity index storage during graph rebuilds, and switching several runtime input/output updates to `clone_from`-style buffer reuse instead of replacing vectors wholesale each pass.
- Added connection-aware subgraph boundary summaries in `GraphDocument`, surfaced them in editor diagnostics, and updated subgraph capture status to report internal wires plus omitted incoming/outgoing boundary links so selection capture semantics are explicit before and after saving a subgraph.
- Fixed an output-port hover panic in the connection diagnostics path by replacing an eager `then_some(connections.targets[0])` access with a lazy checked lookup, so unconnected outputs no longer trigger an index-out-of-bounds crash.
- Fixed the new port-preview tooltip system to use explicitly disjoint Bevy queries, removing a `B0001` runtime panic caused by overlapping mutable `Node` and `Text` access inside the tooltip sync pass.
- Added hover/pinned port value previews as a dedicated tooltip, including resolved value text, type/status context, and live color swatches for `Color` ports so port-level debugging no longer depends entirely on the inspector panel.
- Added structured runtime tracing with persisted settings, recent per-node execution entries, and diagnostics-panel integration so graph execution order and reprocessing reasons can be inspected without always enabling noisy logging.
- Added an explicit `ConnectionPolicy` layer on port definitions, surfaced it in the connection inspector, and made wire creation reject unsupported multi-source declarations with a clear message instead of relying on an implicit single-input rule.
- Added wire and port diagnostics in the editor UI, including focused-port state, per-port health styling, diagnostic wire coloring/thickness, and a new floating `Connection Inspector` panel with persisted visibility settings.
- Added a dedicated architecture note at `docs/connection-architecture.md` describing the first-class `GraphConnection` model, authored-vs-resolved input split, runtime adjacency indexing, and the propagation fixes for widget-driven source nodes.
- Restored live value propagation after the `GraphConnection` refactor by separating authored node inputs from runtime-resolved inputs, initializing authored inputs at spawn/load time, and teaching persistence plus popup editing to read/write the authored buffer instead of serializing transient resolved values.
- Restored propagation from custom-body source nodes such as widget-driven inputs by polling `sync_visual` nodes every frame, avoiding no-op output writes, and teaching the runtime to treat externally updated outputs as dirty so downstream nodes reprocess when source widgets change.
- Rebuilt live graph wiring around first-class `GraphConnection` entities instead of a central `Connecting(Vec<...>)` resource, keeping save/load on `GraphDocument.edges` while moving editor state, wire rendering, and live document capture onto independent link entities.
- Added a compiled runtime connectivity layer with per-node input/output adjacency maps plus cached resolved inputs and output signatures, so node processing now propagates dirty state through dependency order instead of scanning every edge for every node on each pass.
- Expanded document-level coverage with a dedicated `document_workflows` target for prefab upserts, subgraph capture/instancing, and live-document entity-retention behavior, keeping structural graph tests focused on document operations instead of node-family details.
- Added editor/workflow smoke coverage for `box select`, `frame selected`, duplicate snapshots, and prefab/subgraph capture-and-reinsert flows through new `editor_smoke` and `workflow_assets_smoke` targets.
- Extended the staged verification script and GitHub Actions matrix with a dedicated `workflows` stage plus opt-in `fmt`, `clippy-core`, and `clippy-editor` passes, so maintenance checks stay sequential locally while CI covers formatting and lint drift.
- Added `CONTRIBUTING.md` and refreshed the verification docs so the crate boundaries, node-extension path, and staged verification workflow are easier for contributors to follow.
- Re-centered the product definition around `Graph Core`, `Scene Authoring`, and `Editor UX`, moved editor command messages out of `univis_node_graph` into a dedicated `univis_editor_commands` crate, and refreshed the README so the documented scope matches the active architecture more closely.

## 2026-03-14

- Hardened graph-editor mutation flows by sanitizing stale drag, wire, popup, selection, and live-document state after delete/load/undo/redo paths instead of letting invalid entity references linger.
- Added `workflow_smoke` persistence coverage for `spawn -> connect -> save/load` and `spawn -> connect -> undo/redo`, and wired it into the staged verification script.
- Reduced idle editor work by refreshing wire visuals only when links, drag state, settings, or port transforms actually change, and by skipping infinite-grid sync passes when settings are unchanged.
- Expanded editor workflow persistence with saved session settings, tracked `Recent Files`, and automatic persistence of those entries into `.univis/editor_settings.json`.
- Extended the `CanvasIsland` file/settings surfaces with `Reset Settings`, `Recent Files`, and `Recover Latest Autosave`, backed by a shared latest-backup lookup in graph persistence.
- Continued polishing the graph-native editor presentation by persisting grid and wire display settings and ignoring local IDE metadata in Git.
- Formalized `SceneDocument` as the shared Scene IR between runtime world materialization and the app-side scene preview panel, so scene sinks now flow through one explicit document model instead of ad-hoc `EntityValue` consumption.
- Expanded the core graph-native scene toolset with typed `Visibility` and `Anchor` components plus new `visibility`, `anchor`, `scale`, `rotation`, `z-order`, and `group` nodes so common scene-authoring work no longer depends on ad-hoc transform composition alone.
- Started a practical graph-asset workflow by preserving document `prefabs` and `subgraphs` through live sync and persistence, adding a `scene/prefab_instance` node, and wiring island actions for capturing prefabs/subgraphs plus instancing saved subgraphs back into the canvas.
- Upgraded the floating diagnostics panel from a coarse health summary to issue-focused editor feedback, including selected-node findings, blocked-path reporting, unused-branch reachability hints, and clearer validation/runtime messages tied back to node labels.
- Improved daily graph editing workflow with additive selection, empty-canvas box selection, `Duplicate Selected` snapshot duplication, and `Frame Selected` camera framing, and exposed the new duplicate/frame actions from the island `Edit` surface.
- Added lightweight workflow nodes for canvas organization: `logic/reroute` for cleaner wire routing and `logic/note` for inline graph comments without affecting runtime outputs.
- Added a staged GitHub Actions workflow that runs `scripts/verify_workspace.sh` in a `core`/`builtin`/`persistence`/`app` matrix, keeping CI aligned with the same sequential verification path used locally.
- Started the codebase-organization pass by splitting `crates/univis_editor_ui/src/interaction.rs` into focused modules for `selection`, `drag`, `camera`, `workflow_shortcuts`, `box_selection`, and state/document synchronization while keeping the existing editor behavior intact.
- Continued the codebase-organization pass by splitting `crates/univis_editor_persistence/src/graph_persistence.rs` into focused `state`, `history`, `apply`, and `io` modules without changing the public persistence surface used by the app, tests, and editor workflow.
- Continued the runtime cleanup by splitting `crates/univis_editor_runtime/src/lib.rs` into dedicated `processing`, `diagnostics`, `scene_outputs`, and `world_sync` modules, keeping the plugin surface stable while separating node processing from scene-output/world-sync concerns.
- Started the architecture-boundary pass by moving prefab-instance runtime data into `univis_scene`, which lets `univis_editor_runtime` stop depending on `univis_editor_nodes_builtin` for scene-node-specific `custom_data` plumbing.
- Continued slimming `univis_editor_app` by splitting `crates/univis_editor_app/src/graph_assets.rs` into focused workflow modules for duplication, asset capture, instancing, and status updates while preserving the same graph-asset commands.
- Continued the architecture-boundary pass by moving overlay focus state out of `univis_node_graph` into `univis_editor_ui`, removing prefab cache helpers from `univis_scene`, introducing runtime system sets, and extracting graph/prefab workflow logic into a dedicated `univis_editor_workflows` crate so `runtime` stays execution-focused and `app` remains a thin orchestrator.
- Started the consistency pass by standardizing recently touched systems onto `*_system` names, splitting `crates/univis_node_graph/src/document.rs` into `types/state/snapshots/operations` modules, and adding brief intent comments in wire/workflow/document hotspots so the newer architecture is easier to follow.
- Continued the consistency pass across the remaining legacy UI/app files by renaming `menu`, `node_popup`, `CanvasIsland`, and editor-settings persistence systems onto the same `*_system` convention, so the newer module structure now reads consistently across runtime, persistence, UI, workflows, and app orchestration.

## 2026-03-10

- Upgraded the workspace Bevy dependency to `0.18.0`.
- Updated UI code to match Bevy 0.18 node styling changes by moving `border_radius` into `Node`.
- Updated the infinite grid render pipeline to use Bevy 0.18 pipeline layout descriptors.
- Adjusted persistence and game editor status UI to remain compatible with the new UI API.
- Ignored `assets/graphs/` so runtime graph state is no longer tracked by Git.
- Added `GraphDocument` as the shared graph save/load schema in `univis_editor_core`.
- Added typed `EntityValue` support, entity ports, and merge semantics for graph-native scene composition.
- Converted built-in scene nodes from tagged JSON payloads to typed `EntityValue` composition nodes.
- Moved graph persistence to `GraphDocument` and introduced explicit activation resources instead of coupling persistence to editor mode.
- Moved graph editing UI gating behind `GraphEditingUiActivation` so `univis_editor_ui` no longer depends on editor mode state.
- Moved `EditorMode` out of `univis_editor_core` into `univis_editor_game_editor`.
- Added a transitional `EditorScene -> EntityValue` projection in the game editor to start bridging toward a graph-native scene model.
- Detached `univis_editor_app` from the legacy `univis_editor_game_editor` path so the shipped app stays canvas-only.
- Removed `univis_editor_game_editor` from the workspace members to keep the active build focused on the graph-native editor path.
- Deleted the legacy `univis_editor_game_editor` crate from the repository.
- Clarified the active UI direction as canvas-only with floating contextual surfaces instead of fixed panels.
- Added a floating `CanvasIsland` shell in `univis_editor_app` as the new compact top-level UI surface for file/edit actions.
- Added an explicit `DeleteSelectedNodesRequest` in `univis_editor_ui` so floating app surfaces can trigger edit actions without keyboard-only coupling.
- Fixed a Bevy B0001 startup panic in `CanvasIsland` by making its mutable UI queries explicitly disjoint.
- Moved persistence status messaging into the `CanvasIsland` flow and removed the separate persistence status overlay.
- Added a shared `GraphCommandRequest` layer so keyboard shortcuts, the island, and the context menu route through the same command path for save/open/delete/spawn actions.
- Added a shared `GraphOverlayState` so the `CanvasIsland` menu and the context menu no longer behave as unrelated floating surfaces.
- Reworked `CanvasIsland` to use an explicit surface state (`Compact`, `FileMenu`, `EditMenu`) instead of a bare open/closed flag.
- Added a `Search` surface inside `CanvasIsland`, with `Ctrl+K`, keyboard query capture, filtered node results, and direct spawn commands at the graph camera center.
- Migrated `node_popup` from `bevy_ui` widgets to `univis_ui` world-space UI so popup rendering and interaction now stay on the same UI stack as the graph surface.
- Hooked `node_popup` into `GraphOverlayState`, so popup focus is now coordinated with the island and context menu instead of floating independently.
- Fixed `node_popup` property rows so they are rebuilt only when the popup target or node values change, which allows `univis_ui` text/content children to initialize and remain visible.
- Added graph validation helpers in `univis_editor_core` for topological analysis, cycle detection, and `GraphDocument` structural checks.
- Reused the shared cycle detection in wire creation so new links that would introduce a cycle are rejected before they enter the graph.
- Added `GraphRuntimeDiagnostics` and switched runtime ordering to the shared topology analysis so blocked nodes from cycles are tracked explicitly instead of being skipped silently.
- Integrated `GraphDocument` validation into save/load persistence flows so invalid documents now surface as warnings during save, load, and autosave instead of passing unnoticed.
- Expanded `GraphDocument` from a passive schema into an operation-bearing model with node spawn/insert/delete, edge connect/disconnect, selection, and camera state helpers.
- Added `LiveGraphDocumentState` and a `PostUpdate` sync path so the current editor world now maintains a stable, continuously updated `GraphDocument` projection instead of leaving document state only to persistence.
- Removed the temporary built-in node families `assembly`, `materials`, `output`, and `scene_components` so the active built-in registry is back to `input`, `math`, and `logic` only.
- Removed the separate world preview surface from the shipped app so the editor is canvas-only again.
- Kept compatibility for older graph files by relying on the existing placeholder-node load path when removed built-in node definitions are encountered.
- Added a first graph-native `scene` family with `Transform`, `Sprite`, `Camera 2D`, `Merge Entity`, and `Add Child` nodes built directly on `EntityValue`.
- Added optional `Entity<Component>` input requirements at the node-definition layer so scene nodes can declare typed entity dependencies without splitting `Entity` into separate runtime value kinds.
- Restored the node popup to `bevy_ui` and replaced its internal string editing with a lightweight popup-local text field implementation instead of relying on `univis_ui` widgets.
- Added visual differentiation for required `Entity<Component>` ports by tinting both the port marker and its label from the required component kind.
- Expanded the graph-native `scene` family with `Name` and `Text` nodes so the rebuilt scene layer can name entities and emit typed 2D text directly from `EntityValue`.
- Extended `EntityValue` with a typed `Text2D` component for graph-native scene composition.
- Tightened `Entity<Component>` semantics so constrained inputs now require a pure single-component entity, with matching validation in both wire creation and document validation.
- Removed the temporary `scene/world_output` node and its runtime consumption path so scene rebuilding stays focused on graph-native composition semantics instead of a premature output stage.
- Moved node-specific visual synchronization into `NodeDefinition::sync_visual`, removing the separate visual-hook registry path and letting each node own its visual-to-data sync behavior directly.
- Limited visual sync passes to node definitions that explicitly need them, so ordinary nodes no longer pay for custom-body lifecycle work they do not use.
- Extracted shared graph-document snapshot builders into `univis_editor_core`, so live editor syncing and persistence now build documents through the same stable code path.
- Consolidated repeated scene-node composition helpers for base entities, transform fallback ports, transform application, and final entity-result handling to keep scene semantics consistent as the graph-native family grows.
- Moved selection, delete-selected, and input-disconnect decisions onto `LiveGraphDocumentState` helpers so core document operations now drive more of the editor behavior instead of duplicating those rules in UI systems.
- Extracted the node-graph engine into a new standalone `univis_node_graph` crate, moved all graph infrastructure there, and reduced `univis_editor_core` to a thin compatibility facade.
- Moved graph-native scene values into a separate `univis_scene` crate and switched scene nodes to flow through `CustomTag`/`TaggedData` contracts plus generic port requirements, so `univis_node_graph` no longer owns `EntityValue` or scene component semantics directly.
- Removed the now-unused `univis_editor_core` compatibility crate from the workspace after all active code paths were switched to `univis_node_graph` directly.
- Promoted scene entities from tagged payloads to a first-class `ValueType::Entity` / `NodeValue::Entity`, while making `univis_scene` a pure value crate and keeping scene-specific input constraints in the scene nodes themselves.
- Replaced the central `EntityComponentKind` enum with open scene component keys plus shared helpers for labels, colors, and pure-input requirement tokens, so scene composition no longer bottlenecks on a single enum for extensibility.
- Extracted reusable scene-graph helper functions into `crates/univis_editor_nodes_builtin/src/scene_support.rs`, keeping `scene` nodes thinner and avoiding premature expansion of `NodeDefinition` for scene-specific workflow helpers.
- Added a `custom_scene_node` example that defines external `Mesh2D`-style scene nodes on top of `EntityValue::Custom` from outside the built-in scene family, and exposed `scene_support` publicly for that extension path.
- Restored `scene/scene` as a world display sink that spawns the incoming `EntityValue` into the editor world instead of reflecting it only inside the node body.
- Kept `Camera2D` components inert in the temporary world-display sink because `univis_ui` currently assumes a single active `Camera2d` for graph interaction and would otherwise stop responding.
- Verified the project with `CARGO_BUILD_JOBS=1 cargo check`.
