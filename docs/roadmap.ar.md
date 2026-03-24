# خارطة طريق دمج التشغيل وتنظيف الإرث

## الهدف

نقل المشروع من:

- معمارية مختلطة أصبح فيها `ExecutableGraph` موجودًا، لكن ما زالت حوله طبقات انتقالية في runtime وUI

إلى:

- نموذج تنفيذ واحد يتمحور حول `ExecutableGraph`
- طبقة runtime أرفع وأنحف
- ومكدس editor وpersistence أنظف بعد إزالة المسارات القديمة

النتيجة المستهدفة هي أن تبقى `GraphDocument` و`AuthoredNodeInputs` هما الحقيقة
التحريرية، وأن تصبح `ExecutableGraph` هي الحقيقة التنفيذية، وأن تبقى أي إسقاطات
داخل ECS مجرد cache للعرض أو التكامل.

## القرارات المعتمدة قبل التنظيف

- `ExecutableGraph` هي الحقيقة التنفيذية الوحيدة.
- `GraphDocument` و`AuthoredNodeInputs` يبقيان الحقيقة التحريرية.
- `GraphNode.values` داخل ECS هي بيانات cache أو projection، وليست حالة تنفيذية معتمدة.
- `GraphConnectivityIndex` و`GraphResolvedInputs` موارد توافق انتقالية ويجب حذفها.
- التحرير عبر popup متقاعد، والتحرير inline داخل العقد هو المسار الوحيد في الواجهة.
- التحقق يجب أن يُستهلك من `graph_core` أو `LiveGraphValidationState`، لا أن يُعاد بناؤه داخل الـ adapters.
- التنظيف يجب أن يفضل الحذف والاستخدام المباشر بدل إضافة wrappers جديدة.

## مؤشرات النجاح

- لا يوجد نظام runtime يعتمد على `GraphConnectivityIndex`.
- لا يوجد نظام runtime أو UI يعتمد على `GraphResolvedInputs`.
- يختفي `node_popup.rs` و`NodePopupState` بالكامل.
- تقرأ تشخيصات runtime وscene outputs من `ExecutableGraph` مباشرة.
- يمر التحقق عبر `GraphValidationReport` و`node diagnostics` التنفيذية من دون مسارات قديمة موازية.
- يصبح السطح العام في runtime وUI وnode-graph أصغر وأوضح من الوضع الحالي.

## Phase 0: تثبيت نموذج التنظيف

- [x] حصر البنى الانتقالية المتبقية وتصنيف كل واحدة على أنها:
  - [x] truth
  - [x] cache
  - [x] adapter
  - [x] legacy
- [x] تحديث `docs/graph-core-execution-model.md` حيث تغير خطة التنظيف لغة الملكية أو الحدود
- [x] توضيح أي موارد ECS ما زالت موجودة فقط لأجل التوافق المرحلي
- [x] تحديد ترتيب الحذف حتى يتم التنظيف من دون كسر المحرر

## Phase 1: حذف موارد الإسقاط في runtime

- [x] نقل المستهلكين بعيدًا عن:
  - [x] `GraphConnectivityIndex`
  - [x] `GraphResolvedInputs`
- [x] جعل القرّاء التالية تعتمد على `GraphExecutableRuntimeState.graph` مباشرة:
  - [x] جمع scene outputs
  - [x] connection diagnostics
  - [x] اختبارات runtime التي ما زالت تفحص resolved inputs المسقطة
- [x] إضافة أي واجهات قراءة ناقصة على `ExecutableGraph` أو `GraphExecutableRuntimeState`
- [x] حذف:
  - [x] `GraphConnectivityIndex`
  - [x] `GraphResolvedInputs`
  - [x] `project_runtime_resources`

## Phase 2: ترشيق طبقة runtime adapter

- [x] إبقاء `GraphExecutableRuntimeState` مركزة على:
  - [x] `ExecutableGraph`
  - [x] الربط من entity إلى node-id
  - [x] الربط من node-id إلى entity
- [x] إزالة أي معرفة runtime مكررة موجودة أصلًا داخل `ExecutableGraph`
- [x] إبقاء `GraphRuntimeDiagnostics` كمورد عرض فقط، لا كمصدر حقيقة منافس
- [x] تدقيق أنظمة runtime بحثًا عن تكرار في:
  - [x] adjacency logic
  - [x] execution-order logic
  - [x] blocked-state logic
