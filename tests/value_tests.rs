//! اختبارات NodeValue

use univis_editor::core::value::NodeValue;
use bevy::prelude::{Vec2, Vec3};

// ═════════════════════════════════════════════════════
// اختبارات إنشاء القيم
// ═════════════════════════════════════════════════════

#[test]
fn test_float_creation() {
    let value = NodeValue::float(3.14);
    assert_eq!(value.as_float(), Some(3.14));
}

#[test]
fn test_int_creation() {
    let value = NodeValue::int(42);
    assert_eq!(value.as_int(), Some(42));
}

#[test]
fn test_bool_creation() {
    let value = NodeValue::bool(true);
    assert_eq!(value.as_bool(), Some(true));
}

#[test]
fn test_string_creation() {
    let value = NodeValue::string("Hello");
    assert_eq!(value.as_string(), Some("Hello"));
}

#[test]
fn test_vec2_creation() {
    let value = NodeValue::Vec2(Vec2::new(1.0, 2.0));
    let vec = value.as_vec2();
    assert!(vec.is_some());
    let v = vec.unwrap();
    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
}

#[test]
fn test_vec3_creation() {
    let value = NodeValue::Vec3(Vec3::new(1.0, 2.0, 3.0));
    let vec = value.as_vec3();
    assert!(vec.is_some());
    let v = vec.unwrap();
    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
    assert_eq!(v.z, 3.0);
}

// ═════════════════════════════════════════════════════
// اختبارات التحويلات
// ═════════════════════════════════════════════════════

#[test]
fn test_float_to_int_conversion() {
    let value = NodeValue::float(3.14);
    assert_eq!(value.as_int(), Some(3)); // 3.14 -> 3
}

#[test]
fn test_int_to_float_conversion() {
    let value = NodeValue::int(42);
    assert_eq!(value.as_float(), Some(42.0));
}

#[test]
fn test_bool_as_string_returns_none() {
    let value = NodeValue::bool(true);
    assert_eq!(value.as_string(), None);
}

#[test]
fn test_none_value() {
    let value = NodeValue::None;
    assert_eq!(value.as_float(), None);
    assert_eq!(value.as_int(), None);
    assert_eq!(value.as_bool(), None);
    assert_eq!(value.as_string(), None);
}

// ═════════════════════════════════════════════════════
// اختبارات Clone
// ═════════════════════════════════════════════════════

#[test]
fn test_clone_float() {
    let original = NodeValue::float(42.0);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_clone_string() {
    let original = NodeValue::string("Test");
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ═════════════════════════════════════════════════════
// اختبارات Default
// ═════════════════════════════════════════════════════

#[test]
fn test_default_is_none() {
    let value = NodeValue::default();
    assert!(matches!(value, NodeValue::None));
}

// ═════════════════════════════════════════════════════
// اختبارات is_none
// ═════════════════════════════════════════════════════

#[test]
fn test_is_none() {
    assert!(NodeValue::None.is_none());
    assert!(!NodeValue::float(0.0).is_none());
    assert!(!NodeValue::int(0).is_none());
    assert!(!NodeValue::bool(false).is_none());
    assert!(!NodeValue::string("").is_none());
}
