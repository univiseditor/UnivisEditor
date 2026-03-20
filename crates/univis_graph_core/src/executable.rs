use std::any::Any;
use std::collections::{HashMap, HashSet};

use crate::document::GraphDocument;
use crate::identity::NodeId;
use crate::ports::{PortDefinition, PortSchema};
use crate::processing::{ProcessContext, ProcessResult};
use crate::registry::GraphNodeRegistry;
use crate::schema::GraphSchema;
use crate::validation::{
    GraphValidationIssue, GraphValidationIssueKind, GraphValidationReport, validate_graph_document,
};

/// Stable reference to one executable port on one node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExecutablePortRef {
    pub node_id: u64,
    pub port_index: usize,
}

impl ExecutablePortRef {
    pub fn new(node_id: u64, port_index: usize) -> Self {
        Self {
            node_id,
            port_index,
        }
    }
}

/// Direct adjacency owned by one executable node.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutableDirectLinks {
    incoming_by_input: Vec<Vec<ExecutablePortRef>>,
    outgoing_by_output: Vec<Vec<ExecutablePortRef>>,
}

impl ExecutableDirectLinks {
    pub fn new(input_count: usize, output_count: usize) -> Self {
        Self {
            incoming_by_input: vec![Vec::new(); input_count],
            outgoing_by_output: vec![Vec::new(); output_count],
        }
    }

    pub fn input_count(&self) -> usize {
        self.incoming_by_input.len()
    }

    pub fn output_count(&self) -> usize {
        self.outgoing_by_output.len()
    }

    pub fn incoming_links_for_input(&self, input_index: usize) -> Option<&[ExecutablePortRef]> {
        self.incoming_by_input.get(input_index).map(Vec::as_slice)
    }

    pub fn outgoing_links_for_output(&self, output_index: usize) -> Option<&[ExecutablePortRef]> {
        self.outgoing_by_output.get(output_index).map(Vec::as_slice)
    }

    pub fn has_any_incoming_links(&self) -> bool {
        self.incoming_by_input.iter().any(|links| !links.is_empty())
    }

    pub fn add_incoming_link(&mut self, input_index: usize, source: ExecutablePortRef) {
        if let Some(links) = self.incoming_by_input.get_mut(input_index) {
            links.push(source);
        }
    }

    pub fn add_outgoing_link(&mut self, output_index: usize, target: ExecutablePortRef) {
        if let Some(links) = self.outgoing_by_output.get_mut(output_index) {
            links.push(target);
        }
    }
}

/// Mutable execution state tracked for one node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeExecutionState {
    pub enabled: bool,
    pub dirty: bool,
    pub blocked_by_build: bool,
    pub blocked: bool,
    pub ready: bool,
    pub last_result: Option<ProcessResult>,
    pub last_run_revision: u64,
}

impl Default for NodeExecutionState {
    fn default() -> Self {
        Self {
            enabled: true,
            dirty: true,
            blocked_by_build: false,
            blocked: false,
            ready: false,
            last_result: None,
            last_run_revision: 0,
        }
    }
}

impl NodeExecutionState {
    pub fn is_runnable(&self) -> bool {
        self.enabled && !self.blocked && self.ready
    }
}

/// Execution-time resolution status for one input slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableInputResolutionState {
    Authored,
    PendingUpstream,
    Upstream,
    MissingUpstream,
}

/// Build status for one node in the executable graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableNodeBuildStatus {
    Built,
    Degraded,
    Blocked,
    Omitted,
}

/// Primary reason explaining why a node was blocked, degraded, or omitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableNodeBlockReason {
    pub kind: GraphValidationIssueKind,
    pub edge_index: Option<usize>,
    pub message: String,
}

impl From<&GraphValidationIssue> for ExecutableNodeBlockReason {
    fn from(issue: &GraphValidationIssue) -> Self {
        Self {
            kind: issue.kind,
            edge_index: issue.edge_index,
            message: issue.message.clone(),
        }
    }
}

