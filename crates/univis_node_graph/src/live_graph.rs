//! Live Bevy ECS components for the graph adapter.
use std::any::Any;

use bevy::prelude::*;

use crate::node_definition::NodeId;
use crate::value::{NodeValue, NodeValues, ValueType};

#[derive(Component)]
pub struct GraphNode {
    pub definition_id: NodeId,
    pub values: NodeValues,
    pub custom_data: Option<Box<dyn Any + Send + Sync>>,
}

impl GraphNode {
    pub fn new(definition_id: NodeId, input_count: usize, output_count: usize) -> Self {
        Self {
            definition_id,
            values: NodeValues::new(input_count, output_count),
            custom_data: None,
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct AuthoredNodeInputs {
    pub values: Vec<NodeValue>,
}

#[derive(Component)]
pub struct GraphPort {
    pub node_entity: Entity,
    pub port_type: PortType,
    pub index: usize,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    Input,
    Output,
}

#[derive(Component, Debug, Clone, Default)]
pub struct InputConnection {
    pub source_node: Option<Entity>,
    pub source_port_index: Option<usize>,
    pub source_port: Option<Entity>,
    pub connection_entity: Option<Entity>,
}

#[derive(Component, Debug, Clone, Default)]
pub struct OutputConnections {
    pub targets: Vec<OutputTarget>,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputTarget {
    pub target_node: Entity,
    pub target_port_index: usize,
    pub target_port: Entity,
    pub connection_entity: Entity,
}

#[derive(Component)]
pub struct InputPort {
    pub node_entity: Entity,
    pub index: usize,
    pub value_type: ValueType,
    pub source: Option<PortRef>,
}

#[derive(Component)]
pub struct OutputPort {
    pub node_entity: Entity,
    pub index: usize,
    pub value_type: ValueType,
    pub value: NodeValue,
    pub targets: Vec<PortRef>,
}

#[derive(Debug, Clone, Copy)]
pub struct PortRef {
    pub node: Entity,
    pub port_index: usize,
}

impl InputPort {
    pub fn get_value(
        &self,
        outputs: &std::collections::HashMap<Entity, Vec<NodeValue>>,
    ) -> NodeValue {
        if let Some(source) = self.source {
            if let Some(node_outputs) = outputs.get(&source.node) {
                if source.port_index < node_outputs.len() {
                    return node_outputs[source.port_index].clone();
                }
            }
        }
        NodeValue::None
    }
}

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct ValueDisplayLabel {
    pub node_entity: Entity,
}

#[derive(Component)]
pub struct Dragging;
