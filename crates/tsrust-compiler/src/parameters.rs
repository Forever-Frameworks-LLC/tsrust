//! Source omission is distinct from parameter initialization and native access.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ParameterOmission {
    #[default]
    Required,
    Optional,
    Defaulted,
}
impl ParameterOmission {
    pub fn is_omittable(self) -> bool {
        !matches!(self, Self::Required)
    }
}
