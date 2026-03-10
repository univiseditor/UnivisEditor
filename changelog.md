# Changelog

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
- Verified the project with `CARGO_BUILD_JOBS=1 cargo check`.
