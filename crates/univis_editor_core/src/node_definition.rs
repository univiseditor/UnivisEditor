//! تعريف العُقد - النظام القابل للتوسع
//! كل عقدة هي تنفيذ لـ NodeDefinition trait

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::any::Any;
use std::sync::Arc;

use super::value::{EntityValue, NodeValue, NodeValues, ValueType};

/// معرف فريد للعقدة
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// تصنيف العُقد
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeCategory(pub String);

impl NodeCategory {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// تصنيفات مدمجة
    pub const MATH: &'static str = "Math";
    pub const LOGIC: &'static str = "Logic";
    pub const INPUT: &'static str = "Input";
    pub const OUTPUT: &'static str = "Output";
    pub const ADVANCED: &'static str = "Advanced";
}

/// تعريف المنفذ (مدخل أو مخرج)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDefinition {
    /// اسم المنفذ للعرض
    pub name: String,
    /// نوع البيانات المتوقع
    pub value_type: ValueType,
    /// وصف اختياري
    pub description: Option<String>,
    /// قيمة افتراضية (للمداخل)
    pub default_value: Option<NodeValue>,
    /// لون مخصص للمنفذ (اختياري)
    pub color: Option<Color>,
    /// هل يظهر هذا المدخل للتحرير داخل نافذة الإعدادات المنبثقة؟
    #[serde(default)]
    pub editable_in_popup: bool,
    /// خطوة التعديل في UI (للأرقام)
    #[serde(default)]
    pub ui_step: Option<f64>,
    /// الحد الأدنى في UI
    #[serde(default)]
    pub ui_min: Option<f64>,
    /// الحد الأقصى في UI
    #[serde(default)]
    pub ui_max: Option<f64>,
}

impl PortDefinition {
    pub fn new(name: impl Into<String>, value_type: ValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
            description: None,
            default_value: None,
            color: None,
            editable_in_popup: false,
            ui_step: None,
            ui_min: None,
            ui_max: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_default(mut self, value: NodeValue) -> Self {
        self.default_value = Some(value);
        self
    }

    /// مع لون مخصص للمنفذ
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn editable_in_popup(mut self) -> Self {
        self.editable_in_popup = true;
        self
    }

    pub fn with_ui_step(mut self, step: f64) -> Self {
        self.ui_step = Some(step);
        self
    }

    pub fn with_ui_min(mut self, min: f64) -> Self {
        self.ui_min = Some(min);
        self
    }

    pub fn with_ui_max(mut self, max: f64) -> Self {
        self.ui_max = Some(max);
        self
    }

    pub fn with_ui_range(mut self, min: f64, max: f64) -> Self {
        self.ui_min = Some(min);
        self.ui_max = Some(max);
        self
    }

    /// الحصول على لون المنفذ (مخصص أو حسب النوع)
    pub fn resolve_color(&self) -> Color {
        self.color.unwrap_or_else(|| self.value_type.port_color())
    }

    /// منفذ مدخل float
    pub fn input_float(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Float)
    }

    /// منفذ مدخل int
    pub fn input_int(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Int)
    }

    /// منفذ مدخل bool
    pub fn input_bool(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Bool)
    }

    /// منفذ مدخل string
    pub fn input_string(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::String)
    }

    /// منفذ مدخل vec3
    pub fn input_vec3(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Vec3)
    }

    /// منفذ مدخل entity
    pub fn input_entity(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Entity)
    }

    /// منفذ مدخل بعلامة نوع مخصصة
    pub fn input_tag(name: impl Into<String>, tag: impl Into<String>) -> Self {
        Self::new(name, ValueType::CustomTag(tag.into()))
    }

    /// منفذ مخرج float
    pub fn output_float(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Float)
    }

    /// منفذ مخرج any
    pub fn output_any(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Any)
    }

    /// منفذ مخرج entity
    pub fn output_entity(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Entity)
    }

    /// منفذ مخرج بعلامة نوع مخصصة
    pub fn output_tag(name: impl Into<String>, tag: impl Into<String>) -> Self {
        Self::new(name, ValueType::CustomTag(tag.into()))
    }
}

/// سياق المعالجة للعقدة
pub struct ProcessContext<'a> {
    /// قيم المداخل
    pub inputs: &'a [NodeValue],
    /// قيم المخارج (للكتابة)
    pub outputs: &'a mut [NodeValue],
    /// وقت الإطار
    pub delta_time: f32,
    /// بيانات مخصصة للعقدة
    pub custom_data: &'a mut Option<Box<dyn Any + Send + Sync>>,
}

