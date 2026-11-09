//! Convert owned backend ranges into checked source records without copying text.
use super::*;
use crate::{Span,native_signatures::{NativeTypeId,Signature}};
impl Emitter<'_> {
    fn regex_record(&self,id:usize,fields:Vec<String>)->String {
        let fields=fields.into_iter().enumerate().map(|(index,value)|format!("field{index}:{value}")).collect::<Vec<_>>().join(",");
        format!("Record{}{{{fields}{}}}",self.native_record_declaration(id),self.schema_record_order(id))
    }
    fn regex_value(&self,ty:Type,value:&str)->String {
        match ty {
            Type::String=>format!("String::Owned({value})"),
            Type::Array(element)=>{
                let item=self.regex_value(element.ty(),"item");
                format!("({value}).into_iter().map(|item|{item}).collect::<{}>()",self.native_type(ty))
            },
            Type::Union(id)|Type::BorrowedUnion(id)=>{
                let members=&self.program.unions[id.index()].members;
                let absent=members.iter().position(|ty|*ty==Type::Undefined).expect("optional regex result");
                let present=1-absent;let item=self.regex_value(members[present],"item");
                format!("match {value}{{Some(item)=>Union{}::V{present}({item}),None=>Union{}::V{absent}(TsrUndefined)}}",id.index(),id.index())
            },
            Type::Record(id)=>match self.program.records[id].native_type {
                Some(NativeTypeId::RegexMatch)=>format!("{{let matched={value};{}}}",self.regex_record(id,vec!["matched.start".into(),"matched.end".into()])),
                Some(NativeTypeId::RegexCaptures)=>{
                    let fields=&self.program.records[id].fields;
                    let matched=self.regex_value(fields[0].ty.unwrap(),"captures.matched");
                    let groups=self.regex_value(fields[1].ty.unwrap(),"captures.groups");
                    format!("{{let captures={value};{}}}",self.regex_record(id,vec![matched,groups]))
                },
                _=>value.into(),
            },
            _=>value.into(),
        }
    }
    fn regex_result(&self,ty:Type,call:&str)->String {
        let Type::Union(id)=ty else{unreachable!("regex Result")};let members=&self.program.unions[id.index()].members;
        let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
        let tag=|record|members.iter().position(|ty|*ty==Type::Record(record)).unwrap();
        let success=self.regex_record(ok,vec![self.regex_value(self.program.records[ok].fields[0].ty.unwrap(),"value")]);
        let Type::Record(error)=self.program.records[err].fields[0].ty.unwrap()else{unreachable!()};
        let payload=self.regex_record(error,vec!["String::Borrowed(error.code)".into(),"String::Owned(error.message)".into()]);
        let failure=self.regex_record(err,vec![payload]);
        format!("match {call}{{Ok(value)=>Union{}::V{}({success}),Err(error)=>Union{}::V{}({failure})}}",id.index(),tag(ok),id.index(),tag(err))
    }
    fn regex_options(&self,ty:Type,value:&str)->String {
        let Type::Record(id)=ty else{unreachable!("RegexOptions")};
        let fields=self.program.records[id].fields.iter().enumerate().map(|(field,metadata)|{
            let Type::Union(id)=metadata.ty.unwrap()else{unreachable!()};
            let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(value)=>{}",id.index(),if *ty==Type::Undefined{"None"}else{"Some(*value)"})).collect::<Vec<_>>().join(",");
            format!("match &({value}).field{field}{{{arms}}}")
        }).collect::<Vec<_>>();
        format!("tsrust_native_regex::Options{{size_limit:{},dfa_size_limit:{},nest_limit:{}}}",fields[0],fields[1],fields[2])
    }
    pub(super) fn regex_forward(&self,fun:&CheckedFunction,parameters:&[usize],span:Span,id:usize)->Option<String>{
        if crate::native_bindings::BINDINGS[id].module!="std:regex"{return None;}
        let arguments=parameters.iter().map(|&id|{let binding=&fun.bindings[id];let mut value=Expr::new(ExprKind::Name(binding.name.clone()),span);value.binding=Some(id);value.ty=Some(binding.ty);value}).collect::<Vec<_>>();
        let mut result=Expr::new(ExprKind::Undefined,span);result.ty=Some(fun.result);self.regex_native(fun,&result,id,&arguments)
    }
    pub(super) fn regex_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
        let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:regex"{return None;}
        self.native_bindings.borrow_mut().insert(id);
        let mut setup=String::new();let mut values=Vec::new();
        for(index,parameter)in binding.parameters.iter().enumerate(){
            let Some(argument)=args.get(index)else{values.push("None".into());continue;};
            let name=format!("regex_arg{index}");
            let value=if parameter.ty.signature()==Signature::String{self.string_observation(fun,argument)}else{self.expr(fun,argument,parameter.access)};
            if parameter.optional {
                let optional=self.native_optional_argument(parameter,value,argument.ty.unwrap());
                let signature=crate::native_binding_types::resolve(&self.program.records,&parameter.ty.signature()).expect("prepared optional RegexOptions");
                let ty=self.program.unions[signature.union().expect("optional RegexOptions").index()].members.iter().copied().find(|ty|*ty!=Type::Undefined).expect("RegexOptions payload");
                let options=format!("({optional}).map(|value:&{}|{})",self.native_type(ty),self.regex_options(ty,"value"));
                setup.push_str(&format!("let regex_options={options};"));values.push("regex_options.as_ref()".into());
            }else{
                setup.push_str(&format!("let {name}={value};"));
                if parameter.ty.signature()==Signature::String{values.push(format!("{name}.as_ref()"));}else{values.push(name);}
            }
        }
        let function=match binding.name{"compile"=>"compile","escape"=>"escape","__isMatch"=>"is_match","__find"=>"find","__findAll"=>"find_all","__captures"=>"captures","__capturesAll"=>"captures_all","__replace"=>"replace","__replaceAll"=>"replace_all","__split"=>"split","__stringMatch"=>"string_match","__stringMatchAll"=>"string_match_all","__stringSearch"=>"string_search",_=>unreachable!()};
        let call=format!("tsrust_native_regex::{function}({})",values.join(","));
        let result=if matches!(binding.result.signature(),Signature::Result(..)){self.regex_result(expr.ty.unwrap(),&call)}else{self.regex_value(expr.ty.unwrap(),&call)};
        Some(format!("{{{setup}{result}}}"))
    }
}
