#[path = "support/mod.rs"]
mod support;

use support::{build_demo_registry, DemoValue};
use univis_graph_core::{
    document::{GraphDocument, GraphDocumentEdge},
    executable::ExecutableGraph,
    identity::NodeId,
    topology::analyze_graph_topology,
    validation::{validate_graph_document, validate_graph_document_structure},
};

// Normal authoring should prefer GraphDocument::connect(...). This helper
// exists only so the example can inject intentionally invalid edges and show
// how structure, validation, topology, and executable-build diagnostics react.
fn push_raw_edge<Value, Prefab>(
    document: &mut GraphDocument<Value, Prefab>,
    from_node_id: u64,
    from_index: usize,
    to_node_id: u64,
    to_index: usize,
) {
    document.edges.push(GraphDocumentEdge {
        from_node_id,
        from_index,
        to_node_id,
        to_index,
    });
}

fn main() {
    let registry = build_demo_registry();
    let mut document = GraphDocument::<DemoValue, ()>::default();

    let input_number = document.spawn_node(NodeId::new("demo/input_number"), [0.0, 0.0], 1, 1);
    let add = document.spawn_node(NodeId::new("demo/add"), [220.0, 0.0], 2, 1);
    let format = document.spawn_node(NodeId::new("demo/format"), [440.0, 0.0], 1, 1);
    let input_text = document.spawn_node(NodeId::new("demo/input_text"), [0.0, 160.0], 1, 1);
    let missing = document.spawn_node(NodeId::new("demo/missing"), [220.0, 160.0], 0, 1);

    document
        .node_mut(input_number)
        .expect("number input should exist")
        .inputs[0] = DemoValue::number(12.0);
    document
        .node_mut(input_text)
        .expect("text input should exist")
        .inputs[0] = DemoValue::text("oops");

    document
        .connect(input_number, 0, add, 0)
        .expect("number -> add should be a legal authored edge");
    document
        .connect(input_text, 0, add, 1)
        .expect("text -> add is structurally legal even though schema validation will flag it");
    document
        .connect(add, 0, format, 0)
        .expect("add -> format should be a legal authored edge");

    push_raw_edge(&mut document, add, 0, input_number, 0);
    push_raw_edge(&mut document, missing, 0, add, 1);

    let structure_report = validate_graph_document_structure(&document, &registry);
    let validation_report = validate_graph_document(&document, &registry);
    let topology = analyze_graph_topology(
        document.nodes.iter().map(|node| node.id),
        document
            .edges
            .iter()
            .map(|edge| (edge.from_node_id, edge.to_node_id)),
    );

    println!("structure issues: {}", structure_report.issue_count());
    println!(
        "validation issues: {} blocked={:?} ordered={:?}",
        validation_report.issue_count(),
        validation_report.topology.blocked_nodes,
        validation_report.topology.ordered_nodes
    );
    println!(
        "direct topology analysis: ordered={:?} blocked={:?}",
        topology.ordered_nodes, topology.blocked_nodes
    );

    for issue in &validation_report.issues {
        println!("  - {:?}: {}", issue.kind, issue.message);
    }

    let build = ExecutableGraph::build(&document, &registry);
    println!(
        "partial={} build_ready={} can_execute={} blocked_nodes={:?}",
        build.is_partial(),
        build.graph().is_build_ready(),
        build.graph().can_execute(),
        build.graph().blocked_node_ids()
    );
    println!(
        "execution order inside executable graph: {:?}",
        build.graph().execution_order()
    );

    let mut node_ids = build.node_diagnostics().keys().copied().collect::<Vec<_>>();
    node_ids.sort_unstable();
    for node_id in node_ids {
        let diagnostic = build
            .node_diagnostic(node_id)
            .expect("node diagnostic should exist");
        println!(
            "node {node_id}: {:?} reasons={}",
            diagnostic.status,
            diagnostic.reasons.len()
        );
        for reason in &diagnostic.reasons {
            println!("    * {:?}: {}", reason.kind, reason.message);
        }
    }
}
