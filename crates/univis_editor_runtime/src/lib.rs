use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use univis_editor_core::prelude::{
    Connecting, GraphNode, NodeRegistry, NodeValue, ProcessContext, ProcessResult,
};

pub struct NodeRuntimePlugin;

impl Plugin for NodeRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (initialize_node_defaults, propagate_and_process_nodes).chain(),
        );
    }
}

fn initialize_node_defaults(
    registry: Res<NodeRegistry>,
    mut q_nodes: Query<(Entity, &mut GraphNode), Added<GraphNode>>,
) {
    for (_entity, mut node) in q_nodes.iter_mut() {
        let Some(definition) = registry.get(&node.definition_id) else {
            continue;
        };

        let default_inputs = definition.default_input_values();
        for (i, default_val) in default_inputs.iter().enumerate() {
            if i < node.values.inputs.len() {
                node.values.inputs[i] = default_val.clone();
            }
        }

        for output in node.values.outputs.iter_mut() {
            *output = NodeValue::None;
        }
    }
}

fn propagate_and_process_nodes(
    registry: Res<NodeRegistry>,
    graph: Res<Connecting>,
    mut q_nodes: Query<(Entity, &mut GraphNode)>,
    time: Res<Time>,
) {
    let all_entities: Vec<Entity> = q_nodes.iter().map(|(e, _)| e).collect();

    let mut reverse_graph: HashMap<Entity, HashSet<Entity>> = HashMap::new();
    let mut in_degree: HashMap<Entity, usize> = HashMap::new();

    for entity in &all_entities {
        reverse_graph.entry(*entity).or_default();
        in_degree.entry(*entity).or_insert(0);
    }

    for conn in &graph.connections {
        if in_degree.contains_key(&conn.from_node) && in_degree.contains_key(&conn.to_node) {
            reverse_graph
                .entry(conn.from_node)
                .or_default()
                .insert(conn.to_node);
            *in_degree.entry(conn.to_node).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<Entity> = VecDeque::new();
    let mut sorted_nodes: Vec<Entity> = Vec::new();

    for (entity, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(*entity);
        }
    }

    while let Some(entity) = queue.pop_front() {
        sorted_nodes.push(entity);

        if let Some(dependents) = reverse_graph.get(&entity) {
            for &dependent in dependents {
                if let Some(degree) = in_degree.get_mut(&dependent) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }
    }

    let mut processed_outputs: HashMap<Entity, Vec<NodeValue>> = HashMap::new();
    for (entity, node) in q_nodes.iter() {
        processed_outputs.insert(entity, node.values.outputs.clone());
    }

    for entity in sorted_nodes {
        let Ok((_, mut node)) = q_nodes.get_mut(entity) else {
            continue;
        };

        for conn in &graph.connections {
            if conn.to_node == entity {
                if let Some(outputs) = processed_outputs.get(&conn.from_node) {
                    if conn.from_index < outputs.len() && conn.to_index < node.values.inputs.len() {
                        node.values.inputs[conn.to_index] = outputs[conn.from_index].clone();
                    }
                }
            }
        }

        let definition_id = node.definition_id.clone();
        let Some(definition) = registry.get(&definition_id) else {
            continue;
        };

        let inputs = node.values.inputs.clone();
        let mut outputs = node.values.outputs.clone();
        let custom_data = &mut node.custom_data;

        let mut context = ProcessContext {
            inputs: &inputs,
            outputs: &mut outputs,
            delta_time: time.delta_secs(),
            custom_data,
        };

        let result = definition.process(&mut context);
        match result {
            ProcessResult::Success => {}
            ProcessResult::Error(msg) => {
                warn!("Node {} error: {}", definition_id, msg);
            }
            ProcessResult::MissingInput(index) => {
                debug!("Node {} missing input at index {}", definition_id, index);
            }
        }

        node.values.outputs = outputs;
        processed_outputs.insert(entity, node.values.outputs.clone());
    }
}

pub mod prelude {
    pub use crate::NodeRuntimePlugin;
}
