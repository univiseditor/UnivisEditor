use serde::{Deserialize, Serialize};

/// Stable identifier for a node definition.
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

/// Display category used for grouping nodes in menus.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeCategory(pub String);

impl NodeCategory {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const MATH: &'static str = "Math";
    pub const LOGIC: &'static str = "Logic";
    pub const INPUT: &'static str = "Input";
    pub const OUTPUT: &'static str = "Output";
    pub const SCENE: &'static str = "Scene";
    pub const ADVANCED: &'static str = "Advanced";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConnectionPolicy {
    #[default]
    Single,
    /// Reserved for future fan-in support.
    ///
    /// The current workspace treats multi-source inputs as unsupported:
    /// validation reports them explicitly, the UI rejects new links, and
    /// persistence skips them during apply/load.
    Multiple,
}
