# تحليل `univis_graph_core`

هذا الملف يلخص مراجعة static لـ `crates/univis_graph_core` داخل الـ workspace الحالي، مع التركيز على:

- مدى استغلاله من طرف الـ crates الأخرى
- الأجزاء المستغلة جيداً والأجزاء المستغلة جزئياً
- التكرار المعماري أو المنطقي حوله
- أهم نقاط الانحراف التي تستحق قراراً تصميمياً واضحاً

تم إعداد هذا التقرير بدون تشغيل أي `test`.

## مراجع مرتبطة

- خارطة الجودة الحالية: [`roadmap.ar.md`](./roadmap.ar.md)
- النسخة الإنجليزية المختصرة من الخارطة: [`roadmap.md`](./roadmap.md)
- خريطة الحدود الرسمية وتصنيف الواجهة العامة: [`graph-core-boundaries.md`](./graph-core-boundaries.md)

## الخلاصة التنفيذية

`univis_graph_core` هو العمود الفقري المعماري لمنظومة الـ graph داخل المشروع، لكنه ليس مستغلاً 100% كواجهة عامة.

الاستغلال الحالي يمكن وصفه كالتالي:

- الاستغلال الوظيفي مرتفع
- الاستغلال المباشر للواجهة العامة متوسط إلى جيد
- هناك تكرار واضح في طبقة `univis_node_graph`
- توجد فجوة مهمة بين ما تسمح به بعض APIات `graph_core` نظرياً وما يدعمه التطبيق فعلياً

بشكل عملي:

- `document`, `validation`, و `ExecutableGraph` مستغلة بقوة
- عدة أنواع منخفضة المستوى في التنفيذ public أكثر من الحاجة الحالية
- بعض القواعد المعمارية مكررة بين `graph_core`, `editor_ui`, و `editor_persistence`
- `ConnectionPolicy::Multiple` غير مكتمل end-to-end

## خريطة الاعتماد

الـ crates التي تعتمد مباشرة على `univis_graph_core`:

- `crates/univis_node_graph`
- `crates/univis_editor_runtime`
- `crates/univis_editor_ui`
- `crates/univis_editor_persistence`
- `crates/univis_editor_app`

أهم ما يستهلكه كل جزء:

### 1. `univis_node_graph`

هو المستهلك الأكبر فعلياً، لأنه يمثل طبقة adapter فوق `graph_core`.

يعتمد على `graph_core` من أجل:

- أنواع الوثيقة `GraphDocument` عبر aliases في [`../crates/univis_node_graph/src/document/types.rs`](../crates/univis_node_graph/src/document/types.rs)
- schema والـ processing contracts في [`../crates/univis_node_graph/src/node_definition.rs`](../crates/univis_node_graph/src/node_definition.rs)
- registry الأساسية في [`../crates/univis_node_graph/src/node_registry.rs`](../crates/univis_node_graph/src/node_registry.rs)
- validation report في [`../crates/univis_node_graph/src/graph_validation.rs`](../crates/univis_node_graph/src/graph_validation.rs)

هذا يعني أن أغلب crates الأعلى في المشروع تستهلك `graph_core` بشكل غير مباشر عبر `univis_node_graph`.

### 2. `univis_editor_runtime`

يستغل جزء التنفيذ الحقيقي من `graph_core`:

- يبني `ExecutableGraph` من snapshots في [`../crates/univis_editor_runtime/src/connectivity.rs`](../crates/univis_editor_runtime/src/connectivity.rs)
- يشغل العقد عبر `run_ready_nodes` في [`../crates/univis_editor_runtime/src/diagnostics.rs`](../crates/univis_editor_runtime/src/diagnostics.rs)
- يقرأ `resolved_inputs` و `outputs` لتوليد scene outputs في [`../crates/univis_editor_runtime/src/scene_outputs.rs`](../crates/univis_editor_runtime/src/scene_outputs.rs)

### 3. `univis_editor_persistence`

يستخدم validation من `graph_core` مباشرة عند:

- تجهيز ملفات الحفظ
- التحميل
- تطبيق document جديدة

الملفات الأساسية:

- [`../crates/univis_editor_persistence/src/format.rs`](../crates/univis_editor_persistence/src/format.rs)
- [`../crates/univis_editor_persistence/src/graph_persistence/io.rs`](../crates/univis_editor_persistence/src/graph_persistence/io.rs)
- [`../crates/univis_editor_persistence/src/graph_persistence/apply.rs`](../crates/univis_editor_persistence/src/graph_persistence/apply.rs)

### 4. `univis_editor_ui`

استهلاكه المباشر محدود نسبياً. أهم استعمال مباشر هو:

- `connected_input_mask`
- `would_create_cycle`

وهذا يظهر في [`../crates/univis_editor_ui/src/wire.rs`](../crates/univis_editor_ui/src/wire.rs)

### 5. `univis_editor_app`

يستهلك `GraphValidationIssue` للعرض والتشخيص فقط، في:

