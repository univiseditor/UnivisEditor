//! اختبارات عُقد الرياضيات

use univis_editor::core::node_definition::{NodeDefinition, ProcessContext, ProcessResult};
use univis_editor::core::value::NodeValue;
use univis_editor::nodes::math::*;
// ═════════════════════════════════════════════════════
// اختبارات عقدة الجمع (AddNode)
// ═════════════════════════════════════════════════════

#[test]
fn test_add_node_basic() {
    let node = AddNode;
    
    assert_eq!(node.id().as_str(), "math/add");
    assert_eq!(node.display_name(), "Add");
    assert_eq!(node.inputs().len(), 2);
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn test_add_node_process() {
    let node = AddNode;
    
    let inputs = vec![NodeValue::float(5.0), NodeValue::float(3.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    let result = node.process(&mut ctx);
    assert!(matches!(result, ProcessResult::Success));
    assert_eq!(ctx.outputs[0], NodeValue::float(8.0));
}

#[test]
fn test_add_node_negative_numbers() {
    let node = AddNode;
    
    let inputs = vec![NodeValue::float(-5.0), NodeValue::float(3.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::float(-2.0));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة الطرح (SubtractNode)
// ═════════════════════════════════════════════════════

#[test]
fn test_subtract_node_basic() {
    let node = SubtractNode;
    
    assert_eq!(node.id().as_str(), "math/subtract");
    assert_eq!(node.display_name(), "Subtract");
}

#[test]
fn test_subtract_node_process() {
    let node = SubtractNode;
    
    let inputs = vec![NodeValue::float(10.0), NodeValue::float(3.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::float(7.0));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة الضرب (MultiplyNode)
// ═════════════════════════════════════════════════════

#[test]
fn test_multiply_node_basic() {
    let node = MultiplyNode;
    
    assert_eq!(node.id().as_str(), "math/multiply");
    assert_eq!(node.display_name(), "Multiply");
}

#[test]
fn test_multiply_node_process() {
    let node = MultiplyNode;
    
    let inputs = vec![NodeValue::float(4.0), NodeValue::float(5.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::float(20.0));
}

#[test]
fn test_multiply_node_by_zero() {
    let node = MultiplyNode;
    
    let inputs = vec![NodeValue::float(100.0), NodeValue::float(0.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::float(0.0));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة القسمة (DivideNode)
// ═════════════════════════════════════════════════════

#[test]
fn test_divide_node_basic() {
    let node = DivideNode;
    
    assert_eq!(node.id().as_str(), "math/divide");
    assert_eq!(node.display_name(), "Divide");
}

#[test]
fn test_divide_node_process() {
    let node = DivideNode;
    
    let inputs = vec![NodeValue::float(20.0), NodeValue::float(4.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    let result = node.process(&mut ctx);
    assert!(matches!(result, ProcessResult::Success));
    assert_eq!(ctx.outputs[0], NodeValue::float(5.0));
}

#[test]
fn test_divide_node_by_zero() {
    let node = DivideNode;
    
    let inputs = vec![NodeValue::float(10.0), NodeValue::float(0.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    let result = node.process(&mut ctx);
    assert!(matches!(result, ProcessResult::Error(_)));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة Clamp
// ═════════════════════════════════════════════════════

#[test]
fn test_clamp_node_basic() {
    let node = ClampNode;
    
    assert_eq!(node.id().as_str(), "math/clamp");
    assert_eq!(node.inputs().len(), 3);
}

#[test]
fn test_clamp_node_within_range() {
    let node = ClampNode;
    
    let inputs = vec![
        NodeValue::float(0.5),  // value
        NodeValue::float(0.0),  // min
        NodeValue::float(1.0),  // max
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
    assert_eq!(ctx.outputs[0], NodeValue::float(0.5));
}

#[test]
fn test_clamp_node_below_min() {
    let node = ClampNode;
    
    let inputs = vec![
        NodeValue::float(-5.0), // value below min
        NodeValue::float(0.0),  // min
        NodeValue::float(1.0),  // max
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
    assert_eq!(ctx.outputs[0], NodeValue::float(0.0)); // clamped to min
}

#[test]
fn test_clamp_node_above_max() {
    let node = ClampNode;
    
    let inputs = vec![
        NodeValue::float(10.0), // value above max
        NodeValue::float(0.0),  // min
        NodeValue::float(1.0),  // max
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
    assert_eq!(ctx.outputs[0], NodeValue::float(1.0)); // clamped to max
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة Lerp
// ═════════════════════════════════════════════════════

#[test]
fn test_lerp_node_basic() {
    let node = LerpNode;
    assert_eq!(node.id().as_str(), "math/lerp");
}

#[test]
fn test_lerp_node_midpoint() {
    let node = LerpNode;
    
    let inputs = vec![
        NodeValue::float(0.0),  // A
        NodeValue::float(10.0), // B
        NodeValue::float(0.5),  // T (50%)
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
    assert_eq!(ctx.outputs[0], NodeValue::float(5.0));
}

#[test]
fn test_lerp_node_at_start() {
    let node = LerpNode;
    
    let inputs = vec![
        NodeValue::float(0.0),  // A
        NodeValue::float(10.0), // B
        NodeValue::float(0.0),  // T (0%)
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
    assert_eq!(ctx.outputs[0], NodeValue::float(0.0));
}

#[test]
fn test_lerp_node_at_end() {
    let node = LerpNode;
    
    let inputs = vec![
        NodeValue::float(0.0),  // A
        NodeValue::float(10.0), // B
        NodeValue::float(1.0),  // T (100%)
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
    assert_eq!(ctx.outputs[0], NodeValue::float(10.0));
}

// ═════════════════════════════════════════════════════
// اختبارات عقد Min/Max
// ═════════════════════════════════════════════════════

#[test]
fn test_min_node() {
    let node = MinNode;
    
    let inputs = vec![NodeValue::float(3.0), NodeValue::float(7.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::float(3.0));
}

#[test]
fn test_max_node() {
    let node = MaxNode;
    
    let inputs = vec![NodeValue::float(3.0), NodeValue::float(7.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::float(7.0));
}

// ═════════════════════════════════════════════════════
// اختبارات عقد Abs
// ═════════════════════════════════════════════════════

#[test]
fn test_abs_node_positive() {
    let node = AbsNode;
    
    let inputs = vec![NodeValue::float(5.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::float(5.0));
}

#[test]
fn test_abs_node_negative() {
    let node = AbsNode;
    
    let inputs = vec![NodeValue::float(-5.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::float(5.0));
}

// ═════════════════════════════════════════════════════
// اختبارات عقد Sin/Cos
// ═════════════════════════════════════════════════════

#[test]
fn test_sin_node() {
    let node = SinNode;
    
    let inputs = vec![NodeValue::float(0.0)]; // sin(0) = 0
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::float(0.0));
}

#[test]
fn test_cos_node() {
    let node = CosNode;
    
    let inputs = vec![NodeValue::float(0.0)]; // cos(0) = 1
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::float(1.0));
}