/// Primary build diagnostic for one node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableNodeDiagnostic {
    pub node_id: u64,
    pub status: ExecutableNodeBuildStatus,
    pub reasons: Vec<ExecutableNodeBlockReason>,
}

impl ExecutableNodeDiagnostic {
    pub fn clean(node_id: u64) -> Self {
        Self {
            node_id,
            status: ExecutableNodeBuildStatus::Built,
            reasons: Vec::new(),
        }
    }

    pub fn add_reason(&mut self, reason: ExecutableNodeBlockReason) {
        self.reasons.push(reason);
    }
}

/// Structured output of a tolerant executable-graph build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableNodeRunStatus {
    Executed,
    SkippedDisabled,
    SkippedBlocked,
    MissingDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableNodeRunOutcome {
    pub node_id: u64,
    pub status: ExecutableNodeRunStatus,
    pub result: Option<ProcessResult>,
    pub outputs_changed: bool,
}

pub struct ExecutableGraphBuildReport<Value> {
    pub graph: ExecutableGraph<Value>,
    pub validation_report: GraphValidationReport,
    pub node_diagnostics: HashMap<u64, ExecutableNodeDiagnostic>,
    pub is_partial: bool,
}

impl<Value> ExecutableGraphBuildReport<Value> {
    pub fn node_diagnostic(&self, node_id: u64) -> Option<&ExecutableNodeDiagnostic> {
        self.node_diagnostics.get(&node_id)
    }
}

/// Execution-time representation of one graph node.
pub struct ExecutableNode<Value> {
    node_id: u64,
    definition_id: NodeId,
    authored_inputs: Vec<Value>,
    resolved_inputs: Vec<Value>,
    outputs: Vec<Value>,
    input_resolution: Vec<ExecutableInputResolutionState>,
    links: ExecutableDirectLinks,
    execution: NodeExecutionState,
    custom_data: Option<Box<dyn Any + Send + Sync>>,
}

impl<Value> ExecutableNode<Value> {
    pub fn from_parts(
        node_id: u64,
        definition_id: NodeId,
        authored_inputs: Vec<Value>,
        resolved_inputs: Vec<Value>,
        outputs: Vec<Value>,
    ) -> Self {
        assert_eq!(
            authored_inputs.len(),
            resolved_inputs.len(),
            "authored and resolved input buffers must have matching lengths",
        );

        let input_count = authored_inputs.len();
        let output_count = outputs.len();

        Self {
            node_id,
            definition_id,
            authored_inputs,
            resolved_inputs,
            outputs,
            input_resolution: vec![ExecutableInputResolutionState::Authored; input_count],
            links: ExecutableDirectLinks::new(input_count, output_count),
            execution: NodeExecutionState::default(),
            custom_data: None,
        }
    }

    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    pub fn definition_id(&self) -> &NodeId {
        &self.definition_id
    }

    pub fn input_count(&self) -> usize {
        self.authored_inputs.len()
    }

    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    pub fn authored_inputs(&self) -> &[Value] {
        &self.authored_inputs
    }

    pub fn resolved_inputs(&self) -> &[Value] {
        &self.resolved_inputs
    }

    pub fn outputs(&self) -> &[Value] {
        &self.outputs
    }

    pub fn input_resolution(&self) -> &[ExecutableInputResolutionState] {
        &self.input_resolution
    }

    pub fn links(&self) -> &ExecutableDirectLinks {
        &self.links
    }

    pub fn execution_state(&self) -> &NodeExecutionState {
        &self.execution
    }

    pub fn is_ready(&self) -> bool {
        self.execution.ready
    }

    pub fn is_blocked(&self) -> bool {
        self.execution.blocked
    }

    fn has_pending_upstream_inputs(&self) -> bool {
        self.input_resolution
            .iter()
            .any(|state| *state == ExecutableInputResolutionState::PendingUpstream)
    }

    fn has_missing_upstream_inputs(&self) -> bool {
        self.input_resolution
            .iter()
            .any(|state| *state == ExecutableInputResolutionState::MissingUpstream)
    }

