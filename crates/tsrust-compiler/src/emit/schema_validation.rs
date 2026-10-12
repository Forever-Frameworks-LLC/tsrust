//! Generate consuming typed validators while borrowing reusable schema plans.
use super::*;
impl Emitter<'_> {
    pub(super) fn schema_validate(&self,fun:&CheckedFunction,expr:&Expr,args:&[Expr])->String {
        let Type::Record(root)=args[0].ty.unwrap() else{unreachable!()};
        let mut generated=String::new();
        let json=matches!(args[1].ty,Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue));
        self.schema_owned_validator(root,&mut generated);
        if json {self.schema_json_validator(root,&mut generated);}
        let validator=if json {format!("validate_json{root}")}else{format!("validate{root}")};
        let input=self.expr(fun,&args[1],Mode::Consume);
        let input=if json {format!("({input}).value")}else{input};
        let Type::Union(result)=expr.ty.unwrap() else{unreachable!()};
        let members=&self.program.unions[result.index()].members;
        let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
        let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
        format!("{{ {generated} let schema={schema}; let input={input}; let mut issues=tsrust_schema_issues::Issues::new(64,32,1024); let mut path=Vec::new(); match {validator}(schema,input,&mut path,&mut issues) {{ Some(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:value}}),None=>Union{result}::V{err_tag}(Record{err}{{field0:tsrust_schema_error(issues)}}) }} }}",schema=self.expr(fun,&args[0],Mode::Read),input=input,result=result.index(),ok_tag=tag(ok),err_tag=tag(err))
    }
    fn schema_owned_validator(&self,id:usize,generated:&mut String) {
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let native=self.native_type(output);
        let body=match output {
            Type::Array(element)=>{
                let child=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)&&record.application.as_ref().is_some_and(|a|a.resolved_arguments==[element.ty()])).unwrap();
                self.schema_owned_validator(child,generated);
                format!("tsrust_schema_plan::validate_array(input.tsr_owned_items(),|index,value|{{path.push(tsrust_schema_issues::Segment::Index(index));let result=validate{child}(&schema.element,value,path,issues);path.pop();result}}).map(|values|{{let mut output=<{native}>::from([]);for value in values{{output.push(value);}}output}})")
            },
            Type::String|Type::Int|Type::Number=>{
                let (value,kind)=match output{Type::String=>("input.as_ref()","String"),Type::Int=>("input","Integer"),_=>("input","Number")};
                format!("if let Some(code)=schema.bounds.check({value}) {{let code=match code{{\"too-small\"=>tsrust_schema_issues::Code::TooSmall,\"too-large\"=>tsrust_schema_issues::Code::TooLarge,_=>tsrust_schema_issues::Code::NumberNonfinite}};issues.push(path,code,tsrust_schema_issues::Kind::{kind},tsrust_schema_issues::Kind::{kind});None}}else{{Some(input)}}")
            },
            Type::Boolean|Type::Null=>"Some(input)".into(),
            _=>unreachable!("unsupported schema output"),
        };
        generated.push_str(&format!("fn validate{id}(schema:&Record{id},input:{native},path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues)->Option<{native}>{{{body}}}"));
    }
    fn schema_json_validator(&self,id:usize,generated:&mut String) {
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let native=self.native_type(output);
        let (expected,success)=match output {
            Type::Array(element)=>{
                let child=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)&&record.application.as_ref().is_some_and(|a|a.resolved_arguments==[element.ty()])).unwrap();
                self.schema_json_validator(child,generated);
                ("Array",format!("if let tsrust_native_json::Value::Array(values)=input {{return tsrust_schema_plan::validate_array(values,|index,value|{{path.push(tsrust_schema_issues::Segment::Index(index));let result=validate_json{child}(&schema.element,value,path,issues);path.pop();result}}).map(|values|{{let mut output=<{native}>::from([]);for value in values{{output.push(value);}}output}});}}"))
            },
            Type::String=>("String",format!("if let tsrust_native_json::Value::String(value)=input {{return validate{id}(schema,String::Owned(value),path,issues);}}")),
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

}