impl<'a> ProcessContext<'a> {
    // ═════════════════════════════════════════════════════
    // Helper Methods - لتبسيط الوصول للقيم
    // ═════════════════════════════════════════════════════

    /// قراءة float من مدخل
    pub fn get_float(&self, index: usize) -> Option<f64> {
        self.inputs.get(index)?.as_float()
    }

    /// قراءة float مع قيمة افتراضية
    pub fn get_float_or(&self, index: usize, default: f64) -> f64 {
        self.get_float(index).unwrap_or(default)
    }

    /// قراءة int من مدخل
    pub fn get_int(&self, index: usize) -> Option<i64> {
        self.inputs.get(index)?.as_int()
    }

    /// قراءة int مع قيمة افتراضية
    pub fn get_int_or(&self, index: usize, default: i64) -> i64 {
        self.get_int(index).unwrap_or(default)
    }

    /// قراءة bool من مدخل
    pub fn get_bool(&self, index: usize) -> Option<bool> {
        self.inputs.get(index)?.as_bool()
    }

    /// قراءة bool مع قيمة افتراضية
    pub fn get_bool_or(&self, index: usize, default: bool) -> bool {
        self.get_bool(index).unwrap_or(default)
    }

    /// قراءة Vec2 من مدخل
    pub fn get_vec2(&self, index: usize) -> Option<Vec2> {
        self.inputs.get(index)?.as_vec2()
    }

    /// قراءة Vec3 من مدخل
    pub fn get_vec3(&self, index: usize) -> Option<Vec3> {
        self.inputs.get(index)?.as_vec3()
    }

    /// قراءة String من مدخل
    pub fn get_string(&self, index: usize) -> Option<&str> {
        self.inputs.get(index)?.as_string()
    }

    /// قراءة قيمة TaggedData من مدخل
    pub fn get_tagged(&self, index: usize) -> Option<(&str, &JsonValue)> {
        self.inputs.get(index)?.as_tagged()
    }

    /// قراءة EntityValue من مدخل
    pub fn get_entity(&self, index: usize) -> Option<&EntityValue> {
        self.inputs.get(index)?.as_entity()
    }

    /// كتابة مخرج
    pub fn set(&mut self, index: usize, value: NodeValue) {
        if let Some(out) = self.outputs.get_mut(index) {
            *out = value;
        }
    }

    /// كتابة float في مخرج
    pub fn set_float(&mut self, index: usize, value: f64) {
        self.set(index, NodeValue::float(value));
    }

    /// كتابة int في مخرج
    pub fn set_int(&mut self, index: usize, value: i64) {
        self.set(index, NodeValue::int(value));
    }

    /// كتابة bool في مخرج
    pub fn set_bool(&mut self, index: usize, value: bool) {
        self.set(index, NodeValue::bool(value));
    }

    /// كتابة Vec2 في مخرج
    pub fn set_vec2(&mut self, index: usize, value: Vec2) {
        self.set(index, NodeValue::Vec2(value));
    }

    /// كتابة Vec3 في مخرج
    pub fn set_vec3(&mut self, index: usize, value: Vec3) {
        self.set(index, NodeValue::Vec3(value));
    }

    /// كتابة String في مخرج
    pub fn set_string(&mut self, index: usize, value: impl Into<String>) {
        self.set(index, NodeValue::string(value));
    }

    /// كتابة TaggedData في مخرج
    pub fn set_tagged(&mut self, index: usize, tag: impl Into<String>, payload: JsonValue) {
        self.set(index, NodeValue::tagged(tag, payload));
    }

    /// كتابة EntityValue في مخرج
    pub fn set_entity(&mut self, index: usize, value: EntityValue) {
        self.set(index, NodeValue::entity(value));
    }
}

/// نتيجة المعالجة
#[derive(Debug)]
pub enum ProcessResult {
    /// تمت المعالجة بنجاح
    Success,
    /// خطأ مع رسالة
    Error(String),
    /// العقدة تحتاج لمزيد من المداخل
    MissingInput(usize),
}

/// الـ trait الرئيسي لتعريف العُقد
pub trait NodeDefinition: Send + Sync {
    /// المعرف الفريد للعقدة (مثل "math/add")
    fn id(&self) -> NodeId;

    /// اسم العرض للعقدة
    fn display_name(&self) -> &str;

