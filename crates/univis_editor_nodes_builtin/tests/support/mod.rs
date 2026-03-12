use univis_node_graph::node_definition::{NodeDefinition, ProcessContext, ProcessResult};
use univis_node_graph::value::NodeValue;

pub fn run_node(
    node: &dyn NodeDefinition,
    inputs: Vec<NodeValue>,
) -> (ProcessResult, Vec<NodeValue>) {
    let mut outputs = vec![NodeValue::None; node.outputs().len()];
    let mut custom_data = None;
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };

    let result = node.process(&mut ctx);
    (result, outputs)
}
