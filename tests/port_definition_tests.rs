//! اختبارات PortDefinition و resolve_color

use univis_editor::core::node_definition::PortDefinition;
use univis_editor::core::value::{NodeValue, ValueType};
use bevy::prelude::Color;

/// اختبار إنشاء PortDefinition أساسي
#[test]
fn test_port_definition_basic() {
    let port = PortDefinition::new("TestPort", ValueType::Float);

    assert_eq!(port.name, "TestPort");
    assert_eq!(port.value_type, ValueType::Float);
    assert!(port.description.is_none());
    assert!(port.default_value.is_none());
    assert!(port.color.is_none());
}

/// اختبار الطرق المساعدة لإنشاء المنافذ
#[test]
fn test_port_definition_helpers() {
    // input_float
    let float_port = PortDefinition::input_float("FloatInput");
    assert_eq!(float_port.name, "FloatInput");
    assert_eq!(float_port.value_type, ValueType::Float);

    // input_int
    let int_port = PortDefinition::input_int("IntInput");
    assert_eq!(int_port.value_type, ValueType::Int);

    // input_bool
    let bool_port = PortDefinition::input_bool("BoolInput");
    assert_eq!(bool_port.value_type, ValueType::Bool);

    // input_string
    let str_port = PortDefinition::input_string("StringInput");
    assert_eq!(str_port.value_type, ValueType::String);

    // input_vec3
    let vec_port = PortDefinition::input_vec3("Vec3Input");
    assert_eq!(vec_port.value_type, ValueType::Vec3);

    // output_float
    let out_port = PortDefinition::output_float("FloatOutput");
    assert_eq!(out_port.value_type, ValueType::Float);

    // output_any
    let any_port = PortDefinition::output_any("AnyOutput");
    assert_eq!(any_port.value_type, ValueType::Any);
}

/// اختبار with_description
#[test]
fn test_port_with_description() {
    let port = PortDefinition::new("Value", ValueType::Float)
        .with_description("A floating point value");

    assert_eq!(port.description, Some("A floating point value".to_string()));
}

/// اختبار with_default
#[test]
fn test_port_with_default() {
    let port = PortDefinition::new("Number", ValueType::Float)
        .with_default(NodeValue::float(42.5));

    assert_eq!(port.default_value, Some(NodeValue::float(42.5)));
}

/// اختبار with_color
#[test]
fn test_port_with_color() {
    let custom_color = Color::srgb(1.0, 0.5, 0.0);
    let port = PortDefinition::new("Custom", ValueType::Float)
        .with_color(custom_color);

    assert!(port.color.is_some());
    let port_color = port.color.unwrap();
    // التحقق من اللون (بالتقريب بسبب srgb)
    assert!((port_color.to_srgba().red - 1.0).abs() < 0.01);
    assert!((port_color.to_srgba().green - 0.5).abs() < 0.01);
    assert!((port_color.to_srgba().blue - 0.0).abs() < 0.01);
}

/// اختبار resolve_color مع لون مخصص
#[test]
fn test_resolve_color_custom() {
    let custom_color = Color::srgb(0.8, 0.2, 0.6);
    let port = PortDefinition::new("CustomColored", ValueType::Float)
        .with_color(custom_color);

    let resolved = port.resolve_color();
    // يجب أن يعيد اللون المخصص
    assert!((resolved.to_srgba().red - 0.8).abs() < 0.01);
    assert!((resolved.to_srgba().green - 0.2).abs() < 0.01);
    assert!((resolved.to_srgba().blue - 0.6).abs() < 0.01);
}

/// اختبار resolve_color بدون لون مخصص (لون حسب النوع)
#[test]
fn test_resolve_color_by_type() {
    // Float - يجب أن يعيد لون Float
    let float_port = PortDefinition::new("F", ValueType::Float);
    let float_color = float_port.resolve_color();
    assert!(!float_color.to_srgba().red.is_nan());

    // Int - يجب أن يعيد لون Int
    let int_port = PortDefinition::new("I", ValueType::Int);
    let int_color = int_port.resolve_color();
    assert!(!int_color.to_srgba().red.is_nan());

    // Bool - يجب أن يعيد لون Bool
    let bool_port = PortDefinition::new("B", ValueType::Bool);
    let bool_color = bool_port.resolve_color();
    assert!(!bool_color.to_srgba().red.is_nan());

    // String - يجب أن يعيد لون String
    let str_port = PortDefinition::new("S", ValueType::String);
    let str_color = str_port.resolve_color();
    assert!(!str_color.to_srgba().red.is_nan());

    // Any - يجب أن يعيد لون Any
    let any_port = PortDefinition::new("A", ValueType::Any);
    let any_color = any_port.resolve_color();
    assert!(!any_color.to_srgba().red.is_nan());
}

/// اختبار سلسلة methods (builder pattern)
#[test]
fn test_chained_methods() {
    let port = PortDefinition::input_float("ChainedPort")
        .with_description("A chained port definition")
        .with_default(NodeValue::float(10.0))
        .with_color(Color::srgb(1.0, 0.0, 0.0));

    assert_eq!(port.name, "ChainedPort");
    assert_eq!(port.value_type, ValueType::Float);
    assert_eq!(port.description, Some("A chained port definition".to_string()));
    assert_eq!(port.default_value, Some(NodeValue::float(10.0)));
    assert!(port.color.is_some());
}

/// اختبار أنواع المنافذ المختلفة تنتج ألواناً مختلفة
#[test]
fn test_different_types_different_colors() {
    let float_color = PortDefinition::input_float("F").resolve_color();
    let int_color = PortDefinition::input_int("I").resolve_color();
    let bool_color = PortDefinition::input_bool("B").resolve_color();
    let string_color = PortDefinition::input_string("S").resolve_color();

    // كل نوع يجب أن يكون له لون مميز
    // (هذا اختبار تقريبي - الألوان يجب أن تكون مختلفة)
    let colors = vec![
        float_color.to_srgba(),
        int_color.to_srgba(),
        bool_color.to_srgba(),
        string_color.to_srgba(),
    ];

    // التحقق من أن الألوان مختلفة (على الأقل في مكون واحد)
    for i in 0..colors.len() {
        for j in (i + 1)..colors.len() {
            let different = (colors[i].red - colors[j].red).abs() > 0.01
                || (colors[i].green - colors[j].green).abs() > 0.01
                || (colors[i].blue - colors[j].blue).abs() > 0.01;
            assert!(different, "Colors for different types should be different");
        }
    }
}
