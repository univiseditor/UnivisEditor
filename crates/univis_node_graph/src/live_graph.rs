//! Live Bevy ECS components for the graph adapter.
use std::any::Any;

use bevy::prelude::*;

use crate::node_definition::NodeId;
use crate::value::{NodeValue, NodeValues, ValueType};

#[derive(Component)]
pub struct GraphNode {
    pub definition_id: NodeId,
    /// ECS-facing projection buffers for display and integration.
    ///
    /// `inputs` mirror authored or resolved values for widgets and visuals.
    /// `outputs` mirror executable outputs or externally synced visual values.
    /// These buffers are not the authoritative execution state.
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

    pub fn input_projection_len(&self) -> usize {
        self.values.inputs.len()
    }

    pub fn output_projection_len(&self) -> usize {
        self.values.outputs.len()
    }

    pub fn input_projection_values(&self) -> &[NodeValue] {
        &self.values.inputs
    }

    pub fn output_projection_values(&self) -> &[NodeValue] {
        &self.values.outputs
    }

    pub fn input_projection(&self, index: usize) -> Option<&NodeValue> {
        self.values.inputs.get(index)
    }

    pub fn output_projection(&self, index: usize) -> Option<&NodeValue> {
        self.values.outputs.get(index)
    }

    pub fn set_input_projection(&mut self, index: usize, value: NodeValue) -> bool {
        if let Some(slot) = self.values.inputs.get_mut(index) {
            if *slot != value {
                *slot = value;
                return true;
            }
        }
        false
    }

    pub fn set_output_projection(&mut self, index: usize, value: NodeValue) -> bool {
        if let Some(slot) = self.values.outputs.get_mut(index) {
            if *slot != value {
                *slot = value;
                return true;
            }
        }
        false
    }

    pub fn sync_input_projection_from_authored(&mut self, authored_inputs: &AuthoredNodeInputs) {
        for index in 0..self.input_projection_len() {
            let value = authored_inputs
                .values
                .get(index)
                .cloned()
                .unwrap_or(NodeValue::None);
            let _ = self.set_input_projection(index, value);
        }
    }

    pub fn replace_input_projection(&mut self, values: &[NodeValue]) {
        self.values.inputs.clear();
        self.values.inputs.extend_from_slice(values);
    }

    pub fn replace_output_projection(&mut self, values: &[NodeValue]) {
        self.values.outputs.clear();
        self.values.outputs.extend_from_slice(values);
    }

    pub fn clear_output_projection(&mut self) {
        for output in &mut self.values.outputs {
            *output = NodeValue::None;
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct AuthoredNodeInputs {
    pub values: Vec<NodeValue>,
}

impl AuthoredNodeInputs {
    pub fn ensure_len(&mut self, len: usize) {
        if self.values.len() < len {
            self.values.resize(len, NodeValue::None);
        }
    }
}

pub fn graph_node_authored_inputs_for_snapshot(
    node: &GraphNode,
    authored_inputs: Option<&AuthoredNodeInputs>,
) -> Vec<NodeValue> {
    authored_inputs
        .map(|inputs| inputs.values.clone())
        .unwrap_or_else(|| node.input_projection_values().to_vec())
}

pub fn set_graph_node_authored_input_value(
    graph_node: &mut GraphNode,
    authored_inputs: Option<&mut AuthoredNodeInputs>,
    input_index: usize,
    value: NodeValue,
) -> bool {
    if input_index >= graph_node.input_projection_len() {
        return false;
    }

    let projection_unchanged = graph_node.input_projection(input_index) == Some(&value);
    let authored_unchanged = authored_inputs
        .as_ref()
        .and_then(|inputs| inputs.values.get(input_index))
        == Some(&value);
    if projection_unchanged && authored_unchanged {
        return false;
    }

    let _ = graph_node.set_input_projection(input_index, value.clone());

    if let Some(authored_inputs) = authored_inputs {
        authored_inputs.ensure_len(graph_node.input_projection_len());
        authored_inputs.values[input_index] = value;
    }

    true
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