- [`../crates/univis_editor_app/src/panels.rs`](../crates/univis_editor_app/src/panels.rs)

## تقييم الاستغلال حسب الوحدات

### `document`

هذه أكثر وحدة مستغلة بشكل متوازن.

المستعمل فعلياً:

- `GraphDocument`
- `GraphDocumentNode`
- `GraphDocumentEdge`
- `GraphDocumentPrefab`
- `GraphDocumentSubgraph`
- `selected_subgraph_boundary_summary`
- `capture_selected_subgraph`
- `merged_with_subgraph_instance`

أثر ذلك واضح في:

- workflows الخاصة بالـ prefabs/subgraphs في [`../crates/univis_editor_workflows/src/capture.rs`](../crates/univis_editor_workflows/src/capture.rs)
- instantiation في [`../crates/univis_editor_workflows/src/instantiate.rs`](../crates/univis_editor_workflows/src/instantiate.rs)
- clipboard merge/copy في [`../crates/univis_editor_workflows/src/clipboard.rs`](../crates/univis_editor_workflows/src/clipboard.rs)

النتيجة:

- هذه الوحدة مستغلة جيداً
- وجودها داخل `graph_core` مبرر وواضح

### `validation`

مستغلة جيداً أيضاً، خصوصاً عبر:

- `validate_graph_document`
- `GraphValidationReport`

لكن هناك فرق بين الاستغلال الكامل والاستغلال الجزئي:

- الاستعمال المركزي قوي
- بعض APIات validation العامة لا تُستعمل مباشرة خارج أمثلة `graph_core`

الأكثر وضوحاً:

- `validate_graph_document_structure`

هذه الدالة public، لكن لا تظهر كجزء من التدفق الفعلي لبقية الـ crates.

### `executable`

مستغلة وظيفياً، لكن ليس بكل عمق واجهتها العامة.

المستخدم فعلياً:

- `ExecutableGraph::build`
- `run_ready_nodes`
- `replace_authored_inputs`
- `replace_custom_data`
- `sync_external_outputs`
- `node_diagnostics`
- الوصول إلى `resolved_inputs` و `outputs`

لكن توجد أنواع منخفضة المستوى لا يظهر لها استهلاك خارجي واضح كـ API مستقلة، مثل:

- `ExecutableDirectLinks`
- `ExecutablePortRef`
- `NodeExecutionState`
- أجزاء من `ExecutableGraphBuildReport`

هذا لا يعني أنها بلا قيمة، بل يعني أن الواجهة العامة أوسع من الحاجة الفعلية الحالية.

### `topology`

الاستغلال الحالي انتقائي:

- `would_create_cycle` مستعمل
- `connected_input_mask` مستعملة
- `analyze_graph_topology` كـ API عامة لا تظهر في التدفق الأساسي لبقية الـ crates
- `GraphTopologyAnalysis` يُستهلك غالباً عبر `GraphValidationReport.topology` وليس كـ API منفصلة

### `processing` و `registry`

الاستغلال موجود، لكنه غالباً غير مباشر عبر `univis_node_graph`.

المشروع لا يدفع أغلب node authors إلى تنفيذ `GraphNodeDefinition` من `graph_core` مباشرة، بل إلى تنفيذ trait محلية في `univis_node_graph` ثم التحويل إلى core adapters.

النتيجة:

- الاستغلال موجود
- لكنه محجوب خلف طبقة wrapping سميكة

## أين لا يوجد استغلال 100%

### 1. ليس كل public API تُستهلك خارج `graph_core`

هناك أجزاء ظاهرة كواجهات عامة، لكنها اليوم أقرب إلى:

- API داخلية تم كشفها أكثر من اللازم
- أو API موجودة للأمثلة وللتوسع المستقبلي أكثر من استخدامها الحالي

هذا ينطبق خصوصاً على:

- بعض أنواع `executable`
- `validate_graph_document_structure`
- `analyze_graph_topology` كـ entry point مستقل

### 2. `graph_core` مستغلة أكثر كـ engine layer من كونها facade نهائية

المسار الفعلي اليوم هو:

`graph_core -> node_graph -> runtime/ui/persistence/workflows/app`

وهذا يعني أن الاستهلاك الحقيقي مرتفع، لكن الاستهلاك المباشر للواجهة الأصلية أقل بكثير.

### 3. بعض الأخطاء أو الإمكانات لا تصل إلى UX النهائي

مثلاً `GraphDocumentOperationError` موجودة وغنية، لكن بعض الطبقات الأعلى تتجاهل نتائج عمليات document بدلاً من تمريرها للمستخدم أو التعامل معها مركزياً.

## أهم نقطة انحراف: `ConnectionPolicy::Multiple`

هذه أهم نتيجة في المراجعة.

في `graph_core`:

- `ConnectionPolicy::Multiple` موجود
- `PortDefinition::allow_multiple_connections` موجود
- validator يسمح نظرياً بمدخلات multi-source عندما policy تسمح بذلك

