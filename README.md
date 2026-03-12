# UnivisEditor

<div align="center">

**A node-based visual editor for the Bevy Engine**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/Bevy-0.18.0-blue.svg)](https://bevyengine.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

## 📖 Overview

UnivisEditor is a **node-based visual editor** built on top of the [Bevy Engine](https://bevyengine.org/). It follows the philosophy that **"everything is a node"** - similar to Unreal Engine's Blueprint system or Blender's Node Editor.

The codebase is now organized as a Cargo workspace:
- `univis_node_graph`
- `univis_scene`
- `univis_editor_runtime`
- `univis_editor_ui`
- `univis_editor_persistence`
- `univis_editor_nodes_builtin`
- `univis_editor_app` (facade + prelude)

### Key Features

- 🎨 **Visual Node Editor** - Create complex logic by connecting nodes visually
- 🧭 **Canvas-Only Workflow** - The editor surface is dedicated to node graphs, with transient floating tools instead of fixed side panels
- 🔌 **Extensible Node System** - Easy to create custom nodes with a clean trait-based API
- 🚀 **Built on Bevy 0.18** - Leverages the latest Bevy ECS architecture
- 📦 **World-Space UI** - Custom UI system (`univis_ui`) for 3D world-space interactions
- 🔍 **Search & Filter** - Quickly find nodes with built-in search functionality
- 📜 **Auto-scrolling Menu** - Context menu with scrolling support for large node libraries

---

## 🚀 Getting Started

The active editor path is graph-first and canvas-only. Persistent sidebars and mode-driven scene tooling are intentionally out of the shipped app; contextual UI should appear as compact floating surfaces over the canvas.

### Prerequisites

- Rust 1.75 or later
- Bevy 0.18.0 compatible system

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
univis_editor_app = { path = "path/to/univis_editor/crates/univis_editor_app" }
bevy = "0.18.0"
```

### Basic Usage

```rust
use bevy::prelude::*;
use univis_editor_app::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NodeGraphPlugin)  // Add the editor plugin
        .run();
}
```

### Lightweight Verification

If your machine struggles with running the full workspace test suite in one go, use the sequential verification script:

```bash
bash scripts/verify_workspace.sh
```

The script also supports focused suites:

```bash
bash scripts/verify_workspace.sh core
bash scripts/verify_workspace.sh builtin
bash scripts/verify_workspace.sh persistence
```

The maintained crate-local test commands are:

- `cargo test -p univis_node_graph --test core_api`
- `cargo test -p univis_node_graph --test document_ops`
- `cargo test -p univis_node_graph --test graph_validation`
- `cargo test -p univis_editor_nodes_builtin --test input_nodes`
- `cargo test -p univis_editor_nodes_builtin --test math_nodes`
- `cargo test -p univis_editor_nodes_builtin --test logic_nodes`
- `cargo test -p univis_editor_nodes_builtin --test scene_nodes`
- `cargo test -p univis_editor_persistence --test persistence_defaults`
- `cargo test -p univis_editor_persistence --test persistence_format`

---

## 📚 Creating Custom Nodes

UnivisEditor uses a trait-based system for defining nodes. Simply implement `NodeDefinition`:

```rust
use univis_node_graph::node_definition::{
    NodeDefinition, NodeId, NodeCategory, PortDefinition, 
    ProcessContext, ProcessResult
};
use univis_node_graph::value::NodeValue;
use bevy::prelude::*;

pub struct MyCustomNode;

impl NodeDefinition for MyCustomNode {
    // === Required Methods ===
    
    fn id(&self) -> NodeId {
        NodeId::new("custom/my_node")
    }
    
    fn display_name(&self) -> &str {
        "My Custom Node"
    }
    
    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_float("Input A")
                .with_default(NodeValue::float(0.0)),
            PortDefinition::input_float("Input B")
                .with_default(NodeValue::float(0.0)),
        ]
    }
    
    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_float("Result")]
    }
    
    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let a = ctx.get_float_or(0, 0.0);
        let b = ctx.get_float_or(1, 0.0);
        ctx.set_float(0, a + b);  // Simple addition
        ProcessResult::Success
    }
    
    // === Optional Methods ===
    
    fn category(&self) -> NodeCategory {
        NodeCategory::new("Custom")
    }
    
    fn description(&self) -> Option<&str> {
        Some("Adds two numbers together")
    }
    
    fn color(&self) -> Color {
        Color::srgb(0.3, 0.5, 0.7)
    }
    
    fn icon(&self) -> Option<&str> {
        Some("+")
    }
    
    fn keywords(&self) -> Vec<&str> {
        vec!["add", "sum", "plus"]
    }
    
    fn menu_order(&self) -> i32 {
        1  // Display order in menu
    }
}
```

### Registering Nodes

```rust
use univis_node_graph::node_registry::NodeRegistry;

