//! اختبارات عُقد الإدخال

use univis_editor::core::node_definition::{NodeDefinition, ProcessContext};
use univis_editor::core::value::NodeValue;
use univis_editor::nodes::input::*;
use bevy::prelude::{Vec2, Vec3};

// ═════════════════════════════════════════════════════
// اختبارات عقدة NumberNode
// ═════════════════════════════════════════════════════

#[test]
fn test_number_node_basic() {
    let node = NumberNode;
    
    assert_eq!(node.id().as_str(), "input/number");
    assert_eq!(node.display_name(), "Number");
    assert_eq!(node.inputs().len(), 0); // لا مداخل
    assert_eq!(node.outputs().len(), 1); // مخرج واحد
}

#[test]
fn test_number_node_default_value() {
    let node = NumberNode;
    
    let inputs = vec![];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    // القيمة الافتراضية يجب أن تكون 1.0 (تم تغييرها من 0.0)
    assert_eq!(ctx.outputs[0], NodeValue::float(1.0));
}

#[test]
fn test_number_node_preserves_existing() {
    let node = NumberNode;
    
    let inputs = vec![];
    let mut outputs = vec![NodeValue::float(42.0)]; // قيمة موجودة مسبقاً
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    // يجب أن تحافظ على القيمة الموجودة
    assert_eq!(ctx.outputs[0], NodeValue::float(42.0));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة IntegerNode
// ═════════════════════════════════════════════════════

#[test]
fn test_integer_node_basic() {
    let node = IntegerNode;
    
    assert_eq!(node.id().as_str(), "input/integer");
    assert_eq!(node.display_name(), "Integer");
    assert_eq!(node.inputs().len(), 0);
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn test_integer_node_default_value() {
    let node = IntegerNode;
    
    let inputs = vec![];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    // القيمة الافتراضية يجب أن تكون 1 (تم تغييرها من 0)
    assert_eq!(ctx.outputs[0], NodeValue::int(1));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة BooleanNode
// ═════════════════════════════════════════════════════

#[test]
fn test_boolean_node_basic() {
    let node = BooleanNode;
    
    assert_eq!(node.id().as_str(), "input/boolean");
    assert_eq!(node.display_name(), "Boolean");
    assert_eq!(node.inputs().len(), 0);
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn test_boolean_node_default_value() {
    let node = BooleanNode;
    
    let inputs = vec![];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    // القيمة الافتراضية false
    assert_eq!(ctx.outputs[0], NodeValue::bool(false));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة TextNode
// ═════════════════════════════════════════════════════

#[test]
fn test_text_node_basic() {
    let node = TextNode;
    
    assert_eq!(node.id().as_str(), "input/text");
    assert_eq!(node.display_name(), "Text");
    assert_eq!(node.inputs().len(), 0);
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn test_text_node_default_value() {
    let node = TextNode;
    
    let inputs = vec![];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    // القيمة الافتراضية نص فارغ
    assert_eq!(ctx.outputs[0], NodeValue::string(""));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة Vector2Node
// ═════════════════════════════════════════════════════

#[test]
fn test_vector2_node_basic() {
    let node = Vector2Node;
    
    assert_eq!(node.id().as_str(), "input/vector2");
    assert_eq!(node.display_name(), "Vector2");
    assert_eq!(node.inputs().len(), 2); // X, Y
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn test_vector2_node_process() {
    let node = Vector2Node;
    
    let inputs = vec![
        NodeValue::float(3.0), // X
        NodeValue::float(4.0), // Y
    ];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    assert_eq!(ctx.outputs[0], NodeValue::Vec2(Vec2::new(3.0, 4.0)));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة Vector3Node
// ═════════════════════════════════════════════════════

#[test]
fn test_vector3_node_basic() {
    let node = Vector3Node;
    
    assert_eq!(node.id().as_str(), "input/vector3");
    assert_eq!(node.display_name(), "Vector3");
    assert_eq!(node.inputs().len(), 3); // X, Y, Z
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn test_vector3_node_process() {
    let node = Vector3Node;
    
    let inputs = vec![
        NodeValue::float(1.0), // X
        NodeValue::float(2.0), // Y
        NodeValue::float(3.0), // Z
    ];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    assert_eq!(ctx.outputs[0], NodeValue::Vec3(Vec3::new(1.0, 2.0, 3.0)));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة ColorNode
// ═════════════════════════════════════════════════════

#[test]
fn test_color_node_basic() {
    let node = ColorNode;
    
    assert_eq!(node.id().as_str(), "input/color");
    assert_eq!(node.display_name(), "Color");
    assert_eq!(node.inputs().len(), 4); // R, G, B, A
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn test_color_node_process() {
    let node = ColorNode;
    
    let inputs = vec![
        NodeValue::float(1.0), // R
        NodeValue::float(0.5), // G
        NodeValue::float(0.0), // B
        NodeValue::float(1.0), // A
    ];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    // التحقق من القيمة
    if let NodeValue::Color(color) = &ctx.outputs[0] {
        let srgba = color.to_srgba();
        assert!((srgba.red - 1.0).abs() < 0.01);
        assert!((srgba.green - 0.5).abs() < 0.01);
        assert!((srgba.blue - 0.0).abs() < 0.01);
        assert!((srgba.alpha - 1.0).abs() < 0.01);
    } else {
        panic!("Expected Color value");
    }
}