- [x] حذف أي منطق مكرر متبقٍ بعد نقل النسخة الأساسية أو إعادة استخدامها من core

## Phase 3: حذف إرث popup editing

- [x] حذف `crates/univis_editor_ui/src/node_popup.rs`
- [x] إزالة `NodePopupState` من:
  - [x] `NodeUiPlugin`
  - [x] أنظمة selection
  - [x] أنظمة state-sync
  - [x] حالة التنظيف داخل persistence
  - [x] اختبارات workflow وpersistence
- [x] حذف أي حالة overlay أو surface خاصة بالـ popup ولم يعد لها دور واجهي حقيقي
- [x] التأكد من أن إجراءات الإعدادات أصبحت الآن تشير إلى:
  - [x] inline editing
  - [x] inline section expand أو collapse
  - [x] أو لا شيء، إذا صار الزر نفسه قديمًا

## Phase 4: تبسيط الوصول إلى validation

- [x] نقل المستهلكين إلى `GraphValidationReport` و`node diagnostics` التنفيذية مباشرة كلما كان ذلك عمليًا
- [x] إبقاء مورد live validation والحد الأدنى فقط من glue داخل `node_graph`
- [x] حذف واجهات التوافق التي تعيد فقط تغليف نتائج validation القادمة من core
- [x] تدقيق persistence وUI والاختبارات بحثًا عن أي وصول قديم من نمط `Vec<GraphValidationIssue>`
- [x] تفضيل مسار تحقق واحد عبر:
  - [x] editor
  - [x] persistence
  - [x] runtime

## Phase 5: حسم حدود cache داخل ECS

- [x] توضيح ذلك في الكود والأسماء: `GraphNode.values` هي بيانات projection
- [x] تدقيق الأنظمة التي تقرأ `GraphNode.values` لاتخاذ القرار
- [x] نقل قراءات القرار إلى:
  - [x] `AuthoredNodeInputs`
  - [x] `ExecutableGraph`
- [x] إبقاء إسقاطات ECS فقط عندما تكون مطلوبة لأجل:
  - [x] rendering
  - [x] widgets
  - [x] debug أو inspector output
- [x] إعادة تسمية helpers أو الحقول إذا لزم الأمر لتقليل الغموض

## Phase 6: تنظيف persistence وapply

- [x] إزالة افتراضات التنظيف الخاصة بالـ popup من مسارات persistence
- [x] إعادة استخدام document signatures وvalidation reports من المصادر الموحدة فقط
- [x] تدقيق فروع load وapply التي ما زالت موجودة فقط لأجل التوافق المرحلي
- [x] الإبقاء على هجرة save-file القديمة فقط حيث ما زالت تخدم payloads قديمة فعلًا
- [x] حذف الفروع التي أصبحت قديمة بعد توحيد runtime التنفيذي

## Phase 7: تضييق السطح العام وحذف الكود الميت

- [x] حذف exports وwrappers وhelpers غير المستخدمة
- [x] تقليم prelude exports التي لم تعد تمثل المعمارية المعتمدة
- [x] حذف الإشارات التوثيقية القديمة إلى الأنظمة المحذوفة
- [x] إعادة كتابة الاختبارات الخاصة بطبقات التوافق المحذوفة أو حذفها
- [x] إبقاء الـ public API المتبقية صغيرة ومقصودة

## Phase 8: تغطية الانحدار للشكل الجديد

- [x] إضافة اختبارات موجّهة لـ:
  - [x] قراءة runtime لحقيقة التنفيذ مباشرة من `ExecutableGraph`
  - [x] scene outputs من دون `GraphResolvedInputs`
  - [x] UI diagnostics من دون `GraphConnectivityIndex`
  - [x] تشغيل editor وpersistence من دون موارد popup
  - [x] انتقال تعديل authored input عبر التنفيذ حتى الواجهة أو world state
- [x] إبقاء الاختبارات متمحورة حول الضمانات المعمارية، لا مجرد smoke behavior

## صيانة الوثائق

- [x] إبقاء `docs/roadmap.ar.md` و`docs/roadmap.md` متطابقتين في الترتيب والمضمون
- [x] تحديث `changelog.md` مع هبوط كل مرحلة من مراحل التنظيف
- [x] مراجعة الوثائق المساندة وأرشفة ما تغطيه هذه الخارطة بالكامل فقط
