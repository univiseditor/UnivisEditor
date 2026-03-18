use super::*;

#[derive(Resource, Debug, Clone, Default)]
pub(super) struct CanvasIslandState {
    pub surface: CanvasIslandSurface,
    pub search_query: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum CanvasIslandSurface {
    #[default]
    Compact,
    FileMenu,
    EditMenu,
    Assets,
    Search,
    Settings,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum CanvasIslandMenuAction {
    Save,
    SaveAs,
    Open,
    DeleteSelected,
    DuplicateSelected,
    FrameSelected,
    CapturePrefab,
    CaptureSubgraph,
    InsertLatestSubgraph,
    SpawnLatestPrefabNode,
}

#[derive(Component)]
pub(super) struct CanvasIslandRoot;

#[derive(Component)]
pub(super) struct CanvasIslandStatusDot;

#[derive(Component)]
pub(super) struct CanvasIslandFileNameText;

#[derive(Component)]
pub(super) struct CanvasIslandStatusText;

#[derive(Component)]
pub(super) struct CanvasIslandInteractive;

#[derive(Component)]
pub(super) struct CanvasIslandTriggerButton {
    pub surface: CanvasIslandSurface,
}

#[derive(Component)]
pub(super) struct CanvasIslandMenuPanel {
    pub surface: CanvasIslandSurface,
}

#[derive(Component)]
pub(super) struct CanvasIslandMenuActionButton {
    pub action: CanvasIslandMenuAction,
}

#[derive(Component)]
pub(super) struct CanvasIslandRecentFileButton {
    pub path: String,
}

#[derive(Component)]
pub(super) struct CanvasIslandRecoverAutosaveButton {
    pub path: String,
}

#[derive(Component)]
pub(super) struct CanvasIslandFileDynamicContent;

#[derive(Component)]
pub(super) struct CanvasIslandPrefabAssetButton {
    pub prefab_id: String,
}

#[derive(Component)]
pub(super) struct CanvasIslandUpdatePrefabAssetButton {
    pub prefab_id: String,
}

#[derive(Component)]
pub(super) struct CanvasIslandSubgraphAssetButton {
    pub subgraph_id: String,
}

#[derive(Component)]
pub(super) struct CanvasIslandUpdateSubgraphAssetButton {
    pub subgraph_id: String,
}

#[derive(Component)]
pub(super) struct CanvasIslandAssetsDynamicContent;

#[derive(Debug, Clone, Copy)]
pub(super) enum CanvasIslandSettingsAction {
    ToggleAutosave,
    TogglePrettyJson,
    ToggleHistory,
    DecreaseHistoryLimit,
    IncreaseHistoryLimit,
    ToggleDiagnosticsPanel,
    ToggleScenePreviewPanel,
    ToggleConnectionInspectorPanel,
    ToggleRuntimeTrace,
    ToggleGridDisplayMode,
    CycleGridPalette,
    DecreaseGridPointSize,
    IncreaseGridPointSize,
    CycleWireStyle,
    ToggleWireColorFromOutput,
    ResetToDefaults,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum CanvasIslandSettingsValueKind {
    Autosave,
    PrettyJson,
    HistoryEnabled,
    HistoryLimit,
    DiagnosticsPanel,
    ScenePreviewPanel,
    ConnectionInspectorPanel,
    RuntimeTrace,
    GridDisplayMode,
    GridColorPalette,
    GridPointSize,
    WireStyle,
    WireColorFromOutput,
}

#[derive(Component)]
pub(super) struct CanvasIslandSettingsActionButton {
    pub action: CanvasIslandSettingsAction,
}

#[derive(Component)]
pub(super) struct CanvasIslandSettingsValueText {
    pub kind: CanvasIslandSettingsValueKind,
}

#[derive(Component)]
pub(super) struct CanvasIslandSearchQueryText;

#[derive(Component)]
pub(super) struct CanvasIslandSearchResults;

#[derive(Component)]
pub(super) struct CanvasIslandSearchResultButton {
    pub definition_id: NodeId,
}

#[derive(Debug, Clone)]
pub(super) struct CanvasIslandSearchEntry {
    pub definition_id: NodeId,
    pub display_name: String,
    pub category: String,
}
