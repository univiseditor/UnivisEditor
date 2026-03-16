use crate::ports::PortSchema;

/// Higher-level schema contract for graph validation rules.
pub trait GraphSchema: PortSchema {
    fn requirement_satisfied(
        requirement: Option<&Self::Requirement>,
        output_requirement_token: Option<&str>,
    ) -> bool;

    fn requirement_label(requirement: &Self::Requirement) -> String;
}
