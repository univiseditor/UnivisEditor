//! عقد التركيب - تحويل المواد/الأجزاء إلى أجزاء أعلى مستوى

use bevy::prelude::*;
use univis_editor_core::register_node;
use serde_json::json;
use univis_editor_core::node_definition::{
    NodeCategory, NodeDefinition, NodeId, PortDefinition, ProcessContext, ProcessResult,
};

const ASSEMBLY_COLOR: Color = Color::srgb(0.63, 0.42, 0.24);

pub struct WheelAssemblyNode;

impl NodeDefinition for WheelAssemblyNode {
    fn id(&self) -> NodeId {
        NodeId::new("assembly/wheel")
    }

    fn display_name(&self) -> &str {
        "Wheel Assembly"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Craft/Assembly")
    }

    fn description(&self) -> Option<&str> {
        Some("Build a wheel from metal + rubber + air")
    }

    fn color(&self) -> Color {
        ASSEMBLY_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_tag("Metal", "material/metal"),
            PortDefinition::input_tag("Rubber", "material/rubber"),
            PortDefinition::input_tag("Air", "material/air"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_tag("Wheel", "part/wheel")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let Some((metal_tag, metal)) = ctx.get_tagged(0) else {
            return ProcessResult::MissingInput(0);
        };
        let Some((rubber_tag, rubber)) = ctx.get_tagged(1) else {
            return ProcessResult::MissingInput(1);
        };
        let Some((air_tag, air)) = ctx.get_tagged(2) else {
            return ProcessResult::MissingInput(2);
        };

        if metal_tag != "material/metal" {
            return ProcessResult::Error(format!("Expected material/metal, got {}", metal_tag));
        }
        if rubber_tag != "material/rubber" {
            return ProcessResult::Error(format!("Expected material/rubber, got {}", rubber_tag));
        }
        if air_tag != "material/air" {
            return ProcessResult::Error(format!("Expected material/air, got {}", air_tag));
        }

        ctx.set_tagged(
            0,
            "part/wheel",
            json!({
                "recipe": "wheel",
                "metal": metal,
                "rubber": rubber,
                "air": air
            }),
        );
        ProcessResult::Success
    }
}

pub struct CranePartAssemblyNode;

impl NodeDefinition for CranePartAssemblyNode {
    fn id(&self) -> NodeId {
        NodeId::new("assembly/crane_part")
    }

    fn display_name(&self) -> &str {
        "Crane Part Assembly"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Craft/Assembly")
    }

    fn description(&self) -> Option<&str> {
        Some("Build a crane part from metal + wheel")
    }

    fn color(&self) -> Color {
        ASSEMBLY_COLOR
    }

    fn inputs(&self) -> Vec<PortDefinition> {
        vec![
            PortDefinition::input_tag("Metal", "material/metal"),
            PortDefinition::input_tag("Wheel", "part/wheel"),
        ]
    }

    fn outputs(&self) -> Vec<PortDefinition> {
        vec![PortDefinition::output_tag("Crane Part", "part/crane_part")]
    }

    fn process(&self, ctx: &mut ProcessContext) -> ProcessResult {
        let Some((metal_tag, metal)) = ctx.get_tagged(0) else {
            return ProcessResult::MissingInput(0);
        };
        let Some((wheel_tag, wheel)) = ctx.get_tagged(1) else {
            return ProcessResult::MissingInput(1);
        };

        if metal_tag != "material/metal" {
            return ProcessResult::Error(format!("Expected material/metal, got {}", metal_tag));
        }
        if wheel_tag != "part/wheel" {
            return ProcessResult::Error(format!("Expected part/wheel, got {}", wheel_tag));
        }

        ctx.set_tagged(
            0,
            "part/crane_part",
            json!({
                "recipe": "crane_part",
                "metal": metal,
                "wheel": wheel
            }),
        );
        ProcessResult::Success
    }
}

register_node!(WheelAssemblyNode);
register_node!(CranePartAssemblyNode);
