//! اختبارات ProcessContext helper methods

use univis_editor::core::node_definition::ProcessContext;
use univis_editor::core::value::NodeValue;
use bevy::prelude::Vec2;

/// اختبار قراءة وكتابة float
#[test]
fn test_float_operations() {
    let inputs = vec![NodeValue::float(5.5), NodeValue::float(10.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;

    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };

    // اختبار get_float
    assert_eq!(ctx.get_float(0), Some(5.5));
    assert_eq!(ctx.get_float(1), Some(10.0));
    assert_eq!(ctx.get_float(2), None); // خارج النطاق

    // اختبار get_float_or
    assert_eq!(ctx.get_float_or(0, 0.0), 5.5);
    assert_eq!(ctx.get_float_or(2, 99.0), 99.0); // قيمة افتراضية

    // اختبار set_float
    ctx.set_float(0, 42.5);
    assert_eq!(ctx.outputs[0], NodeValue::float(42.5));
}

/// اختبار قراءة وكتابة int
#[test]
fn test_int_operations() {
    let inputs = vec![NodeValue::int(100), NodeValue::int(-50)];
    let mut outputs = vec![NodeValue::None, NodeValue::None];
    let mut custom_data = None;

    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };

    // اختبار get_int
    assert_eq!(ctx.get_int(0), Some(100));
    assert_eq!(ctx.get_int(1), Some(-50));
    assert_eq!(ctx.get_int(2), None);

    // اختبار get_int_or
    assert_eq!(ctx.get_int_or(0, 0), 100);
    assert_eq!(ctx.get_int_or(2, 999), 999);

    // اختبار set_int
    ctx.set_int(0, 777);
    assert_eq!(ctx.outputs[0], NodeValue::int(777));
}

/// اختبار قراءة وكتابة bool
#[test]
fn test_bool_operations() {
    let inputs = vec![NodeValue::bool(true), NodeValue::bool(false)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;

    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };

    // اختبار get_bool
    assert_eq!(ctx.get_bool(0), Some(true));
    assert_eq!(ctx.get_bool(1), Some(false));
    assert_eq!(ctx.get_bool(2), None);

    // اختبار get_bool_or
    assert_eq!(ctx.get_bool_or(0, false), true);
    assert_eq!(ctx.get_bool_or(2, true), true); // قيمة افتراضية

    // اختبار set_bool
    ctx.set_bool(0, false);
    assert_eq!(ctx.outputs[0], NodeValue::bool(false));
}

/// اختبار قراءة وكتابة Vec2
#[test]
fn test_vec2_operations() {
    let inputs = vec![NodeValue::Vec2(Vec2::new(3.0, 4.0))];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;

    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };

    // اختبار get_vec2
    let vec = ctx.get_vec2(0);
    assert!(vec.is_some());
    let v = vec.unwrap();
    assert_eq!(v.x, 3.0);
    assert_eq!(v.y, 4.0);

    // اختبار set_vec2
    ctx.set_vec2(0, Vec2::new(10.0, 20.0));
    assert_eq!(ctx.outputs[0], NodeValue::Vec2(Vec2::new(10.0, 20.0)));
}

/// اختبار قراءة وكتابة String
#[test]
fn test_string_operations() {
    let inputs = vec![NodeValue::string("Hello, World!")];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;

    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };

    // اختبار get_string
    assert_eq!(ctx.get_string(0), Some("Hello, World!"));
    assert_eq!(ctx.get_string(1), None);

    // اختبار set_string
    ctx.set_string(0, "Test String");
    assert_eq!(ctx.outputs[0], NodeValue::string("Test String"));
}

/// اختبار الكتابة العامة set
#[test]
fn test_generic_set() {
    let inputs = vec![];
    let mut outputs = vec![NodeValue::None, NodeValue::None, NodeValue::None];
    let mut custom_data = None;

    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };

    // كتابة أنواع مختلفة
    ctx.set(0, NodeValue::float(1.5));
    ctx.set(1, NodeValue::int(42));
    ctx.set(2, NodeValue::bool(true));

    assert_eq!(ctx.outputs[0], NodeValue::float(1.5));
    assert_eq!(ctx.outputs[1], NodeValue::int(42));
    assert_eq!(ctx.outputs[2], NodeValue::bool(true));
}

/// اختبار عدم تجاوز الفهرس
#[test]
fn test_index_bounds() {
    let inputs = vec![NodeValue::float(1.0)];
    let mut outputs = vec![NodeValue::None];
    let mut custom_data = None;

    let mut ctx = ProcessContext {
        inputs: &inputs,
        outputs: &mut outputs,
        delta_time: 0.016,
        custom_data: &mut custom_data,
    };

    // محاولة الكتابة خارج النطاق - لا يجب أن يسبب panic
    ctx.set(100, NodeValue::float(1.0));
    ctx.set_float(100, 1.0);

    // محاولة القراءة خارج النطاق
    assert_eq!(ctx.get_float(100), None);
    assert_eq!(ctx.get_int(100), None);
    assert_eq!(ctx.get_bool(100), None);
}
