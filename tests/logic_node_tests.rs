//! اختبارات عُقد المنطق

use univis_editor::core::node_definition::{NodeDefinition, ProcessContext};
use univis_editor::core::value::NodeValue;
use univis_editor::nodes::logic::*;

// ═════════════════════════════════════════════════════
// اختبارات عقدة CompareNode
// ═════════════════════════════════════════════════════

#[test]
fn test_compare_node_basic() {
    let node = CompareNode;
    
    assert_eq!(node.id().as_str(), "logic/compare");
    assert_eq!(node.display_name(), "Compare");
    assert_eq!(node.inputs().len(), 2);  // A, B
    assert_eq!(node.outputs().len(), 4); // Equal, NotEqual, Greater, Less
}

#[test]
fn test_compare_node_equal() {
    let node = CompareNode;
    
    let inputs = vec![NodeValue::float(5.0), NodeValue::float(5.0)];
    let mut outputs = vec![NodeValue::None; 4];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    assert_eq!(ctx.outputs[0], NodeValue::bool(true));  // Equal
    assert_eq!(ctx.outputs[1], NodeValue::bool(false)); // Not Equal
    assert_eq!(ctx.outputs[2], NodeValue::bool(false)); // Greater
    assert_eq!(ctx.outputs[3], NodeValue::bool(false)); // Less
}

#[test]
fn test_compare_node_greater() {
    let node = CompareNode;
    
    let inputs = vec![NodeValue::float(10.0), NodeValue::float(5.0)];
    let mut outputs = vec![NodeValue::None; 4];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    assert_eq!(ctx.outputs[0], NodeValue::bool(false)); // Equal
    assert_eq!(ctx.outputs[1], NodeValue::bool(true));  // Not Equal
    assert_eq!(ctx.outputs[2], NodeValue::bool(true));  // Greater
    assert_eq!(ctx.outputs[3], NodeValue::bool(false)); // Less
}

#[test]
fn test_compare_node_less() {
    let node = CompareNode;
    
    let inputs = vec![NodeValue::float(3.0), NodeValue::float(5.0)];
    let mut outputs = vec![NodeValue::None; 4];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    
    assert_eq!(ctx.outputs[0], NodeValue::bool(false)); // Equal
    assert_eq!(ctx.outputs[1], NodeValue::bool(true));  // Not Equal
    assert_eq!(ctx.outputs[2], NodeValue::bool(false)); // Greater
    assert_eq!(ctx.outputs[3], NodeValue::bool(true));  // Less
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة BranchNode
// ═════════════════════════════════════════════════════

#[test]
fn test_branch_node_basic() {
    let node = BranchNode;
    
    assert_eq!(node.id().as_str(), "logic/branch");
    assert_eq!(node.display_name(), "Branch");
    assert_eq!(node.inputs().len(), 3);  // Condition, True, False
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn test_branch_node_returns_true_value() {
    let node = BranchNode;
    
    let inputs = vec![
        NodeValue::bool(true),           // Condition = true
        NodeValue::float(100.0),         // True value
        NodeValue::float(200.0),         // False value
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
    
    assert_eq!(ctx.outputs[0], NodeValue::float(100.0)); // يجب أن يعيد True value
}

#[test]
fn test_branch_node_returns_false_value() {
    let node = BranchNode;
    
    let inputs = vec![
        NodeValue::bool(false),          // Condition = false
        NodeValue::float(100.0),         // True value
        NodeValue::float(200.0),         // False value
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
    
    assert_eq!(ctx.outputs[0], NodeValue::float(200.0)); // يجب أن يعيد False value
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة AndNode
// ═════════════════════════════════════════════════════

#[test]
fn test_and_node_basic() {
    let node = AndNode;
    
    assert_eq!(node.id().as_str(), "logic/and");
    assert_eq!(node.display_name(), "And");
    assert_eq!(node.inputs().len(), 2);
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn test_and_node_true_true() {
    let node = AndNode;
    
    let inputs = vec![NodeValue::bool(true), NodeValue::bool(true)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::bool(true));
}

#[test]
fn test_and_node_true_false() {
    let node = AndNode;
    
    let inputs = vec![NodeValue::bool(true), NodeValue::bool(false)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::bool(false));
}

#[test]
fn test_and_node_false_false() {
    let node = AndNode;
    
    let inputs = vec![NodeValue::bool(false), NodeValue::bool(false)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::bool(false));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة OrNode
// ═════════════════════════════════════════════════════

#[test]
fn test_or_node_basic() {
    let node = OrNode;
    
    assert_eq!(node.id().as_str(), "logic/or");
    assert_eq!(node.display_name(), "Or");
}

#[test]
fn test_or_node_false_false() {
    let node = OrNode;
    
    let inputs = vec![NodeValue::bool(false), NodeValue::bool(false)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::bool(false));
}

#[test]
fn test_or_node_true_false() {
    let node = OrNode;
    
    let inputs = vec![NodeValue::bool(true), NodeValue::bool(false)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::bool(true));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة NotNode
// ═════════════════════════════════════════════════════

#[test]
fn test_not_node_basic() {
    let node = NotNode;
    
    assert_eq!(node.id().as_str(), "logic/not");
    assert_eq!(node.display_name(), "Not");
    assert_eq!(node.inputs().len(), 1);
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn test_not_node_true() {
    let node = NotNode;
    
    let inputs = vec![NodeValue::bool(true)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::bool(false));
}

#[test]
fn test_not_node_false() {
    let node = NotNode;
    
    let inputs = vec![NodeValue::bool(false)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    node.process(&mut ctx);
    assert_eq!(ctx.outputs[0], NodeValue::bool(true));
}
