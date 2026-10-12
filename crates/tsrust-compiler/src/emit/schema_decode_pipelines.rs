//! Fused source decoding preserves exact callback plans and commitment boundaries.
use super::*;
use crate::stdlib::schema::pipelines::{Operation,ObjectPolicy,WrapperKind};

impl Emitter<'_> {
    pub(super) fn schema_text_missing(&self,id:usize,generated:&mut String,owned:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->String {
        let name=format!("decode_missing{id}");let marker=format!("/*{name}*/");
        if generated.contains(&marker){return name;}generated.push_str(&marker);
        let trial=self.schema_pipeline_trial_impl(id,Type::Undefined,true,generated,owned,errors);
        generated.push_str(&format!("fn {name}(schema:&Record{},path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>,committed:&mut bool)->Option<Option<{}>>{{use tsrust_schema_trial::Trial;match {trial}(schema,TsrUndefined,path,issues,failure){{Trial::Absent=>None,Trial::Prepared{{value,..}}=>Some(Some(value)),Trial::Rejected(_)=>Some(None),Trial::Committed(value)=>{{*committed=true;Some(value)}},Trial::Aborted(error)=>{{*failure=Some(error);Some(None)}}}}}}",self.native_record_declaration(id),self.native_type(self.schema_pipeline_output(id))));
        name
    }
    pub(super) fn schema_text_pipeline_body(&self,id:usize,generated:&mut String,owned:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->String {
        if let Some(plan)=self.program.schema_pipelines.wrapper(id){
            let child=self.schema_text_validator(plan.source,generated,owned,errors);
            let call=format!("{child}(&schema.field0,input,path,issues,failure,policy,remaining,committed)");
            return match plan.kind{
                WrapperKind::Default(_)=>call,
                WrapperKind::Message=>format!("{{let issue_start=issues.items.len();let result={call};issues.message_since(issue_start,schema.field1.as_ref());result}}"),
                WrapperKind::Presence(added)=>{
                    let null=if added==Type::Null{format!("if input.kind()==\"null\"{{return Some({});}}",self.schema_output_injection(Type::Null,plan.output,"TsrNull"))}else{String::new()};
                    let inject=self.schema_output_injection(self.schema_pipeline_output(plan.source),plan.output,"value");
                    format!("{null}{call}.map(|value|{inject})")
                }
            };
        }
        if let Some(stage)=self.program.schema_pipelines.get(id){
            let call=self.schema_pipeline_callback_result(stage.callback,"schema.field1",errors);
            if stage.operation==Operation::Coerce{
                // This conversion constructs the callback's declared owned input
                // directly. A JsonValue callback explicitly requests dynamic data.
                let raw=self.schema_text_input(stage.callback_input,generated);
                let child=self.schema_pipeline_validator(stage.source,stage.callback_output,generated,owned,errors);
                return format!("let mut value={raw}(input,path,issues)?;*committed=true;let converted=match {call}{{Ok(value)=>value,Err(error)=>{{*failure=Some(error);return None;}}}};{child}(&schema.field0,converted,path,issues,failure)");
            }
            let child=self.schema_text_validator(stage.source,generated,owned,errors);
            let mut body=format!("let mut value={child}(&schema.field0,input,path,issues,failure,policy,remaining,committed)?;");
            match stage.operation{
                Operation::Transform=>body.push_str(&format!("*committed=true;match {call}{{Ok(value)=>Some(value),Err(error)=>{{*failure=Some(error);None}}}}")),
                Operation::Refine=>{
                    let setup=self.schema_refinement_path_setup(stage);
                    let message=self.schema_refinement_message(stage);
                    body.push_str(&format!("match {call}{{Ok(true)=>Some(value),Ok(false)=>{{{setup}let issue_start=issues.items.len();issues.push(&refinement_path,Code::Refinement,Kind::{kind},Kind::{kind});{message}None}},Err(error)=>{{*failure=Some(error);None}}}}",kind=self.schema_kind(stage.output)));
                },
                Operation::Coerce=>unreachable!(),
            }
            return body;
        }
        if let Some(plan)=self.program.schema_pipelines.collection(id){
            assert_eq!(plan.kind,crate::stdlib::schema::pipelines::CollectionKind::Record,"checked text collection domain");
            let child=self.schema_text_validator(plan.element,generated,owned,errors);
            let native=self.native_type(plan.output);
            // Dynamic names borrow each local key only for that validation call.
            // The outer path must not retain a reference to the temporary index.
            return format!("let fields=match input.fields(){{Ok(fields)=>fields,Err(_)=>{{issues.push(path,Code::Type,Kind::Object,observed);return None;}}}};let mut values=Vec::new();for(key,input)in fields{{let mut field_path=path.clone();field_path.push(tsrust_schema_issues::Segment::Field(key.as_ref()));let value={child}(&schema.field0,input,&mut field_path,issues,failure,None,&mut None,committed);let Some(value)=value else{{return None;}};values.push((String::Owned(key.into_owned()),value));}}Some(<{native}>::from_values(values))");
        }
        if let Some(plan)=self.program.schema_pipelines.array(id){
            let child=self.schema_text_validator(plan.element,generated,owned,errors);
            let native=self.native_type(plan.output);
            return format!("if input.kind()!=\"array\"{{issues.push(path,Code::Type,Kind::Array,observed);return None;}}let mut values=Some(<{native}>::from([]));input.elements(|index,input|{{path.push(tsrust_schema_issues::Segment::Index(index));let value={child}(&schema.field0,input,path,issues,failure,None,&mut None,committed);path.pop();match(value,&mut values){{(Some(value),Some(values))=>values.push(value),(None,_)=>values=None,_=>{{}}}}failure.is_none()&&values.is_some()}}).expect(\"validated array cursor\");values");
        }
        if let Some(plan)=self.program.schema_pipelines.union(id){
            let mut body=String::from("let checkpoint=issues.checkpoint();");
            if let Some(selector)=&plan.selector{body.push_str(&format!("let object=match input.fields(){{Ok(fields)=>fields,Err(_)=>{{issues.push(path,Code::Type,Kind::Object,observed);return None;}}}};let Some(tag)=object.get({:?}).copied()else{{path.push(tsrust_schema_issues::Segment::Field({:?}));issues.push(path,Code::Missing,Kind::Union,Kind::Missing);path.pop();return None;}};",selector.field,selector.field));}
            for(branch,child)in plan.branches.iter().enumerate(){
                let validator=self.schema_text_validator(*child,generated,owned,errors);
                let inject=self.schema_output_injection(self.schema_pipeline_output(*child),plan.output,"value");
                let attempt=format!("let mut branch_committed=false;let result={validator}(&schema.field{branch},input,path,issues,failure,policy,remaining,&mut branch_committed);*committed|=branch_committed;if let Some(value)=result{{issues.restore(checkpoint);return Some({inject});}}if failure.is_some()||branch_committed{{return None;}}");
                if let Some(selector)=&plan.selector{
                    let condition=selector.tags[branch].iter().map(|tag|self.schema_text_tag("tag",tag)).collect::<Vec<_>>().join("||");
                    body.push_str(&format!("if {condition}{{{attempt}issues.reject();return None;}}"));
                }else{body.push_str(&attempt);}
            }
            if let Some(selector)=&plan.selector{body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({:?}));issues.push(path,Code::Discriminator,Kind::Union,tsrust_source_kind(tag));path.pop();",selector.field));}
            body.push_str("issues.reject();None");return body;
        }
        let plan=self.program.schema_pipelines.object(id).expect("schema pipeline decoder kind");
        let Type::Record(record)=plan.value else{unreachable!()};
        let fields=&self.program.records[record].fields;
        let mut body=String::from("let mut fields=match input.fields(){Ok(fields)=>fields,Err(_)=>{issues.push(path,Code::Type,Kind::Object,observed);return None;}};");
        // Match in-memory pipeline behavior: reject unknown keys before callbacks.
        let known=fields.iter().map(|field|format!("{:?}",field.name.text)).collect::<Vec<_>>().join("|");
        let unknown=if known.is_empty(){"true".into()}else{format!("!matches!(key.as_ref(),{known})")};
        body.push_str(&format!("if policy.unwrap_or({}){{let mut invalid=false;for(key,value)in &fields{{if {unknown}{{invalid=true;let mut unknown_path=path.clone();unknown_path.push(tsrust_schema_issues::Segment::Field(key));issues.push(&unknown_path,Code::UnknownField,Kind::Missing,tsrust_source_kind(*value));}}}}if invalid{{return None;}}}}",plan.policy==ObjectPolicy::Reject));
        for(index,field)in fields.iter().enumerate(){
            let child=self.schema_text_validator(plan.fields[index],generated,owned,errors);
            let missing=self.schema_text_missing(plan.fields[index],generated,owned,errors);
            body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({key:?}));let field{index}=match fields.remove({key:?}){{Some(input)=>{child}(&schema.field0.field{index},input,path,issues,failure,None,&mut None,committed),None=>match {missing}(&schema.field0.field{index},path,issues,failure,committed){{Some(value)=>value,None=>{{issues.push(path,Code::Missing,Kind::{kind},Kind::Missing);None}}}}}};path.pop();let field{index}=field{index}?;",key=field.name.text,kind=self.schema_kind(field.ty.unwrap())));
        }
        let values=(0..fields.len()).map(|field|format!("field{field}")).collect::<Vec<_>>().join(",");
        let value=format!("Record{}{{{values}}}",self.native_record_declaration(record));
        if plan.policy==ObjectPolicy::Retain{
            let Type::Record(retained)=plan.output else{unreachable!()};
            let extra=self.native_type(self.program.records[retained].fields[1].ty.unwrap());
            let json=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::JsonValue)).unwrap();
            body.push_str(&format!("Some(Record{}{{field0:{value},field1:<{extra}>::from_values(fields.into_iter().map(|(key,value)|(String::Owned(key.into_owned()),Record{}{{value:value.retain()}})))}})",self.native_record_declaration(retained),self.native_record_declaration(json)));
        }else{body.push_str(&format!("*remaining=Some(fields);Some({value})"));}
        body
    }
}
