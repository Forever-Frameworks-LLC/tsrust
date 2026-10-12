//! Generate consuming typed validators while borrowing reusable schema plans.
use super::*;
impl Emitter<'_> {
    pub(super) fn schema_validate(&self,fun:&CheckedFunction,expr:&Expr,args:&[Expr])->String {
        let Type::Record(root)=args[0].ty.unwrap() else{unreachable!()};
        let mut generated=String::new();
        let json=matches!(args[1].ty,Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue));
        let mut seen=std::collections::HashSet::new();
        if json {self.schema_json_validator(root,&mut generated,&mut seen);}else{self.schema_owned_validator(root,args[1].ty.unwrap(),&mut generated,&mut seen);}
        let validator=if json {format!("validate_json{root}")}else{format!("validate{root}")};
        let input=self.expr(fun,&args[1],Mode::Consume);
        let input=if json {format!("({input}).value")}else{input};
        let Type::Union(result)=expr.ty.unwrap() else{unreachable!()};
        let members=&self.program.unions[result.index()].members;
        let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
        let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
        format!("{{ {generated} let schema={schema}; let input={input}; let mut issues=tsrust_schema_issues::Issues::new(64,32,1024); let mut path=Vec::new(); match {validator}(schema,input,&mut path,&mut issues) {{ Some(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:value}}),None=>Union{result}::V{err_tag}(Record{err}{{field0:tsrust_schema_error(issues)}}) }} }}",schema=self.expr(fun,&args[0],Mode::Read),input=input,result=result.index(),ok_tag=tag(ok),err_tag=tag(err))
    }
    fn schema_owned_validator(&self,id:usize,input_type:Type,generated:&mut String,seen:&mut std::collections::HashSet<usize>) {
        if !seen.insert(id){return;}
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let native=self.native_type(output);
        let input_native=self.native_type(input_type);
        let body=match output {
            Type::Union(union)=>{
                let base=self.schema_presence_element(output).expect("presence schema union");let child=self.schema_child(base);
                let members=&self.program.unions[union.index()].members;
                if input_type==Type::Int&&base==Type::Number {
                    self.schema_owned_validator(child,Type::Int,generated,seen);
                    let tag=members.iter().position(|ty|*ty==base).unwrap();
                    format!("validate{child}(&schema.inner,input,path,issues).map(Union{}::V{tag})",union.index())
                }else{
                self.schema_owned_validator(child,base,generated,seen);
                let Type::Union(input)=input_type else{unreachable!("presence input is injected")};
                let arms=self.program.unions[input.index()].members.iter().enumerate().map(|(tag,ty)|{
                    let target=members.iter().position(|member|member==ty).unwrap();
                    if matches!(ty,Type::Null|Type::Undefined){format!("Union{}::V{tag}(value)=>Some(Union{}::V{target}(value))",input.index(),union.index())}
                    else{format!("Union{}::V{tag}(value)=>validate{child}(&schema.inner,value,path,issues).map(Union{}::V{target})",input.index(),union.index())}
                }).collect::<Vec<_>>().join(",");
                format!("match input{{{arms}}}")
                }
            },
            Type::Array(element)=>{
                let child=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)&&record.application.as_ref().is_some_and(|a|a.resolved_arguments==[element.ty()])).unwrap();
                let Type::Array(input_element)=input_type else{unreachable!()};
                self.schema_owned_validator(child,input_element.ty(),generated,seen);
                format!("tsrust_schema_plan::validate_array(input.tsr_owned_items(),|index,value|{{path.push(tsrust_schema_issues::Segment::Index(index));let result=validate{child}(&schema.element,value,path,issues);path.pop();result}}).map(|values|{{let mut output=<{native}>::from([]);for value in values{{output.push(value);}}output}})")
            },
            Type::Record(object) if self.program.records[object].name.text.starts_with("@schema:retained:")=>{
                let value=self.program.records[object].fields[0].ty.unwrap();let extra=self.native_type(self.program.records[object].fields[1].ty.unwrap());
                let (child_input,input,extras)=if input_type==output {(value,"input.field0",String::from("input.field1"))}else{(input_type,"input",format!("<{extra}>::new()"))};
                let child=self.schema_child(value);self.schema_owned_validator(child,child_input,generated,seen);
                format!("validate{child}(&schema.inner,{input},path,issues).map(|value|Record{}{{field0:value,field1:{extras}}})",self.native_record_declaration(object))
            },
            Type::Record(object)=>{
                let mut body=String::from("let mut valid=true;");
                for (field,value) in self.program.records[object].fields.iter().enumerate(){
                    let child=self.schema_child(value.ty.unwrap());
                    self.schema_owned_validator(child,value.ty.unwrap(),generated,seen);
                    body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({name:?}));let mut field{field}=validate{child}(&schema.field{field},input.field{field},path,issues);path.pop();if field{field}.is_none(){{valid=false;}}",name=value.name.text));
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
        generated.push_str(&format!("fn validate{id}(schema:&Record{id},input:{input_native},path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues)->Option<{native}>{{{body}}}"));
    }
    fn schema_json_validator(&self,id:usize,generated:&mut String,seen:&mut std::collections::HashSet<usize>) {
        if !seen.insert(id){return;}
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let native=self.native_type(output);
        let (expected,success)=match output {
            Type::Union(union)=>{
                let base=self.schema_presence_element(output).expect("presence schema union");let child=self.schema_child(base);
                self.schema_json_validator(child,generated,seen);
                let members=&self.program.unions[union.index()].members;
                let null=members.iter().position(|ty|*ty==Type::Null).map(|tag|format!("if matches!(input,tsrust_native_json::Value::Null){{return Some(Union{}::V{tag}(TsrNull));}}",union.index())).unwrap_or_default();
                let tag=members.iter().position(|ty|*ty==base).unwrap();
                let body=format!("{null}return validate_json{child}(&schema.inner,input,path,issues).map(Union{}::V{tag});",union.index());
                (self.schema_kind(base),body)
            },
            Type::Array(element)=>{
                let child=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)&&record.application.as_ref().is_some_and(|a|a.resolved_arguments==[element.ty()])).unwrap();
                self.schema_json_validator(child,generated,seen);
                ("Array",format!("if let tsrust_native_json::Value::Array(values)=input {{return tsrust_schema_plan::validate_array(values,|index,value|{{path.push(tsrust_schema_issues::Segment::Index(index));let result=validate_json{child}(&schema.element,value,path,issues);path.pop();result}}).map(|values|{{let mut output=<{native}>::from([]);for value in values{{output.push(value);}}output}});}}"))
            },
            Type::Record(output_object)=>{
                let retained=self.program.records[output_object].name.text.starts_with("@schema:retained:");
                let object=if retained {let Some(Type::Record(object))=self.program.records[output_object].fields[0].ty else{unreachable!()};object}else{output_object};
                let plan=if retained {"schema.inner"}else{"schema"};
                let mut body=String::from("if let tsrust_native_json::Value::Object(mut input)=input {let mut valid=true;");
                for (field,value) in self.program.records[object].fields.iter().enumerate(){
                    let ty=value.ty.unwrap();let child=self.schema_child(ty);
                    self.schema_json_validator(child,generated,seen);
                    let kind=self.schema_kind(ty);
                    let missing=if let Type::Union(union)=ty && let Some(tag)=self.program.unions[union.index()].members.iter().position(|ty|*ty==Type::Undefined) {format!("Some(Union{}::V{tag}(TsrUndefined))",union.index())}else{format!("issues.push(path,Code::Missing,Kind::{kind},Kind::Missing);None")};
                    body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({name:?}));let mut field{field}=match input.remove({name:?}){{Some(value)=>validate_json{child}(&{plan}.field{field},value,path,issues),None=>{{{missing}}}}};path.pop();if field{field}.is_none(){{valid=false;}}",name=value.name.text));
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
                self.schema_owned_validator(id,Type::String,generated,&mut std::collections::HashSet::new());
                ("String",format!("if let tsrust_native_json::Value::String(value)=input {{return validate{id}(schema,String::Owned(value),path,issues);}}"))
            },
            Type::Boolean=>("Boolean","if let tsrust_native_json::Value::Bool(value)=input {return Some(value);}".into()),
            Type::Null=>("Null","if let tsrust_native_json::Value::Null=input {return Some(TsrNull);}".into()),
            Type::Int|Type::Number=>{
                let (reader,kind)=if output==Type::Int {("read_i64","Integer")}else{("read_f64","Number")};
                (kind,format!("let failure=match tsrust_native_json::{reader}(&input){{Ok(value)=>match schema.bounds.check(value){{None=>return Some(value),Some(code)=>code}},Err(error)=>error.code}};let code=match failure{{\"integer-range\"=>Code::IntegerRange,\"integer-fractional\"=>Code::IntegerFractional,\"number-precision\"=>Code::NumberPrecision,\"number-nonfinite\"=>Code::NumberNonfinite,\"too-small\"=>Code::TooSmall,\"too-large\"=>Code::TooLarge,\"expected-number\"=>Code::Type,_=>unreachable!(\"numeric schema error\")}};issues.push(path,code,Kind::{kind},observed);return None;"))
            },
            _=>unreachable!("unsupported JSON schema output"),
        };
        generated.push_str(&format!("fn validate_json{id}(schema:&Record{id},input:tsrust_native_json::Value,path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues)->Option<{native}>{{use tsrust_schema_issues::{{Kind,Code}};let observed=match &input{{tsrust_native_json::Value::Null=>Kind::Null,tsrust_native_json::Value::Bool(_)=>Kind::Boolean,tsrust_native_json::Value::Number(_)=>Kind::Number,tsrust_native_json::Value::String(_)=>Kind::String,tsrust_native_json::Value::Array(_)=>Kind::Array,tsrust_native_json::Value::Object(_)=>Kind::Object}};{success} issues.push(path,Code::Type,Kind::{expected},observed);None}}"));
    }

    pub(super) fn schema_presence_element(&self,output:Type)->Option<Type> {
        let Type::Union(id)=output else{return None};
        let members=&self.program.unions[id.index()].members;
        if !members.iter().any(|ty|matches!(ty,Type::Null|Type::Undefined)){return None;}
        let present=members.iter().copied().filter(|ty|!matches!(ty,Type::Null|Type::Undefined)).collect::<Vec<_>>();
        match present.as_slice(){[value]=>Some(*value),[]=>Some(Type::Null),_=>None}
    }
    pub(super) fn schema_child(&self,output:Type)->usize {
        self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)&&record.application.as_ref().is_some_and(|a|a.resolved_arguments==[output])).expect("checked schema child")
    }
    fn schema_kind(&self,ty:Type)->&'static str {
        match ty{Type::String=>"String",Type::Int=>"Integer",Type::Number=>"Number",Type::Boolean=>"Boolean",Type::Null=>"Null",Type::Array(_)=>"Array",Type::Record(_)=>"Object",Type::Union(_)=>self.schema_kind(self.schema_presence_element(ty).expect("presence kind")),_=>unreachable!("checked schema output")}
    }

}
