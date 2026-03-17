use univis_graph_core::prelude::{
    GraphDocument, GraphNodeDefinition, GraphNodeRegistry, GraphSchema, NodeCategory, NodeId,
    PortDefinition, PortSchema, ProcessContext, ProcessResult, ProcessValueAccess,
};

#[derive(Clone, Debug, PartialEq)]
enum MiniValue {
    Number(f64),
    Text(String),
    None,
}

impl Default for MiniValue {
    fn default() -> Self {
        Self::None
    }
}

impl ProcessValueAccess for MiniValue {
    type Vec2 = ();
    type Vec3 = ();
    type TaggedPayload = ();
    type Entity = ();

    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    fn as_float(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    fn as_int(&self) -> Option<i64> {
        self.as_float().map(|value| value as i64)
    }

    fn as_bool(&self) -> Option<bool> {
        None
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    fn as_vec2(&self) -> Option<Self::Vec2> {
        None
    }

    fn as_vec3(&self) -> Option<Self::Vec3> {
        None
    }

    fn as_tagged(&self) -> Option<(&str, &Self::TaggedPayload)> {
        None
    }

    fn as_entity(&self) -> Option<&Self::Entity> {
        None
    }

    fn from_float(value: f64) -> Self {
        Self::Number(value)
    }

    fn from_int(value: i64) -> Self {
        Self::Number(value as f64)
    }

    fn from_bool(_value: bool) -> Self {
        Self::None
    }

    fn from_string(value: String) -> Self {
        Self::Text(value)
    }

    fn from_vec2(_value: Self::Vec2) -> Self {
        Self::None
    }

    fn from_vec3(_value: Self::Vec3) -> Self {
        Self::None
    }

    fn from_tagged(_tag: String, _payload: Self::TaggedPayload) -> Self {
        Self::None
    }

    fn from_entity(_value: Self::Entity) -> Self {
        Self::None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MiniTypeTag {
    Number,
    Text,
    Any,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MiniRequirement {
    token: &'static str,
    label: &'static str,
}

struct MiniSchema;

impl PortSchema for MiniSchema {
    type TypeTag = MiniTypeTag;
    type Requirement = MiniRequirement;
    type DefaultValue = MiniValue;

    fn ports_compatible(from: &Self::TypeTag, to: &Self::TypeTag) -> bool {
        match (from, to) {
            (a, b) if a == b => true,
            (_, MiniTypeTag::Any) => true,
            (MiniTypeTag::Any, _) => true,
            _ => false,
        }
    }
}

impl GraphSchema for MiniSchema {
    fn requirement_satisfied(
        requirement: Option<&Self::Requirement>,
        output_requirement_token: Option<&str>,
    ) -> bool {
        match requirement {
            Some(requirement) => output_requirement_token == Some(requirement.token),
            None => true,
        }
    }

    fn requirement_label(requirement: &Self::Requirement) -> String {
        requirement.label.to_string()
    }
}

type MiniPort = PortDefinition<MiniSchema>;

struct AddNode;

impl GraphNodeDefinition<MiniValue, MiniPort> for AddNode {
    fn id(&self) -> NodeId {
        NodeId::new("mini/add")
    }

    fn display_name(&self) -> &str {
        "Add"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Math")
    }

    fn inputs(&self) -> Vec<MiniPort> {
        vec![
            MiniPort::new("A", MiniTypeTag::Number),
            MiniPort::new("B", MiniTypeTag::Number),
        ]
    }

    fn outputs(&self) -> Vec<MiniPort> {
        vec![MiniPort::new("Sum", MiniTypeTag::Number)]
    }

    fn process(&self, context: &mut ProcessContext<'_, MiniValue>) -> ProcessResult {
        let a = context.get_float_or(0, 0.0);
        let b = context.get_float_or(1, 0.0);
        context.set_float(0, a + b);
        ProcessResult::Success
    }
}

struct FormatNode;

impl GraphNodeDefinition<MiniValue, MiniPort> for FormatNode {
    fn id(&self) -> NodeId {
        NodeId::new("mini/format")
    }

    fn display_name(&self) -> &str {
        "Format"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::new("Text")
    }

    fn inputs(&self) -> Vec<MiniPort> {
        vec![
            MiniPort::new("Value", MiniTypeTag::Number).with_requirement(MiniRequirement {
                token: "numeric-output",
                label: "Numeric Output",
            }),
        ]
    }

    fn outputs(&self) -> Vec<MiniPort> {
        vec![MiniPort::new("Text", MiniTypeTag::Text)]
    }

    fn process(&self, context: &mut ProcessContext<'_, MiniValue>) -> ProcessResult {
        let value = context.get_float_or(0, 0.0);
        context.set_string(0, format!("sum={value:.1}"));
        ProcessResult::Success
    }
}

fn main() {
    let mut registry = GraphNodeRegistry::<MiniValue, MiniPort>::new();
    registry.register(AddNode);
    registry.register(FormatNode);

    let mut document = GraphDocument::<MiniValue, ()>::default();
    let add_node_id = document.spawn_node(NodeId::new("mini/add"), [0.0, 0.0], 2, 1);
    let format_node_id = document.spawn_node(NodeId::new("mini/format"), [180.0, 0.0], 1, 1);
    document
        .connect(add_node_id, 0, format_node_id, 0)
        .expect("valid edge");

    let add_node = document.node_mut(add_node_id).expect("add node");
    add_node.inputs = vec![MiniValue::Number(2.0), MiniValue::Number(3.5)];

    let add_definition = registry
        .get(&NodeId::new("mini/add"))
        .expect("add definition");
    let format_definition = registry
        .get(&NodeId::new("mini/format"))
        .expect("format definition");

    let mut add_outputs = vec![MiniValue::None];
    let mut format_outputs = vec![MiniValue::None];
    let mut add_custom_data = None;
    let mut format_custom_data = None;

    let add_inputs = document.node(add_node_id).expect("add node").inputs.clone();
    let mut add_context = ProcessContext {
        inputs: &add_inputs,
        outputs: &mut add_outputs,
        delta_time: 0.0,
        custom_data: &mut add_custom_data,
    };
    assert_eq!(
        add_definition.process(&mut add_context),
        ProcessResult::Success
    );

    let mut format_context = ProcessContext {
        inputs: &add_outputs,
        outputs: &mut format_outputs,
        delta_time: 0.0,
        custom_data: &mut format_custom_data,
    };
    assert_eq!(
        format_definition.process(&mut format_context),
        ProcessResult::Success
    );

    assert_eq!(format_outputs[0], MiniValue::Text("sum=5.5".to_string()));
    assert!(MiniSchema::ports_compatible(
        &MiniTypeTag::Number,
        &MiniTypeTag::Number
    ));
    assert!(MiniSchema::ports_compatible(
        &MiniTypeTag::Number,
        &MiniTypeTag::Any
    ));
    assert!(MiniSchema::requirement_satisfied(
        Some(&MiniRequirement {
            token: "numeric-output",
            label: "Numeric Output",
        }),
        Some("numeric-output"),
    ));

    println!(
        "Built a pure graph document with {} nodes and {} edge.",
        document.nodes.len(),
        document.edges.len()
    );
    println!("Result: {:?}", format_outputs[0]);
}