    /// التصنيف
    fn category(&self) -> NodeCategory {
        NodeCategory::new(NodeCategory::ADVANCED)
    }

    /// وصف العقدة
    fn description(&self) -> Option<&str> {
        None
    }

    /// لون العقدة
    fn color(&self) -> Color {
        Color::srgb(0.2, 0.2, 0.25)
    }

    /// لون العنوان (اختياري)
    fn title_color(&self) -> Color {
        Color::WHITE
    }

    /// لون الأسلاك المتصلة بالعقدة (اختياري)
    fn wire_color(&self) -> Color {
        self.color()
    }

    /// أيقونة العقدة (اختياري)
    fn icon(&self) -> Option<&str> {
        None
    }

    /// تعريف المداخل
    fn inputs(&self) -> Vec<PortDefinition>;

    /// تعريف المخارج
    fn outputs(&self) -> Vec<PortDefinition>;

    /// دالة المعالجة الرئيسية
    fn process(&self, context: &mut ProcessContext) -> ProcessResult;

    /// ═══════════════════════════════════════════════════════════
    /// 🎯 جديد: التسجيل التلقائي
    /// ═══════════════════════════════════════════════════════════

    /// هل تريد تسجيل هذه العقدة تلقائياً في القائمة؟
    /// ابحث عن true افتراضياً - معظم العُقد تظهر في القائمة
    fn show_in_menu(&self) -> bool {
        true
    }

    /// ترتيب العرض في القائمة (أقل = أولاً)
    /// مفيد لترتيب العُقد داخل التصنيف
    fn menu_order(&self) -> i32 {
        0
    }

    /// كلمات مفتاحية للبحث
    fn keywords(&self) -> Vec<&str> {
        vec![]
    }

    /// ═══════════════════════════════════════════════════════════

    /// هل يمكن للعقدة أن يكون لها أطفال؟
    fn can_have_children(&self) -> bool {
        false
    }

    /// الحصول على القيم الافتراضية للمداخل
    fn default_input_values(&self) -> Vec<NodeValue> {
        self.inputs()
            .iter()
            .map(|p| p.default_value.clone().unwrap_or(NodeValue::None))
            .collect()
    }

    /// الحصول على ألوان المنافذ (باستخدام resolve_color)
    fn input_port_colors(&self) -> Vec<Color> {
        self.inputs().iter().map(|p| p.resolve_color()).collect()
    }

    fn output_port_colors(&self) -> Vec<Color> {
        self.outputs().iter().map(|p| p.resolve_color()).collect()
    }

    /// عرض الـ body (اختياري - للتخصيص)
    fn body_width(&self) -> f32 {
        150.0
    }

    /// ارتفاع الـ body (اختياري - للتخصيص)
    fn body_height(&self) -> f32 {
        let ins = self.inputs().len();
        let outs = self.outputs().len();
        50.0 + (ins.max(outs) as f32 * 22.0)
    }

    /// بناء محتوى جسم العقدة المخصص
    /// هذه الدالة تُستدعى بعد إنشاء الهيكل الأساسي للعقدة
    ///
    /// # المعاملات
    /// - `body`: أوامر إنشاء العناصر الأبناء للـ body
    /// - `node_entity`: كيان العقدة الرئيسي (للرجوع إليه)
    ///
    /// # مثال
    /// ```ignore
    /// fn build_body(&self, body: &mut ChildSpawnerCommands, node_entity: Entity) {
    ///     body.spawn(UTextLabel {
    ///         text: "Custom Content".to_string(),
    ///         font_size: 12.0,
    ///         color: Color::WHITE,
    ///         ..default()
    ///     });
    /// }
    /// ```
    fn build_body(&self, _body: &mut ChildSpawnerCommands, _node_entity: Entity) {
        // التنفيذ الافتراضي: لا يفعل شيئاً
        // العُقد يمكنها تجاوز هذه الدالة لإضافة محتوى مخصص
    }

    /// هل تحتاج العقدة لـ body مخصص؟
    /// إذا كانت true، سيتم استدعاء build_body
    fn has_custom_body(&self) -> bool {
        false
    }
}

/// تعريف العقدة كـ Arc للتخزين الآمن
pub type ArcNodeDefinition = Arc<dyn NodeDefinition>;

