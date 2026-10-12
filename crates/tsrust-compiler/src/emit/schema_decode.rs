//! Construct schema outputs from validated source cursors, never dynamic trees.
use super::*;

impl Emitter<'_> {
    pub(super) fn schema_text_validator(&self,id:usize,generated:&mut String,owned:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->String {
        let name=format!("decode_schema{id}");
        let marker=format!("/*{name}*/");
        if generated.contains(&marker){return name;}
        generated.push_str(&marker);
        if !generated.contains("fn tsrust_source_kind(") {
            generated.push_str("fn tsrust_source_kind(input:tsrust_native_json::source::Node<'_>)->tsrust_schema_issues::Kind{use tsrust_schema_issues::Kind;match input.kind(){\"null\"=>Kind::Null,\"boolean\"=>Kind::Boolean,\"string\"=>Kind::String,\"number\"=>Kind::Number,\"array\"=>Kind::Array,_=>Kind::Object}}");
        }
        let output=self.schema_pipeline_output(id);
        let body=if self.program.schema_pipelines.types(id).is_some(){self.schema_text_pipeline_body(id,generated,owned,errors)}else{self.schema_text_native_body(id,generated,owned,errors)};
        let record=self.native_record_declaration(id);
        let native=self.native_type(output);
        generated.push_str(&format!("fn {name}<'source>(schema:&Record{record},input:tsrust_native_json::source::Node<'source>,path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>,policy:Option<bool>,remaining:&mut Option<std::collections::BTreeMap<std::borrow::Cow<'source,str>,tsrust_native_json::source::Node<'source>>>,committed:&mut bool)->Option<{native}>{{if failure.is_some(){{return None;}}use tsrust_schema_issues::{{Code,Kind}};let observed=tsrust_source_kind(input);{body}}}"));
        name
    }

    pub(super) fn schema_text_tag(&self,value:&str,tag:&crate::stdlib::schema::plans::Tag)->String {
        use crate::stdlib::schema::plans::Tag;
        match tag {
            Tag::String(text)=>format!("{value}.string().is_ok_and(|value|value=={text:?})"),
            Tag::Boolean(boolean)=>format!("{value}.boolean().is_ok_and(|value|value=={boolean})"),
            Tag::Null=>format!("{value}.kind()==\"null\""),
            Tag::Integer(integer)=>format!("{value}.int().is_ok_and(|value|value=={integer}i64)"),
            Tag::Number(bits)=>format!("{value}.number().is_ok_and(|value|value=={})",float_literal(f64::from_bits(*bits))),
        }
    }

    fn schema_text_native_body(&self,id:usize,generated:&mut String,owned:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->String {
        let output=self.schema_pipeline_output(id);
        let record=self.native_record_declaration(id);
        let mut body=String::new();
        // Runtime branch identities remain distinct even when output types coincide.
        let mut arms=Vec::new();
        for(plan,choice)in self.program.schema_choices.for_output(output){
            let bindings=(0..choice.branches.len()).map(|branch|format!("field{branch}")).collect::<Vec<_>>().join(",");
            let mut attempt=String::from("let checkpoint=issues.checkpoint();");
            if choice.presence==Some(Type::Null){attempt.push_str(&format!("if input.kind()==\"null\"{{return Some({});}}",self.schema_output_injection(Type::Null,output,"TsrNull")));}
            if let Some(selector)=&choice.selector{
                attempt.push_str(&format!("let object=match input.fields(){{Ok(fields)=>fields,Err(_)=>{{issues.push(path,Code::Type,Kind::Object,observed);return None;}}}};let Some(tag)=object.get({:?}).copied()else{{path.push(tsrust_schema_issues::Segment::Field({:?}));issues.push(path,Code::Missing,Kind::Union,Kind::Missing);path.pop();return None;}};",selector.field,selector.field));
            }
            for(branch,child_output)in choice.branches.iter().enumerate(){
                let child=self.schema_text_validator(self.schema_child(*child_output),generated,owned,errors);
                let inject=self.schema_output_injection(*child_output,output,"value");
                let bound=if matches!(output,Type::String|Type::Int|Type::Number){
                    let read=if output==Type::String{"AsRef::<str>::as_ref(&value)"}else{"value"};
                    format!("if let Some(code)=schema.bounds.check({read}){{issues.push(path,tsrust_schema_numeric_code(code),Kind::{},observed);return None;}}",self.schema_kind(output))
                }else{String::new()};
                let code=format!("let mut branch_committed=false;let result={child}(field{branch},input,path,issues,failure,policy,remaining,&mut branch_committed);*committed|=branch_committed;if let Some(value)=result{{issues.restore(checkpoint);let value={inject};{bound}return Some(value);}}if failure.is_some()||branch_committed{{return None;}}");
                if let Some(selector)=&choice.selector {
                    let condition=selector.tags[branch].iter().map(|tag|self.schema_text_tag("tag",tag)).collect::<Vec<_>>().join("||");
                    attempt.push_str(&format!("if {condition}{{{code}issues.reject();return None;}}"));
                }else{attempt.push_str(&code);}
            }
            if let Some(selector)=&choice.selector{attempt.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({:?}));issues.push(path,Code::Discriminator,Kind::Union,tsrust_source_kind(tag));path.pop();",selector.field));}
            attempt.push_str("issues.reject();return None;");
            arms.push(format!("SchemaChoice{record}::P{plan}{{{bindings}}}=>{{{attempt}}}"));
        }
        if !arms.is_empty(){
            let policy=if matches!(output,Type::Record(object) if !self.program.records[object].name.text.starts_with("@schema:retained:")){"let policy=policy.or(schema.choice_policy);"}else{""};
            body.push_str(&format!("if let Some(choice)=&schema.choice{{{policy}match choice.as_ref(){{{}}}}}",arms.join(",")));
        }
        let mismatch=|kind:&str|format!("issues.push(path,Code::Type,Kind::{kind},observed);return None;");
        let base=match output {
            Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Undefined|Type::Literal(_,_)|Type::Enum(_,_)=>{
                let base=output.scalar_base();
                let base=match output{Type::Enum(_,crate::syntax::EnumKind::String)=>Type::String,Type::Enum(_,_)=>Type::Number,_=>base};
                let expected=self.schema_kind(output);
                let read=match base{
                    Type::String=>"input.string().map(|value|String::Owned(value.into_owned()))".to_owned(),
                    Type::Int=>"input.int()".into(),Type::Number=>"input.number()".into(),Type::Boolean=>"input.boolean()".into(),
                    Type::Null=>format!("if input.kind()==\"null\"{{Ok(TsrNull)}}else{{{}}}",mismatch("Null")),
                    Type::Undefined=>return format!("{body}{}",mismatch("Missing")),
                    _=>unreachable!("scalar schema decoder"),
                };
                let mut code=if base==Type::Null {"if input.kind()!=\"null\"{issues.push(path,Code::Type,Kind::Null,observed);return None;}let value=TsrNull;".to_owned()}
                    else{format!("let value=match {read}{{Ok(value)=>value,Err(error)=>{{issues.push(path,tsrust_schema_numeric_code(error.code),Kind::{expected},observed);return None;}}}};")};
                if matches!(output,Type::String|Type::Int|Type::Number){
                    let read=if output==Type::String{"AsRef::<str>::as_ref(&value)"}else{"value"};
                    code.push_str(&format!("if let Some(code)=schema.bounds.check({read}){{issues.push(path,tsrust_schema_numeric_code(code),Kind::{expected},observed);return None;}}"));
                }
                if matches!(output,Type::Literal(_,_)){code.push_str(&format!("if value!=schema.expected{{issues.push(path,Code::Literal,Kind::{expected},observed);return None;}}"));}
                if let Type::Enum(enumeration,_)=output{
                    for(tag,member)in self.program.records[enumeration].enum_definition.as_ref().unwrap().members.iter().enumerate(){
                        let expected=match &member.value{crate::syntax::EnumValue::String(value)=>format!("{value:?}"),crate::syntax::EnumValue::Number(value)=>float_literal(*value)};
                        code.push_str(&format!("if value=={expected}{{return Some(Enum{enumeration}::V{tag});}}"));
                    }
                    code.push_str(&format!("issues.push(path,Code::Literal,Kind::{expected},observed);None"));
                }else{code.push_str("Some(value)");}
                code
            },
            Type::Union(union) if self.schema_enum_members(output).is_some()=>{
                let mut code=format!("let value=match input.string(){{Ok(value)=>String::Owned(value.into_owned()),Err(_)=>{{{}}}}};",mismatch("String"));
                for(tag,_)in self.program.unions[union.index()].members.iter().enumerate(){code.push_str(&format!("if value==schema.field{tag}.expected{{return Some(Union{}::V{tag}(value));}}",union.index()));}
                code.push_str("issues.push(path,Code::Literal,Kind::String,observed);None");code
            },
            Type::Union(union)=>{
                if let Some(base)=self.schema_presence_element(output){
                    let child=self.schema_text_validator(self.schema_child(base),generated,owned,errors);
                    let null=self.program.unions[union.index()].members.contains(&Type::Null).then(||format!("if input.kind()==\"null\"{{return Some({});}}",self.schema_output_injection(Type::Null,output,"TsrNull"))).unwrap_or_default();
                    let value=self.schema_output_injection(base,output,"value");
                    format!("{null}{child}(&schema.inner,input,path,issues,failure,policy,remaining,committed).map(|value|{value})")
                }else{"unreachable!(\"ordered union plan required\")".into()}
            },
            Type::Array(element)=>{
                let child=self.schema_text_validator(self.schema_child(element.ty()),generated,owned,errors);
                let native=self.native_type(output);
                format!("if input.kind()!=\"array\"{{{}}}let mut values=Some(<{native}>::from([]));input.elements(|index,input|{{path.push(tsrust_schema_issues::Segment::Index(index));let value={child}(&schema.element,input,path,issues,failure,None,&mut None,committed);path.pop();match(value,&mut values){{(Some(value),Some(values))=>values.push(value),(None,_)=>values=None,_=>{{}}}}failure.is_none()&&(!*committed||values.is_some())}}).expect(\"validated array cursor\");values",mismatch("Array"))
            },
            Type::Record(object) if self.program.records[object].name.text.starts_with("@schema:retained:")=>{
                let fields=&self.program.records[object].fields;let value=fields[0].ty.unwrap();
                let child=self.schema_text_validator(self.schema_child(value),generated,owned,errors);
                let extra=self.native_type(fields[1].ty.unwrap());
                let json=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::JsonValue)).unwrap();
                format!("let mut extra=None;let value={child}(&schema.inner,input,path,issues,failure,Some(false),&mut extra,committed)?;Some(Record{}{{field0:value,field1:<{extra}>::from_values(extra.unwrap_or_default().into_iter().map(|(key,value)|(String::Owned(key.into_owned()),Record{}{{value:value.retain()}}))){order}}})",self.native_record_declaration(object),self.native_record_declaration(json),order=self.schema_record_order(object))
            },
            Type::Record(object)=>{
                let mut code=format!("let mut fields=match input.fields(){{Ok(fields)=>fields,Err(_)=>{{{}}}}};let mut valid=true;",mismatch("Object"));
                let mut values=Vec::new();
                for(field,member)in self.program.records[object].fields.iter().enumerate(){
                    let output=member.ty.unwrap();let child=self.schema_text_validator(self.schema_child(output),generated,owned,errors);
                    let fallback=self.schema_default_value(output,&format!("schema.field{field}"),generated,owned,errors);
                    let absent=if let Type::Union(union)=output && self.program.unions[union.index()].members.contains(&Type::Undefined){format!("Some({})",self.schema_output_injection(Type::Undefined,output,"TsrUndefined"))}
                        else{format!("issues.push(path,Code::Missing,Kind::{},Kind::Missing);None",self.schema_kind(output))};
                    code.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({key:?}));let field{field}=match fields.remove({key:?}){{Some(input)=>{child}(&schema.field{field},input,path,issues,failure,None,&mut None,committed),None=>match {fallback}{{Some(value)=>value,None=>{{{absent}}}}}}};path.pop();if field{field}.is_none(){{valid=false;}}if failure.is_some()||(*committed&&!valid){{return None;}}",key=member.name.text));
                    values.push(format!("field{field}:field{field}.unwrap()"));
                }
                code.push_str("if policy.unwrap_or(schema.reject_unknown){for(key,value)in &fields{valid=false;let mut unknown=path.clone();unknown.push(tsrust_schema_issues::Segment::Field(key));issues.push(&unknown,Code::UnknownField,Kind::Missing,tsrust_source_kind(*value));}}if !valid{return None;}*remaining=Some(fields);");
                code.push_str(&format!("Some({})",self.schema_output_record(object,&values.join(","))));code
            },
            _=>unreachable!("checked schema text output"),
        };
        body.push_str(&base);body
    }
}