fn register_my_nodes(registry: &mut NodeRegistry) {
    registry.register(MyCustomNode);
    // Add more nodes...
}
```

---

## 🎯 Node API Reference

### ProcessContext Helpers

The `ProcessContext` provides convenient methods for reading and writing values:

```rust
// Reading values
let value = ctx.get_float(0);           // Option<f64>
let value = ctx.get_float_or(0, 0.0);   // f64 with default
let value = ctx.get_int(0);
let value = ctx.get_bool(0);
let value = ctx.get_vec2(0);
let value = ctx.get_vec3(0);
let value = ctx.get_string(0);

// Writing values
ctx.set_float(0, 42.0);
ctx.set_int(0, 42);
ctx.set_bool(0, true);
ctx.set_vec2(0, Vec2::new(1.0, 2.0));
ctx.set_vec3(0, Vec3::new(1.0, 2.0, 3.0));
ctx.set_string(0, "Hello");
```

### PortDefinition Builders

```rust
PortDefinition::input_float("Value")
    .with_description("A float input")
    .with_default(NodeValue::float(1.0))
    .with_color(Color::srgb(1.0, 0.5, 0.0));

PortDefinition::output_float("Result");
PortDefinition::output_any("Pass Through");
```

### NodeDefinition Trait Methods

| Method | Required | Description |
|--------|----------|-------------|
| `id()` | ✅ | Unique identifier (e.g., "math/add") |
| `display_name()` | ✅ | Human-readable name |
| `inputs()` | ✅ | Input port definitions |
| `outputs()` | ✅ | Output port definitions |
| `process()` | ✅ | Processing logic |
| `category()` | ❌ | Node category for menu |
| `description()` | ❌ | Tooltip/description |
| `color()` | ❌ | Node header color |
| `icon()` | ❌ | Unicode icon |
| `show_in_menu()` | ❌ | Show in context menu (default: true) |
| `menu_order()` | ❌ | Sort order in menu |
| `keywords()` | ❌ | Search keywords |
| `has_custom_body()` | ❌ | Custom UI body |
| `build_body()` | ❌ | Build custom UI |

---

## 📦 Built-in Nodes

### Math Nodes
| Node | Description |
|------|-------------|
| Add | Addition of two numbers |
| Subtract | Subtraction |
| Multiply | Multiplication |
| Divide | Division (with zero check) |
| Clamp | Clamp value to range |
| Lerp | Linear interpolation |
| Min / Max | Minimum / Maximum |
| Abs | Absolute value |
| Sin / Cos | Trigonometric functions |

### Input Nodes
| Node | Description |
|------|-------------|
| Number | Float value input |
| Integer | Integer value input |
| Boolean | True/false toggle |
| Text | String input |
| Vector2 | 2D vector |
| Vector3 | 3D vector |
| Color | RGBA color |

### Logic Nodes
| Node | Description |
|------|-------------|
| Compare | Compare two values |
| Branch | If/else conditional |
| And / Or / Not | Boolean operations |

### Output Nodes
| Node | Description |
|------|-------------|
| View | Display value in node |
| Watch | Named value display |
| Debug | Print to console |

---

## 🏗️ Project Structure

```
src/
├── core/
│   ├── node_definition.rs   # Node trait and types
│   ├── node_registry.rs     # Node registration system
│   ├── value.rs             # Dynamic value types
│   ├── menu.rs              # Context menu system
│   ├── wire.rs              # Wire/connection logic
│   └── interaction.rs       # User interaction
├── nodes/
│   ├── math.rs              # Math nodes
│   ├── input.rs             # Input nodes
│   ├── output.rs            # Output nodes
│   └── logic.rs             # Logic nodes
├── data/
│   └── node_spawn.rs        # Node spawning system
├── widgets/
│   └── infinity_grid.rs     # Background grid
├── editor/
│   └── editor.rs            # Editor camera & controls
└── lib.rs                   # Main plugin
```

---

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test math_node_tests

# Run with verbose output
cargo test -- --nocapture
```

### Test Categories

- `value_tests.rs` - NodeValue type tests
- `process_context_tests.rs` - Context helper tests
- `port_definition_tests.rs` - Port builder tests
- `math_node_tests.rs` - Math node functionality
- `input_node_tests.rs` - Input node tests
- `output_node_tests.rs` - Output node tests
- `logic_node_tests.rs` - Logic node tests
- `integration_tests.rs` - End-to-end tests

---

## 🔧 Architecture

### Data Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Input Node │────▶│  Math Node  │────▶│ Output Node │
│   (Value)   │     │  (Process)  │     │   (View)    │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       └───────────────────┴───────────────────┘
                           │
                    Topological Sort
                    (Kahn's Algorithm)
```

### Node Processing Pipeline

1. **Initialize** - Set default values for new nodes
2. **Sort** - Topological ordering for correct execution
3. **Propagate** - Transfer values through connections
4. **Process** - Execute each node's logic
5. **Output** - Results available for next frame

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/univiseditor/univis_editor
cd univis_editor

# Run tests
cargo test

# Run the example
cargo run --example simple_editor
```

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [Bevy Engine](https://bevyengine.org/) - The game engine this is built on
- [Blender](https://www.blender.org/) - Inspiration for the node system architecture
- [Unreal Engine Blueprints](https://docs.unrealengine.com/) - Design philosophy reference

---

<div align="center">

**Built with ❤️ using Bevy Engine**

</div>
