use crate::identity::ConnectionPolicy;

pub trait PortSchema: Send + Sync + 'static {
    type TypeTag: Clone;
    type Requirement: Clone;
    type DefaultValue: Clone;

    fn ports_compatible(from: &Self::TypeTag, to: &Self::TypeTag) -> bool;
}

/// Engine-independent port schema without editor or renderer styling.
#[derive(Debug, Clone)]
pub struct PortDefinition<S: PortSchema> {
    pub name: String,
    pub description: Option<String>,
    pub type_tag: S::TypeTag,
    pub default_value: Option<S::DefaultValue>,
    pub requirement: Option<S::Requirement>,
    pub connection_policy: ConnectionPolicy,
}

impl<S: PortSchema> PortDefinition<S> {
    pub fn new(name: impl Into<String>, type_tag: S::TypeTag) -> Self {
        Self {
            name: name.into(),
            description: None,
            type_tag,
            default_value: None,
            requirement: None,
            connection_policy: ConnectionPolicy::Single,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_default(mut self, value: S::DefaultValue) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn with_requirement(mut self, requirement: S::Requirement) -> Self {
        self.requirement = Some(requirement);
        self
    }

    pub fn with_connection_policy(mut self, policy: ConnectionPolicy) -> Self {
        self.connection_policy = policy;
        self
    }

    /// Declares future multi-source intent.
    ///
    /// Current workspace flows still treat `ConnectionPolicy::Multiple` as
    /// unsupported until runtime fan-in semantics are implemented end-to-end.
    pub fn allow_multiple_connections(mut self) -> Self {
        self.connection_policy = ConnectionPolicy::Multiple;
        self
    }

    pub fn accepts_multiple_connections(&self) -> bool {
        self.connection_policy == ConnectionPolicy::Multiple
    }
}