/// مكون لعقدة في المشهد
#[derive(Component)]
pub struct GraphNode {
    /// معرف تعريف العقدة
    pub definition_id: NodeId,
    /// قيم المداخل والمخارج
    pub values: NodeValues,
    /// بيانات مخصصة
    pub custom_data: Option<Box<dyn Any + Send + Sync>>,
}

impl GraphNode {
    pub fn new(definition_id: NodeId, input_count: usize, output_count: usize) -> Self {
        Self {
            definition_id,
            values: NodeValues::new(input_count, output_count),
            custom_data: None,
        }
    }
}

/// مكون لمنفذ العقدة
#[derive(Component)]
pub struct GraphPort {
    /// الكيان الأب (العقدة)
    pub node_entity: Entity,
    /// نوع المنفذ (مدخل/مخرج)
    pub port_type: PortType,
    /// فهرس المنفذ
    pub index: usize,
    /// نوع البيانات
    pub value_type: ValueType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    Input,
    Output,
}

/// ✨ جديد: اتصال محلي للمدخل (يعرف مصدره مباشرة)
#[derive(Component, Default)]
pub struct InputConnection {
    /// الكيان المصدر (العقدة المُرسلة)
    pub source_node: Option<Entity>,
    /// فهرس المخرج في العقدة المصدر
    pub source_port_index: Option<usize>,
}

/// ✨ جديد: اتصالات محلية للمخرج (يعرف المستقبلين مباشرة)
#[derive(Component, Default)]
pub struct OutputConnections {
    /// قائمة المستقبلين (العقدة المستقبلة + فهرس المدخل)
    pub targets: Vec<OutputTarget>,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputTarget {
    pub target_node: Entity,
    pub target_port_index: usize,
}

/// ═══════════════════════════════════════════════════════════════
/// 🎯 تصميم على طريقة Blender
/// ═══════════════════════════════════════════════════════════════
///
/// في Blender، كل Socket (منفذ) له:
/// 1. مؤشر مباشر للقيمة (value)
/// 2. مؤشر مباشر للاتصال (link)
///
/// هذا يسمح بـ O(1) للوصول للبيانات - لا بحث!
///
/// هنا نطبق نفس المبدأ:
/// - InputPort يعرف مصدره مباشرة (مثل Blender link)
/// - OutputPort يعرف مستقبليه مباشرة
/// - لا حاجة للبحث في Connecting Resource
///
/// ═══════════════════════════════════════════════════════════════

/// منفذ مدخل - مثل bNodeSocket في Blender
#[derive(Component)]
pub struct InputPort {
    /// العقدة الأب
    pub node_entity: Entity,
    /// فهرس المنفذ
    pub index: usize,
    /// نوع البيانات
    pub value_type: ValueType,

    /// 🎯 المؤشر المباشر للمصدر (مثل Blender link)
    /// إذا كان Some، القيمة تأتي من هنا مباشرة
    pub source: Option<PortRef>,
}

/// منفذ مخرج - مثل bNodeSocket في Blender
#[derive(Component)]
pub struct OutputPort {
    /// العقدة الأب
    pub node_entity: Entity,
    /// فهرس المنفذ
    pub index: usize,
    /// نوع البيانات
    pub value_type: ValueType,

    /// 🎯 القيمة الحالية (مخزنة هنا مباشرة)
    pub value: NodeValue,

    /// المستقبلون (للنشر السريع)
    pub targets: Vec<PortRef>,
}

/// مرجع لمنفذ (مثل المؤشر في C)
#[derive(Debug, Clone, Copy)]
pub struct PortRef {
    pub node: Entity,
    pub port_index: usize,
}

impl InputPort {
    /// الحصول على القيمة - O(1) مثل Blender!
    pub fn get_value(
        &self,
        outputs: &std::collections::HashMap<Entity, Vec<NodeValue>>,
    ) -> NodeValue {
        if let Some(ref source) = self.source {
            // وصول مباشر - لا بحث!
            if let Some(node_outputs) = outputs.get(&source.node) {
                if source.port_index < node_outputs.len() {
                    return node_outputs[source.port_index].clone();
                }
            }
        }
        NodeValue::None
    }
}

/// مكون لتتبع العُقد المحددة
#[derive(Component)]
pub struct Selected;

/// مكون لربط نص العرض داخل عقدة Output/View بالعقدة الأم
#[derive(Component)]
pub struct ValueDisplayLabel {
    pub node_entity: Entity,
}

/// مكون لتتبع حالة السحب
#[derive(Component)]
pub struct Dragging;
