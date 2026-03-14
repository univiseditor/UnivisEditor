use bevy::prelude::*;
use univis_node_graph::node_definition::{
    PortDefinition, PortRequirement, ProcessContext, ProcessResult,
};
use univis_node_graph::value::{NodeValue, ValueType};
use univis_scene::{
    component_display_name, component_port_color,
    pure_entity_requirement_token as scene_pure_entity_requirement_token, EntityComponentValue,
    EntityValue, TransformComponentValue, TRANSFORM_COMPONENT_KEY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformResolutionSource {
    SpecificInput,
    BaseEntity,
    LocalDefault,
}

#[derive(Debug, Clone)]
pub struct ResolvedTransform {
    pub value: TransformComponentValue,
    pub source: TransformResolutionSource,
}

pub fn popup_float(
    name: impl Into<String>,
    default: f64,
    step: f64,
    min: Option<f64>,
    max: Option<f64>,
) -> PortDefinition {
    let mut port = PortDefinition::input_float(name)
        .with_default(NodeValue::float(default))
        .editable_in_popup()
        .with_ui_step(step);

    if let Some(min) = min {
        port = port.with_ui_min(min);
    }
    if let Some(max) = max {
        port = port.with_ui_max(max);
    }

    port
}

pub fn popup_bool(name: impl Into<String>, default: bool) -> PortDefinition {
    PortDefinition::input_bool(name)
        .with_default(NodeValue::bool(default))
        .editable_in_popup()
}

pub fn scene_entity_requirement(component_key: &str) -> PortRequirement {
    PortRequirement::new(
        scene_pure_entity_requirement_token(component_key),
        component_display_name(component_key),
    )
    .with_color(component_port_color(component_key))
}

pub fn pure_entity_requirement_token(component_key: &str) -> String {
    scene_pure_entity_requirement_token(component_key)
}

pub fn scene_entity_output(name: impl Into<String>) -> PortDefinition {
    PortDefinition::output_entity(name)
}

pub fn scene_entity_input(name: impl Into<String>) -> PortDefinition {
    PortDefinition::input_entity(name)
}

pub fn entity_extension_input() -> PortDefinition {
    scene_entity_input("Entity").with_description("Optional base entity to extend")
}

pub fn transform_override_input() -> PortDefinition {
    scene_entity_input("Transform")
        .with_requirement(scene_entity_requirement(TRANSFORM_COMPONENT_KEY))
        .with_description("Optional Transform entity override")
}

pub fn transform_fallback_inputs() -> [PortDefinition; 7] {
    [
        popup_float("Tx", 0.0, 1.0, None, None),
        popup_float("Ty", 0.0, 1.0, None, None),
        popup_float("Tz", 0.0, 1.0, None, None),
        popup_float("Rotation", 0.0, 1.0, None, None),
        popup_float("Sx", 1.0, 0.1, Some(0.0), None),
        popup_float("Sy", 1.0, 0.1, Some(0.0), None),
        popup_float("Sz", 1.0, 0.1, Some(0.0), None),
    ]
}

pub fn base_entity_or_empty(ctx: &ProcessContext, index: usize) -> EntityValue {
    ctx.get_entity(index).unwrap_or_default()
}

pub fn fallback_transform(
    tx: f64,
    ty: f64,
    tz: f64,
    rotation_deg: f64,
    sx: f64,
    sy: f64,
    sz: f64,
) -> TransformComponentValue {
    TransformComponentValue {
        translation: Vec3::new(tx as f32, ty as f32, tz as f32),
        rotation_deg: rotation_deg as f32,
        scale: Vec3::new(sx as f32, sy as f32, sz as f32),
    }
}

pub fn resolve_transform(
    ctx: &ProcessContext,
    base_entity: &EntityValue,
    specific_index: usize,
    fallback_index_start: usize,
    input_name: &str,
) -> (ResolvedTransform, Option<String>) {
    let fallback = ResolvedTransform {
        value: fallback_transform(
            ctx.get_float_or(fallback_index_start, 0.0),
            ctx.get_float_or(fallback_index_start + 1, 0.0),
            ctx.get_float_or(fallback_index_start + 2, 0.0),
            ctx.get_float_or(fallback_index_start + 3, 0.0),
            ctx.get_float_or(fallback_index_start + 4, 1.0),
            ctx.get_float_or(fallback_index_start + 5, 1.0),
            ctx.get_float_or(fallback_index_start + 6, 1.0),
        ),
        source: TransformResolutionSource::LocalDefault,
    };

    if let Some(entity) = ctx.get_entity(specific_index) {
        if entity.is_pure_component(TRANSFORM_COMPONENT_KEY) {
            if let Some(transform) = entity
                .component(TRANSFORM_COMPONENT_KEY)
                .and_then(EntityComponentValue::as_transform)
            {
                return (
                    ResolvedTransform {
                        value: transform.clone(),
                        source: TransformResolutionSource::SpecificInput,
                    },
                    None,
                );
            }
        }

        if let Some(transform) = base_entity.transform() {
            return (
                ResolvedTransform {
                    value: transform.clone(),
                    source: TransformResolutionSource::BaseEntity,
                },
                Some(format!(
                    "Input '{}' requires a pure Transform entity; using the base entity Transform instead.",
                    input_name
                )),
            );
        }

        return (
            fallback,
            Some(format!(
                "Input '{}' requires a pure Transform entity; using local defaults instead.",
                input_name
            )),
        );
    }

    if let Some(transform) = base_entity.transform() {
        return (
            ResolvedTransform {
                value: transform.clone(),
                source: TransformResolutionSource::BaseEntity,
            },
            None,
        );
    }

    (fallback, None)
}

pub fn apply_resolved_transform(entity: &mut EntityValue, transform: &ResolvedTransform) {
    if matches!(
        transform.source,
        TransformResolutionSource::SpecificInput | TransformResolutionSource::LocalDefault
    ) {
        entity.set_component(EntityComponentValue::Transform(transform.value.clone()));
    }
}

pub fn finish_entity_process(
    ctx: &mut ProcessContext,
    entity: EntityValue,
    warning: Option<String>,
) -> ProcessResult {
    ctx.set_entity(0, entity);
    warning.map_or(ProcessResult::Success, ProcessResult::Error)
}

pub fn missing_entity_input(ctx: &mut ProcessContext, index: usize) -> ProcessResult {
    ctx.set(0, NodeValue::None);
    ProcessResult::MissingInput(index)
}

pub fn popup_color(name: impl Into<String>, default: Color) -> PortDefinition {
    PortDefinition::new(name, ValueType::Color)
        .with_default(NodeValue::Color(default))
        .editable_in_popup()
        .with_ui_step(0.05)
        .with_ui_range(0.0, 1.0)
}

pub fn popup_string(name: impl Into<String>, default: impl Into<String>) -> PortDefinition {
    PortDefinition::input_string(name)
        .with_default(NodeValue::string(default))
        .editable_in_popup()
}
