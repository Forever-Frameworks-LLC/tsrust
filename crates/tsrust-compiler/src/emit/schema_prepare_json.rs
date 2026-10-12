//! Prepare JSON validation without taking input; commit only the selected plan.
use super::*;
use std::collections::HashSet;

impl Emitter<'_> {
    pub(super) fn schema_output_injection(&self, source: Type, target: Type, value: &str) -> String {
        if source==target { value.into() }
        else if let Type::Union(target)=target { self.schema_widen_value(source,target,value) }
        else { assert!(target.accepts_scalar_value(source));value.into() }
    }
    pub(super) fn schema_json_tag_test(&self, value: &str, tag: &crate::stdlib::schema::plans::Tag) -> String {
        use crate::stdlib::schema::plans::Tag;
        match tag {
            Tag::String(text)=>format!("matches!({value},tsrust_native_json::Value::String(value) if value=={text:?})"),
            Tag::Boolean(expected)=>format!("matches!({value},tsrust_native_json::Value::Bool(value) if *value=={expected})"),
            Tag::Null=>format!("matches!({value},tsrust_native_json::Value::Null)"),
            Tag::Integer(expected)=>{
                let float=if (*expected as f64) as i128==*expected as i128 {format!("||tsrust_native_json::read_f64({value}).is_ok_and(|value|value=={})",float_literal(*expected as f64))}else{String::new()};
                format!("(tsrust_native_json::read_i64({value}).is_ok_and(|value|value=={expected}i64){float})")
            },
            Tag::Number(bits)=>format!("tsrust_native_json::read_f64({value}).is_ok_and(|value|value=={})",float_literal(f64::from_bits(*bits))),
        }
    }
    pub(super) fn schema_json_prepared_validator(&self, id:usize, generated:&mut String, seen:&mut HashSet<usize>, owned_seen:&mut HashSet<(usize,Type)>, errors:&crate::error_effects::FailureSet) {
        let first=seen.is_empty();
        if !seen.insert(id){return;}
        if first { generated.push_str("fn tsrust_schema_observed(input:&tsrust_native_json::Value)->tsrust_schema_issues::Kind{use tsrust_schema_issues::Kind;match input{tsrust_native_json::Value::Null=>Kind::Null,tsrust_native_json::Value::Bool(_)=>Kind::Boolean,tsrust_native_json::Value::Number(_)=>Kind::Number,tsrust_native_json::Value::String(_)=>Kind::String,tsrust_native_json::Value::Array(_)=>Kind::Array,tsrust_native_json::Value::Object(_)=>Kind::Object}}"); }
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let native=self.native_type(output);
        let record=self.native_record_declaration(id);
        let name=format!("PreparedJson{id}");
        let mut variants=Vec::new();
        let mut commits=Vec::new();
        let mut choice_body=String::new();
        for (plan,choice) in self.program.schema_choices.for_output(output) {
            let bindings=(0..choice.branches.len()).map(|branch|format!("field{branch}")).collect::<Vec<_>>().join(",");
            let mut body=String::from("let checkpoint=issues.checkpoint();");
            if choice.presence==Some(Type::Null){
                variants.push(format!("P{plan}Absent"));
                commits.push(format!("{name}::P{plan}Absent=>{}",self.schema_output_injection(Type::Null,output,"TsrNull")));
                body.push_str(&format!("if matches!(input,tsrust_native_json::Value::Null){{return Some({name}::P{plan}Absent);}}"));
            }
            if let Some(selector)=&choice.selector {
                body.push_str(&format!("let tsrust_native_json::Value::Object(object)=input else{{issues.push(path,Code::Type,Kind::Object,observed);return None;}};path.push(tsrust_schema_issues::Segment::Field({:?}));let Some(tag)=object.get({:?})else{{issues.push(path,Code::Missing,Kind::Union,Kind::Missing);path.pop();return None;}};path.pop();",selector.field,selector.field));
            }
            for (branch,child_output) in choice.branches.iter().enumerate() {
                let child=self.schema_child(*child_output);
                self.schema_json_prepared_validator(child,generated,seen,owned_seen,errors);
                variants.push(format!("P{plan}B{branch}(Box<PreparedJson{child}>)"));
                let commit=format!("commit_json{child}(input,*state,remaining)");
                commits.push(format!("{name}::P{plan}B{branch}(state)=>{}",self.schema_output_injection(*child_output,output,&commit)));
                let attempt=format!("if let Some(state)=prepare_json{child}(field{branch},input,path,issues,failure,policy){{issues.restore(checkpoint);return Some({name}::P{plan}B{branch}(Box::new(state)));}}if failure.is_some(){{issues.restore(checkpoint);return None;}}");
                if let Some(selector)=&choice.selector {
                    let condition=selector.tags[branch].iter().map(|tag|self.schema_json_tag_test("tag",tag)).collect::<Vec<_>>().join("||");
                    body.push_str(&format!("if {condition}{{{attempt}issues.reject();return None;}}"));
                }else{body.push_str(&attempt);}
            }
            if let Some(selector)=&choice.selector {
                body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({:?}));issues.push(path,Code::Discriminator,Kind::Union,tsrust_schema_observed(tag));path.pop();",selector.field));
            }
            body.push_str("issues.reject();return None;");
            choice_body.push_str(&format!("SchemaChoice{record}::P{plan}{{{bindings}}}=>{{{body}}},"));
        }
        let choice_dispatch=if choice_body.is_empty(){String::new()}else{
            let policy=if matches!(output,Type::Record(object) if !self.program.records[object].name.text.starts_with("@schema:retained:")){"let policy=policy.or(schema.choice_policy);"}else{""};
            format!("if let Some(choice)=&schema.choice{{{policy}match choice.as_ref(){{{choice_body}}}}}")
        };
        let (state,prepare,commit)=self.schema_json_base_preparation(id,generated,seen,owned_seen,errors);
        variants.insert(0,format!("Base({state})"));
        commits.insert(0,format!("{name}::Base(state)=>{{{commit}}}"));
        generated.push_str(&format!("enum {name}{{{}}}",variants.join(",")));
        let guard=self.schema_json_outer_bounds(output);
        generated.push_str(&format!("fn prepare_json{id}(schema:&Record{record},input:&tsrust_native_json::Value,path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>,policy:Option<bool>)->Option<{name}>{{if failure.is_some(){{return None;}}use tsrust_schema_issues::{{Code,Kind}};let observed=tsrust_schema_observed(input);{guard}{choice_dispatch}{prepare}}}"));
        generated.push_str(&format!("fn commit_json{id}(input:tsrust_native_json::Value,prepared:{name},remaining:&mut Option<tsrust_native_json::Map<std::string::String,tsrust_native_json::Value>>)->{native}{{match prepared{{{}}}}}",commits.join(",")));
        generated.push_str(&format!("fn validate_json{id}(schema:&Record{record},input:tsrust_native_json::Value,path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>)->Option<{native}>{{let prepared=prepare_json{id}(schema,&input,path,issues,failure,None)?;Some(commit_json{id}(input,prepared,&mut None))}}"));
    }

    fn schema_json_outer_bounds(&self, output:Type)->String {
        let (value,expected)=match output {
            Type::String=>("if let tsrust_native_json::Value::String(value)=input{Some(value.as_str())}else{None}".to_owned(),"String"),
            Type::Int=>("tsrust_native_json::read_i64(input).ok()".into(),"Integer"),
            Type::Number=>("tsrust_native_json::read_f64(input).ok()".into(),"Number"),
            _=>return String::new(),
        };
        format!("if let Some(value)={value}{{if let Some(code)=schema.bounds.check(value){{issues.push(path,tsrust_schema_numeric_code(code),Kind::{expected},observed);return None;}}}}")
    }

    fn schema_json_base_preparation(&self,id:usize,generated:&mut String,seen:&mut HashSet<usize>,owned_seen:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->(String,String,String) {
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let name=format!("PreparedJson{id}");
        let ready=|value:&str|format!("return Some({name}::Base({value}));");
        let mismatch=|expected:&str|format!("issues.push(path,Code::Type,Kind::{expected},observed);None");
        match output {
            Type::String=> ("()".into(),format!("if matches!(input,tsrust_native_json::Value::String(_)){{{}}}{}",ready("()"),mismatch("String")),"let tsrust_native_json::Value::String(value)=input else{unreachable!(\"prepared string\")};String::Owned(value)".into()),
            Type::Boolean=> ("bool".into(),format!("if let tsrust_native_json::Value::Bool(value)=input{{{}}}{}",ready("*value"),mismatch("Boolean")),"state".into()),
            Type::Null=> ("()".into(),format!("if matches!(input,tsrust_native_json::Value::Null){{{}}}{}",ready("()"),mismatch("Null")),"TsrNull".into()),
            Type::Undefined=> ("std::convert::Infallible".into(),mismatch("Missing"),"match state{}".into()),
            Type::Int|Type::Number=>{
                let (reader,expected)=if output==Type::Int{("read_i64","Integer")}else{("read_f64","Number")};
                (self.native_type(output),format!("match tsrust_native_json::{reader}(input){{Ok(value)=>Some({name}::Base(value)),Err(error)=>{{issues.push(path,tsrust_schema_numeric_code(error.code),Kind::{expected},observed);None}}}}"),"state".into())
            },
            Type::Literal(_,kind)=>{
                let expected=self.schema_kind(output);
                let (native,read,commit)=match kind {
                    crate::literal_types::LiteralKind::String=>("()".into(),"if let tsrust_native_json::Value::String(value)=input{if value.as_str()==schema.expected.as_ref(){Some(())}else{issues.push(path,Code::Literal,Kind::String,observed);None}}else{issues.push(path,Code::Type,Kind::String,observed);None}".into(),"let tsrust_native_json::Value::String(value)=input else{unreachable!(\"prepared string literal\")};String::Owned(value)".into()),
                    crate::literal_types::LiteralKind::Boolean=>("bool".into(),"if let tsrust_native_json::Value::Bool(value)=input{if *value==schema.expected{Some(*value)}else{issues.push(path,Code::Literal,Kind::Boolean,observed);None}}else{issues.push(path,Code::Type,Kind::Boolean,observed);None}".into(),"state".into()),
                    _=>{let reader=if kind==crate::literal_types::LiteralKind::Int{"read_i64"}else{"read_f64"};(self.native_type(kind.base()),format!("match tsrust_native_json::{reader}(input){{Ok(value) if value==schema.expected=>Some(value),Ok(_)=>{{issues.push(path,Code::Literal,Kind::{expected},observed);None}},Err(error)=>{{issues.push(path,tsrust_schema_numeric_code(error.code),Kind::{expected},observed);None}}}}"),"state".into())},
                };
                (native,format!("({read}).map({name}::Base)"),commit)
            },
            Type::Enum(enumeration,kind)=>{
                let definition=self.program.records[enumeration].enum_definition.as_ref().unwrap();
                let mut body=if kind==crate::syntax::EnumKind::String {format!("let tsrust_native_json::Value::String(value)=input else{{{}}};",mismatch("String").replace("None","return None"))}
                    else {"let value=match tsrust_native_json::read_f64(input){Ok(value)=>value,Err(error)=>{issues.push(path,tsrust_schema_numeric_code(error.code),Kind::Number,observed);return None;}};".into()};
                for (tag,member) in definition.members.iter().enumerate(){let value=match &member.value{crate::syntax::EnumValue::String(value)=>format!("{value:?}"),crate::syntax::EnumValue::Number(value)=>float_literal(*value)};body.push_str(&format!("if value=={value}{{{}}}",ready(&format!("Enum{enumeration}::V{tag}"))));}
                body.push_str(&format!("issues.push(path,Code::Literal,Kind::{},observed);None",self.schema_kind(output)));
                (self.native_type(output),body,"state".into())
            },
            Type::Union(union) if self.schema_enum_members(output).is_some()=>{
                let mut body=format!("let tsrust_native_json::Value::String(value)=input else{{{}}};",mismatch("String").replace("None","return None"));
                let mut arms=Vec::new();
                for (tag,_) in self.program.unions[union.index()].members.iter().enumerate(){body.push_str(&format!("if value.as_str()==schema.field{tag}.expected.as_ref(){{{}}}",ready(&tag.to_string())));arms.push(format!("{tag}=>Union{}::V{tag}(String::Owned(value))",union.index()));}
                body.push_str("issues.push(path,Code::Literal,Kind::String,observed);None");
                ("usize".into(),body,format!("let tsrust_native_json::Value::String(value)=input else{{unreachable!(\"prepared string enum\")}};match state{{{},_=>unreachable!(\"prepared enum alternative\")}}",arms.join(",")))
            },
            Type::Union(union)=>{
                let Some(base)=self.schema_presence_element(output) else{return ("std::convert::Infallible".into(),"unreachable!(\"union has a choice plan\")".into(),"match state{}".into());};
                let child=self.schema_child(base);self.schema_json_prepared_validator(child,generated,seen,owned_seen,errors);
                let output_native=self.native_type(output);
                let null=self.program.unions[union.index()].members.iter().position(|ty|*ty==Type::Null).map(|tag|format!("if matches!(input,tsrust_native_json::Value::Null){{{}}}",ready(&format!("PreparedPresence::Absent(Union{}::V{tag}(TsrNull))",union.index())))).unwrap_or_default();
                let inject=self.schema_output_injection(base,output,&format!("commit_json{child}(input,state,remaining)"));
                (format!("PreparedPresence<{output_native},PreparedJson{child}>"),format!("{null}prepare_json{child}(&schema.inner,input,path,issues,failure,policy).map(|state|{name}::Base(PreparedPresence::Present(state)))"),format!("match state{{PreparedPresence::Absent(value)=>value,PreparedPresence::Present(state)=>{inject}}}"))
            },
            Type::Array(element)=>{
                let child=self.schema_child(element.ty());self.schema_json_prepared_validator(child,generated,seen,owned_seen,errors);
                let native=self.native_type(output);
                let prepare=format!("let tsrust_native_json::Value::Array(values)=input else{{{}}};let mut states=Some(Vec::with_capacity(values.len()));for(index,value)in values.iter().enumerate(){{path.push(tsrust_schema_issues::Segment::Index(index));let state=prepare_json{child}(&schema.element,value,path,issues,failure,None);path.pop();match(state,&mut states){{(Some(state),Some(states))=>states.push(state),(None,_)=>{{states=None;}},_=>{{}}}}if failure.is_some(){{return None;}}}}states.map({name}::Base)",mismatch("Array").replace("None","return None"));
                let commit=format!("let tsrust_native_json::Value::Array(values)=input else{{unreachable!(\"prepared array\")}};let mut output=<{native}>::from([]);for(value,state)in values.into_iter().zip(state){{output.push(commit_json{child}(value,state,&mut None));}}output");
                (format!("Vec<PreparedJson{child}>"),prepare,commit)
            },
            Type::Record(object) if self.program.records[object].name.text.starts_with("@schema:retained:")=>{
                let value=self.program.records[object].fields[0].ty.unwrap();let child=self.schema_child(value);self.schema_json_prepared_validator(child,generated,seen,owned_seen,errors);
                let extra=self.native_type(self.program.records[object].fields[1].ty.unwrap());
                let json=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::JsonValue)).unwrap();
                let commit=format!("let mut extra=None;let value=commit_json{child}(input,state,&mut extra);Record{}{{field0:value,field1:<{extra}>::from_values(extra.unwrap_or_default().into_iter().map(|(key,value)|(String::Owned(key),Record{}{{value}})))}}",self.native_record_declaration(object),self.native_record_declaration(json));
                (format!("PreparedJson{child}"),format!("prepare_json{child}(&schema.inner,input,path,issues,failure,Some(false)).map({name}::Base)"),commit)
            },
            Type::Record(object)=>self.schema_json_object_preparation(id,object,generated,seen,owned_seen,errors),
            _=>unreachable!("checked schema preparation output"),
        }
    }

    fn schema_json_object_preparation(&self,id:usize,object:usize,generated:&mut String,seen:&mut HashSet<usize>,owned_seen:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->(String,String,String) {
        let mut types=Vec::new();let mut prepare=String::from("let tsrust_native_json::Value::Object(input)=input else{issues.push(path,Code::Type,Kind::Object,observed);return None;};let mut valid=true;");let mut commit=String::from("let tsrust_native_json::Value::Object(mut input)=input else{unreachable!(\"prepared object\")};");let mut names=Vec::new();let mut fields=Vec::new();
        for (field,member) in self.program.records[object].fields.iter().enumerate(){
            let output=member.ty.unwrap();let child=self.schema_child(output);self.schema_json_prepared_validator(child,generated,seen,owned_seen,errors);
            types.push(format!("PreparedField<PreparedJson{child},{}>",self.native_type(output)));
            let missing=if let Type::Union(union)=output && let Some(tag)=self.program.unions[union.index()].members.iter().position(|ty|*ty==Type::Undefined){format!("Some(PreparedField::Default(Union{}::V{tag}(TsrUndefined)))",union.index())}else{format!("issues.push(path,Code::Missing,Kind::{},Kind::Missing);None",self.schema_kind(output))};
            let fallback=self.schema_default_value(output,&format!("schema.field{field}"),generated,owned_seen,errors);
            let missing=format!("if failure.is_some(){{None}}else{{match {fallback}{{Some(value)=>value.map(PreparedField::Default),None=>{{{missing}}}}}}}");
            prepare.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({key:?}));let mut field{field}=match input.get({key:?}){{Some(value)=>prepare_json{child}(&schema.field{field},value,path,issues,failure,None).map(PreparedField::Present),None=>{{{missing}}}}};path.pop();if field{field}.is_none(){{valid=false;}}if !valid{{{drop}}}",key=member.name.text,drop=(0..=field).map(|field|format!("field{field}.take();")).collect::<String>()));
            commit.push_str(&format!("let field{field}=match state.{field}{{PreparedField::Present(state)=>commit_json{child}(input.remove({key:?}).expect(\"prepared field\"),state,&mut None),PreparedField::Default(value)=>value}};",key=member.name.text));
            names.push(format!("{:?}",member.name.text));fields.push(format!("field{field}"));
        }
        let known=if names.is_empty(){"false".into()}else{format!("matches!(key.as_str(),{})",names.join("|"))};
        prepare.push_str(&format!("if policy.unwrap_or(schema.reject_unknown){{for(key,value)in input{{if !({known}){{valid=false;let mut unknown_path=path.clone();unknown_path.push(tsrust_schema_issues::Segment::Field(key));issues.push(&unknown_path,Code::UnknownField,Kind::Missing,tsrust_schema_observed(value));}}}}}}if valid{{Some(PreparedJson{id}::Base(({})))}}else{{None}}",fields.iter().map(|field|format!("{field}.unwrap(),")).collect::<String>()));
        commit.push_str(&format!("*remaining=Some(input);Record{}{{{}}}",self.native_record_declaration(object),fields.join(",")));
        (format!("({})",types.iter().map(|ty|format!("{ty},")).collect::<String>()),prepare,commit)
    }
}
