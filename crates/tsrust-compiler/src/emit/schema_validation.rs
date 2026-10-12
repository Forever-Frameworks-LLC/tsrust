//! Generate consuming typed validators while borrowing reusable schema plans.
use super::*;
impl Emitter<'_> {
    fn schema_factory_value(&self,output:Type,factory:&str,errors:&crate::error_effects::FailureSet)->String {
        let family=self.program.schema_factories[&output];
        let source=self.result_storage.borrow().get(&family).cloned();
        let Some((source_name,source))=source else{return format!("{factory}.call()")};
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError"&&record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
        let mapping=source.native_injection(errors,&unions,runtime,crate::Span::default()).expect("schema factory error injection");
        let conversion=super::exception_emit::error_conversion("error",&source_name,"SchemaFactoryError",&mapping);
        format!("match {factory}.call(){{Ok(value)=>value,Err(error)=>{{*failure=Some({conversion});return None;}}}}")
    }
    // Outer None means no factory; inner None means generated data failed validation.
    pub(super) fn schema_default_value(&self,output:Type,plan:&str,generated:&mut String,seen:&mut std::collections::HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->String {
        if !self.program.schema_choices.is_empty(){return self.schema_prepared_default_value(output,plan,generated,seen,errors);}
        let mut absent=if let Type::Union(union)=output && let Some(base)=self.schema_presence_element(output) {
            let child=self.schema_default_value(base,&format!("{plan}.inner"),generated,seen,errors);
            let injection=self.schema_widen_value(base,union,"value");
            format!("({child}).map(|value|value.map(|value|{injection}))")
        }else if let Type::Record(record)=output && self.program.records[record].name.text.starts_with("@schema:retained:") {
            let fields=&self.program.records[record].fields;
            let child=self.schema_default_value(fields[0].ty.unwrap(),&format!("{plan}.inner"),generated,seen,errors);
            let extra=self.native_type(fields[1].ty.unwrap());
            format!("({child}).map(|value|value.map(|value|Record{}{{field0:value,field1:<{extra}>::new()}}))",self.native_record_declaration(record))
        }else{format!("None::<Option<{}>>",self.native_type(output))};
        for (source,_) in self.schema_inherited_factories(output).into_iter().rev(){
            let Type::Union(from)=source else{unreachable!()};let Type::Union(to)=output else{unreachable!()};
            let arms=self.program.unions[from.index()].members.iter().enumerate().map(|(tag,ty)|{let target=self.program.unions[to.index()].members.iter().position(|candidate|candidate==ty).unwrap();format!("Union{}::V{tag}(value)=>Union{}::V{target}(value)",from.index(),to.index())}).collect::<Vec<_>>().join(",");
            let factory=self.schema_factory_value(source,"factory",errors);
            let validator=self.schema_owned_validator(self.schema_child(output),output,generated,seen,errors);
            absent=format!("if let Some(factory)=&{plan}.factory_from{}{{let generated=match {factory}{{{arms}}};Some({validator}(&{plan},generated,path,issues,failure))}}else{{{absent}}}",from.index());
        }
        if self.program.schema_factories.contains_key(&output){
            let validator=self.schema_owned_validator(self.schema_child(output),output,generated,seen,errors);
            let value=self.schema_factory_value(output,"factory",errors);
            format!("if let Some(factory)=&{plan}.factory{{Some({validator}(&{plan},{value},path,issues,failure))}}else{{{absent}}}")
        }else{absent}
    }

    fn schema_prepared_default_value(&self,output:Type,plan:&str,generated:&mut String,seen:&mut std::collections::HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->String {
        let id=self.schema_child(output);let record=self.native_record_declaration(id);
        let name=format!("missing_schema{id}");let marker=format!("/*{name}*/");
        if generated.contains(&marker){return format!("{name}(&{plan},path,issues,failure)");}
        // Mark before descending: runtime choice plans can recurse through the
        // same output type even though each uniquely owned plan is finite.
        generated.push_str(&marker);
        let mut body=String::from("if failure.is_some(){return Some(None);}");
        if self.program.schema_factories.contains_key(&output){
            let value=self.schema_factory_value(output,"factory",errors);
            let validator=self.schema_owned_validator(id,output,generated,seen,errors);
            body.push_str(&format!("if let Some(factory)=&schema.factory{{return Some({validator}(schema,{value},path,issues,failure));}}"));
        }
        for(source,_)in self.schema_inherited_factories(output){
            let Type::Union(union)=source else{unreachable!()};
            let value=self.schema_factory_value(source,"factory",errors);
            let value=self.schema_output_injection(source,output,&value);
            let validator=self.schema_owned_validator(id,output,generated,seen,errors);
            body.push_str(&format!("if let Some(factory)=&schema.factory_from{}{{return Some({validator}(schema,{value},path,issues,failure));}}",union.index()));
        }
        let mut choices=String::new();
        for(plan,choice)in self.program.schema_choices.for_output(output){
            let bindings=(0..choice.branches.len()).map(|branch|format!("field{branch}")).collect::<Vec<_>>().join(",");
            let mut branch_body=String::from("let checkpoint=issues.checkpoint();let mut attempted=false;");
            for(branch,child)in choice.branches.iter().enumerate(){
                let fallback=self.schema_prepared_default_value(*child,&format!("field{branch}"),generated,seen,errors);
                let value=self.schema_output_injection(*child,output,"value");
                let constraint=if matches!(output,Type::String|Type::Int|Type::Number){
                    let observed=self.schema_kind(output);
                    let read=if output==Type::String{"AsRef::<str>::as_ref(&value)"}else{"value"};
                    format!("if let Some(code)=schema.bounds.check({read}){{issues.push(path,tsrust_schema_numeric_code(code),tsrust_schema_issues::Kind::{observed},tsrust_schema_issues::Kind::{observed});return Some(None);}}")
                }else{String::new()};
                branch_body.push_str(&format!("match {fallback}{{Some(Some(value))=>{{issues.restore(checkpoint);let value={value};{constraint}return Some(Some(value));}},Some(None)=>attempted=true,None=>{{}}}}if failure.is_some(){{issues.restore(checkpoint);return Some(None);}}"));
            }
            branch_body.push_str("return if attempted{Some(None)}else{None};");
            choices.push_str(&format!("SchemaChoice{record}::P{plan}{{{bindings}}}=>{{{branch_body}}},"));
        }
        if !choices.is_empty(){body.push_str(&format!("if let Some(choice)=&schema.choice{{match choice.as_ref(){{{choices}}}}}"));}
        if let Type::Union(_)=output && let Some(base)=self.schema_presence_element(output){
            let fallback=self.schema_prepared_default_value(base,"schema.inner",generated,seen,errors);
            let value=self.schema_output_injection(base,output,"value");
            body.push_str(&format!("({fallback}).map(|value|value.map(|value|{value}))"));
        }else if let Type::Record(object)=output && self.program.records[object].name.text.starts_with("@schema:retained:"){
            let fields=&self.program.records[object].fields;
            let fallback=self.schema_prepared_default_value(fields[0].ty.unwrap(),"schema.inner",generated,seen,errors);
            body.push_str(&format!("({fallback}).map(|value|value.map(|value|Record{}{{field0:value,field1:<{}>::new()}}))",self.native_record_declaration(object),self.native_type(fields[1].ty.unwrap())));
        }else{body.push_str("None");}
        generated.push_str(&format!("fn {name}(schema:&Record{record},path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>)->Option<Option<{}>>{{{body}}}",self.native_type(output)));
        format!("{name}(&{plan},path,issues,failure)")
    }
    pub(super) fn schema_validate(&self,fun:&CheckedFunction,expr:&Expr,args:&[Expr])->String {
        let Type::Record(root)=args[0].ty.unwrap() else{unreachable!()};
        let mut generated=String::from("fn tsrust_schema_numeric_code(code:&str)->tsrust_schema_issues::Code{use tsrust_schema_issues::Code;match code{\"integer-range\"=>Code::IntegerRange,\"integer-fractional\"=>Code::IntegerFractional,\"number-precision\"=>Code::NumberPrecision,\"number-nonfinite\"=>Code::NumberNonfinite,\"too-small\"=>Code::TooSmall,\"too-large\"=>Code::TooLarge,_=>Code::Type}}");
        let errors=expr.schema_invocations.as_deref().unwrap_or_default().iter().fold(crate::error_effects::FailureSet::default(),|errors,family|errors|self.program.error_effects.storage_errors.get(family).cloned().unwrap_or_default());
        let fallible=errors.runtime||errors.opaque||!errors.explicit.is_empty();
        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError"&&record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
        let (declaration,error_name)=if fallible{self.completion_error_layout("SchemaFactoryError",&errors,runtime,expr.span)}else{("enum SchemaFactoryError {}".into(),"SchemaFactoryError".into())};
        let declaration=format!("{declaration}type SchemaFactoryFailure={error_name};enum PreparedField<P,T>{{Present(P),Default(T)}}enum PreparedPresence<T,P>{{Absent(T),Present(P)}}");
        let json=matches!(args[1].ty,Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue));
        let mut seen=std::collections::HashSet::new();
        let validator=if json {self.schema_json_validator(root,&mut generated,&mut seen,&mut std::collections::HashSet::new(),&errors);format!("validate_json{root}")}else{self.schema_owned_validator(root,args[1].ty.unwrap(),&mut generated,&mut std::collections::HashSet::new(),&errors)};
        let input=self.expr(fun,&args[1],Mode::Consume);
        let input=if json {format!("({input}).value")}else{input};
        let Type::Union(result)=expr.ty.unwrap() else{unreachable!()};
        let members=&self.program.unions[result.index()].members;
        let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
        let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
        let body=format!("{{ {generated} let schema={schema}; let input={input}; let mut issues=tsrust_schema_issues::Issues::new(64,32,1024); let mut path=Vec::new();let mut failure=None;let validated={validator}(schema,input,&mut path,&mut issues,&mut failure);if let Some(error)=failure{{Err(error)}}else{{Ok(match validated {{ Some(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:value}}),None=>Union{result}::V{err_tag}(Record{err}{{field0:tsrust_schema_error(issues)}}) }})}} }}",schema=self.expr(fun,&args[0],Mode::Read),input=input,result=result.index(),ok_tag=tag(ok),err_tag=tag(err));
        if fallible {
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
            let call=self.exception_regions.result_call(&body,&error_name,&errors,&unions,expr.span).expect("checked schema factory exception transport");
            format!("{{{declaration}{call}}}")
        }else{format!("{{{declaration}match {body}{{Ok(value)=>value,Err(error)=>match error{{}}}}}}")}

    }
    fn schema_owned_validator(&self,id:usize,input_type:Type,generated:&mut String,seen:&mut std::collections::HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->String {
        if !self.program.schema_choices.is_empty(){return self.schema_owned_prepared_validator(id,input_type,generated,seen,errors);}
        let name=format!("validate{id}_{input_type:?}").chars().map(|c|if c.is_ascii_alphanumeric(){c}else{'_'}).collect::<String>();
        if !seen.insert((id,input_type)){return name;}
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let native=self.native_type(output);
        let input_native=self.native_type(input_type);
        let body=match output {
            Type::Union(union) if self.schema_enum_members(output).is_some()=>{
                let mut body=if let Type::Union(input)=input_type {
                    let arms=self.program.unions[input.index()].members.iter().enumerate().map(|(tag,_)|format!("Union{}::V{tag}(value)=>value",input.index())).collect::<Vec<_>>().join(",");format!("let input=match input{{{arms}}};")
                }else{String::new()};
                for (tag,_) in self.program.unions[union.index()].members.iter().enumerate(){body.push_str(&format!("if input==schema.field{tag}.expected{{return Some(Union{}::V{tag}(input));}}",union.index()));}
                body.push_str("issues.push(path,tsrust_schema_issues::Code::Literal,tsrust_schema_issues::Kind::String,tsrust_schema_issues::Kind::String);None");body
            },
            Type::Enum(enumeration,kind)=>{
                if output==input_type {String::from("Some(input)")}else if let Type::Union(input)=input_type {
                    let arms=self.program.unions[input.index()].members.iter().enumerate().map(|(tag,ty)|{let validator=self.schema_owned_validator(id,*ty,generated,seen,errors);format!("Union{}::V{tag}(value)=>{validator}(schema,value,path,issues,failure)",input.index())}).collect::<Vec<_>>().join(",");format!("match input{{{arms}}}")
                }else{
                    let definition=self.program.records[enumeration].enum_definition.as_ref().unwrap();
                    let expected=self.schema_kind(output);
                    let mut body=if kind==crate::syntax::EnumKind::Number&&input_type.scalar_base()==Type::Int {
                        String::from("let number=input as f64;if number as i128!=input as i128{issues.push(path,tsrust_schema_issues::Code::NumberPrecision,tsrust_schema_issues::Kind::Number,tsrust_schema_issues::Kind::Integer);return None;}let input=number;")
                    }else if kind==crate::syntax::EnumKind::Number {
                        String::from("if !input.is_finite(){issues.push(path,tsrust_schema_issues::Code::NumberNonfinite,tsrust_schema_issues::Kind::Number,tsrust_schema_issues::Kind::Number);return None;}")
                    }else{String::new()};
                    for (tag,member) in definition.members.iter().enumerate(){let value=match &member.value{crate::syntax::EnumValue::String(value)=>format!("{value:?}"),crate::syntax::EnumValue::Number(value)=>float_literal(*value)};body.push_str(&format!("if input=={value}{{return Some(Enum{enumeration}::V{tag});}}"));}
                    body.push_str(&format!("issues.push(path,tsrust_schema_issues::Code::Literal,tsrust_schema_issues::Kind::{expected},tsrust_schema_issues::Kind::{expected});None"));body
                }
            },
            Type::Literal(_,_)=>{
                let kind=self.schema_kind(output);
                let conversion=if output.scalar_base()==Type::Number&&input_type.scalar_base()==Type::Int {
                    "let number=input as f64;if number as i128!=input as i128{issues.push(path,tsrust_schema_issues::Code::NumberPrecision,tsrust_schema_issues::Kind::Number,tsrust_schema_issues::Kind::Integer);return None;}let input=number;"
                }else if output.scalar_base()==Type::Number {
                    "if !input.is_finite(){issues.push(path,tsrust_schema_issues::Code::NumberNonfinite,tsrust_schema_issues::Kind::Number,tsrust_schema_issues::Kind::Number);return None;}"
                }else{""};
                format!("{conversion}if input==schema.expected{{Some(input)}}else{{issues.push(path,tsrust_schema_issues::Code::Literal,tsrust_schema_issues::Kind::{kind},tsrust_schema_issues::Kind::{kind});None}}")
            },
            Type::Union(union)=>{
                let base=self.schema_presence_element(output).expect("presence schema union");let child=self.schema_child(base);
                let injection=self.schema_widen_value(base,union,"value");
                if matches!(input_type,Type::Null|Type::Undefined){let tag=self.program.unions[union.index()].members.iter().position(|member|*member==input_type).expect("checked absent schema input");format!("Some(Union{}::V{tag}(input))",union.index())}
                else if let Type::Union(input)=input_type {
                    let arms=self.program.unions[input.index()].members.iter().enumerate().map(|(tag,ty)|{
                        if matches!(ty,Type::Null|Type::Undefined){let target=self.program.unions[union.index()].members.iter().position(|member|member==ty).unwrap();format!("Union{}::V{tag}(value)=>Some(Union{}::V{target}(value))",input.index(),union.index())}
                        else{let validator=self.schema_owned_validator(child,*ty,generated,seen,errors);format!("Union{}::V{tag}(value)=>{validator}(&schema.inner,value,path,issues,failure).map(|value|{injection})",input.index())}
                    }).collect::<Vec<_>>().join(",");format!("match input{{{arms}}}")
                }else{let validator=self.schema_owned_validator(child,input_type,generated,seen,errors);format!("{validator}(&schema.inner,input,path,issues,failure).map(|value|{injection})")}
            },
            Type::Array(element)=>{
                let child=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)&&record.application.as_ref().is_some_and(|a|a.resolved_arguments==[element.ty()])).unwrap();
                let Type::Array(input_element)=input_type else{unreachable!()};
                let validator=self.schema_owned_validator(child,input_element.ty(),generated,seen,errors);
                format!("tsrust_schema_plan::validate_array(input.tsr_owned_items(),|index,value|{{path.push(tsrust_schema_issues::Segment::Index(index));let result={validator}(&schema.element,value,path,issues,failure);path.pop();result}}).map(|values|{{let mut output=<{native}>::from([]);for value in values{{output.push(value);}}output}})")
            },
            Type::Record(object) if self.program.records[object].name.text.starts_with("@schema:retained:")=>{
                let value=self.program.records[object].fields[0].ty.unwrap();let extra=self.native_type(self.program.records[object].fields[1].ty.unwrap());
                let (child_input,input,extras)=if input_type==output {(value,"input.field0",String::from("input.field1"))}else{(input_type,"input",format!("<{extra}>::new()"))};
                let child=self.schema_child(value);let validator=self.schema_owned_validator(child,child_input,generated,seen,errors);
                format!("{validator}(&schema.inner,{input},path,issues,failure).map(|value|Record{}{{field0:value,field1:{extras}}})",self.native_record_declaration(object))
            },
            Type::Record(object)=>{
                let mut body=String::from("let mut valid=true;");
                for (field,value) in self.program.records[object].fields.iter().enumerate(){
                    let child=self.schema_child(value.ty.unwrap());
                    let Type::Record(input_object)=input_type else{unreachable!("owned record input")};
                    let provided=self.program.records[input_object].fields.iter().enumerate().find(|(_,candidate)|candidate.name.text==value.name.text);
                    let validated=if let Some((position,input))=provided {
                        let validator=self.schema_owned_validator(child,input.ty.unwrap(),generated,seen,errors);
                        format!("{validator}(&schema.field{field},input.field{position},path,issues,failure)")
                    }else{
                        let kind=self.schema_kind(value.ty.unwrap());
                        let missing=if let Type::Union(union)=value.ty.unwrap() && let Some(tag)=self.program.unions[union.index()].members.iter().position(|ty|*ty==Type::Undefined) {format!("Some(Union{}::V{tag}(TsrUndefined))",union.index())}else{format!("issues.push(path,tsrust_schema_issues::Code::Missing,tsrust_schema_issues::Kind::{kind},tsrust_schema_issues::Kind::Missing);None")};
                        let default=self.schema_default_value(value.ty.unwrap(),&format!("schema.field{field}"),generated,seen,errors);
                        format!("if failure.is_some(){{None}}else{{match {default}{{Some(value)=>value,None=>{{{missing}}}}}}}")
                    };
                    body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({name:?}));let mut field{field}={validated};path.pop();if field{field}.is_none(){{valid=false;}}",name=value.name.text));
                    body.push_str(&format!("if !valid{{{}}}",(0..=field).map(|field|format!("field{field}.take();")).collect::<String>()));
                }
                let fields=(0..self.program.records[object].fields.len()).map(|field|format!("field{field}:field{field}.unwrap()")).collect::<Vec<_>>().join(",");
                body.push_str(&format!("if valid{{Some(Record{}{{{fields}}})}}else{{None}}",self.native_record_declaration(object)));
                body
            },
            Type::Number if input_type==Type::Int=>{
                "let value=input as f64;if value as i128!=input as i128 {issues.push(path,tsrust_schema_issues::Code::NumberPrecision,tsrust_schema_issues::Kind::Number,tsrust_schema_issues::Kind::Integer);None}else if let Some(code)=schema.bounds.check(value){let code=match code{\"too-small\"=>tsrust_schema_issues::Code::TooSmall,\"too-large\"=>tsrust_schema_issues::Code::TooLarge,_=>unreachable!(\"finite integer conversion\")};issues.push(path,code,tsrust_schema_issues::Kind::Number,tsrust_schema_issues::Kind::Integer);None}else{Some(value)}".into()
            },
            Type::String|Type::Int|Type::Number=>{
                let (value,kind)=match output{Type::String=>("input.as_ref()","String"),Type::Int=>("input","Integer"),_=>("input","Number")};
                format!("if let Some(code)=schema.bounds.check({value}) {{let code=match code{{\"too-small\"=>tsrust_schema_issues::Code::TooSmall,\"too-large\"=>tsrust_schema_issues::Code::TooLarge,_=>tsrust_schema_issues::Code::NumberNonfinite}};issues.push(path,code,tsrust_schema_issues::Kind::{kind},tsrust_schema_issues::Kind::{kind});None}}else{{Some(input)}}")
            },
            Type::Boolean|Type::Null=>"Some(input)".into(),
            _=>unreachable!("unsupported schema output"),
        };
        generated.push_str(&format!("fn {name}(schema:&Record{id},input:{input_native},path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>)->Option<{native}>{{if failure.is_some(){{return None;}}{body}}}"));
        name
    }
    fn schema_json_validator(&self,id:usize,generated:&mut String,seen:&mut std::collections::HashSet<usize>,owned_seen:&mut std::collections::HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet) {
        if !self.program.schema_choices.is_empty(){return self.schema_json_prepared_validator(id,generated,seen,owned_seen,errors);}
        if !seen.insert(id){return;}
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let native=self.native_type(output);
        let (expected,success)=match output {
            Type::Union(_) if self.schema_enum_members(output).is_some()=>{
                let validator=self.schema_owned_validator(id,Type::String,generated,owned_seen,errors);
                ("String",format!("if let tsrust_native_json::Value::String(value)=input{{return {validator}(schema,String::Owned(value),path,issues,failure);}}"))
            },
            Type::Enum(_,kind)=>{
                let base=if kind==crate::syntax::EnumKind::String{Type::String}else{Type::Number};let validator=self.schema_owned_validator(id,base,generated,owned_seen,errors);
                if kind==crate::syntax::EnumKind::String {("String",format!("if let tsrust_native_json::Value::String(value)=input{{return {validator}(schema,String::Owned(value),path,issues,failure);}}"))}
                else {("Number",format!("match tsrust_native_json::read_f64(&input){{Ok(value)=>return {validator}(schema,value,path,issues,failure),Err(error)=>{{let code=match error.code{{\"number-precision\"=>Code::NumberPrecision,\"number-nonfinite\"=>Code::NumberNonfinite,_=>Code::Type}};issues.push(path,code,Kind::Number,observed);return None;}}}}"))}
            },
            Type::Literal(_,kind)=>{
                let validator=self.schema_owned_validator(id,kind.base(),generated,owned_seen,errors);
                let expected=self.schema_kind(output);
                let code=match kind {
                    crate::literal_types::LiteralKind::String=>format!("if let tsrust_native_json::Value::String(value)=input{{return {validator}(schema,String::Owned(value),path,issues,failure);}}"),
                    crate::literal_types::LiteralKind::Boolean=>format!("if let tsrust_native_json::Value::Bool(value)=input{{return {validator}(schema,value,path,issues,failure);}}"),
                    _=>{let reader=if kind==crate::literal_types::LiteralKind::Int{"read_i64"}else{"read_f64"};format!("match tsrust_native_json::{reader}(&input){{Ok(value)=>return {validator}(schema,value,path,issues,failure),Err(error)=>{{let code=match error.code{{\"integer-range\"=>Code::IntegerRange,\"integer-fractional\"=>Code::IntegerFractional,\"number-precision\"=>Code::NumberPrecision,\"number-nonfinite\"=>Code::NumberNonfinite,_=>Code::Type}};issues.push(path,code,Kind::{expected},observed);return None;}}}}")}
                };(expected,code)
            },
            Type::Union(union)=>{
                let base=self.schema_presence_element(output).expect("presence schema union");let child=self.schema_child(base);
                self.schema_json_validator(child,generated,seen,owned_seen,errors);
                let members=&self.program.unions[union.index()].members;
                let null=members.iter().position(|ty|*ty==Type::Null).map(|tag|format!("if matches!(input,tsrust_native_json::Value::Null){{return Some(Union{}::V{tag}(TsrNull));}}",union.index())).unwrap_or_default();
                let injection=self.schema_widen_value(base,union,"value");
                let body=format!("{null}return validate_json{child}(&schema.inner,input,path,issues,failure).map(|value|{injection});");
                (self.schema_kind(base),body)
            },
            Type::Array(element)=>{
                let child=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)&&record.application.as_ref().is_some_and(|a|a.resolved_arguments==[element.ty()])).unwrap();
                self.schema_json_validator(child,generated,seen,owned_seen,errors);
                ("Array",format!("if let tsrust_native_json::Value::Array(values)=input {{return tsrust_schema_plan::validate_array(values,|index,value|{{path.push(tsrust_schema_issues::Segment::Index(index));let result=validate_json{child}(&schema.element,value,path,issues,failure);path.pop();result}}).map(|values|{{let mut output=<{native}>::from([]);for value in values{{output.push(value);}}output}});}}"))
            },
            Type::Record(output_object)=>{
                let retained=self.program.records[output_object].name.text.starts_with("@schema:retained:");
                let object=if retained {let Some(Type::Record(object))=self.program.records[output_object].fields[0].ty else{unreachable!()};object}else{output_object};
                let plan=if retained {"schema.inner"}else{"schema"};
                let mut body=String::from("if let tsrust_native_json::Value::Object(mut input)=input {let mut valid=true;");
                for (field,value) in self.program.records[object].fields.iter().enumerate(){
                    let ty=value.ty.unwrap();let child=self.schema_child(ty);
                    self.schema_json_validator(child,generated,seen,owned_seen,errors);
                    let kind=self.schema_kind(ty);
                    let missing=if let Type::Union(union)=ty && let Some(tag)=self.program.unions[union.index()].members.iter().position(|ty|*ty==Type::Undefined) {format!("Some(Union{}::V{tag}(TsrUndefined))",union.index())}else{format!("issues.push(path,Code::Missing,Kind::{kind},Kind::Missing);None")};
                    let default=self.schema_default_value(ty,&format!("{plan}.field{field}"),generated,owned_seen,errors);
                    let missing=format!("if failure.is_some(){{None}}else{{match {default}{{Some(value)=>value,None=>{{{missing}}}}}}}");
                    body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({name:?}));let mut field{field}=match input.remove({name:?}){{Some(value)=>validate_json{child}(&{plan}.field{field},value,path,issues,failure),None=>{{{missing}}}}};path.pop();if field{field}.is_none(){{valid=false;}}",name=value.name.text));
                    body.push_str(&format!("if !valid{{{}}}",(0..=field).map(|field|format!("field{field}.take();")).collect::<String>()));
                }
                if !retained {body.push_str("if schema.reject_unknown {for (name,value) in input {valid=false;let mut unknown_path=path.clone();unknown_path.push(tsrust_schema_issues::Segment::Field(&name));let observed=match value{tsrust_native_json::Value::Null=>Kind::Null,tsrust_native_json::Value::Bool(_)=>Kind::Boolean,tsrust_native_json::Value::Number(_)=>Kind::Number,tsrust_native_json::Value::String(_)=>Kind::String,tsrust_native_json::Value::Array(_)=>Kind::Array,tsrust_native_json::Value::Object(_)=>Kind::Object};issues.push(&unknown_path,Code::UnknownField,Kind::Missing,observed);}}");}
                let fields=(0..self.program.records[object].fields.len()).map(|field|format!("field{field}:field{field}.unwrap()")).collect::<Vec<_>>().join(",");
                let result=if retained {
                    let extra=self.native_type(self.program.records[output_object].fields[1].ty.unwrap());
                    let json=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::JsonValue)).unwrap();
                    format!("Record{}{{field0:Record{}{{{fields}}},field1:<{extra}>::from_values(input.into_iter().map(|(key,value)|(String::Owned(key),Record{}{{value}})))}}",self.native_record_declaration(output_object),self.native_record_declaration(object),self.native_record_declaration(json))
                }else{format!("Record{}{{{fields}}}",self.native_record_declaration(object))};
                body.push_str(&format!("return if valid{{Some({result})}}else{{None}};}}"));
                ("Object",body)
            },
            Type::String=>{
                let validator=self.schema_owned_validator(id,Type::String,generated,owned_seen,errors);
                ("String",format!("if let tsrust_native_json::Value::String(value)=input {{return {validator}(schema,String::Owned(value),path,issues,failure);}}"))
            },
            Type::Boolean=>("Boolean","if let tsrust_native_json::Value::Bool(value)=input {return Some(value);}".into()),
            Type::Null=>("Null","if let tsrust_native_json::Value::Null=input {return Some(TsrNull);}".into()),
            Type::Int|Type::Number=>{
                let (reader,kind)=if output==Type::Int {("read_i64","Integer")}else{("read_f64","Number")};
                (kind,format!("let failure=match tsrust_native_json::{reader}(&input){{Ok(value)=>match schema.bounds.check(value){{None=>return Some(value),Some(code)=>code}},Err(error)=>error.code}};let code=match failure{{\"integer-range\"=>Code::IntegerRange,\"integer-fractional\"=>Code::IntegerFractional,\"number-precision\"=>Code::NumberPrecision,\"number-nonfinite\"=>Code::NumberNonfinite,\"too-small\"=>Code::TooSmall,\"too-large\"=>Code::TooLarge,\"expected-number\"=>Code::Type,_=>unreachable!(\"numeric schema error\")}};issues.push(path,code,Kind::{kind},observed);return None;"))
            },
            _=>unreachable!("unsupported JSON schema output"),
        };
        generated.push_str(&format!("fn validate_json{id}(schema:&Record{id},input:tsrust_native_json::Value,path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>)->Option<{native}>{{if failure.is_some(){{return None;}}use tsrust_schema_issues::{{Kind,Code}};let observed=match &input{{tsrust_native_json::Value::Null=>Kind::Null,tsrust_native_json::Value::Bool(_)=>Kind::Boolean,tsrust_native_json::Value::Number(_)=>Kind::Number,tsrust_native_json::Value::String(_)=>Kind::String,tsrust_native_json::Value::Array(_)=>Kind::Array,tsrust_native_json::Value::Object(_)=>Kind::Object}};{success} issues.push(path,Code::Type,Kind::{expected},observed);None}}"));
    }

    pub(super) fn schema_widen_value(&self,source:Type,target:UnionId,value:&str)->String {
        let destination=&self.program.unions[target.index()].members;
        if let Type::Union(source)=source {
            let arms=self.program.unions[source.index()].members.iter().enumerate().map(|(tag,ty)|{let to=destination.iter().position(|member|member==ty).expect("schema output alternative");format!("Union{}::V{tag}(value)=>Union{}::V{to}(value)",source.index(),target.index())}).collect::<Vec<_>>().join(",");
            format!("match {value}{{{arms}}}")
        }else{let tag=destination.iter().position(|member|*member==source).expect("schema output member");format!("Union{}::V{tag}({value})",target.index())}
    }
    pub(super) fn schema_enum_members(&self,output:Type)->Option<&[Type]> {
        let Type::Union(id)=output else{return None};let members=&self.program.unions[id.index()].members;
        members.iter().all(|ty|matches!(ty,Type::Literal(_,crate::literal_types::LiteralKind::String))).then_some(members)
    }
    pub(super) fn schema_presence_element(&self,output:Type)->Option<Type> {
        let Type::Union(id)=output else{return None};
        let members=&self.program.unions[id.index()].members;
        if !members.iter().any(|ty|matches!(ty,Type::Null|Type::Undefined)){return None;}
        let present=members.iter().copied().filter(|ty|!matches!(ty,Type::Null|Type::Undefined)).collect::<Vec<_>>();
        let base=match present.as_slice(){[value]=>Some(*value),[]=>Some(Type::Null),_=>self.program.unions.iter().enumerate().find(|(_,union)|union.members==present).map(|(index,union)|Type::Union(UnionId{index:index.try_into().unwrap(),copy:union.members.iter().all(|member|member.is_copy())}))};
        base.filter(|base|self.program.records.iter().any(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)&&record.application.as_ref().is_some_and(|application|application.resolved_arguments==[*base])))
    }
    pub(super) fn schema_child(&self,output:Type)->usize {
        self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)&&record.application.as_ref().is_some_and(|a|a.resolved_arguments==[output])).expect("checked schema child")
    }
    pub(super) fn schema_kind(&self,ty:Type)->&'static str {
        if self.schema_enum_members(ty).is_some(){return "String";}
        match ty.scalar_base(){Type::Enum(_,crate::syntax::EnumKind::String)=>"String",Type::Enum(_,crate::syntax::EnumKind::Number)=>"Number",Type::String=>"String",Type::Int=>"Integer",Type::Number=>"Number",Type::Boolean=>"Boolean",Type::Null=>"Null",Type::Undefined=>"Missing",Type::Array(_)=>"Array",Type::Record(_)=>"Object",Type::Union(_)=>self.schema_presence_element(ty).map(|base|self.schema_kind(base)).unwrap_or("Union"),_=>unreachable!("checked schema output")}
    }

}
