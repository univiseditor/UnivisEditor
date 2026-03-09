//! اختبارات عُقد الإخراج

use univis_editor::core::node_definition::{NodeDefinition, ProcessContext, ProcessResult};
use univis_editor::core::value::NodeValue;
use univis_editor::nodes::output::*;

// ═════════════════════════════════════════════════════
// اختبارات عقدة ViewNode
// ═════════════════════════════════════════════════════

#[test]
fn test_view_node_basic() {
    let node = ViewNode;
    
    assert_eq!(node.id().as_str(), "output/view");
    assert_eq!(node.display_name(), "View");
    assert_eq!(node.inputs().len(), 1);  // Value
    assert_eq!(node.outputs().len(), 1); // Pass Through
}

#[test]
fn test_view_node_pass_through() {
    let node = ViewNode;
    
    let inputs = vec![NodeValue::float(42.5)];
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
    // يجب أن يمرر القيمة كما هي
    assert_eq!(ctx.outputs[0], NodeValue::float(42.5));
}

#[test]
fn test_view_node_has_custom_body() {
    let node = ViewNode;
    
    // ViewNode يجب أن يكون لها body مخصص
    assert!(node.has_custom_body());
}

#[test]
fn test_view_node_with_different_types() {
    let node = ViewNode;
    
    // اختبار مع bool
    {
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
        assert_eq!(ctx.outputs[0], NodeValue::bool(true));
    }
    
    // اختبار مع string
    {
        let inputs = vec![NodeValue::string("Hello")];
        let mut outputs = vec![NodeValue::None];
        let mut custom_data = None;
        
        let mut ctx = ProcessContext {
            inputs: &inputs,
            outputs: &mut outputs,
            delta_time: 0.016,
            custom_data: &mut custom_data,
        };
        
        node.process(&mut ctx);
        assert_eq!(ctx.outputs[0], NodeValue::string("Hello"));
    }
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة WatchNode
// ═════════════════════════════════════════════════════

#[test]
fn test_watch_node_basic() {
    let node = WatchNode;
    
    assert_eq!(node.id().as_str(), "output/watch");
    assert_eq!(node.display_name(), "Watch");
    assert_eq!(node.inputs().len(), 2);  // Name, Value
    assert_eq!(node.outputs().len(), 1); // Pass Through
}

#[test]
fn test_watch_node_pass_through() {
    let node = WatchNode;
    
    let inputs = vec![
        NodeValue::string("MyValue"),  // Name
        NodeValue::float(123.0),       // Value
    ];
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
    // يجب أن يمرر القيمة الثانية (Value) فقط
    assert_eq!(ctx.outputs[0], NodeValue::float(123.0));
}

// ═════════════════════════════════════════════════════
// اختبارات عقدة DebugNode
// ═════════════════════════════════════════════════════

#[test]
fn test_debug_node_basic() {
    let node = DebugNode;
    
    assert_eq!(node.id().as_str(), "output/debug");
    assert_eq!(node.display_name(), "Debug");
    assert_eq!(node.inputs().len(), 1);  // Value
    assert_eq!(node.outputs().len(), 1); // Pass Through
}

#[test]
fn test_debug_node_pass_through() {
    let node = DebugNode;
    
    let inputs = vec![NodeValue::float(99.0)];
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
    // يجب أن يمرر القيمة كما هي
    assert_eq!(ctx.outputs[0], NodeValue::float(99.0));
}
