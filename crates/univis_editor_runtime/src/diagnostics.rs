use bevy::prelude::*;
use std::collections::HashMap;
use univis_node_graph::prelude::{
    analyze_graph_topology, Connecting, GraphNode, NodeRegistry, NodeValue, ProcessContext,
    ProcessResult,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphRuntimeDiagnostics {
    pub blocked_nodes: Vec<Entity>,
    pub node_issues: Vec<GraphRuntimeNodeIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphRuntimeIssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRuntimeNodeIssue {
    pub node: Entity,
    pub definition_id: String,
    pub severity: GraphRuntimeIssueSeverity,
    pub message: String,
}

pub(super) fn propagate_and_process_nodes_system(
    registry: Res<NodeRegistry>,
    graph: Res<Connecting>,
    mut diagnostics: ResMut<GraphRuntimeDiagnostics>,
    mut q_nodes: Query<(Entity, &mut GraphNode)>,
    time: Res<Time>,
) {
    let all_entities: Vec<Entity> = q_nodes.iter().map(|(e, _)| e).collect();
    let topology = analyze_graph_topology(
        all_entities.iter().copied(),
        graph
            .connections
            .iter()
            .map(|conn| (conn.from_node, conn.to_node)),
    );
    let sorted_nodes = topology.ordered_nodes;
    let blocked_nodes = topology.blocked_nodes;

    if diagnostics.blocked_nodes != blocked_nodes {
        if blocked_nodes.is_empty() {
            diagnostics.blocked_nodes.clear();
        } else {
            diagnostics.blocked_nodes = blocked_nodes.clone();
            warn!(
                "Graph runtime skipped {} node(s) because the graph contains a cycle or blocked dependency path.",
                diagnostics.blocked_nodes.len()
            );
        }
    }
    diagnostics.node_issues.clear();

    let mut processed_outputs = HashMap::<Entity, Vec<NodeValue>>::new();
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
                diagnostics.node_issues.push(GraphRuntimeNodeIssue {
                    node: entity,
                    definition_id: definition_id.to_string(),
                    severity: GraphRuntimeIssueSeverity::Error,
                    message: format!("Processing error: {}", msg),
                });
            }
            ProcessResult::MissingInput(index) => {
                debug!("Node {} missing input at index {}", definition_id, index);
                diagnostics.node_issues.push(GraphRuntimeNodeIssue {
                    node: entity,
                    definition_id: definition_id.to_string(),
                    severity: GraphRuntimeIssueSeverity::Warning,
                    message: format!("Missing input: {}", index),
                });
            }
        }

        node.values.outputs = outputs;
        processed_outputs.insert(entity, node.values.outputs.clone());
    }
}
