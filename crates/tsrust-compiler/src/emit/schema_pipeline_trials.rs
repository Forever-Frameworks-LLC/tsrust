//! Preserve union fallback until the first user callback consumes its input.
use super::*;
use crate::stdlib::schema::pipelines::Operation;

impl Emitter<'_> {
    pub(super) fn schema_pipeline_output(&self, id: usize) -> Type {
        self.program.schema_pipelines.types(id).map(|(_, output, _)| output)
            .unwrap_or_else(|| self.program.records[id].application.as_ref().unwrap().resolved_arguments[0])
    }

    pub(super) fn schema_pipeline_trial(
        &self, id: usize, input: Type, generated: &mut String,
        seen: &mut HashSet<(usize, Type)>, errors: &crate::error_effects::FailureSet,
    ) -> String {
        self.schema_pipeline_trial_impl(id, input, false, generated, seen, errors)
    }

    pub(super) fn schema_pipeline_trial_impl(
        &self, id: usize, input: Type, missing: bool, generated: &mut String,
        seen: &mut HashSet<(usize, Type)>, errors: &crate::error_effects::FailureSet,
    ) -> String {
        let name = format!("trial_pipeline{id}_{input:?}_{missing}").chars().map(|ch|if ch.is_ascii_alphanumeric(){ch}else{'_'}).collect::<String>();
        let marker = format!("/*{name}*/");
        if generated.contains(&marker) { return name; }
        generated.push_str(&marker);
        let output = self.schema_pipeline_output(id);
        let raw = self.native_type(input);
        let native = self.native_type(output);
        let json = matches!(input,Type::Record(record) if self.program.records[record].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue));
        if json && !generated.contains("fn tsrust_schema_observed(") {
            generated.push_str("fn tsrust_schema_observed(input:&tsrust_native_json::Value)->tsrust_schema_issues::Kind{use tsrust_schema_issues::Kind;match input{tsrust_native_json::Value::Null=>Kind::Null,tsrust_native_json::Value::Bool(_)=>Kind::Boolean,tsrust_native_json::Value::Number(_)=>Kind::Number,tsrust_native_json::Value::String(_)=>Kind::String,tsrust_native_json::Value::Array(_)=>Kind::Array,tsrust_native_json::Value::Object(_)=>Kind::Object}}");
        }
        let body = if let Type::Union(union) = input {
            // Select the concrete input representation before a callback gains
            // ownership. Merely unwrapping a union is not a user transformation.
            let arms = self.program.unions[union.index()].members.iter().enumerate().map(|(tag, member)| {
                let child = self.schema_pipeline_trial(id, *member, generated, seen, errors);
        let child_roots=self.schema_root_arguments(id);
                let extracted=self.union_move_payload(union,*member,"input");
                let restored=self.union_store_payload(union,*member,"restoration(value)");
                let rejected=self.union_store_payload(union,*member,"input");
                format!("Union{}::V{tag}(input)=>match {child}({child_roots}schema,{extracted},path,issues,failure){{Trial::Prepared{{value,restoration}}=>Trial::Prepared{{value,restoration:Box::new(move|value|Union{}::V{tag}({restored}))}},Trial::Rejected(input)=>Trial::Rejected(Union{}::V{tag}({rejected})),Trial::Absent=>Trial::Absent,Trial::Committed(value)=>Trial::Committed(value),Trial::Aborted(error)=>Trial::Aborted(error)}}", union.index(),union.index(),union.index())
            }).collect::<Vec<_>>().join(",");
            format!("match input{{{arms}}}")
        } else if let Some(plan)=self.program.schema_pipelines.recursive(id) {
            let child=self.schema_pipeline_trial_impl(plan.body,input,missing,generated,seen,errors);
            let child_roots=self.schema_root_arguments(plan.body);
            let source=self.schema_pipeline_output(plan.body);
            let inject=self.schema_recursive_output(source,output,"value");
            let inverse=self.schema_recursive_output(output,source,"value");
            format!("let schema_root{id}=schema;{child}({child_roots}&schema.field0,input,path,issues,failure).inject(|value|{inject},|value|{inverse})")
        } else if let Some(reference)=self.program.schema_pipelines.reference(id) {
            let root=reference.root;
            let child=self.schema_pipeline_trial_impl(root,input,missing,generated,seen,errors);
            let child_roots=self.schema_root_arguments(root);
            let source=self.schema_pipeline_output(root);
            let inject=self.schema_output_injection(source,output,"value");
            let inverse=self.schema_selected_output(source,output,"value");
            format!("{child}({child_roots}schema_root{root},input,path,issues,failure).inject(|value|{inject},|value|{inverse})")
        } else if self.program.schema_pipelines.wrapper(id).is_some() {
            self.schema_pipeline_wrapper_trial_body(id,input,missing,generated,seen,errors)
        } else if let Some(plan) = self.program.schema_pipelines.union(id) {
            let mut body = String::from("let checkpoint=issues.checkpoint();let mut input=input;let mut attempted=false;");
            for (branch, child_id) in plan.branches.iter().enumerate() {
                let child = self.schema_pipeline_trial_impl(*child_id, input, missing, generated, seen, errors);
        let child_roots=self.schema_root_arguments(*child_id);
                let child_output = self.schema_pipeline_output(*child_id);
                let inject = self.schema_output_injection(child_output, output, "value");
                let inverse = self.schema_selected_output(child_output, output, "value");
                let absent = if missing { "{}" } else { "unreachable!(\"supplied union branch trial\")" };
                let attempt = format!("match {child}({child_roots}&schema.field{branch},input,path,issues,failure).inject(|value|{inject},|value|{inverse}){{Trial::Rejected(restored)=>{{attempted=true;input=restored;}},Trial::Absent=>{absent},Trial::Committed(None)=>return Trial::Committed(None),selected=>{{issues.restore(checkpoint);return selected;}}}}");
                if let Some(selector) = &plan.selector {
                    let condition = if json {
                        let tags = selector.tags[branch].iter().map(|tag|self.schema_json_tag_test("value",tag)).collect::<Vec<_>>().join("||");
                        format!("matches!(&input.value,tsrust_native_json::Value::Object(fields) if fields.get({:?}).is_some_and(|value|{tags}))",selector.field)
                    } else { self.schema_owned_discriminator_test(input,"&input",selector,branch) };
                    body.push_str(&format!("if {condition}{{{attempt}issues.reject();return Trial::Rejected(input);}}"));
                } else { body.push_str(&attempt); }
            }
            if let Some(selector) = &plan.selector {
                body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({:?}));issues.push(path,tsrust_schema_issues::Code::Discriminator,tsrust_schema_issues::Kind::Union,tsrust_schema_issues::Kind::Union);path.pop();",selector.field));
            }
            if missing { body.push_str("if !attempted{return Trial::Absent;}"); }
            body.push_str("issues.reject();Trial::Rejected(input)");body
        } else if let Some(stage) = self.program.schema_pipelines.get(id) {
            let call = self.schema_pipeline_callback_result(stage.callback,"schema.field1",errors);
            if stage.operation == Operation::Coerce {
                let compatible = stage.callback_input == input || stage.callback_input.accepts_scalar_value(input)
                    || matches!(stage.callback_input,Type::Union(union) if self.program.unions[union.index()].members.contains(&input));
                if !compatible {
                    format!("issues.push(path,tsrust_schema_issues::Code::Type,tsrust_schema_issues::Kind::{},tsrust_schema_issues::Kind::{});Trial::Rejected(input)",self.schema_kind(stage.callback_input),self.schema_kind(input))
                } else {
                    let child = self.schema_pipeline_trial(stage.source,stage.callback_output,generated,seen,errors);
        let child_roots=self.schema_root_arguments(stage.source);
                    let inject = self.schema_output_injection(input,stage.callback_input,"input");
                    format!("let mut value={inject};match {call}{{Err(error)=>Trial::Aborted(error),Ok(input)=>match {child}({child_roots}&schema.field0,input,path,issues,failure).finish(){{Ok(value)=>Trial::Committed(value),Err(error)=>Trial::Aborted(error)}}}}")
                }
            } else {
                let child = self.schema_pipeline_trial_impl(stage.source,input,missing,generated,seen,errors);
        let child_roots=self.schema_root_arguments(stage.source);
                let trial = format!("{child}({child_roots}&schema.field0,input,path,issues,failure)");
                if stage.operation == Operation::Transform {
                    format!("{trial}.transform(|mut value|{call}).boxed()")
                } else {
                    let binding = if stage.output.is_copy(){"let value=*borrowed;"}else{"let value=borrowed;"};
                    {
                        let setup=self.schema_refinement_path_setup(stage);
                        let message=self.schema_refinement_message(stage);
                        format!("{{let trial={trial};{setup}let issue_start=issues.items.len();let result=trial.refine(&refinement_path,issues,tsrust_schema_issues::Kind::{},|borrowed|{{{binding}{call}}});{message}result}}",self.schema_kind(stage.output))
                    }
                }
            }
        } else if missing {
            let fallback = if self.program.schema_pipelines.types(id).is_some() { "None".to_owned() }
                else { self.schema_default_value(output,"schema",generated,seen,errors) };
            let absent = if let Type::Union(union)=output
                && let Some(tag)=self.program.unions[union.index()].members.iter().position(|ty|*ty==Type::Undefined) {
                format!("Trial::Prepared{{value:Union{}::V{tag}(TsrUndefined),restoration:Box::new(|_|TsrUndefined)}}",union.index())
            } else { String::from("Trial::Absent") };
            format!("match {fallback}{{Some(Some(value))=>Trial::Prepared{{value,restoration:Box::new(|_|TsrUndefined)}},Some(None)=>match failure.take(){{Some(error)=>Trial::Aborted(error),None=>Trial::Rejected(input)}},None=>{{{absent}}}}}")
        } else if self.program.schema_pipelines.collection(id).is_some() {
            self.schema_collection_trial_body(id,input,generated,seen,errors)
        } else if self.program.schema_pipelines.array(id).is_some() {
            self.schema_pipeline_array_trial_body(id,input,generated,seen,errors)
        } else if self.program.schema_pipelines.object(id).is_some() {
            self.schema_pipeline_object_trial_body(id,input,generated,seen,errors)
        } else if json {
            self.schema_json_trial(id,generated,seen,errors);
            let Type::Record(record) = input else { unreachable!() };
            let record = self.native_record_declaration(record);
            format!("match trial_json{id}(schema,input.value,path,issues,failure){{Trial::Prepared{{value,restoration}}=>Trial::Prepared{{value,restoration:Box::new(move|value|Record{record}{{value:tsrust_schema_trial::Restore::restore(restoration,value)}})}},Trial::Rejected(value)=>Trial::Rejected(Record{record}{{value}}),Trial::Absent=>Trial::Absent,Trial::Committed(value)=>Trial::Committed(value),Trial::Aborted(error)=>Trial::Aborted(error)}}")
        } else {
            let child = self.schema_owned_projection(id,input,generated,seen,errors);
            format!("match prepare_{child}(schema,&input,path,issues,failure,None){{Some(state)=>{{let(value,restoration)=project_{child}(input,state);Trial::Prepared{{value,restoration}}}},None=>match failure.take(){{Some(error)=>Trial::Aborted(error),None=>Trial::Rejected(input)}}}}")
        };
        let roots=self.schema_root_parameters(id);
        generated.push_str(&format!("fn {name}<'a>({roots}schema:&Record{},input:{raw},path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>)->tsrust_schema_trial::Trial<{raw},{native},Box<dyn FnOnce({native})->{raw}+'a>,SchemaFactoryFailure>where {raw}:'a,{native}:'a{{use tsrust_schema_trial::Trial;{body}}}",self.native_record_declaration(id)));
        name
    }
}
