use bevy::prelude::{Color, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::time::{SystemTime, UNIX_EPOCH};
use univis_graph_core::prelude::{GraphValidationReport, validate_graph_document};
use univis_node_graph::document::{
    GRAPH_DOCUMENT_VERSION, GraphDocument, GraphDocumentCameraState, GraphDocumentEdge,
    GraphDocumentNode, GraphDocumentPrefab, GraphDocumentSubgraph, GraphDocumentViewState,
    graph_document_signature,
};
use univis_node_graph::node_definition::NodeId;
use univis_node_graph::node_registry::NodeRegistry;
use univis_node_graph::value::NodeValue;
use univis_scene::{
    AnchorComponentValue, Camera2DComponentValue, EntityComponentValue, EntityValue,
    SpriteComponentValue, Text2DComponentValue, TransformComponentValue, VisibilityComponentValue,
};

pub(crate) const GRAPH_SAVE_FILE_FORMAT: &str = "univis.graph";
pub(crate) const GRAPH_SAVE_FILE_VERSION: u32 = 1;
const GRAPH_SAVE_COLOR_SPACE_SRGBA: &str = "srgba";

#[derive(Debug, Clone)]
pub struct ParsedGraphDocument {
    pub document: GraphDocument,
    pub migration_note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedGraphWrite {
    pub document: GraphDocument,
    pub payload: String,
    pub document_signature: String,
    pub validation_report: GraphValidationReport,
}

impl PreparedGraphWrite {
    pub fn validation_issue_count(&self) -> usize {
        self.validation_report.issue_count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSaveFileV1 {
    pub format: String,
    pub format_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generator: Option<GraphSaveGeneratorV1>,
    #[serde(default)]
    pub meta: GraphSaveMetaV1,
    pub document: GraphDocumentV1,
}

impl GraphSaveFileV1 {
    pub(crate) fn from_document(document: &GraphDocument) -> Self {
        Self::from_document_with_meta(document, GraphSaveMetaV1::fresh())
    }

    pub(crate) fn from_document_with_meta(document: &GraphDocument, meta: GraphSaveMetaV1) -> Self {
        Self {
            format: GRAPH_SAVE_FILE_FORMAT.to_string(),
            format_version: GRAPH_SAVE_FILE_VERSION,
            generator: Some(GraphSaveGeneratorV1::current()),
            meta,
            document: GraphDocumentV1::from_document(document),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSaveGeneratorV1 {
    pub app: String,
    pub app_version: String,
}

impl GraphSaveGeneratorV1 {
    fn current() -> Self {
        Self {
            app: "UnivisEditor".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphSaveMetaV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl GraphSaveMetaV1 {
    fn fresh() -> Self {
        let now = current_timestamp_string();
        Self {
            created_at: Some(now.clone()),
            updated_at: Some(now),
        }
    }

    pub(crate) fn refreshed(mut self) -> Self {
        let now = current_timestamp_string();
        if self.created_at.is_none() {
            self.created_at = Some(now.clone());
        }
        self.updated_at = Some(now);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphDocumentV1 {
    #[serde(default)]
    pub nodes: Vec<GraphDocumentNodeV1>,
    #[serde(default)]
    pub edges: Vec<GraphDocumentEdgeV1>,
    #[serde(default)]
    pub prefabs: Vec<GraphDocumentPrefabV1>,
    #[serde(default)]
    pub subgraphs: Vec<GraphDocumentSubgraphV1>,
    #[serde(default)]
    pub view: GraphDocumentViewStateV1,
}

impl GraphDocumentV1 {
    pub(crate) fn from_document(document: &GraphDocument) -> Self {
        Self {
            nodes: document
                .nodes
                .iter()
                .map(GraphDocumentNodeV1::from_node)
                .collect(),
            edges: document
                .edges
                .iter()
                .map(GraphDocumentEdgeV1::from_edge)
                .collect(),
            prefabs: document
                .prefabs
                .iter()
                .map(GraphDocumentPrefabV1::from_prefab)
                .collect(),
            subgraphs: document
                .subgraphs
                .iter()
                .map(GraphDocumentSubgraphV1::from_subgraph)
                .collect(),
            view: GraphDocumentViewStateV1::from_view(&document.view),
        }
    }

    fn into_document(self) -> GraphDocument {
        GraphDocument {
            version: GRAPH_DOCUMENT_VERSION,
            nodes: self
                .nodes
                .into_iter()
                .map(GraphDocumentNodeV1::into_node)
                .collect(),
            edges: self
                .edges
                .into_iter()
                .map(GraphDocumentEdgeV1::into_edge)
                .collect(),
            prefabs: self
                .prefabs
                .into_iter()
                .map(GraphDocumentPrefabV1::into_prefab)
                .collect(),
            subgraphs: self
                .subgraphs
                .into_iter()
                .map(GraphDocumentSubgraphV1::into_subgraph)
                .collect(),
            view: self.view.into_view(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocumentNodeV1 {
    pub id: u64,
    pub definition_id: String,
    pub position: [f32; 2],
    #[serde(default)]
    pub authored_inputs: Vec<GraphValueV1>,
    pub input_count: usize,
    pub output_count: usize,
}

impl GraphDocumentNodeV1 {
    fn from_node(node: &GraphDocumentNode) -> Self {
        Self {
            id: node.id,
            definition_id: node.definition_id.as_str().to_string(),
            position: node.position,
            authored_inputs: node.inputs.iter().map(GraphValueV1::from_value).collect(),
            input_count: node.input_count,
            output_count: node.output_count,
        }
    }

    fn into_node(self) -> GraphDocumentNode {
        GraphDocumentNode {
            id: self.id,
            definition_id: NodeId::new(self.definition_id),
            position: self.position,
            inputs: self
                .authored_inputs
                .into_iter()
                .map(GraphValueV1::into_value)
                .collect(),
            input_count: self.input_count,
            output_count: self.output_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocumentEdgeV1 {
    pub from: GraphEdgeEndpointV1,
    pub to: GraphEdgeEndpointV1,
}

impl GraphDocumentEdgeV1 {
    fn from_edge(edge: &GraphDocumentEdge) -> Self {
        Self {
            from: GraphEdgeEndpointV1 {
                node_id: edge.from_node_id,
                port_index: edge.from_index,
            },
            to: GraphEdgeEndpointV1 {
                node_id: edge.to_node_id,
                port_index: edge.to_index,
            },
        }
    }

    fn into_edge(self) -> GraphDocumentEdge {
        GraphDocumentEdge {
            from_node_id: self.from.node_id,
            from_index: self.from.port_index,
            to_node_id: self.to.node_id,
            to_index: self.to.port_index,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdgeEndpointV1 {
    pub node_id: u64,
    pub port_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocumentPrefabV1 {
    pub id: String,
    pub name: String,
    pub root: EntityValueV1,
}

impl GraphDocumentPrefabV1 {
    fn from_prefab(prefab: &GraphDocumentPrefab) -> Self {
        Self {
            id: prefab.id.clone(),
            name: prefab.name.clone(),
            root: EntityValueV1::from_entity_value(&prefab.root),
        }
    }

    fn into_prefab(self) -> GraphDocumentPrefab {
        GraphDocumentPrefab {
            id: self.id,
            name: self.name,
            root: self.root.into_entity_value(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocumentSubgraphV1 {
    pub id: String,
    pub name: String,
    pub document: Box<GraphDocumentV1>,
}

impl GraphDocumentSubgraphV1 {
    fn from_subgraph(subgraph: &GraphDocumentSubgraph) -> Self {
        Self {
            id: subgraph.id.clone(),
            name: subgraph.name.clone(),
            document: Box::new(GraphDocumentV1::from_document(&subgraph.document)),
        }
    }

    fn into_subgraph(self) -> GraphDocumentSubgraph {
        GraphDocumentSubgraph {
            id: self.id,
            name: self.name,
            document: Box::new(self.document.into_document()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphDocumentViewStateV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera: Option<GraphDocumentCameraStateV1>,
    #[serde(default)]
    pub selected_node_ids: Vec<u64>,
}

impl GraphDocumentViewStateV1 {
    fn from_view(view: &GraphDocumentViewState) -> Self {
        Self {
            camera: view
                .camera
                .as_ref()
                .map(GraphDocumentCameraStateV1::from_camera),
            selected_node_ids: view.selected_node_ids.clone(),
        }
    }

    fn into_view(self) -> GraphDocumentViewState {
        GraphDocumentViewState {
            camera: self.camera.map(GraphDocumentCameraStateV1::into_camera),
            selected_node_ids: self.selected_node_ids,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocumentCameraStateV1 {
    pub translation: [f32; 3],
    pub ortho_scale: f32,
}

impl GraphDocumentCameraStateV1 {
    fn from_camera(camera: &GraphDocumentCameraState) -> Self {
        Self {
            translation: camera.translation,
            ortho_scale: camera.ortho_scale,
        }
    }

    fn into_camera(self) -> GraphDocumentCameraState {
        GraphDocumentCameraState {
            translation: self.translation,
            ortho_scale: self.ortho_scale,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GraphValueV1 {
    None,
    Float { value: f64 },
    Int { value: i64 },
    Bool { value: bool },
    String { value: String },
    Vec2 { value: [f32; 2] },
    Vec3 { value: [f32; 3] },
    Vec4 { value: [f32; 4] },
    Color { space: String, value: [f32; 4] },
    Entity { value: EntityValueV1 },
    Tagged { tag: String, payload: JsonValue },
}

impl GraphValueV1 {
    fn from_value(value: &NodeValue) -> Self {
        match value {
            NodeValue::None => Self::None,
            NodeValue::Float(value) => Self::Float { value: *value },
            NodeValue::Int(value) => Self::Int { value: *value },
            NodeValue::Bool(value) => Self::Bool { value: *value },
            NodeValue::String(value) => Self::String {
                value: value.clone(),
            },
            NodeValue::Vec2(value) => Self::Vec2 {
                value: [value.x, value.y],
            },
            NodeValue::Vec3(value) => Self::Vec3 {
                value: [value.x, value.y, value.z],
            },
            NodeValue::Vec4(value) => Self::Vec4 {
                value: [value.x, value.y, value.z, value.w],
            },
            NodeValue::Color(value) => Self::Color {
                space: GRAPH_SAVE_COLOR_SPACE_SRGBA.to_string(),
                value: color_to_array(value),
            },
            NodeValue::Entity(value) => Self::Entity {
                value: EntityValueV1::from_entity_value(value),
            },
            NodeValue::TaggedData { tag, payload } => Self::Tagged {
                tag: tag.clone(),
                payload: payload.clone(),
            },
        }
    }

    fn into_value(self) -> NodeValue {
        match self {
            Self::None => NodeValue::None,
            Self::Float { value } => NodeValue::Float(value),
            Self::Int { value } => NodeValue::Int(value),
            Self::Bool { value } => NodeValue::Bool(value),
            Self::String { value } => NodeValue::String(value),
            Self::Vec2 { value } => NodeValue::Vec2(Vec2::new(value[0], value[1])),
            Self::Vec3 { value } => NodeValue::Vec3(Vec3::new(value[0], value[1], value[2])),
            Self::Vec4 { value } => {
                NodeValue::Vec4(Vec4::new(value[0], value[1], value[2], value[3]))
            }
            Self::Color { value, .. } => {
                NodeValue::Color(Color::srgba(value[0], value[1], value[2], value[3]))
            }
            Self::Entity { value } => NodeValue::Entity(value.into_entity_value()),
            Self::Tagged { tag, payload } => NodeValue::TaggedData { tag, payload },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityValueV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub components: Vec<EntityComponentV1>,
    #[serde(default)]
    pub children: Vec<EntityValueV1>,
}

impl EntityValueV1 {
    fn from_entity_value(entity: &EntityValue) -> Self {
        Self {
            name: entity.name.clone(),
            components: entity
                .components
                .iter()
                .map(EntityComponentV1::from_component)
                .collect(),
            children: entity
                .children
                .iter()
                .map(EntityValueV1::from_entity_value)
                .collect(),
        }
    }

    fn into_entity_value(self) -> EntityValue {
        EntityValue {
            name: self.name,
            components: self
                .components
                .into_iter()
                .map(EntityComponentV1::into_component)
                .collect(),
            children: self
                .children
                .into_iter()
                .map(EntityValueV1::into_entity_value)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EntityComponentV1 {
    Transform {
        translation: [f32; 3],
        #[serde(default)]
        rotation_deg: f32,
        #[serde(default = "default_scale_array")]
        scale: [f32; 3],
    },
    Sprite {
        #[serde(default = "default_size_array")]
        size: [f32; 2],
        color: [f32; 4],
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sprite_id: Option<String>,
    },
    #[serde(rename = "camera_2d")]
    Camera2d {
        zoom: f32,
    },
    #[serde(rename = "text_2d")]
    Text2d {
        content: String,
        font_size: f32,
        color: [f32; 4],
    },
    Visibility {
        visible: bool,
    },
    Anchor {
        position: [f32; 2],
    },
    Custom {
        key: String,
        payload: JsonValue,
    },
}

impl EntityComponentV1 {
    fn from_component(component: &EntityComponentValue) -> Self {
        match component {
            EntityComponentValue::Transform(value) => Self::Transform {
                translation: vec3_to_array(value.translation),
                rotation_deg: value.rotation_deg,
                scale: vec3_to_array(value.scale),
            },
            EntityComponentValue::Sprite(value) => Self::Sprite {
                size: vec2_to_array(value.size),
                color: color_to_array(&value.color),
                sprite_id: value.sprite_id.clone(),
            },
            EntityComponentValue::Camera2D(value) => Self::Camera2d { zoom: value.zoom },
            EntityComponentValue::Text2D(value) => Self::Text2d {
                content: value.content.clone(),
                font_size: value.font_size,
                color: color_to_array(&value.color),
            },
            EntityComponentValue::Visibility(value) => Self::Visibility {
                visible: value.visible,
            },
            EntityComponentValue::Anchor(value) => Self::Anchor {
                position: vec2_to_array(value.position),
            },
            EntityComponentValue::Custom { key, payload } => Self::Custom {
                key: key.clone(),
                payload: payload.clone(),
            },
        }
    }

    fn into_component(self) -> EntityComponentValue {
        match self {
            Self::Transform {
                translation,
                rotation_deg,
                scale,
            } => EntityComponentValue::Transform(TransformComponentValue {
                translation: Vec3::new(translation[0], translation[1], translation[2]),
                rotation_deg,
                scale: Vec3::new(scale[0], scale[1], scale[2]),
            }),
            Self::Sprite {
                size,
                color,
                sprite_id,
            } => EntityComponentValue::Sprite(SpriteComponentValue {
                size: Vec2::new(size[0], size[1]),
                color: Color::srgba(color[0], color[1], color[2], color[3]),
                sprite_id,
            }),
            Self::Camera2d { zoom } => {
                EntityComponentValue::Camera2D(Camera2DComponentValue { zoom })
            }
            Self::Text2d {
                content,
                font_size,
                color,
            } => EntityComponentValue::Text2D(Text2DComponentValue {
                content,
                font_size,
                color: Color::srgba(color[0], color[1], color[2], color[3]),
            }),
            Self::Visibility { visible } => {
                EntityComponentValue::Visibility(VisibilityComponentValue { visible })
            }
            Self::Anchor { position } => EntityComponentValue::Anchor(AnchorComponentValue {
                position: Vec2::new(position[0], position[1]),
            }),
            Self::Custom { key, payload } => EntityComponentValue::Custom { key, payload },
        }
    }
}

fn default_scale_array() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_size_array() -> [f32; 2] {
    [1.0, 1.0]
}

fn vec2_to_array(value: Vec2) -> [f32; 2] {
    [value.x, value.y]
}

fn vec3_to_array(value: Vec3) -> [f32; 3] {
    [value.x, value.y, value.z]
}

fn color_to_array(value: &Color) -> [f32; 4] {
    let srgba = value.to_srgba();
    [srgba.red, srgba.green, srgba.blue, srgba.alpha]
}

fn current_timestamp_string() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis().to_string(),
        Err(_) => "0".to_string(),
    }
}

fn serialize_save_file(save_file: &GraphSaveFileV1, pretty_json: bool) -> Result<String, String> {
    if pretty_json {
        serde_json::to_string_pretty(save_file)
            .map_err(|err| format!("cannot serialize JSON payload: {}", err))
    } else {
        serde_json::to_string(save_file)
            .map_err(|err| format!("cannot serialize JSON payload: {}", err))
    }
}

pub(crate) fn is_graph_save_file_envelope(value: &JsonValue) -> bool {
    value
        .get("format")
        .and_then(|field| field.as_str())
        .is_some()
}

pub(crate) fn parse_graph_save_file_value(value: JsonValue) -> Result<GraphSaveFileV1, String> {
    let save_file: GraphSaveFileV1 = serde_json::from_value(value)
        .map_err(|err| format!("invalid save-file v1 payload: {}", err))?;

    if save_file.format != GRAPH_SAVE_FILE_FORMAT {
        return Err(format!(
            "unsupported save-file format {} (expected {})",
            save_file.format, GRAPH_SAVE_FILE_FORMAT
        ));
    }

    if save_file.format_version != GRAPH_SAVE_FILE_VERSION {
        return Err(format!(
            "unsupported save-file version {} (latest supported {})",
            save_file.format_version, GRAPH_SAVE_FILE_VERSION
        ));
    }

    Ok(save_file)
}

pub fn parse_graph_document_payload(content: &str) -> Result<ParsedGraphDocument, String> {
    let value: JsonValue =
        serde_json::from_str(content).map_err(|err| format!("invalid JSON: {}", err))?;
    let parsed = crate::migrations::parse_save_file_payload(value)?;

    Ok(ParsedGraphDocument {
        document: parsed.save_file.document.into_document(),
        migration_note: parsed.migration_note,
    })
}

pub fn serialize_graph_document(
    document: &GraphDocument,
    pretty_json: bool,
) -> Result<String, String> {
    serialize_save_file(&GraphSaveFileV1::from_document(document), pretty_json)
}

pub fn parse_graph_save_metadata(content: &str) -> Result<Option<GraphSaveMetaV1>, String> {
    let value: JsonValue =
        serde_json::from_str(content).map_err(|err| format!("invalid JSON: {}", err))?;

    if !is_graph_save_file_envelope(&value) {
        return Ok(None);
    }

    let save_file = parse_graph_save_file_value(value)?;
    Ok(Some(save_file.meta))
}

fn finalize_prepared_graph_write(
    document: GraphDocument,
    pretty_json: bool,
    meta: GraphSaveMetaV1,
    validation_report: GraphValidationReport,
) -> Result<PreparedGraphWrite, String> {
    let payload = serialize_save_file(
        &GraphSaveFileV1::from_document_with_meta(&document, meta.refreshed()),
        pretty_json,
    )?;
    let document_signature = graph_document_signature(&document)?;

    Ok(PreparedGraphWrite {
        document,
        payload,
        document_signature,
        validation_report,
    })
}

pub(crate) fn prepare_graph_document_write_with_meta_and_validation_report(
    document: GraphDocument,
    pretty_json: bool,
    meta: GraphSaveMetaV1,
    validation_report: GraphValidationReport,
) -> Result<PreparedGraphWrite, String> {
    finalize_prepared_graph_write(document, pretty_json, meta, validation_report)
}

pub(crate) fn prepare_graph_document_write_with_meta(
    document: GraphDocument,
    pretty_json: bool,
    registry: &NodeRegistry,
    meta: GraphSaveMetaV1,
) -> Result<PreparedGraphWrite, String> {
    let validation_report = validate_graph_document(&document, registry.core_registry());
    finalize_prepared_graph_write(document, pretty_json, meta, validation_report)
}

pub fn prepare_graph_document_write(
    document: GraphDocument,
    pretty_json: bool,
    registry: &NodeRegistry,
) -> Result<PreparedGraphWrite, String> {
    prepare_graph_document_write_with_meta(
        document,
        pretty_json,
        registry,
        GraphSaveMetaV1::fresh(),
    )
}
