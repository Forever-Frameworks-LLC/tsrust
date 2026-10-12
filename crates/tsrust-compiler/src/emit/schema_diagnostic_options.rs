//! Shared emission of checked relative paths and schema-owned messages.
use super::*;
use crate::stdlib::schema::pipelines::{DiagnosticSegment, Stage};

impl Emitter<'_> {
    pub(super) fn schema_refinement_path_setup(&self, stage: &Stage) -> String {
        if stage.diagnostic_path.is_empty() { return "let refinement_path=&*path;".into(); }
        let mut setup = String::from("let mut refinement_path=path.clone();");
        for segment in &stage.diagnostic_path {
            setup.push_str(&match segment {
                DiagnosticSegment::Field(name) => format!("refinement_path.push(tsrust_schema_issues::Segment::Field({name:?}));"),
                DiagnosticSegment::Index(index) => format!("refinement_path.push(tsrust_schema_issues::Segment::Index({index}));"),
            });
        }
        setup
    }
    pub(super) fn schema_refinement_message(&self, stage: &Stage) -> &'static str {
        if stage.custom_message { "issues.message_since(issue_start,schema.field2.as_ref());" } else { "" }
    }
}
