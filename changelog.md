# Changelog

## 2026-03-10

- Upgraded the workspace Bevy dependency to `0.18.0`.
- Updated UI code to match Bevy 0.18 node styling changes by moving `border_radius` into `Node`.
- Updated the infinite grid render pipeline to use Bevy 0.18 pipeline layout descriptors.
- Adjusted persistence and game editor status UI to remain compatible with the new UI API.
- Ignored `assets/graphs/current_graph.json` so runtime graph state is no longer tracked by Git.
- Verified the project with `CARGO_BUILD_JOBS=1 cargo check`.
