//! Bounded validation errors containing type summaries, never input values.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind { Missing, Null, Boolean, Number, Integer, String, Array, Object, Union }
impl Kind {
    pub(super) fn as_str(self)->&'static str {
        match self {Self::Missing=>"missing",Self::Null=>"null",Self::Boolean=>"boolean",Self::Number=>"number",Self::Integer=>"integer",Self::String=>"string",Self::Array=>"array",Self::Object=>"object",Self::Union=>"union"}
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Code { Type, Missing, IntegerRange, IntegerFractional, NumberPrecision, NumberNonfinite, TooSmall, TooLarge, Literal, UnknownField, Discriminator, Refinement, Collision }
impl Code {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::NumberNonfinite => "number-nonfinite",
            Self::UnknownField => "unknown-field",
            Self::Literal => "literal-mismatch",
            Self::Discriminator => "discriminator-mismatch",
            Self::Collision => "collection-collision",
            Self::Refinement => "refinement-failed",
            Self::Type => "type-mismatch", Self::Missing => "missing-field",
            Self::IntegerRange => "integer-range", Self::IntegerFractional => "integer-fractional",
            Self::NumberPrecision => "number-precision", Self::TooSmall => "too-small", Self::TooLarge => "too-large",
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub(super) enum Segment<'a> { Field(&'a str), Index(usize) }
#[derive(Debug, PartialEq, Eq)]
pub(super) enum OwnedSegment { Field(String), Index(usize) }
#[derive(Debug)]
pub(super) struct Issue {
    pub code: Code,
    pub path: Vec<OwnedSegment>,
    pub path_truncated: bool,
    pub expected: Kind,
    pub observed: Kind,
}
#[derive(Debug)]
pub(super) struct Issues {
    pub items: Vec<Issue>,
    pub truncated: bool,
    invalid: bool,
    max_issues: usize,
    max_segments: usize,
    max_path_bytes: usize,
}
/// A speculative branch saves bookkeeping, not input data or copied issues.
#[derive(Debug)]
pub(super) struct Checkpoint { length: usize, truncated: bool, invalid: bool }
impl Issues {
    pub(super) fn checkpoint(&self) -> Checkpoint {
        Checkpoint { length: self.items.len(), truncated: self.truncated, invalid: self.invalid }
    }
    pub(super) fn restore(&mut self, checkpoint: Checkpoint) {
        debug_assert!(checkpoint.length <= self.items.len());
        self.items.truncate(checkpoint.length);
        self.truncated = checkpoint.truncated;
        self.invalid = checkpoint.invalid;
    }
    pub(super) fn reject(&mut self) { self.invalid = true; }
    pub(super) fn new(max_issues: usize, max_segments: usize, max_path_bytes: usize) -> Self {
        Self { items: Vec::new(), truncated: false, invalid: false, max_issues, max_segments, max_path_bytes }
    }
    pub(super) fn is_valid(&self) -> bool { !self.invalid }
    pub(super) fn push(&mut self, source: &[Segment<'_>], code: Code, expected: Kind, observed: Kind) {
        self.invalid = true;
        if self.items.len() == self.max_issues { self.truncated = true; return; }
        let mut path = Vec::new();
        let mut remaining = self.max_path_bytes;
        let mut path_truncated = source.len() > self.max_segments;
        for segment in source.iter().take(self.max_segments) {
            match segment {
                Segment::Index(index) => path.push(OwnedSegment::Index(*index)),
                Segment::Field(name) => {
                    let mut end = name.len().min(remaining);
                    while !name.is_char_boundary(end) { end -= 1; }
                    path.push(OwnedSegment::Field(name[..end].to_owned()));
                    remaining -= end;
                    if end < name.len() { path_truncated = true; break; }
                }
            }
        }
        self.items.push(Issue { code, path, path_truncated, expected, observed });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_issue_budget_cannot_turn_failure_into_success() {
        let mut errors = Issues::new(0, 8, 64);
        assert!(errors.is_valid());
        errors.push(&[], Code::Type, Kind::String, Kind::Number);
        assert!(!errors.is_valid());
        assert!(errors.truncated);
        assert!(errors.items.is_empty());
    }
    #[test]
    fn issue_limit_preserves_order_and_reports_omission() {
        let mut errors = Issues::new(2, 8, 64);
        for index in 0..3 {
            errors.push(&[Segment::Index(index)], Code::Type, Kind::String, Kind::Number);
        }
        assert_eq!(errors.items.len(), 2);
        assert_eq!(errors.items[1].path, vec![OwnedSegment::Index(1)]);
        assert!(errors.truncated);
        assert!(!errors.is_valid());
    }
    #[test]
    fn paths_preserve_property_index_distinction_and_bound_unicode_bytes() {
        let mut errors = Issues::new(2, 3, 5);
        errors.push(&[Segment::Field("é😀secret"), Segment::Index(2)], Code::Type, Kind::String, Kind::Boolean);
        assert_eq!(errors.items[0].path, vec![OwnedSegment::Field("é".into())]);
        assert!(errors.items[0].path_truncated);
        errors.push(&[Segment::Field("0"), Segment::Index(0)], Code::Type, Kind::Array, Kind::Object);
        assert_eq!(errors.items[1].path, vec![OwnedSegment::Field("0".into()), OwnedSegment::Index(0)]);
        assert!(!errors.items[1].path_truncated);
    }
    #[test]
    fn path_limits_apply_across_segments_without_losing_failure() {
        let mut errors = Issues::new(2, 1, 3);
        errors.push(&[Segment::Field("abc"), Segment::Index(1)], Code::Missing, Kind::String, Kind::Missing);
        assert!(errors.items[0].path_truncated);
        let mut errors = Issues::new(1, 4, 3);
        errors.push(&[Segment::Field("ab"), Segment::Field("cd")], Code::Type, Kind::Null, Kind::Boolean);
        assert_eq!(errors.items[0].path, vec![OwnedSegment::Field("ab".into()), OwnedSegment::Field("c".into())]);
        assert!(errors.items[0].path_truncated);
    }
    #[test]
    fn issue_payload_has_only_controlled_type_summaries() {
        let mut errors = Issues::new(1, 8, 64);
        errors.push(&[Segment::Field("password")], Code::Type, Kind::String, Kind::Number);
        let issue = &errors.items[0];
        assert_eq!(issue.code.as_str(), "type-mismatch");
        assert_eq!(issue.expected, Kind::String);
        assert_eq!(issue.observed, Kind::Number);
        assert!(!issue.path_truncated);
        assert!(!errors.truncated);
    }
}
