#[path = "support/mod.rs"]
mod support;

use support::{DemoValue, build_demo_registry, describe_values};
use univis_graph_core::prelude::{NodeId, ProcessContext};

fn main() {
    let registry = build_demo_registry();

    let mut categories = registry.get_categories().cloned().collect::<Vec<_>>();
    categories.sort();

    let menu_nodes = registry
        .get_menu_nodes_sorted()
        .into_iter()
        .map(|definition| {
            format!(
                "{} [{}] children={}",
                definition.display_name(),
                definition.category().as_str(),
                definition.can_have_children()
            )
        })
        .collect::<Vec<_>>();

    let hidden_matches = registry
        .search("counter")
        .into_iter()
        .map(|definition| definition.display_name().to_string())
        .collect::<Vec<_>>();

    let math_nodes = registry
        .get_by_category("Math")
        .into_iter()
        .map(|definition| definition.display_name().to_string())
        .collect::<Vec<_>>();

    println!("registered ids: {:?}", registry.get_all_ids());
    println!("categories: {categories:?}");
    println!("menu nodes: {menu_nodes:?}");
    println!("search(\"counter\"): {hidden_matches:?}");
    println!("math nodes: {math_nodes:?}");

    let add = registry
        .get(&NodeId::new("demo/add"))
        .expect("demo/add should exist");
    let publish = registry
        .get(&NodeId::new("demo/publish_number"))
        .expect("demo/publish_number should exist");

    let mut outputs = vec![DemoValue::default()];
    let mut custom_data = None;
    let inputs = [DemoValue::number(4.0), DemoValue::number(6.5)];
    let result = {
        let mut context = ProcessContext {
            inputs: &inputs,
            outputs: &mut outputs,
            delta_time: 0.016,
            custom_data: &mut custom_data,
        };
        add.process(&mut context)
    };

    println!("manual add result: {result:?}");
    println!("manual add outputs: {:?}", describe_values(&outputs));
    println!(
        "publish_number requirement token: {:?}",
        publish.output_requirement_token(0, &[true])
    );
}
