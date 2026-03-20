# خارطة طريق تنفيذ `graph_core`

## الهدف

تحويل `graph_core` من:

- نواة توصيف وتحقق

إلى:

- الحقيقة التحريرية للغراف
- والحقيقة التنفيذية للغراف

النتيجة المستهدفة هي أن تبقى `GraphDocument` هي authored truth، بينما تصبح
`ExecutableGraph` هي execution truth.

## القرارات المعتمدة قبل التنفيذ

- سطح `ExecutableGraph` لا يحمل `S` كجزء من الواجهة العامة.
- الـ schema تبقى داخل:
  - `GraphNodeRegistry`
  - `PortDefinition`
  - مسارات `build` و`validation`
- البناء من document يجب أن يكون متسامحًا:
  - يبني ما يمكن بناؤه
  - ولا يسقط كل graph على أول مشكلة
- ناتج البناء ليس `Result` بسيطًا، بل تقرير بناء صريح.
- API التنفيذ في البداية داخلية وتجريبية، وليست واجهة عامة مستقرة.

## Build Semantics

- يحدث التحقق قبل البناء أو أثناءه
- قد ينتج البناء graph تنفيذية جزئية
- `node diagnostics` هي الحقيقة الأساسية على مستوى كل node لحالات الحجب أو التدهور أثناء البناء

## Phase 0: تثبيت نموذج التنفيذ

- [x] إنشاء ملف: `docs/graph-core-execution-model.md`
- [x] تعريف الفرق بين:
  - [x] `GraphDocument`
  - [x] `ExecutableGraph`
  - [x] `ExecutableNode`
- [x] تحديد ownership لكل من:
  - [x] `authored_inputs`
  - [x] `resolved_inputs`
  - [x] `outputs`
- [x] تحديد ownership لكل من:
  - [x] `edges`
  - [x] `direct links`
  - [x] `execution state`
- [x] توضيح أن:
  - [x] `GraphDocument` ليست runtime graph
  - [x] `ExecutableGraph` هي البنية المشتقة للتنفيذ

## Phase 1: إدخال الطبقة التنفيذية

- [x] إنشاء ملف: `crates/univis_graph_core/src/executable.rs`
- [x] تعريف:
  - [x] `ExecutableGraph<Value>`
  - [x] `ExecutableNode<Value>`
  - [x] `NodeExecutionState`
- [x] إضافة بنية علاقات مباشرة:
  - [x] incoming links
  - [x] outgoing links
- [x] إضافة البيانات الأساسية لكل node:
  - [x] `node_id`
  - [x] `definition_id`
  - [x] `authored_inputs`
  - [x] `resolved_inputs`
  - [x] `outputs`
- [x] إضافة حالة التنفيذ الأساسية:
  - [x] `enabled`
  - [x] `dirty`
  - [x] `blocked`
  - [x] `ready`
  - [x] `last_result`
  - [x] `last_run_revision`

## Phase 2: البناء من `GraphDocument`

- [x] تعريف API بناء صريحة مثل:
  - [x] `ExecutableGraph::build(document, registry)`
- [x] تمرير:
  - [x] `&GraphDocument`
  - [x] `&GraphNodeRegistry`
- [x] تعريف:
  - [x] `ExecutableGraphBuildReport<Value>`
  - [x] `ExecutableNodeDiagnostic`
  - [x] `ExecutableNodeBuildStatus`
  - [x] `ExecutableNodeBlockReason`

### شكل تقرير البناء المستهدف

- [x] يحتوي `ExecutableGraphBuildReport` على:
  - [x] `graph`
  - [x] `validation_report`
  - [x] `node_diagnostics`
  - [x] `is_partial`
- [x] لا يعتمد التقرير على فصل:
  - [x] `issues`
  - [x] `blocked_node_ids`
- [x] بدل ذلك، يملك لكل node:
  - [x] status واضح
  - [x] reasons صريحة للحجب أو التقييد

### مسؤوليات البناء

- [x] تحويل:
  - [x] `GraphDocumentNode -> ExecutableNode`
  - [x] `GraphDocumentEdge -> direct links`
- [x] ربط المنافذ باستخدام definitions من `registry`
- [x] تهيئة:
  - [x] `authored_inputs`
  - [x] buffers الابتدائية لـ `resolved_inputs`
  - [x] buffers الابتدائية لـ `outputs`
