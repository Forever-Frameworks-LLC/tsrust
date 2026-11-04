//! Finalize non-branching recursive JSON plans without constructing rollback state.
use super::*;
use crate::stdlib::schema::pipelines::ObjectPolicy;

impl Emitter<'_> {
    pub(super) fn schema_pipeline_direct_json(&self,id:usize,input:Type)->bool {
        if self.program.schema_pipelines.recursive(id).is_none()
            || !matches!(input,Type::Record(record) if self.program.records[record].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue)){return false;}
        let plans=&self.program.schema_pipelines;
        let mut pending=vec![id];let mut visited=HashSet::new();
        while let Some(id)=pending.pop(){
            if !visited.insert(id){continue;}
            if let Some(plan)=plans.recursive(id){if !plan.json_input{return false;}pending.push(plan.body);}
            else if let Some(plan)=plans.reference(id){if !plan.json_input{return false;}pending.push(plan.root);}
            else if plans.wrapper(id).is_some()||plans.get(id).is_some()||plans.union(id).is_some()||plans.collection(id).is_some(){return false;}
            else if let Some(plan)=plans.array(id){if !plan.json_input{return false;}pending.push(plan.element);}
            else if let Some(plan)=plans.object(id){
                if !plan.json_input||plan.policy==ObjectPolicy::Retain{return false;}
                pending.extend(plan.fields.iter().copied());
            }else if self.program.records[id].native_type!=Some(crate::native_signatures::NativeTypeId::Schema){return false;}
        }
        true
    }

    pub(super) fn schema_pipeline_json_validator(
        &self,id:usize,generated:&mut String,seen:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet,
    )->String {
        let name=format!("direct_json_pipeline{id}");let marker=format!("/*{name}*/");
        if generated.contains(&marker){return name;}
        generated.push_str(&marker);
        if !generated.contains("fn tsrust_schema_observed("){
            generated.push_str("fn tsrust_schema_observed(input:&tsrust_native_json::Value)->tsrust_schema_issues::Kind{use tsrust_schema_issues::Kind;match input{tsrust_native_json::Value::Null=>Kind::Null,tsrust_native_json::Value::Bool(_)=>Kind::Boolean,tsrust_native_json::Value::Number(_)=>Kind::Number,tsrust_native_json::Value::String(_)=>Kind::String,tsrust_native_json::Value::Array(_)=>Kind::Array,tsrust_native_json::Value::Object(_)=>Kind::Object}}");
        }
        let plans=&self.program.schema_pipelines;let output=self.schema_pipeline_output(id);
        let body=if let Some(plan)=plans.recursive(id){
            let child=self.schema_pipeline_json_validator(plan.body,generated,seen,errors);
            let roots=self.schema_root_arguments(plan.body);
            let inject=self.schema_recursive_output(self.schema_pipeline_output(plan.body),output,"value");
            format!("let schema_root{id}=schema;{child}({roots}&schema.field0,input,path,issues,failure).map(|value|{inject})")
        }else if let Some(plan)=plans.reference(id){
            let root=plan.root;let child=self.schema_pipeline_json_validator(root,generated,seen,errors);
            let roots=self.schema_root_arguments(root);
            let inject=self.schema_output_injection(self.schema_pipeline_output(root),output,"value");
            format!("{child}({roots}schema_root{root},input,path,issues,failure).map(|value|{inject})")
        }else if let Some(plan)=plans.array(id){
            let child=self.schema_pipeline_json_validator(plan.element,generated,seen,errors);
            let roots=self.schema_root_arguments(plan.element);let native=self.native_type(output);
            let Type::Array(element)=output else{unreachable!("checked array pipeline")};
            let inject=self.schema_output_injection(self.schema_pipeline_output(plan.element),element.ty(),"value");
            let storage=if native==format!("Vec<{}>",self.native_type(element.ty())){"Vec::with_capacity(items.len())".to_owned()}else{format!("<{native}>::from([])")};
            format!("let items=match input{{tsrust_native_json::Value::Array(items)=>items,value=>{{issues.push(path,tsrust_schema_issues::Code::Type,tsrust_schema_issues::Kind::Array,tsrust_schema_observed(&value));return None;}}}};let mut output={storage};for(index,value)in items.into_iter().enumerate(){{path.push(tsrust_schema_issues::Segment::Index(index));let value={child}({roots}&schema.field0,value,path,issues,failure);path.pop();output.push({{let value=value?;{inject}}});}}Some(output)")
        }else if let Some(plan)=plans.object(id){
            let Type::Record(record)=plan.value else{unreachable!("checked object pipeline")};
            let fields=&self.program.records[record].fields;
            let mut body=String::from("let mut fields=match input{tsrust_native_json::Value::Object(fields)=>fields,value=>{issues.push(path,tsrust_schema_issues::Code::Type,tsrust_schema_issues::Kind::Object,tsrust_schema_observed(&value));return None;}};");
            if plan.policy==ObjectPolicy::Reject{
                let names=fields.iter().map(|field|format!("{:?}",field.name.text)).collect::<Vec<_>>().join("|");
                let unknown=if names.is_empty(){"true".to_owned()}else{format!("!matches!(key.as_str(),{names})")};
                body.push_str(&format!("let mut unknown=false;for(key,value)in &fields{{if {unknown}{{let mut field_path=path.clone();field_path.push(tsrust_schema_issues::Segment::Field(key));issues.push(&field_path,tsrust_schema_issues::Code::UnknownField,tsrust_schema_issues::Kind::Missing,tsrust_schema_observed(value));unknown=true;}}}}if unknown{{return None;}}"));
            }
            for(index,field)in fields.iter().enumerate(){
                let child_id=plan.fields[index];let child=self.schema_pipeline_json_validator(child_id,generated,seen,errors);let roots=self.schema_root_arguments(child_id);
                let absent=self.schema_pipeline_trial_impl(child_id,Type::Undefined,true,generated,seen,errors);let absent_roots=self.schema_root_arguments(child_id);
                let missing=format!("match {absent}({absent_roots}&schema.field0.field{index},TsrUndefined,path,issues,failure){{tsrust_schema_trial::Trial::Prepared{{value,..}}|tsrust_schema_trial::Trial::Committed(Some(value))=>Some(value),tsrust_schema_trial::Trial::Absent=>{{issues.push(path,tsrust_schema_issues::Code::Missing,tsrust_schema_issues::Kind::{},tsrust_schema_issues::Kind::Missing);None}},tsrust_schema_trial::Trial::Rejected(_)|tsrust_schema_trial::Trial::Committed(None)=>None,tsrust_schema_trial::Trial::Aborted(error)=>{{*failure=Some(error);None}}}}",self.schema_kind(field.ty.unwrap()));
                body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({name:?}));let field{index}=match fields.remove({name:?}){{Some(value)=>{child}({roots}&schema.field0.field{index},value,path,issues,failure),None=>{{{missing}}}}};path.pop();let field{index}=field{index}?;",name=field.name.text));
            }
            let values=(0..fields.len()).map(|index|format!("field{index}")).collect::<Vec<_>>().join(",");
            body.push_str(&format!("Some({})",self.schema_output_record(record,&values)));body
        }else{
            self.schema_json_prepared_validator(id,generated,&mut HashSet::new(),seen,errors);
            format!("validate_json{id}(schema,input,path,issues,failure)")
        };
        let roots=self.schema_root_parameters(id);
        generated.push_str(&format!("fn {name}({roots}schema:&Record{},input:tsrust_native_json::Value,path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>)->Option<{}>{{if failure.is_some(){{return None;}}{body}}}",self.native_record_declaration(id),self.native_type(output)));
        name
    }
}