    fn first_missing_upstream_input(&self) -> Option<usize> {
        self.input_resolution
            .iter()
            .position(|state| *state == ExecutableInputResolutionState::MissingUpstream)
    }

    fn refresh_execution_state(&mut self) {
        let blocked_by_inputs = self.has_missing_upstream_inputs();
        self.execution.blocked = self.execution.blocked_by_build || blocked_by_inputs;
        self.execution.ready = self.execution.enabled
            && self.execution.dirty
            && !self.execution.blocked
            && !self.has_pending_upstream_inputs();
    }

    fn set_build_blocked(&mut self, blocked: bool) {
        self.execution.blocked_by_build = blocked;
        self.refresh_execution_state();
    }
}

impl<Value> ExecutableNode<Value>
where
    Value: Clone,
{
    pub fn set_authored_input(&mut self, index: usize, value: Value) -> bool {
        if let Some(slot) = self.authored_inputs.get_mut(index) {
            *slot = value;
            true
        } else {
            false
        }
    }

    pub fn seed_resolved_inputs_from_authored(&mut self) {
        self.resolved_inputs.clone_from(&self.authored_inputs);
        self.input_resolution = self
            .links
            .incoming_by_input
            .iter()
            .map(|links| {
                if links.is_empty() {
                    ExecutableInputResolutionState::Authored
                } else {
                    ExecutableInputResolutionState::PendingUpstream
                }
            })
            .collect();
        self.refresh_execution_state();
    }

    pub fn resolve_input_from_upstream(&mut self, index: usize, value: Value) -> bool {
        let has_upstream_link = self
            .links
            .incoming_by_input
            .get(index)
            .is_some_and(|links| !links.is_empty());
        if !has_upstream_link {
            return false;
        }

        let Some(slot) = self.resolved_inputs.get_mut(index) else {
            return false;
        };
        let Some(state) = self.input_resolution.get_mut(index) else {
            return false;
        };

        *slot = value;
        *state = ExecutableInputResolutionState::Upstream;
        self.refresh_execution_state();
        true
    }

    pub fn mark_input_missing_upstream(&mut self, index: usize) -> bool {
        let has_upstream_link = self
            .links
            .incoming_by_input
            .get(index)
            .is_some_and(|links| !links.is_empty());
        if !has_upstream_link {
            return false;
        }

        let Some(state) = self.input_resolution.get_mut(index) else {
            return false;
        };

        *state = ExecutableInputResolutionState::MissingUpstream;
        self.refresh_execution_state();
        true
    }
}

impl<Value> ExecutableNode<Value>
where
    Value: Clone + Default,
{
    pub fn new(
        node_id: u64,
        definition_id: NodeId,
        input_count: usize,
        output_count: usize,
    ) -> Self {
        Self::from_parts(
            node_id,
            definition_id,
            vec![Value::default(); input_count],
            vec![Value::default(); input_count],
            vec![Value::default(); output_count],
        )
    }
}

/// Derived execution graph built from authored graph state.
pub struct ExecutableGraph<Value> {
    nodes: HashMap<u64, ExecutableNode<Value>>,
    revision: u64,
}

impl<Value> Default for ExecutableGraph<Value> {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            revision: 0,
        }
    }
}

impl<Value> ExecutableGraph<Value> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn contains_node(&self, node_id: u64) -> bool {
        self.nodes.contains_key(&node_id)
    }

    pub fn insert_node(&mut self, node: ExecutableNode<Value>) -> Option<ExecutableNode<Value>> {
        self.nodes.insert(node.node_id(), node)
    }

    pub fn get_node(&self, node_id: u64) -> Option<&ExecutableNode<Value>> {
        self.nodes.get(&node_id)
    }

    pub fn get_node_mut(&mut self, node_id: u64) -> Option<&mut ExecutableNode<Value>> {
        self.nodes.get_mut(&node_id)
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = &ExecutableNode<Value>> {
        self.nodes.values()
    }

    pub fn iter_nodes_mut(&mut self) -> impl Iterator<Item = &mut ExecutableNode<Value>> {
        self.nodes.values_mut()
    }

    fn bump_revision(&mut self) -> u64 {
        self.revision = self.revision.saturating_add(1);
        self.revision
    }

    fn stable_node_ids(&self) -> Vec<u64> {
        let mut ids = self.nodes.keys().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }
}

