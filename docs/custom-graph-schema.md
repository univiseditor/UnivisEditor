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

```rust
use univis_graph_core::prelude::*;

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

For a complete pure-core example without Bevy, see:

- [crates/univis_graph_core/examples/minimal_schema.rs](/home/abdellah/Desktop/Univis/UnivisEditor/crates/univis_graph_core/examples/minimal_schema.rs)