- [x] تسجيل الحالات التالية داخل التقرير:
  - [x] missing definitions
  - [x] invalid ports
  - [x] cycles / blocked topology
  - [x] type incompatibility
  - [x] requirement mismatch

### قاعدة البناء

- [x] البناء متسامح:
  - [x] لا يفشل كليًا على أول خطأ
  - [x] يسجل ما بُني
  - [x] ويسجل ما حُجب ولماذا

## Phase 3: تثبيت فصل البيانات التنفيذية

- [x] داخل `ExecutableNode`:
  - [x] تثبيت الفصل بين `authored_inputs`
  - [x] تثبيت الفصل بين `resolved_inputs`
  - [x] تثبيت الفصل بين `outputs`
- [x] تعريف flow صريح:
  - [x] كيف تتحول `authored_inputs` إلى `resolved_inputs`
- [x] منع:
  - [x] أي خلط بين authored وresolved
- [x] ضمان:
  - [x] أن `outputs` لا تعدل authored state
- [x] تعريف:
  - [x] ready condition
  - [x] blocked condition

## Phase 4: API تنفيذ داخلية وتجريبية

> هذه API داخلية وغير مستقرة في هذه المرحلة.

- [x] إضافة API داخل `ExecutableGraph`:
  - [x] `enable_node(node_id)`
  - [x] `disable_node(node_id)`
  - [x] `mark_dirty(node_id)`
  - [x] `set_authored_input(node_id, index, value)`
  - [x] `resolve_inputs(node_id)`
  - [x] `run_node(node_id)`
  - [x] `run_ready_nodes()`
  - [x] `run_from(node_id)`
- [x] إضافة واجهات قراءة:
  - [x] `get_outputs(node_id)`
  - [x] `get_upstream(node_id)`
  - [x] `get_downstream(node_id)`

## Phase 5: Dirty Propagation

- [x] تعريف dirty propagation model
- [x] عند تغير authored input أو external mutation:
  - [x] mark node dirty
- [x] نشر dirty إلى:
  - [x] downstream nodes
- [x] تنفيذ:
  - [x] ready nodes فقط
- [x] بعد التنفيذ:
  - [x] مقارنة outputs السابقة والجديدة
- [x] إذا لم تتغير outputs:
  - [x] إيقاف propagation
- [x] إذا تغيرت outputs:
  - [x] متابعة propagation

## Phase 6: إنزال منطق التنفيذ إلى core

- [x] نقل:
  - [x] adjacency logic
  - [x] propagation
  - [x] ready / blocked logic
  - [x] scheduling
  - [x] execution traversal
- [x] إبقاء خارج core:
  - [x] Bevy ECS
  - [x] world mutation
  - [x] rendering
  - [x] UI
- [x] جعل runtime الأعلى:
  - [x] adapter فوق core

## Phase 7: ربط validation بالتنفيذ

- [x] جعل `validation_report` جزءًا من build readiness
- [x] ربط:
  - [x] blocked node diagnostics ← validation causes
  - [x] topology ← seed لترتيب التنفيذ
- [x] تمكين core من الإجابة عن:
  - [x] هل graph قابلة للتنفيذ؟
  - [x] ما هي النود المحجوبة؟
  - [x] لماذا هي محجوبة؟

## Phase 8: نموذج الأداء

- [ ] إنشاء ملف: `docs/graph-core-performance-model.md`
- [ ] تعريف:
  - [ ] ما يُخزن دائمًا في `ExecutableGraph`
  - [ ] ما يُعاد بناؤه فقط عند تغير البنية
  - [ ] ما الذي يسبب dirty
  - [ ] متى تعتبر outputs متغيرة
  - [ ] تكلفة العمليات الأساسية
- [ ] تحديد:
  - [ ] متى نعيد build
  - [ ] متى نعيد execution

## Phase 9: اختبارات التنفيذ

- [ ] اختبار:
  - [ ] build من document
  - [ ] direct linking
  - [ ] node diagnostics
  - [ ] dirty propagation
  - [ ] disabled nodes
  - [ ] blocked nodes
  - [ ] partial execution
  - [ ] unchanged output short-circuit

## صيانة الوثائق

- [ ] إبقاء `docs/roadmap.ar.md` و`docs/roadmap.md` متطابقتين في الترتيب والمضمون
- [ ] إبقاء القرارات المعتمدة في أعلى الخارطة محدثة عند أي تغيير في التنفيذ
