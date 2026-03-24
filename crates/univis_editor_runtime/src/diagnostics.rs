use std::collections::HashMap;

use bevy::prelude::*;
use univis_graph_core::prelude::{ExecutableNodeBuildStatus, ExecutableNodeRunStatus};
use univis_node_graph::prelude::{AuthoredNodeInputs, GraphNode, NodeRegistry, ProcessResult};

use crate::connectivity::{GraphExecutableRuntimeState, NodeInputSignature, NodeOutputSignature};

#[derive(Resource, Debug, Clone, Default)]
pub struct GraphRuntimeDiagnostics {
    /// Presentation-oriented runtime issues derived from executable truth.
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
    mut executable_state: ResMut<GraphExecutableRuntimeState>,
    mut diagnostics: ResMut<GraphRuntimeDiagnostics>,
    trace_settings: Res<GraphRuntimeTraceSettings>,
    mut runtime_trace: ResMut<GraphRuntimeTrace>,
    mut q_nodes: Query<(
        Entity,
        &mut GraphNode,
        &AuthoredNodeInputs,
        &mut NodeInputSignature,
        &mut NodeOutputSignature,
    )>,
    time: Res<Time>,
) {
    let executable_graph_changed = executable_state.is_changed();
    let node_count_hint = executable_state.entity_to_node_id.len().max(1);
    let mut dirty_reasons = HashMap::<Entity, Vec<String>>::with_capacity(node_count_hint);
    let mut definition_ids = HashMap::<Entity, String>::with_capacity(node_count_hint);

    if trace_settings.enabled {
        runtime_trace.entries.clear();
    }

    for (entity, mut node, authored_inputs, mut input_signature, output_signature) in
        q_nodes.iter_mut()
    {
        definition_ids.insert(entity, node.definition_id.to_string());

        let Some(node_id) = executable_state.node_id_for_entity(entity) else {
            input_signature.inputs.clone_from(&authored_inputs.values);
            continue;
        };

        if node.custom_data.is_some() {
            let custom_data = node.custom_data.take();
            let _ = executable_state
                .graph
                .replace_custom_data(node_id, custom_data);
        }

        if input_signature.inputs != authored_inputs.values
            && executable_state
                .graph
                .replace_authored_inputs(node_id, &authored_inputs.values)
        {
            dirty_reasons
                .entry(entity)
                .or_default()
                .push("authored inputs changed".to_string());
        }

        if output_signature.outputs != node.output_projection_values()
            && executable_state
                .graph
                .sync_external_outputs(node_id, node.output_projection_values())
        {
            dirty_reasons
                .entry(entity)
                .or_default()
                .push("visual or external output changed".to_string());
        }

        if executable_graph_changed
            && executable_state
                .graph
                .get_node(node_id)
                .is_some_and(|node| node.is_dirty())
        {
            dirty_reasons
                .entry(entity)
                .or_default()
                .push("graph structure changed".to_string());
        }

        input_signature.inputs.clone_from(&authored_inputs.values);
    }

    let outcomes = executable_state
        .graph
        .run_ready_nodes(registry.core_registry(), time.delta_secs());

    let mut issues_by_node = HashMap::<Entity, GraphRuntimeNodeIssue>::new();
    for (node_id, diagnostic) in executable_state.graph.node_diagnostics() {
        let Some(entity) = executable_state.entity_for_node_id(*node_id) else {
            continue;
        };
        let Some(message) = diagnostic
            .reasons
            .first()
            .map(|reason| reason.message.clone())
        else {
            continue;
        };
        let Some(definition_id) = executable_state
            .graph
            .get_node(*node_id)
            .map(|node| node.definition_id().to_string())
            .or_else(|| definition_ids.get(&entity).cloned())
        else {
            continue;
        };

        let severity = match diagnostic.status {
            ExecutableNodeBuildStatus::Built => continue,
            ExecutableNodeBuildStatus::Degraded => GraphRuntimeIssueSeverity::Warning,
            ExecutableNodeBuildStatus::Blocked | ExecutableNodeBuildStatus::Omitted => {
                GraphRuntimeIssueSeverity::Error
            }
        };

        issues_by_node.insert(
            entity,
            GraphRuntimeNodeIssue {
                node: entity,
                definition_id,
                severity,
                message,
            },
        );
    }

    for outcome in outcomes {
        let Some(entity) = executable_state.entity_for_node_id(outcome.node_id) else {
            continue;
        };
        let definition_id = executable_state
            .graph
            .get_node(outcome.node_id)
            .map(|node| node.definition_id().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let result_summary = summarize_result(outcome.result.as_ref());

        if trace_settings.enabled {
            runtime_trace.entries.push(GraphRuntimeTraceEntry {
                node: entity,
                definition_id: definition_id.clone(),
                reasons: dirty_reasons
                    .remove(&entity)
                    .filter(|reasons| !reasons.is_empty())
                    .unwrap_or_else(|| vec!["node marked dirty".to_string()]),
                result: result_summary,
                outputs_changed: outcome.outputs_changed,
            });
        }

        match (outcome.status, outcome.result) {
            (ExecutableNodeRunStatus::Executed, Some(ProcessResult::Success)) => {}
            (ExecutableNodeRunStatus::Executed, Some(ProcessResult::Error(message))) => {
                issues_by_node.insert(
                    entity,
                    GraphRuntimeNodeIssue {
                        node: entity,
                        definition_id,
                        severity: GraphRuntimeIssueSeverity::Error,
                        message: format!("Processing error: {message}"),
                    },
                );
            }
            (ExecutableNodeRunStatus::Executed, Some(ProcessResult::MissingInput(index)))
            | (ExecutableNodeRunStatus::SkippedBlocked, Some(ProcessResult::MissingInput(index))) =>
            {
                issues_by_node.insert(
                    entity,
                    GraphRuntimeNodeIssue {
                        node: entity,
                        definition_id,
                        severity: GraphRuntimeIssueSeverity::Warning,
                        message: format!("Missing input: {index}"),
                    },
                );
            }
            (ExecutableNodeRunStatus::MissingDefinition, Some(ProcessResult::Error(message))) => {
                issues_by_node.insert(
                    entity,
                    GraphRuntimeNodeIssue {
                        node: entity,
                        definition_id,
                        severity: GraphRuntimeIssueSeverity::Error,
                        message,
                    },
                );
            }
            _ => {}
        }
    }

    for (entity, mut node, _authored_inputs, _input_signature, mut output_signature) in
        q_nodes.iter_mut()
    {
        let Some(node_id) = executable_state.node_id_for_entity(entity) else {
            output_signature.outputs = node.output_projection_values().to_vec();
            continue;
        };

        let Some(executable_node) = executable_state.graph.get_node(node_id) else {
            output_signature.outputs = node.output_projection_values().to_vec();
            continue;
        };

        node.replace_input_projection(executable_node.resolved_inputs());
        node.replace_output_projection(executable_node.outputs());
        output_signature.outputs = node.output_projection_values().to_vec();
    }

    diagnostics.node_issues = sorted_issues(issues_by_node);

    let blocked_entities = executable_state.blocked_entities();
    if !blocked_entities.is_empty() && executable_graph_changed {
        warn!(
            "Graph runtime skipped {} node(s) because the graph contains a cycle or blocked dependency path.",
            blocked_entities.len()
        );
    }

    if trace_settings.enabled && runtime_trace.entries.len() > trace_settings.max_entries {
        let keep_from = runtime_trace.entries.len() - trace_settings.max_entries;
        runtime_trace.entries.drain(0..keep_from);
    }
}

fn summarize_result(result: Option<&ProcessResult>) -> String {
    match result {
        Some(ProcessResult::Success) => "success".to_string(),
        Some(ProcessResult::Error(message)) => format!("error: {message}"),
        Some(ProcessResult::MissingInput(index)) => format!("missing input: {index}"),
        None => "skipped".to_string(),
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