لكن end-to-end، هذا غير مكتمل:

### على مستوى document authoring

`GraphDocument::connect` يمنع أي وصلة ثانية على نفس input بدون الرجوع إلى policy.

هذا يعني أن multi-input لا يمكن إنشاؤه بشكل طبيعي عبر API الأساسية للوثيقة.

### على مستوى UI

طبقة wire preview في `editor_ui` ترفض `Multiple` صراحة وتعتبرها غير مدعومة runtime حالياً.

### على مستوى persistence/apply

عند تطبيق graph محمّلة، توجد فلترة تمنع عملياً وجود أكثر من source لنفس input.

### على مستوى التنفيذ

`ExecutableDirectLinks` يخزن incoming links على شكل vectors، لكن `resolve_inputs` يقرأ أول link فقط.

النتيجة:

- هذه الإمكانية موجودة في التصميم
- لكنها غير مستغلة بالكامل
- بل يمكن اعتبارها اليوم feature غير مكتملة وليست مستعملة فعلياً

وهذا أقوى دليل على أن `graph_core` ليست مستغلة 100% حالياً.

## مناطق التكرار

### 1. تكرار `PortDefinition`

يوجد `PortDefinition` عام في `graph_core`، ثم يوجد `PortDefinition` آخر في `univis_node_graph` يعيد معظم الحقول نفسها ثم يضيف معلومات UI مثل:

- اللون
- editable_inline
- ui_step / ui_min / ui_max

المشكلة ليست فقط في وجود wrapper، بل في إعادة تعريف نفس البنية تقريباً مع إعادة نفس builder methods ثم التحويل إلى core عبر `as_core()`.

هذا يرفع كلفة الصيانة ويزيد احتمال drift.

### 2. تكرار `GraphNodeDefinition`

يوجد trait عامة صافية في `graph_core`، ثم trait أخرى في `univis_node_graph` تكاد تعيد نفس العقد الأساسي مع إضافات Bevy/UI.

ثم تأتي adapters فقط لإرجاعها إلى core.

هذا يعني أن:

- العقدة تُعرّف نفسها مرة في الواجهة المحلية
- ثم يجري تكييفها إلى واجهة core

بدلاً من composition أو extension أبسط.

### 3. تكرار registry

`NodeRegistry` في `univis_node_graph` تحتفظ بـ:

- registry محلية
- و `GraphNodeRegistry` من `graph_core`

ثم تمرر كثيراً من الوظائف بينهما.

هذا عملي، لكنه يكرر ownership والمسؤولية ويجعل بعض الميزات تحتاج التحديث في طبقتين.

### 4. تكرار منطق validation الجزئية

قواعد الربط ليست ممركزة بالكامل.

حالياً جزء من القواعد موجود في:

- `graph_core::validation`
- `editor_ui::wire`
- `editor_persistence::graph_persistence::apply`

وهذا يخلق 3 مصادر truth بدلاً من واحد.

## مخاطر هذا الشكل الحالي

- drift بين core validator و UI validator
- drift بين persistence filtering و runtime expectations
- واجهة عامة أوسع من الاستعمال الفعلي، ما يصعب فهم حدود crate
- wrapping كثيف في `univis_node_graph` يجعل `graph_core` أقل ظهوراً كمصدر الحقيقة

## ماذا أوصي به

### أولوية 1

اتخاذ قرار واضح حول `ConnectionPolicy::Multiple`:

- إما دعمها end-to-end
- أو تقليصها مؤقتاً من الواجهة العامة حتى تصبح جاهزة فعلاً

### أولوية 2

تجميع قواعد قبول الوصلات في helper واحدة مشتركة، بحيث:

- UI تستعمل نفس القاعدة
- persistence تستعمل نفس القاعدة
- runtime تبقى متوافقة معها

### أولوية 3

تقليل mirroring داخل `univis_node_graph` عبر:

- composition فوق `graph_core::PortDefinition`
- أو جعل `BevyNodeDefinition` امتداداً فعلياً لـ core contract بدل إعادة تعريفه من الصفر

### أولوية 4

مراجعة public surface في `graph_core`:

- ما هو intended for external consumption
- ما هو internal detail
- وما هو future API لم يُستخدم بعد

## التقييم النهائي

إذا كان السؤال هو:

"هل تعتمد المنظومة الحالية على `univis_graph_core` بشكل حقيقي؟"

فالجواب: نعم، وبقوة.

أما إذا كان السؤال هو:

"هل كل ما تعرضه `univis_graph_core` كواجهة عامة مستغل فعلاً من بقية الـ crates بدون تكرار أو انحراف؟"

فالجواب: لا.

التوصيف الأدق هو:

- crate أساسية وناجحة من ناحية التموضع المعماري
- لكنها ليست مستغلة بالكامل كـ public API
- وحولها طبقة adapter كبيرة فيها تكرار واضح
- وفيها feature مهمة غير مكتملة end-to-end وهي multi-source inputs
