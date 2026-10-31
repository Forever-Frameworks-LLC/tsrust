//! Move-only array adapters preserve commitment from nested callback pipelines.
use super::*;

impl Emitter<'_> {
    pub(in crate::emit) fn schema_pipeline_array_trial_body(
        &self, id: usize, input: Type, generated: &mut String,
        seen: &mut HashSet<(usize, Type)>, errors: &crate::error_effects::FailureSet,
    ) -> String {
        let plan = self.program.schema_pipelines.array(id).unwrap();
        let output = self.native_type(plan.output);
        let (child_input, setup, inverse) = match input {
            Type::Array(element) => (
                element.ty(), "let items=input.tsr_owned_items().collect::<Vec<_>>();".to_owned(),
                format!("let mut input=<{}>::from([]);for value in values{{input.push(value);}}input",self.native_type(input)),
            ),
            Type::Record(record) if self.program.records[record].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue) => {
                let record=self.native_record_declaration(record);
                (input,format!("let items=match input.value{{tsrust_native_json::Value::Array(values)=>values.into_iter().map(|value|Record{record}{{value}}).collect::<Vec<_>>(),value=>{{issues.push(path,tsrust_schema_issues::Code::Type,tsrust_schema_issues::Kind::Array,tsrust_schema_observed(&value));return Trial::Rejected(Record{record}{{value}});}}}};"),
                    format!("Record{record}{{value:tsrust_native_json::Value::Array(values.into_iter().map(|value|value.value).collect())}}"))
            }
            _ => return format!("issues.push(path,tsrust_schema_issues::Code::Type,tsrust_schema_issues::Kind::Array,tsrust_schema_issues::Kind::{});Trial::Rejected(input)",self.schema_kind(input)),
        };
        let child=self.schema_pipeline_trial(plan.element,child_input,generated,seen,errors);
        let child_roots=self.schema_root_arguments(plan.element);
        format!("{setup}tsrust_schema_trial::sequence(items,|index,value|{{path.push(tsrust_schema_issues::Segment::Index(index));let trial={child}({child_roots}&schema.field0,value,path,issues,failure);path.pop();trial}}).inject(|values|{{let mut output=<{output}>::from([]);for value in values{{output.push(value);}}output}},|value|value.tsr_owned_items().collect()).remap_input(|values|{{{inverse}}}).boxed()")
    }
}
