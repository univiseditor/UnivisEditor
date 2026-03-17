use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use univis_node_graph::prelude::{
    AuthoredNodeInputs, GraphNode, NodeRegistry, NodeValue, ProcessContext, ProcessResult,
};

use crate::connectivity::{
    GraphConnectivityIndex, GraphResolvedInputs, NodeInputSignature, NodeOutputSignature,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphRuntimeDiagnostics {
    pub blocked_nodes: Vec<Entity>,
    pub node_issues: Vec<GraphRuntimeNodeIssue>,
}

#[derive(Resource, Debug, Clone)]
pub struct GraphRuntimeTraceSettings {
    pub enabled: bool,
    pub max_entries: usize,
}

impl Default for GraphRuntimeTraceSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_entries: 32,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphRuntimeTrace {
    pub entries: Vec<GraphRuntimeTraceEntry>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRuntimeTraceEntry {
    pub node: Entity,
    pub definition_id: String,
    pub reasons: Vec<String>,
    pub result: String,
    pub outputs_changed: bool,
}

pub(super) fn propagate_and_process_nodes_system(
    registry: Res<NodeRegistry>,
    connectivity: Res<GraphConnectivityIndex>,
    mut diagnostics: ResMut<GraphRuntimeDiagnostics>,
    trace_settings: Res<GraphRuntimeTraceSettings>,
    mut runtime_trace: ResMut<GraphRuntimeTrace>,
    mut resolved_inputs: ResMut<GraphResolvedInputs>,
    mut q_nodes: ParamSet<(
        Query<(
            Entity,
            &GraphNode,
            &AuthoredNodeInputs,
            &NodeInputSignature,
            &NodeOutputSignature,
        )>,
        Query<(
            Entity,
            &mut GraphNode,
            &AuthoredNodeInputs,
            &mut NodeInputSignature,
            &mut NodeOutputSignature,
        )>,
    )>,
    time: Res<Time>,
) {
    let connectivity_changed = connectivity.is_changed();
    if diagnostics.blocked_nodes != connectivity.blocked_nodes {
        diagnostics.blocked_nodes = connectivity.blocked_nodes.clone();
        if !diagnostics.blocked_nodes.is_empty() {
            warn!(
                "Graph runtime skipped {} node(s) because the graph contains a cycle or blocked dependency path.",
                diagnostics.blocked_nodes.len()
            );
        }
    }

    let node_count_hint = q_nodes.p0().iter().len();
    let mut known_nodes = HashSet::with_capacity(node_count_hint);
    let mut dirty_nodes = HashSet::with_capacity(node_count_hint);
    let mut dirty_reasons = HashMap::<Entity, Vec<String>>::with_capacity(node_count_hint);
    let mut outputs_by_node = HashMap::<Entity, Vec<NodeValue>>::with_capacity(node_count_hint);

    {
        let nodes = q_nodes.p0();
        for (entity, node, authored_inputs, input_signature, output_signature) in nodes.iter() {
            known_nodes.insert(entity);
            outputs_by_node.insert(entity, node.values.outputs.clone());
            resolved_inputs
                .by_node
                .entry(entity)
                .or_insert_with(|| authored_inputs.values.clone());

            let mut reasons = Vec::new();
            if connectivity_changed {
                reasons.push("graph connectivity changed".to_string());
            }
            if input_signature.inputs != authored_inputs.values {
                reasons.push("authored inputs changed".to_string());
            }
            if output_signature.outputs != node.values.outputs {
                reasons.push("visual or external output changed".to_string());
            }

            if !reasons.is_empty() {
                dirty_nodes.insert(entity);
                dirty_reasons.insert(entity, reasons);
            }
        }
    }

    resolved_inputs
        .by_node
        .retain(|entity, _| known_nodes.contains(entity));

    let mut issues_by_node = diagnostics
        .node_issues
        .drain(..)
        .filter(|issue| known_nodes.contains(&issue.node))
        .map(|issue| (issue.node, issue))
        .collect::<HashMap<_, _>>();

    if dirty_nodes.is_empty() && !connectivity_changed {
        diagnostics.node_issues = sorted_issues(issues_by_node);
        if trace_settings.enabled {
            runtime_trace.entries.clear();
        }
        return;
    }

    if trace_settings.enabled {
        runtime_trace.entries.clear();
    }

    for entity in connectivity.ordered_nodes.iter().copied() {
        if !dirty_nodes.contains(&entity) {
            continue;
        }

        let mut mutable_nodes = q_nodes.p1();
        let Ok((_, mut node, authored_inputs, mut input_signature, mut output_signature)) =
            mutable_nodes.get_mut(entity)
        else {
            continue;
        };

        let mut resolved = authored_inputs.values.clone();
        if let Some(sources) = connectivity.incoming_by_node_input.get(&entity) {
            for (input_index, source) in sources.iter().enumerate() {
                let Some(source) = source else {
                    continue;
                };

                let value = outputs_by_node
                    .get(&source.source_node)
                    .and_then(|outputs| outputs.get(source.source_index))
                    .cloned()
                    .unwrap_or(NodeValue::None);

                if let Some(slot) = resolved.get_mut(input_index) {
                    *slot = value;
                }
            }
        }
        match resolved_inputs.by_node.entry(entity) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().clone_from(&resolved);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(resolved.clone());
            }
        }

        let definition_id = node.definition_id.clone();
        let Some(definition) = registry.get(&definition_id) else {
            issues_by_node.insert(
                entity,
                GraphRuntimeNodeIssue {
                    node: entity,
                    definition_id: definition_id.to_string(),
                    severity: GraphRuntimeIssueSeverity::Error,
                    message: "Missing node definition in registry.".to_string(),
                },
            );
            continue;
        };

        let mut outputs = node.values.outputs.clone();

        let result = {
            let custom_data = &mut node.custom_data;
            let mut context = ProcessContext {
                inputs: &resolved,
                outputs: &mut outputs,
                delta_time: time.delta_secs(),
                custom_data,
            };
            definition.process(&mut context)
        };

        let result_summary = match &result {
            ProcessResult::Success => "success".to_string(),
            ProcessResult::Error(message) => format!("error: {message}"),
            ProcessResult::MissingInput(index) => format!("missing input: {index}"),
        };

        match &result {
            ProcessResult::Success => {
                issues_by_node.remove(&entity);
            }
            ProcessResult::Error(message) => {
                issues_by_node.insert(
                    entity,
                    GraphRuntimeNodeIssue {
                        node: entity,
                        definition_id: definition_id.to_string(),
                        severity: GraphRuntimeIssueSeverity::Error,
                        message: format!("Processing error: {}", message),
                    },
                );
            }
            ProcessResult::MissingInput(index) => {
                issues_by_node.insert(
                    entity,
                    GraphRuntimeNodeIssue {
                        node: entity,
                        definition_id: definition_id.to_string(),
                        severity: GraphRuntimeIssueSeverity::Warning,
                        message: format!("Missing input: {}", index),
                    },
                );
            }
        }

        let outputs_changed = output_signature.outputs != outputs;
        if trace_settings.enabled {
            runtime_trace.entries.push(GraphRuntimeTraceEntry {
                node: entity,
                definition_id: definition_id.to_string(),
                reasons: dirty_reasons.remove(&entity).unwrap_or_default(),
                result: result_summary,
                outputs_changed,
            });
        }
        node.values.inputs.clone_from(&resolved);
        node.values.outputs.clone_from(&outputs);
        input_signature.inputs.clone_from(&authored_inputs.values);
        output_signature.outputs.clone_from(&outputs);
        outputs_by_node.insert(entity, outputs);

        if outputs_changed {
            if let Some(outputs) = connectivity.outgoing_by_node_output.get(&entity) {
                for targets in outputs {
                    for target in targets {
                        dirty_nodes.insert(target.target_node);
                        dirty_reasons
                            .entry(target.target_node)
                            .or_default()
                            .push(format!(
                                "upstream output changed: {} -> input {}",
                                definition_id, target.target_index
                            ));
                    }
                }
            }
        }
    }

    diagnostics.node_issues = sorted_issues(issues_by_node);
    if trace_settings.enabled && runtime_trace.entries.len() > trace_settings.max_entries {
        let keep_from = runtime_trace.entries.len() - trace_settings.max_entries;
        runtime_trace.entries.drain(0..keep_from);
    }
}

fn sorted_issues(
    issues_by_node: HashMap<Entity, GraphRuntimeNodeIssue>,
) -> Vec<GraphRuntimeNodeIssue> {
    let mut issues = issues_by_node.into_values().collect::<Vec<_>>();
    issues.sort_by_key(|issue| {
        (
            issue.node.index(),
            issue.definition_id.clone(),
            issue.message.clone(),
        )
    });
    issues
}
