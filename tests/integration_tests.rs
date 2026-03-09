//! اختبارات التكامل - اختبار النظام ككل

use univis_editor::core::node_definition::{NodeDefinition, NodeId, ProcessContext, ProcessResult};
use univis_editor::core::node_registry::NodeRegistry;
use univis_editor::core::value::NodeValue;
use univis_editor::nodes::math::AddNode;
use univis_editor::nodes::input::NumberNode;
use univis_editor::nodes::output::ViewNode;
use univis_editor::nodes::logic::CompareNode;

// ═════════════════════════════════════════════════════
// اختبارات NodeRegistry
// ═════════════════════════════════════════════════════

#[test]
fn test_registry_create() {
    let registry = NodeRegistry::new();
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_registry_register_and_get() {
    let mut registry = NodeRegistry::new();
    
    let node = AddNode;
    let id = node.id();
    
    registry.register(node);
    
    assert_eq!(registry.len(), 1);
    assert!(registry.get(&id).is_some());
}

#[test]
fn test_registry_contains() {
    let mut registry = NodeRegistry::new();
    
    let node = AddNode;
    let id = node.id();
    
    assert!(!registry.contains(&id));
    
    registry.register(node);
    
    assert!(registry.contains(&id));
}

#[test]
fn test_registry_get_nonexistent() {
    let registry = NodeRegistry::new();
    
    let fake_id = NodeId::new("nonexistent/node");
    assert!(registry.get(&fake_id).is_none());
}

// ═════════════════════════════════════════════════════
// اختبارات معالجة سلسلة من العُقد (محاكاة)
// ═════════════════════════════════════════════════════

/// محاكاة تدفق البيانات: NumberNode -> AddNode -> ViewNode
#[test]
fn test_node_chain_simulation() {
    // الخطوة 1: NumberNode ينتج قيمة
    let input_node = NumberNode;
    let mut input_outputs = vec![NodeValue::None];
    let mut custom_data = None;
    
    {
        let inputs: &[NodeValue] = &[];
        let mut ctx = ProcessContext {
            inputs,
            outputs: &mut input_outputs,
            delta_time: 0.016,
            custom_data: &mut custom_data,
        };
        
        input_node.process(&mut ctx);
    }
    
    // القيمة الافتراضية يجب أن تكون 1.0
    let number_output = input_outputs[0].clone();
    assert_eq!(number_output, NodeValue::float(1.0));
    
    // الخطوة 2: AddNode يأخذ القيمة ويضيف عليها
    let add_node = AddNode;
    let mut add_outputs = vec![NodeValue::None];
    let mut custom_data2 = None;
    
    {
        let add_inputs = vec![number_output.clone(), NodeValue::float(5.0)];
        let mut ctx = ProcessContext {
            inputs: &add_inputs,
            outputs: &mut add_outputs,
            delta_time: 0.016,
            custom_data: &mut custom_data2,
        };
        
        add_node.process(&mut ctx);
    }
    
    // 1.0 + 5.0 = 6.0
    assert_eq!(add_outputs[0], NodeValue::float(6.0));
    
    // الخطوة 3: ViewNode يعرض النتيجة
    let view_node = ViewNode;
    let mut view_outputs = vec![NodeValue::None];
    let mut custom_data3 = None;
    
    {
        let view_inputs = vec![add_outputs[0].clone()];
        let mut ctx = ProcessContext {
            inputs: &view_inputs,
            outputs: &mut view_outputs,
            delta_time: 0.016,
            custom_data: &mut custom_data3,
        };
        
        view_node.process(&mut ctx);
    }
    
    // Pass through يجب أن يمرر 6.0
    assert_eq!(view_outputs[0], NodeValue::float(6.0));
}

/// اختبار مقارنة قيمتين من NumberNode
#[test]
fn test_compare_flow() {
    // إنشاء قيمتين
    let inputs_a = vec![NodeValue::float(10.0)];
    let inputs_b = vec![NodeValue::float(5.0)];
    
    // CompareNode يقارن
    let compare_node = CompareNode;
    let mut outputs = vec![NodeValue::None; 4];
    let mut custom_data = None;
    
    // استخدام قيم من "مداخل" مختلفة (محاكاة اتصال)
    let compare_inputs = vec![inputs_a[0].clone(), inputs_b[0].clone()];
    
    let mut ctx = ProcessContext {
        inputs: &compare_inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };
    
    compare_node.process(&mut ctx);
    
    // 10 > 5
    assert_eq!(ctx.outputs[0], NodeValue::bool(false)); // Equal
    assert_eq!(ctx.outputs[1], NodeValue::bool(true));  // Not Equal
    assert_eq!(ctx.outputs[2], NodeValue::bool(true));  // Greater
    assert_eq!(ctx.outputs[3], NodeValue::bool(false)); // Less
}

// ═════════════════════════════════════════════════════
// اختبارات NodeDefinition trait
// ═════════════════════════════════════════════════════

#[test]
fn test_node_definition_default_values() {
    let node = AddNode;
    
    let defaults = node.default_input_values();
    assert_eq!(defaults.len(), 2);
    assert_eq!(defaults[0], NodeValue::float(0.0));
    assert_eq!(defaults[1], NodeValue::float(0.0));
}

#[test]
fn test_node_definition_port_colors() {
    let node = AddNode;
    
    let input_colors = node.input_port_colors();
    let output_colors = node.output_port_colors();
    
    assert_eq!(input_colors.len(), 2);
    assert_eq!(output_colors.len(), 1);
    
    // الألوان يجب أن تكون صالحة (ليس NaN)
    for color in input_colors.iter().chain(output_colors.iter()) {
        let srgba = color.to_srgba();
        assert!(!srgba.red.is_nan());
        assert!(!srgba.green.is_nan());
        assert!(!srgba.blue.is_nan());
    }
}

#[test]
fn test_node_category() {
    use univis_editor::core::node_definition::NodeCategory;
    
    let math_node = AddNode;
    let logic_node = CompareNode;
    let input_node = NumberNode;
    let output_node = ViewNode;
    
    assert_eq!(math_node.category().as_str(), NodeCategory::MATH);
    assert_eq!(logic_node.category().as_str(), NodeCategory::LOGIC);
    assert_eq!(input_node.category().as_str(), NodeCategory::INPUT);
    assert_eq!(output_node.category().as_str(), NodeCategory::OUTPUT);
}

// ═════════════════════════════════════════════════════
// اختبارات ProcessResult
// ═════════════════════════════════════════════════════

#[test]
fn test_process_result_success() {
    let node = AddNode;
    
    let inputs = vec![NodeValue::float(1.0), NodeValue::float(2.0)];
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
}

#[test]
fn test_process_result_error_division_by_zero() {
    use univis_editor::nodes::math::DivideNode;
    
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
    if let ProcessResult::Error(msg) = result {
        assert!(msg.contains("zero"));
    }
}
