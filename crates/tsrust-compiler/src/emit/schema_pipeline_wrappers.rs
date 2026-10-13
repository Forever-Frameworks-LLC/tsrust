//! Presence and default adapters retain omission independently of supplied values.
use super::*;
use crate::stdlib::schema::pipelines::WrapperKind;

impl Emitter<'_> {
    pub(super) fn schema_pipeline_wrapper_trial_body(
        &self,id:usize,input:Type,missing:bool,generated:&mut String,
        seen:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet,
    )->String {
        let plan=self.program.schema_pipelines.wrapper(id).unwrap();
        let source_output=self.schema_pipeline_output(plan.source);
        match plan.kind {
            WrapperKind::Constraint(_)=>{
                let child=self.schema_pipeline_trial_impl(plan.source,input,missing,generated,seen,errors);
                let roots=self.schema_root_arguments(plan.source);
                let read=if source_output==Type::String {"AsRef::<str>::as_ref(value)"}else{"*value"};
                let kind=self.schema_kind(source_output);
                format!("{child}({roots}&schema.field0,input,path,issues,failure).constraint(path,issues,tsrust_schema_issues::Kind::{kind},Some(schema.field1.field1.as_ref()),|value|schema.field1.field0.bounds.check({read}).map(tsrust_schema_numeric_code))")
            }
            WrapperKind::Message=>{
                let child=self.schema_pipeline_trial_impl(plan.source,input,missing,generated,seen,errors);
        let child_roots=self.schema_root_arguments(plan.source);
                format!("{{let issue_start=issues.items.len();let trial={child}({child_roots}&schema.field0,input,path,issues,failure);issues.message_since(issue_start,schema.field1.as_ref());trial}}")
            }

            WrapperKind::Default(family) if missing=>{
                let source_input=self.program.schema_pipelines.types(plan.source).unwrap().0;
                let child=self.schema_pipeline_trial(plan.source,source_input,generated,seen,errors);
        let child_roots=self.schema_root_arguments(plan.source);
                let call=self.schema_pipeline_callback_result(family,"schema.field1",errors);
                // Creating a missing value is not itself a consuming transform.
                // Preserve the child's commitment if its pipeline consumes it.
                format!("match {call}{{Err(error)=>Trial::Aborted(error),Ok(input)=>match {child}({child_roots}&schema.field0,input,path,issues,failure).remap_input(|_|TsrUndefined){{Trial::Absent=>unreachable!(\"supplied default trial\"),trial=>trial}}}}")
            }
            WrapperKind::Default(_)=>{
                let child=self.schema_pipeline_trial(plan.source,input,generated,seen,errors);
        let child_roots=self.schema_root_arguments(plan.source);
                format!("{child}({child_roots}&schema.field0,input,path,issues,failure)")
            }
            WrapperKind::Presence(added)=>{
                let child=self.schema_pipeline_trial_impl(plan.source,input,missing,generated,seen,errors);
        let child_roots=self.schema_root_arguments(plan.source);
                let inject=self.schema_output_injection(source_output,plan.output,"value");
                let inverse=self.schema_selected_output(source_output,plan.output,"value");
                let trial=format!("{child}({child_roots}&schema.field0,input,path,issues,failure).inject(|value|{inject},|value|{inverse})");
                if missing {
                    if added!=Type::Undefined{return trial;}
                    let absent=self.schema_output_injection(Type::Undefined,plan.output,"TsrUndefined");
                    return format!("match {trial}{{Trial::Absent=>Trial::Prepared{{value:{absent},restoration:Box::new(|_|TsrUndefined)}},trial=>trial}}");
                }
                if input==added {
                    return format!("Trial::Prepared{{value:{},restoration:Box::new(|value|{})}}",self.schema_output_injection(input,plan.output,"input"),self.schema_selected_output(input,plan.output,"value"));
                }
                if added==Type::Null && let Type::Record(record)=input
                    && self.program.records[record].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue) {
                    return format!("if matches!(&input.value,tsrust_native_json::Value::Null){{Trial::Prepared{{value:{},restoration:Box::new(|_|Record{}{{value:tsrust_native_json::Value::Null}})}}}}else{{{trial}}}",self.schema_output_injection(Type::Null,plan.output,"TsrNull"),self.native_record_declaration(record));
                }
                trial
            }
        }
    }
}
