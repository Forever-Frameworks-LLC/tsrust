//! Reversible field projections keep stripped data and missing fields intact.
use super::*;
use crate::stdlib::schema::pipelines::ObjectPolicy;

impl Emitter<'_> {
    pub(super) fn schema_pipeline_object_trial_body(
        &self, id: usize, input: Type, generated: &mut String,
        seen: &mut HashSet<(usize, Type)>, errors: &crate::error_effects::FailureSet,
    ) -> String {
        let plan = self.program.schema_pipelines.object(id).unwrap();
        let Type::Record(output_record) = plan.value else { unreachable!() };
        let fields = &self.program.records[output_record].fields;
        let Type::Record(input_record) = input else {
            return format!("issues.push(path,tsrust_schema_issues::Code::Type,tsrust_schema_issues::Kind::Object,tsrust_schema_issues::Kind::{});Trial::Rejected(input)",self.schema_kind(input));
        };
        let json = self.program.records[input_record].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue);
        if !json && self.program.records[input_record].native_type.is_some() {
            return "issues.push(path,tsrust_schema_issues::Code::Type,tsrust_schema_issues::Kind::Object,tsrust_schema_issues::Kind::Object);Trial::Rejected(input)".into();
        }
        let raw_record = self.native_record_declaration(input_record);
        let provided = fields.iter().map(|field|self.program.records[input_record].fields.iter().enumerate()
            .find(|(_,candidate)|candidate.name.text==field.name.text).map(|(position,field)|(position,field.ty.unwrap()))).collect::<Vec<_>>();
        let mut body = String::from("let mut committed=false;");
        if json {
            body.push_str(&format!("let mut values=match input.value{{tsrust_native_json::Value::Object(values)=>values,value=>{{issues.push(path,tsrust_schema_issues::Code::Type,tsrust_schema_issues::Kind::Object,tsrust_schema_observed(&value));return Trial::Rejected(Record{raw_record}{{value}});}}}};"));
            if plan.policy==ObjectPolicy::Reject {
                let known=fields.iter().map(|field|format!("{:?}",field.name.text)).collect::<Vec<_>>().join("|");
                let unknown=if known.is_empty(){"true".into()}else{format!("!matches!(key.as_str(),{known})")};
                body.push_str(&format!("let mut unknown=false;for(key,value)in &values{{if {unknown}{{let mut field_path=path.clone();field_path.push(tsrust_schema_issues::Segment::Field(key));issues.push(&field_path,tsrust_schema_issues::Code::UnknownField,tsrust_schema_issues::Kind::Missing,tsrust_schema_observed(value));unknown=true;}}}}if unknown{{return Trial::Rejected(Record{raw_record}{{value:tsrust_native_json::Value::Object(values)}});}}"));
            }
        } else if plan.policy==ObjectPolicy::Reject {
            let extra=self.program.records[input_record].fields.iter().filter(|candidate|!fields.iter().any(|field|field.name.text==candidate.name.text)).collect::<Vec<_>>();
            if !extra.is_empty() {
                for field in extra {
                    body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({:?}));issues.push(path,tsrust_schema_issues::Code::UnknownField,tsrust_schema_issues::Kind::Missing,tsrust_schema_issues::Kind::{});path.pop();",field.name.text,self.schema_kind(field.ty.unwrap())));
                }
                body.push_str("return Trial::Rejected(input);");return body;
            }
        }
        for (index,field) in fields.iter().enumerate() {
            let raw = if json { input } else { provided[index].map(|(_,ty)|ty).unwrap_or(Type::Undefined) };
            let raw_native=self.native_type(raw);
            let child = self.schema_pipeline_trial(plan.fields[index],raw,generated,seen,errors);
            let absent = self.schema_pipeline_trial_impl(plan.fields[index],Type::Undefined,true,generated,seen,errors);
            let schema = format!("&schema.field0.field{index}");
            let read = if json {
                body.push_str(&format!("let(key{index},raw{index})=match values.remove_entry({:?}){{Some((key,value))=>(Some(key),Some(Record{raw_record}{{value}})),None=>(None,None)}};",field.name.text));
                format!("match raw{index}{{Some(value)=>{child}({schema},value,path,issues,failure).remap_input(Some),None=>{absent}({schema},TsrUndefined,path,issues,failure).remap_input(|_|None::<{}>)}}",self.native_type(raw))
            } else if let Some((position,_)) = provided[index] {
                format!("{child}({schema},input.field{position},path,issues,failure).remap_input(Some)")
            } else {
                format!("{absent}({schema},TsrUndefined,path,issues,failure).remap_input(|_|None::<{}>)",self.native_type(raw))
            };
            let mut reject = String::from("if committed{return Trial::Committed(None);}");
            if json {
                reject.push_str(&format!("if let(Some(key),Some(value))=(key{index},restored){{values.insert(key,value.value);}}"));
                for previous in 0..index {
                    reject.push_str(&format!("if let(Some(key),Some(value))=(key{previous},restore{previous}.expect(\"uncommitted field\")(field{previous})){{values.insert(key,value.value);}}"));
                }
                reject.push_str(&format!("return Trial::Rejected(Record{raw_record}{{value:tsrust_native_json::Value::Object(values)}});"));
            } else {
                let restored=self.program.records[input_record].fields.iter().enumerate().map(|(position,_)| {
                    let value=if provided[index].is_some_and(|(source,_)|source==position){"restored.expect(\"present input field\")".to_owned()}
                    else if let Some(previous)=(0..index).find(|previous|provided[*previous].is_some_and(|(source,_)|source==position)){
                        format!("restore{previous}.expect(\"uncommitted field\")(field{previous}).expect(\"present input field\")")
                    }else{format!("input.field{position}")};format!("field{position}:{value}")
                }).collect::<Vec<_>>().join(",");
                reject.push_str(&format!("return Trial::Rejected(Record{raw_record}{{{restored}}});"));
            }
            body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({:?}));let trial={read};path.pop();let(field{index},restore{index}):({},Option<Box<dyn FnOnce({})->Option<{}>+'a>>)=match trial{{Trial::Prepared{{value,restoration}}=>(value,Some(restoration)),Trial::Committed(Some(value))=>{{committed=true;(value,None)}},Trial::Absent=>{{path.push(tsrust_schema_issues::Segment::Field({:?}));issues.push(path,tsrust_schema_issues::Code::Missing,tsrust_schema_issues::Kind::{},tsrust_schema_issues::Kind::Missing);path.pop();let restored=None::<{raw_native}>;{reject}}},Trial::Rejected(restored)=>{{{reject}}},Trial::Committed(None)=>return Trial::Committed(None),Trial::Aborted(error)=>return Trial::Aborted(error)}};",field.name.text,self.native_type(field.ty.unwrap()),self.native_type(field.ty.unwrap()),self.native_type(raw),field.name.text,self.schema_kind(field.ty.unwrap())));
        }
        let values=(0..fields.len()).map(|index|format!("field{index}")).collect::<Vec<_>>().join(",");
        let mut value=format!("Record{}{{{values}}}",self.native_record_declaration(output_record));
        let mut inverse=String::new();
        if plan.policy==ObjectPolicy::Retain {
            let Type::Record(retained)=plan.output else{unreachable!()};
            let extra=self.native_type(self.program.records[retained].fields[1].ty.unwrap());
            let extras=if json {format!("<{extra}>::from_values(std::mem::take(&mut values).into_iter().map(|(key,value)|(String::Owned(key),Record{raw_record}{{value}})))")}else{format!("<{extra}>::new()")};
            value=format!("Record{}{{field0:{value},field1:{extras}}}",self.native_record_declaration(retained));
            if json {inverse.push_str("for entry in value.field1.tsr_owned_items(){let(key,value)=entry.into_pair();values.insert(key.into_owned(),value.value);}");}
            inverse.push_str("let value=value.field0;");
        }
        if json {
            for index in 0..fields.len() {
                inverse.push_str(&format!("if let(Some(key),Some(value))=(key{index},restore{index}.expect(\"uncommitted field\")(value.field{index})){{values.insert(key,value.value);}}"));
            }
            inverse.push_str(&format!("Record{raw_record}{{value:tsrust_native_json::Value::Object(values)}}"));
        } else {
            let restored=self.program.records[input_record].fields.iter().enumerate().map(|(position,_)|{
                let value=if let Some(index)=provided.iter().position(|provided|provided.is_some_and(|(source,_)|source==position)){
                    format!("restore{index}.expect(\"uncommitted field\")(value.field{index}).expect(\"present input field\")")
                }else{format!("input.field{position}")};format!("field{position}:{value}")
            }).collect::<Vec<_>>().join(",");
            inverse.push_str(&format!("Record{raw_record}{{{restored}}}"));
        }
        body.push_str(&format!("let value={value};if committed{{Trial::Committed(Some(value))}}else{{Trial::Prepared{{value,restoration:Box::new(move|value|{{{inverse}}})}}}}"));
        body
    }
}