impl<Value> ExecutableGraph<Value>
where
    Value: Clone + Default + PartialEq,
{
    pub fn build<S, Prefab>(
        document: &GraphDocument<Value, Prefab>,
        registry: &GraphNodeRegistry<Value, PortDefinition<S>>,
    ) -> ExecutableGraphBuildReport<Value>
    where
        S: GraphSchema<DefaultValue = Value>,
    {
        let validation_report = validate_graph_document(document, registry);
        let blocked_node_ids = validation_report
            .topology
            .blocked_nodes
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        let invalid_edge_indexes = validation_report
            .issues
            .iter()
            .filter_map(|issue| issue.edge_index)
            .collect::<HashSet<_>>();

        let mut graph = Self::new();
        let mut node_diagnostics = HashMap::new();

        for node in &document.nodes {
            node_diagnostics
                .entry(node.id)
                .or_insert_with(|| ExecutableNodeDiagnostic::clean(node.id));
        }

        for issue in &validation_report.issues {
            for &node_id in &issue.node_ids {
                node_diagnostics
                    .entry(node_id)
                    .or_insert_with(|| ExecutableNodeDiagnostic::clean(node_id))
                    .add_reason(issue.into());
            }
        }

        for node in &document.nodes {
            if graph.contains_node(node.id) {
                continue;
            }

            let Some(definition) = registry.get(&node.definition_id) else {
                continue;
            };

            let input_definitions = definition.inputs();
            let output_definitions = definition.outputs();
            let mut authored_inputs = input_definitions
                .iter()
                .map(|port| port.default_value.clone().unwrap_or_default())
                .collect::<Vec<_>>();

            for (slot, value) in authored_inputs.iter_mut().zip(node.inputs.iter()) {
                *slot = value.clone();
            }

            let resolved_inputs = authored_inputs.clone();
            let outputs = vec![Value::default(); output_definitions.len()];

            let mut executable_node = ExecutableNode::from_parts(
                node.id,
                node.definition_id.clone(),
                authored_inputs,
                resolved_inputs,
                outputs,
            );

            if blocked_node_ids.contains(&node.id) {
                executable_node.set_build_blocked(true);
            }

            graph.insert_node(executable_node);
        }

        for (edge_index, edge) in document.edges.iter().enumerate() {
            if invalid_edge_indexes.contains(&edge_index) {
                continue;
            }

            let Some(from_node) = graph.get_node(edge.from_node_id) else {
                continue;
            };
            let Some(to_node) = graph.get_node(edge.to_node_id) else {
                continue;
            };

            if edge.from_index >= from_node.output_count() || edge.to_index >= to_node.input_count()
            {
                continue;
            }

            let source = ExecutablePortRef::new(edge.from_node_id, edge.from_index);
            let target = ExecutablePortRef::new(edge.to_node_id, edge.to_index);

            if let Some(from_node) = graph.get_node_mut(edge.from_node_id) {
                from_node.links.add_outgoing_link(edge.from_index, target);
            }
            if let Some(to_node) = graph.get_node_mut(edge.to_node_id) {
                to_node.links.add_incoming_link(edge.to_index, source);
            }
        }

        for node in graph.iter_nodes_mut() {
            node.seed_resolved_inputs_from_authored();
        }

        for (node_id, diagnostic) in &mut node_diagnostics {
            let was_built = graph.contains_node(*node_id);
            let has_cycle_reason = diagnostic
                .reasons
                .iter()
                .any(|reason| reason.kind == GraphValidationIssueKind::CycleDetected);

            diagnostic.status = if !was_built {
                ExecutableNodeBuildStatus::Omitted
            } else if has_cycle_reason {
                ExecutableNodeBuildStatus::Blocked
            } else if diagnostic.reasons.is_empty() {
                ExecutableNodeBuildStatus::Built
            } else {
                ExecutableNodeBuildStatus::Degraded
            };

            if diagnostic.status == ExecutableNodeBuildStatus::Blocked {
                if let Some(node) = graph.get_node_mut(*node_id) {
                    node.set_build_blocked(true);
                }
            }
        }

        let is_partial = node_diagnostics
            .values()
            .any(|diagnostic| diagnostic.status != ExecutableNodeBuildStatus::Built);

        ExecutableGraphBuildReport {
            graph,
            validation_report,
            node_diagnostics,
            is_partial,
        }
    }

    pub fn enable_node(&mut self, node_id: u64) -> bool {
        let Some(node) = self.get_node_mut(node_id) else {
            return false;
        };
        node.execution.enabled = true;
        node.refresh_execution_state();
        self.bump_revision();
        true
    }

    pub fn disable_node(&mut self, node_id: u64) -> bool {
        let Some(node) = self.get_node_mut(node_id) else {
            return false;
        };
        node.execution.enabled = false;
        node.refresh_execution_state();
        self.bump_revision();
        true
    }

    pub fn mark_dirty(&mut self, node_id: u64) -> bool {
        let Some(node) = self.get_node_mut(node_id) else {
            return false;
        };
        node.execution.dirty = true;
        node.refresh_execution_state();
        self.bump_revision();
        true
    }

    pub fn set_authored_input(&mut self, node_id: u64, index: usize, value: Value) -> bool {
        let Some(node) = self.get_node_mut(node_id) else {
            return false;
        };
        if !node.set_authored_input(index, value) {
            return false;
        }
        node.execution.dirty = true;
        node.seed_resolved_inputs_from_authored();
        self.bump_revision();
        true
    }

    pub fn resolve_inputs(&mut self, node_id: u64) -> bool {
        let Some(node) = self.get_node(node_id) else {
            return false;
        };

        let authored_inputs = node.authored_inputs.clone();
        let incoming = (0..node.input_count())
            .map(|index| {
                node.links()
                    .incoming_links_for_input(index)
                    .map(|links| links.to_vec())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();

        let mut resolved_inputs = authored_inputs.clone();
        let mut resolution = Vec::with_capacity(incoming.len());

        for (input_index, links) in incoming.iter().enumerate() {
            if links.is_empty() {
                resolution.push(ExecutableInputResolutionState::Authored);
                continue;
            }

            let source = links[0];
            let value = self
                .get_node(source.node_id)
                .and_then(|source_node| source_node.outputs().get(source.port_index))
                .cloned();

            if let Some(value) = value {
                if let Some(slot) = resolved_inputs.get_mut(input_index) {
                    *slot = value;
                }
                resolution.push(ExecutableInputResolutionState::Upstream);
            } else {
                resolution.push(ExecutableInputResolutionState::MissingUpstream);
            }
        }

        let Some(node) = self.get_node_mut(node_id) else {
            return false;
        };
        node.resolved_inputs = resolved_inputs;
        node.input_resolution = resolution;
        node.refresh_execution_state();
        true
    }

    pub fn run_node<S>(
        &mut self,
        registry: &GraphNodeRegistry<Value, PortDefinition<S>>,
        node_id: u64,
        delta_time: f32,
    ) -> Option<ExecutableNodeRunOutcome>
    where
        S: PortSchema,
    {
        self.resolve_inputs(node_id);

        let definition_id = self.get_node(node_id)?.definition_id().clone();
        let definition = registry.get(&definition_id);

        if definition.is_none() {
            let result = ProcessResult::Error("Missing node definition in registry.".to_string());
            let revision = self.bump_revision();
            let node = self.get_node_mut(node_id)?;
            node.set_build_blocked(true);
            node.execution.last_result = Some(result.clone());
            node.execution.last_run_revision = revision;
            return Some(ExecutableNodeRunOutcome {
                node_id,
                status: ExecutableNodeRunStatus::MissingDefinition,
                result: Some(result),
                outputs_changed: false,
            });
        }

        {
            let node = self.get_node(node_id)?;
            if !node.execution_state().enabled {
                return Some(ExecutableNodeRunOutcome {
                    node_id,
                    status: ExecutableNodeRunStatus::SkippedDisabled,
                    result: None,
                    outputs_changed: false,
                });
            }

            if node.is_blocked() {
                let result = node
                    .first_missing_upstream_input()
                    .map(ProcessResult::MissingInput)
                    .or_else(|| node.execution_state().last_result.clone());
                return Some(ExecutableNodeRunOutcome {
                    node_id,
                    status: ExecutableNodeRunStatus::SkippedBlocked,
                    result,
                    outputs_changed: false,
                });
            }
        }

        let definition = definition?;
        let (result, outputs_changed) = {
            let node = self.get_node_mut(node_id)?;
            let previous_outputs = node.outputs.clone();
            let mut next_outputs = previous_outputs.clone();

            let result = {
                let mut context = ProcessContext {
                    inputs: &node.resolved_inputs,
                    outputs: &mut next_outputs,
                    delta_time,
                    custom_data: &mut node.custom_data,
                };
                definition.process(&mut context)
            };

            let outputs_changed = next_outputs != previous_outputs;
            node.outputs = next_outputs;
            node.execution.dirty = false;
            node.execution.last_result = Some(result.clone());

            (result, outputs_changed)
        };

        let revision = self.bump_revision();
        let node = self.get_node_mut(node_id)?;
        node.execution.last_run_revision = revision;
        node.refresh_execution_state();

        Some(ExecutableNodeRunOutcome {
            node_id,
            status: ExecutableNodeRunStatus::Executed,
            result: Some(result),
            outputs_changed,
        })
    }

    pub fn run_ready_nodes<S>(
        &mut self,
        registry: &GraphNodeRegistry<Value, PortDefinition<S>>,
        delta_time: f32,
    ) -> Vec<ExecutableNodeRunOutcome>
    where
        S: PortSchema,
    {
        let ready_node_ids = self
            .stable_node_ids()
            .into_iter()
            .filter(|node_id| {
                self.get_node(*node_id)
                    .is_some_and(ExecutableNode::is_ready)
            })
            .collect::<Vec<_>>();

        let mut outcomes = Vec::new();
        for node_id in ready_node_ids {
            if let Some(outcome) = self.run_node(registry, node_id, delta_time) {
                outcomes.push(outcome);
            }
        }
        outcomes
    }

    pub fn run_from<S>(
        &mut self,
        registry: &GraphNodeRegistry<Value, PortDefinition<S>>,
        node_id: u64,
        delta_time: f32,
    ) -> Vec<ExecutableNodeRunOutcome>
    where
        S: PortSchema,
    {
        let mut visited = HashSet::new();
        let mut queue = vec![node_id];
        let mut ordered = Vec::new();

        while let Some(current) = queue.pop() {
            if !visited.insert(current) {
                continue;
            }
            ordered.push(current);
            if let Some(downstream) = self.get_downstream(current) {
                for target in downstream.into_iter().rev() {
                    queue.push(target);
                }
            }
        }

        let mut outcomes = Vec::new();
        for current in ordered {
            self.mark_dirty(current);
            if let Some(outcome) = self.run_node(registry, current, delta_time) {
                outcomes.push(outcome);
            }
        }
        outcomes
    }

    pub fn get_outputs(&self, node_id: u64) -> Option<&[Value]> {
        self.get_node(node_id).map(ExecutableNode::outputs)
    }

    pub fn get_upstream(&self, node_id: u64) -> Option<Vec<u64>> {
        let node = self.get_node(node_id)?;
        let mut upstream = HashSet::new();
        for input_index in 0..node.input_count() {
            if let Some(links) = node.links().incoming_links_for_input(input_index) {
                for link in links {
                    upstream.insert(link.node_id);
                }
            }
        }
        let mut upstream = upstream.into_iter().collect::<Vec<_>>();
        upstream.sort_unstable();
        Some(upstream)
    }

    pub fn get_downstream(&self, node_id: u64) -> Option<Vec<u64>> {
        let node = self.get_node(node_id)?;
        let mut downstream = HashSet::new();
        for output_index in 0..node.output_count() {
            if let Some(links) = node.links().outgoing_links_for_output(output_index) {
                for link in links {
                    downstream.insert(link.node_id);
                }
            }
        }
        let mut downstream = downstream.into_iter().collect::<Vec<_>>();
        downstream.sort_unstable();
        Some(downstream)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ExecutableGraph, ExecutableInputResolutionState, ExecutableNode, ExecutablePortRef,
        NodeExecutionState,
    };
    use crate::identity::NodeId;
    use crate::ports::{PortDefinition, PortSchema};
    use crate::processing::{GraphNodeDefinition, ProcessContext, ProcessResult};
    use crate::registry::GraphNodeRegistry;

    #[derive(Clone)]
    struct TestSchema;

    impl PortSchema for TestSchema {
        type TypeTag = ();
        type Requirement = ();
        type DefaultValue = i32;

        fn ports_compatible(_from: &Self::TypeTag, _to: &Self::TypeTag) -> bool {
            true
        }
    }

    struct EchoNode;

    impl GraphNodeDefinition<i32, PortDefinition<TestSchema>> for EchoNode {
        fn id(&self) -> NodeId {
            NodeId::new("tests/echo")
        }

        fn display_name(&self) -> &str {
            "Echo"
        }

        fn inputs(&self) -> Vec<PortDefinition<TestSchema>> {
            vec![PortDefinition::new("Value", ()).with_default(0)]
        }

        fn outputs(&self) -> Vec<PortDefinition<TestSchema>> {
            vec![PortDefinition::new("Value", ())]
        }

        fn process(&self, context: &mut ProcessContext<'_, i32>) -> ProcessResult {
            let value = context.get(0).copied().unwrap_or_default();
            context.set(0, value + 1);
            ProcessResult::Success
        }
    }

    #[test]
    fn executable_node_separates_authored_resolved_and_outputs() {
        let node = ExecutableNode::<i32>::new(10, NodeId::new("tests/node"), 2, 1);

        assert_eq!(node.input_count(), 2);
        assert_eq!(node.output_count(), 1);
        assert_eq!(node.authored_inputs(), &[0, 0]);
        assert_eq!(node.resolved_inputs(), &[0, 0]);
        assert_eq!(node.outputs(), &[0]);
        assert_eq!(node.links().input_count(), 2);
        assert_eq!(node.links().output_count(), 1);
    }

    #[test]
    fn executable_graph_stores_nodes_by_id() {
        let mut graph = ExecutableGraph::<i32>::new();
        let node = ExecutableNode::<i32>::new(7, NodeId::new("tests/node"), 1, 1);

        graph.insert_node(node);

        assert!(graph.contains_node(7));
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn execution_state_defaults_to_enabled_dirty_and_not_ready() {
        let state = NodeExecutionState::default();

        assert!(state.enabled);
        assert!(state.dirty);
        assert!(!state.blocked_by_build);
        assert!(!state.blocked);
        assert!(!state.ready);
        assert!(!state.is_runnable());
    }

    #[test]
    fn direct_links_store_incoming_and_outgoing_refs() {
        let mut node = ExecutableNode::<i32>::new(1, NodeId::new("tests/node"), 2, 1);
        let source = ExecutablePortRef::new(2, 0);
        let target = ExecutablePortRef::new(3, 1);

        node.links.add_incoming_link(0, source);
        node.links.add_outgoing_link(0, target);

        assert_eq!(
            node.links().incoming_links_for_input(0),
            Some(&[source][..])
        );
        assert_eq!(
            node.links().outgoing_links_for_output(0),
            Some(&[target][..])
        );
    }

    #[test]
    fn seeding_resolved_inputs_keeps_authored_values_and_marks_pending_links() {
        let mut node = ExecutableNode::<i32>::from_parts(
            1,
            NodeId::new("tests/node"),
            vec![10, 20],
            vec![0, 0],
            vec![0],
        );

        node.links
            .add_incoming_link(0, ExecutablePortRef::new(2, 0));
        node.seed_resolved_inputs_from_authored();

        assert_eq!(node.authored_inputs(), &[10, 20]);
        assert_eq!(node.resolved_inputs(), &[10, 20]);
        assert_eq!(
            node.input_resolution(),
            &[
                ExecutableInputResolutionState::PendingUpstream,
                ExecutableInputResolutionState::Authored,
            ]
        );
        assert!(!node.is_blocked());
        assert!(!node.is_ready());
    }

    #[test]
    fn resolving_upstream_input_overrides_only_resolved_buffer() {
        let mut node = ExecutableNode::<i32>::from_parts(
            1,
            NodeId::new("tests/node"),
            vec![10],
            vec![0],
            vec![0],
        );

        node.links
            .add_incoming_link(0, ExecutablePortRef::new(2, 0));
        node.seed_resolved_inputs_from_authored();
        assert!(node.resolve_input_from_upstream(0, 99));

        assert_eq!(node.authored_inputs(), &[10]);
        assert_eq!(node.resolved_inputs(), &[99]);
        assert_eq!(
            node.input_resolution(),
            &[ExecutableInputResolutionState::Upstream]
        );
        assert!(node.is_ready());
        assert!(!node.is_blocked());
    }

    #[test]
    fn missing_upstream_marks_node_blocked() {
        let mut node = ExecutableNode::<i32>::from_parts(
            1,
            NodeId::new("tests/node"),
            vec![10],
            vec![0],
            vec![0],
        );

        node.links
            .add_incoming_link(0, ExecutablePortRef::new(2, 0));
        node.seed_resolved_inputs_from_authored();
        assert!(node.mark_input_missing_upstream(0));

        assert_eq!(
            node.input_resolution(),
            &[ExecutableInputResolutionState::MissingUpstream]
        );
        assert!(node.is_blocked());
        assert!(!node.is_ready());
    }

    #[test]
    fn changing_authored_input_does_not_mutate_resolved_until_reseed() {
        let mut node = ExecutableNode::<i32>::from_parts(
            1,
            NodeId::new("tests/node"),
            vec![1],
            vec![1],
            vec![0],
        );

        node.seed_resolved_inputs_from_authored();
        assert!(node.set_authored_input(0, 7));

        assert_eq!(node.authored_inputs(), &[7]);
        assert_eq!(node.resolved_inputs(), &[1]);

        node.seed_resolved_inputs_from_authored();
        assert_eq!(node.resolved_inputs(), &[7]);
    }

    #[test]
    fn run_node_resolves_upstream_input_and_updates_outputs() {
        let mut registry = GraphNodeRegistry::<i32, PortDefinition<TestSchema>>::new();
        registry.register(EchoNode);

        let mut graph = ExecutableGraph::<i32>::new();

        let mut source =
            ExecutableNode::from_parts(1, NodeId::new("tests/source"), vec![], vec![], vec![5]);
        source.execution.dirty = false;
        source.execution.ready = false;

        let mut target =
            ExecutableNode::from_parts(2, NodeId::new("tests/echo"), vec![1], vec![1], vec![0]);
        target
            .links
            .add_incoming_link(0, ExecutablePortRef::new(1, 0));
        source
            .links
            .add_outgoing_link(0, ExecutablePortRef::new(2, 0));
        target.seed_resolved_inputs_from_authored();

        graph.insert_node(source);
        graph.insert_node(target);

        let outcome = graph.run_node(&registry, 2, 0.016).unwrap();

        assert!(matches!(outcome.result, Some(ProcessResult::Success)));
        assert!(outcome.outputs_changed);
        assert_eq!(graph.get_node(2).unwrap().resolved_inputs(), &[5]);
        assert_eq!(graph.get_outputs(2), Some(&[6][..]));
        assert!(!graph.get_node(2).unwrap().execution_state().dirty);
    }
}
