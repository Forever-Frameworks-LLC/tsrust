//! Export visitors preserve exact runtime plan identities and finite references.
use super::*;
use crate::stdlib::schema::pipelines::{CollectionKind, ObjectPolicy, WrapperKind};

impl Emitter<'_> {
    pub(in crate::emit) fn schema_export_call(&self,fun:&CheckedFunction,expr:&Expr,args:&[Expr],failure:usize)->String {
        let Type::Record(root)=args[0].ty.unwrap() else{unreachable!()};
        let Type::Record(output)=expr.ty.unwrap() else{unreachable!()};
        let mut generated=String::new();
        let visitor=self.schema_export_visitor(root,&mut generated,&mut HashSet::new());
        let errors=crate::error_effects::FailureSet{explicit:vec![Type::Record(failure)],runtime:false,opaque:false};
        let (declaration,error_name)=self.completion_error_layout("SchemaExportFailure",&errors,None,expr.span);
        let failure_record=self.native_record_declaration(failure);
        let order=self.schema_record_order(failure);
        let output_record=self.native_record_declaration(output);
        let receiver=self.expr(fun,&args[0],Mode::Read);
        let body=format!("{{let schema={receiver};let mut definitions=tsrust_native_json::Map::new();match {visitor}(schema,\"\",&mut definitions,false){{Ok(value)=>Ok(Record{output_record}{{value:tsrust_schema_export::document(value,definitions)}}),Err(error)=>Err(SchemaExportFailure::V0(Record{failure_record}{{field0:String::Borrowed(\"unrepresentable\"),field1:String::Owned(error.path),field2:String::Borrowed(error.reason){order}}}))}}}}");
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
        let call=self.exception_regions.result_call(&body,&error_name,&errors,&unions,expr.span).expect("checked schema export exception transport");
        format!("{{{declaration}{generated}{call}}}")
    }

    pub(in crate::emit) fn schema_export_visitor(
        &self,
        id: usize,
        generated: &mut String,
        seen: &mut HashSet<usize>,
    ) -> String {
        let name = format!("export_schema{id}");
        if !seen.insert(id) { return name; }
        let body = self.schema_export_body(id, generated, seen);
        let record = self.native_record_declaration(id);
        generated.push_str(&format!(
            "fn {name}(schema:&Record{record},path:&str,definitions:&mut tsrust_native_json::Map<std::string::String,tsrust_native_json::Value>,optional:bool)->Result<tsrust_native_json::Value,tsrust_schema_export::ExportError>{{use tsrust_schema_export as export;{body}}}"
        ));
        name
    }

    fn schema_export_body(&self,id:usize,generated:&mut String,seen:&mut HashSet<usize>)->String {
        if let Some(plan) = self.program.schema_pipelines.recursive(id) {
            let child = self.schema_export_visitor(plan.body, generated, seen);
            let key = format!("schema{id}");
            return format!("if !definitions.contains_key({key:?}){{definitions.insert({key:?}.to_owned(),tsrust_native_json::Value::Bool(false));let body={child}(&schema.field0,path,definitions,optional)?;definitions.insert({key:?}.to_owned(),body);}}Ok(export::reference({key:?}))");
        }
        if let Some(reference) = self.program.schema_pipelines.reference(id) {
            return format!("Ok(export::reference(\"schema{}\"))",reference.root);
        }
        if self.program.schema_pipelines.get(id).is_some() {
            return "Err(export::unsupported(path,\"Callback rules cannot be represented in JSON Schema\"))".into();
        }
        if let Some(plan) = self.program.schema_pipelines.wrapper(id) {
            if matches!(plan.kind, WrapperKind::Default(_)) {
                return "Err(export::unsupported(path,\"Default factories cannot be represented in JSON Schema\"))".into();
            }
            let child = self.schema_export_visitor(plan.source,generated,seen);
            let call = format!("{child}(&schema.field0,path,definitions,optional)?");
            return match plan.kind {
                WrapperKind::Presence(Type::Null) => format!("Ok(export::nullable({call}))"),
                WrapperKind::Presence(Type::Undefined) => format!("if !optional{{return Err(export::unsupported(path,\"Undefined is representable only as a missing object property\"));}}Ok({call})"),
                WrapperKind::Presence(_) => unreachable!("checked schema presence wrapper"),
                WrapperKind::Message => format!("Ok({call})"),
                WrapperKind::Constraint(_) => {
                    let helper = match self.schema_pipeline_output(plan.source) {
                        Type::String => "string", Type::Int => "integer", Type::Number => "number",
                        _ => unreachable!("checked scalar constraint"),
                    };
                    // Intersect with the source instead of replacing its limits:
                    // a later, weaker bound must not erase an earlier constraint.
                    format!("let source={call};let(min,max)=schema.field1.field0.bounds.limits();Ok(export::intersection(source,export::{helper}(min,max)))")
                },
                WrapperKind::Default(_) => unreachable!(),
            };
        }
        if let Some(plan) = self.program.schema_pipelines.collection(id) {
            if plan.kind != CollectionKind::Record {
                return "Err(export::unsupported(path,\"Native Map and Set schemas have no JSON encoding\"))".into();
            }
            let child = self.schema_export_visitor(plan.element,generated,seen);
            return format!("Ok(export::record({child}(&schema.field0,&export::child_path(path,\"additionalProperties\"),definitions,false)?))");
        }
        if let Some(plan) = self.program.schema_pipelines.array(id) {
            let child = self.schema_export_visitor(plan.element,generated,seen);
            return format!("Ok(export::array({child}(&schema.field0,&export::child_path(path,\"items\"),definitions,false)?))");
        }
        if let Some(plan) = self.program.schema_pipelines.union(id) {
            let branches = plan.branches.iter().enumerate().map(|(index,branch)| {
                let child = self.schema_export_visitor(*branch,generated,seen);
                format!("{child}(&schema.field{index},&export::child_path(&export::child_path(path,\"anyOf\"),\"{index}\"),definitions,optional)?")
            }).collect::<Vec<_>>().join(",");
            return format!("Ok(export::alternatives(vec![{branches}]))");
        }
        if let Some(plan) = self.program.schema_pipelines.object(id) {
            if plan.policy != ObjectPolicy::Reject {
                return "Err(export::unsupported(path,\"Stripping or retaining unknown fields transforms the output\"))".into();
            }
            let Type::Record(object) = plan.value else { unreachable!() };
            return self.schema_export_object_fields(object,&plan.fields,"schema.field0",generated,seen);
        }
        self.schema_export_remaining_body(id,generated,seen)
    }

    fn schema_export_object_fields(&self,object:usize,children:&[usize],receiver:&str,generated:&mut String,seen:&mut HashSet<usize>)->String {
        let mut body=String::from("let mut properties=tsrust_native_json::Map::new();let mut required=Vec::new();");
        for (index,field) in self.program.records[object].fields.iter().enumerate() {
            let child=self.schema_export_visitor(children[index],generated,seen);
            let key=&field.name.text;
            let optional=match field.ty.unwrap() {
                Type::Undefined=>true,
                Type::Union(union)=>self.program.unions[union.index()].members.contains(&Type::Undefined),
                _=>false,
            };
            body.push_str(&format!("properties.insert({key:?}.to_owned(),{child}(&{receiver}.field{index},&export::child_path(&export::child_path(path,\"properties\"),{key:?}),definitions,true)?);"));
            if !optional {
                body.push_str(&format!("required.push(tsrust_native_json::Value::String({key:?}.to_owned()));"));
            }
        }
        body.push_str("Ok(export::fields(properties,required))");
        body
    }

    fn schema_export_native_base(&self,id:usize,generated:&mut String,seen:&mut HashSet<usize>)->String {
        let output=self.schema_pipeline_output(id);
        if matches!(output,Type::String|Type::Int|Type::Number) {
            let helper=match output {Type::String=>"string",Type::Int=>"integer",_=>"number"};
            return format!("let(min,max)=schema.bounds.limits();Ok(export::{helper}(min,max))");
        }
        if let Type::Literal(_,kind)=output {
            let value=if kind.base()==Type::String {"tsrust_native_json::Value::String(schema.expected.as_ref().to_owned())".into()}else{"tsrust_native_json::Value::from(schema.expected)".to_owned()};
            return format!("Ok(export::object(vec![(\"const\",{value})]))");
        }
        if let Some(members)=self.schema_enum_members(output) {
            let branches=members.iter().enumerate().map(|(index,member)|{
                let child=self.schema_export_visitor(self.schema_child(*member),generated,seen);
                format!("{child}(&schema.field{index},path,definitions,optional)?")
            }).collect::<Vec<_>>().join(",");
            return format!("Ok(export::alternatives(vec![{branches}]))");
        }
        if let Some(element)=self.schema_presence_element(output) {
            let Type::Union(union)=output else{unreachable!()};
            let members=&self.program.unions[union.index()].members;
            let child=self.schema_export_visitor(self.schema_child(element),generated,seen);
            let mut body=String::new();
            if members.contains(&Type::Undefined) {body.push_str("if !optional{return Err(export::unsupported(path,\"Undefined is representable only as a missing object property\"));}");}
            let value=format!("{child}(&schema.inner,path,definitions,optional)?");
            body.push_str(&if members.contains(&Type::Null){format!("Ok(export::nullable({value}))")}else{format!("Ok({value})")});
            return body;
        }
        match output {
            Type::Boolean=>"Ok(export::typed(\"boolean\"))".into(),
            Type::Null=>"Ok(export::typed(\"null\"))".into(),
            Type::Undefined=>"if optional{Ok(tsrust_native_json::Value::Bool(false))}else{Err(export::unsupported(path,\"Undefined has no JSON representation\"))}".into(),
            Type::Array(element)=>{
                let child=self.schema_export_visitor(self.schema_child(element.ty()),generated,seen);
                format!("Ok(export::array({child}(&schema.element,&export::child_path(path,\"items\"),definitions,false)?))")
            },
            Type::Record(object)=>{
                if self.program.records[object].name.text.starts_with("@schema:retained:") {
                    return "Err(export::unsupported(path,\"Retaining unknown fields transforms the output\"))".into();
                }
                let children=self.program.records[object].fields.iter().map(|field|self.schema_child(field.ty.unwrap())).collect::<Vec<_>>();
                let body=self.schema_export_object_fields(object,&children,"schema",generated,seen);
                format!("if !schema.reject_unknown{{return Err(export::unsupported(path,\"Stripping unknown fields transforms the output\"));}}{body}")
            },
            Type::Enum(enumeration,_)=>{
                let values=self.program.records[enumeration].enum_definition.as_ref().unwrap().members.iter().map(|member|match &member.value {
                    crate::syntax::EnumValue::String(value)=>format!("tsrust_native_json::Value::String({value:?}.to_owned())"),
                    crate::syntax::EnumValue::Number(value)=>format!("tsrust_native_json::Value::from({})",float_literal(*value)),
                }).collect::<Vec<_>>().join(",");
                format!("Ok(export::object(vec![(\"enum\",tsrust_native_json::Value::Array(vec![{values}]))]))")
            },
            Type::Union(_) if self.schema_has_choices(output)=>
                "unreachable!(\"checked union schema has an active choice\")".into(),
            _=>unreachable!("checked native schema export output"),
        }
    }

    fn schema_export_remaining_body(&self,id:usize,generated:&mut String,seen:&mut HashSet<usize>)->String {
        let output=self.schema_pipeline_output(id);
        let record=self.native_record_declaration(id);
        let mut body=String::new();
        let mut factories=Vec::new();
        if self.program.schema_factories.contains_key(&output){factories.push("schema.factory.is_some()".to_owned());}
        for (source,_) in self.schema_inherited_factories(output) {
            let Type::Union(source)=source else{unreachable!()};
            factories.push(format!("schema.factory_from{}.is_some()",source.index()));
        }
        if !factories.is_empty(){body.push_str(&format!("if {}{{return Err(export::unsupported(path,\"Default factories cannot be represented in JSON Schema\"));}}",factories.join("||")));}
        let mut arms=Vec::new();
        for (plan,choice) in self.program.schema_choices.for_output(output) {
            let bindings=(0..choice.branches.len()).map(|index|format!("field{index}")).collect::<Vec<_>>().join(",");
            let branches=choice.branches.iter().enumerate().map(|(index,branch)|{
                let child=self.schema_export_visitor(self.schema_child(*branch),generated,seen);
                format!("{child}(field{index},&export::child_path(&export::child_path(path,\"anyOf\"),\"{index}\"),definitions,optional)?")
            }).collect::<Vec<_>>().join(",");
            let mut value=format!("export::alternatives(vec![{branches}])");
            if choice.presence==Some(Type::Null){value=format!("export::nullable({value})");}
            let missing=if choice.presence==Some(Type::Undefined){"if !optional{return Err(export::unsupported(path,\"Undefined is representable only as a missing object property\"));}"}else{""};
            let bounds=if matches!(output,Type::String|Type::Int|Type::Number){
                let helper=match output{Type::String=>"string",Type::Int=>"integer",_=>"number"};
                value=format!("export::intersection({value},export::{helper}(min,max))");
                "let(min,max)=schema.bounds.limits();"
            }else{""};
            arms.push(format!("SchemaChoice{record}::P{plan}{{{bindings}}}=>{{{missing}{bounds}return Ok({value});}}"));
        }
        if !arms.is_empty(){
            let policy=if matches!(output,Type::Record(object) if !self.program.records[object].name.text.starts_with("@schema:retained:")) {
                "if schema.choice_policy==Some(false){return Err(export::unsupported(path,\"Stripping unknown fields transforms the output\"));}"
            }else{""};
            body.push_str(&format!("if let Some(choice)=&schema.choice{{{policy}match choice.as_ref(){{{}}}}}",arms.join(",")));
        }
        body.push_str(&self.schema_export_native_base(id,generated,seen));
        body
    }
}
