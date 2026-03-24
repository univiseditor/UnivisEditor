#[path = "support/mod.rs"]
mod support;

use support::{build_demo_registry, describe_values, DemoValue};
use univis_graph_core::{
    document::GraphDocument,
    executable::{ExecutableGraph, ExecutableNodeRunOutcome},
    identity::NodeId,
};

fn print_outcomes(label: &str, outcomes: &[ExecutableNodeRunOutcome]) {
    println!("{label}:");
    for outcome in outcomes {
        println!(
            "  node {} => {:?}, changed={}, result={:?}",
            outcome.node_id, outcome.status, outcome.outputs_changed, outcome.result
        );
    }
}

fn main() {
    let registry = build_demo_registry();
    let mut document = GraphDocument::<DemoValue, ()>::default();

    let left = document.spawn_node(NodeId::new("demo/input_number"), [0.0, 0.0], 1, 1);
    let right = document.spawn_node(NodeId::new("demo/input_number"), [0.0, 120.0], 1, 1);
    let add = document.spawn_node(NodeId::new("demo/add"), [240.0, 60.0], 2, 1);
    let publish = document.spawn_node(NodeId::new("demo/publish_number"), [460.0, 60.0], 1, 1);
    let format = document.spawn_node(NodeId::new("demo/format"), [700.0, 60.0], 1, 1);
    let counter = document.spawn_node(NodeId::new("demo/counter"), [240.0, 220.0], 1, 1);

    document.node_mut(left).expect("left node").inputs[0] = DemoValue::number(2.0);
    document.node_mut(right).expect("right node").inputs[0] = DemoValue::number(5.0);
    document.node_mut(counter).expect("counter node").inputs[0] = DemoValue::number(1.0);

    document.connect(left, 0, add, 0).expect("left -> add");
    document.connect(right, 0, add, 1).expect("right -> add");
    document
        .connect(add, 0, publish, 0)
        .expect("add -> publish");
    document
        .connect(publish, 0, format, 0)
        .expect("publish -> format");
    document
        .connect(left, 0, counter, 0)
        .expect("left -> counter");

    let mut graph = ExecutableGraph::build(&document, &registry).into_graph();
    println!(
        "node_count={} revision={} execution_order={:?}",
        graph.node_count(),
        graph.revision(),
        graph.execution_order()
    );
    println!(
        "upstream(format)={:?} downstream(left)={:?}",
        graph.get_upstream(format),
        graph.get_downstream(left)
    );

    graph.replace_authored_inputs(right, &[DemoValue::number(8.0)]);
    graph.replace_custom_data(counter, Some(Box::new(40.0_f64)));

    let first_pass = graph.run_ready_nodes(&registry, 0.016);
    print_outcomes("initial run_ready_nodes", &first_pass);
    println!(
        "publish input resolution: {:?}",
        graph
            .get_node(publish)
            .expect("publish node")
            .input_resolution()
    );
    println!(
        "format outputs after first pass: {:?}",
        describe_values(graph.get_outputs(format).expect("format outputs"))
    );
    println!(
        "counter outputs after first pass: {:?} custom_data={}",
        describe_values(graph.get_outputs(counter).expect("counter outputs")),
        graph
            .get_node(counter)
            .expect("counter node")
            .has_custom_data()
    );

    graph.set_authored_input(counter, 0, DemoValue::number(2.0));
    let counter_outcome = graph
        .run_node(&registry, counter, 0.016)
        .expect("counter should run directly");
    println!(
        "direct counter run: {:?} => {:?}",
        counter_outcome.status,
        describe_values(graph.get_outputs(counter).expect("counter outputs"))
    );

    graph.disable_node(publish);
    graph.sync_external_outputs(left, &[DemoValue::number(20.0)]);
    let disabled_pass = graph.run_from(&registry, add, 0.016);
    print_outcomes("run_from(add) with publish disabled", &disabled_pass);

    graph.enable_node(publish);
    graph.mark_dirty(publish);
    let resumed_pass = graph.run_from(&registry, publish, 0.016);
    print_outcomes("run_from(publish) after re-enabling", &resumed_pass);
    println!(
        "final format outputs: {:?}",
        describe_values(graph.get_outputs(format).expect("format outputs"))
    );
    println!("final revision={}", graph.revision());
}
