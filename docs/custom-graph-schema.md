# Building A Custom Graph Schema

`univis_graph_core` is meant to stay generic.

It does not own:

- `NodeValue`
- `ValueType`
- Bevy components
- editor styling
- scene semantics

Those belong in an adapter crate such as `univis_node_graph`, or in your own graph package.

## What The Core Owns

The reusable kernel gives you:

- `NodeId`, `NodeCategory`, and connection policies
- generic `PortSchema` and `GraphSchema` contracts
- generic `GraphNodeDefinition<Value, Port>`
- generic `GraphNodeRegistry<Value, Port>`
- generic `GraphDocument<Value, Prefab>`
- topology helpers like `analyze_graph_topology` and `would_create_cycle`

## What You Provide

To build your own graph family, you define:

1. A runtime value type
2. A type tag for ports
3. Optional semantic requirements
4. A schema type that implements `PortSchema` and `GraphSchema`
5. One or more pure `GraphNodeDefinition`s

## Minimal Shape

For ordinary downstream code, `univis_graph_core::prelude` is still a fine
starting point. This note uses explicit module imports so the stable ownership
boundaries remain visible.

```rust
use univis_graph_core::{
    ports::PortSchema,
    schema::GraphSchema,
};

#[derive(Clone, Debug, Default, PartialEq)]
enum MyValue {
    Number(f64),
    Text(String),
    #[default]
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MyTypeTag {
    Number,
    Text,
    Any,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MyRequirement {
    token: String,
    label: String,
}

struct MySchema;

impl PortSchema for MySchema {
    type TypeTag = MyTypeTag;
    type Requirement = MyRequirement;
    type DefaultValue = MyValue;

    fn ports_compatible(from: &Self::TypeTag, to: &Self::TypeTag) -> bool {
        matches!((from, to), (_, MyTypeTag::Any) | (MyTypeTag::Any, _) | (a, b) if a == b)
    }
}

impl GraphSchema for MySchema {
    fn requirement_satisfied(
        requirement: Option<&Self::Requirement>,
        output_requirement_token: Option<&str>,
    ) -> bool {
        match requirement {
            Some(requirement) => output_requirement_token == Some(requirement.token.as_str()),
            None => true,
        }
    }

    fn requirement_label(requirement: &Self::Requirement) -> String {
        requirement.label.clone()
    }
}
```

`GraphDocument` convenience operations such as `spawn_node`, `connect`, and `node_mut` expect
your value type to implement `Default`, so it is best to make your schema value explicitly
defaultable even when `None` is just a sentinel variant.

## Stable Vs Advanced Surfaces

Most downstream integrations should start with:

- `GraphDocument::spawn_node`, `connect`, `insert_node`, and other document helpers
- `GraphNodeRegistry`
- `validate_graph_document`
- `GraphNodeDefinition` plus `ProcessContext` / `ProcessResult`

Use the narrower module-level APIs when you are building tooling or deeper diagnostics:

- `validate_graph_document_structure`
- `analyze_graph_topology`
- `ExecutableGraph`
- raw `GraphDocumentEdge` construction for persistence, migrations, whole-document tooling, or intentionally invalid fixtures

If you need to evaluate a single candidate connection outside the full-document validator,
reuse the shared connection-law helpers in `univis_graph_core::validation`:

- `GraphConnectionCandidate`
- `validate_structural_connection_candidate`
- `validate_schema_connection_candidate`

Current workspace semantics are intentionally single-source per input.
`ConnectionPolicy::Multiple` and `PortDefinition::allow_multiple_connections()` remain
future-facing declarations and are not supported end-to-end today.

## Choosing The Right Layer

Use `univis_graph_core` when you need:

- a pure graph document
- your own values and port semantics
- processing without Bevy or editor UI

Use an adapter crate like `univis_node_graph` when you need:

- Bevy ECS components
- editor-facing hooks
- styled ports
- world-space node bodies
- runtime/editor integration

## Recommended Structure

- `my_graph_core_adapter`
  - owns `MyValue`, `MyTypeTag`, `MyRequirement`, and `MySchema`
- `my_graph_nodes`
  - owns concrete node definitions
- `my_graph_runtime`
  - owns evaluation/runtime integration
- `my_graph_editor`
  - owns UI and editor workflow

## Reference Example

Recommended pure-core examples:

- [crates/univis_graph_core/examples/minimal_schema.rs](/home/abdellah/Desktop/Univis/UnivisEditor/crates/univis_graph_core/examples/minimal_schema.rs)
  Pure stable schema and node-definition setup without Bevy.
- [crates/univis_graph_core/examples/registry_and_processing.rs](/home/abdellah/Desktop/Univis/UnivisEditor/crates/univis_graph_core/examples/registry_and_processing.rs)
  Stable registry lookup, menu/search behavior, and manual processing.
- [crates/univis_graph_core/examples/document_workflows.rs](/home/abdellah/Desktop/Univis/UnivisEditor/crates/univis_graph_core/examples/document_workflows.rs)
  Stable document operations such as subgraph capture, prefab upsert, merge, and selection helpers.
- [crates/univis_graph_core/examples/validation_and_build.rs](/home/abdellah/Desktop/Univis/UnivisEditor/crates/univis_graph_core/examples/validation_and_build.rs)
  Advanced diagnostics example that deliberately injects invalid edges after the normal authored path.
- [crates/univis_graph_core/examples/execution_flow.rs](/home/abdellah/Desktop/Univis/UnivisEditor/crates/univis_graph_core/examples/execution_flow.rs)
  Advanced `ExecutableGraph` orchestration and runtime-style execution flow.
