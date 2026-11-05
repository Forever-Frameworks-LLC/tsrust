//! Thin URL calls keep borrowed substrings and owned query snapshots distinct.
use super::*;
use crate::Span;
use crate::native_signatures::{NativeTypeId,Signature};
impl Emitter<'_> {
    pub(super) fn url_assignment_target<'a>(&self,target:&'a Expr)->Option<(&'a Expr,&'static crate::native_bindings::NativeBinding)> {
        let ExprKind::Field(receiver,name)=&target.kind else{return None};
        if !matches!(receiver.ty,Some(Type::Record(id)) if self.program.records[id].native_type==Some(NativeTypeId::Url)){return None;}
        Some((receiver,crate::stdlib::url::property(name)?.setter.as_ref()?))
    }
    pub(super) fn url_assignment(&self,fun:&CheckedFunction,target:&Expr,value:&Expr,returned:bool)->String {
        let (receiver,setter)=self.url_assignment_target(target).expect("checked URL assignment");
        let (id,_)=crate::native_bindings::by_symbol(&format!("@native:std:url#{}",setter.name)).unwrap();self.native_bindings.borrow_mut().insert(id);
        let selection=self.storage_selection(fun,receiver,false);
        let input=self.expr(fun,value,Mode::Consume);
        let parameter=setter.parameters[1];
        let argument=if parameter.optional {
            if let Some(union)=value.ty.unwrap().union() {
                let arms=self.program.unions[union.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(value)=>{}",union.index(),match ty {Type::Undefined=>"None",Type::Int=>"Some(*value)",Type::String=>"Some(value.as_ref())",_=>unreachable!("URL setter input")})).collect::<Vec<_>>().join(",");
                format!("match &storage_value{{{arms}}}")
            }else if value.ty==Some(Type::Undefined){"None".into()}
            else if value.ty==Some(Type::Int){"Some(storage_value)".into()}
            else{"Some(storage_value.as_ref())".into()}
        }else{"storage_value.as_ref()".into()};
        let function=match setter.name {"__setHref"=>"set_href","__setProtocol"=>"set_protocol","__setUsername"=>"set_username","__setPassword"=>"set_password","__setHostname"=>"set_hostname","__setPort"=>"set_port","__setPathname"=>"set_pathname","__setSearch"=>"set_search","__setHash"=>"set_hash",_=>unreachable!()};
        let Some(CallTarget::ResultUnwrap(error))=target.target else{unreachable!("computed setter error contract")};
        let Type::Record(record)=error else{unreachable!()};
        let payload=format!("Record{}{{field0:String::Borrowed(error.code),field1:String::Borrowed(error.operation),field2:String::Owned(error.message){}}}",self.native_record_declaration(record),self.schema_record_order(record));
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
        let failed=self.exception_regions.throw_value("payload",error,&unions,target.span).expect("checked URL setter propagation");
        let failure=format!("{{let payload={payload};{failed}}}");
        let receiver=&selection.exclusive;
        let invoke=format!("match tsrust_native_url::{function}({receiver},{argument}){{Ok(())=>{{}},Err(error)=>{failure}}}");
        let setup=&selection.setup;let output=if returned{"storage_value"}else{"()"};
        let storage_type=self.native_type(value.ty.expect("checked URL setter value"));
        format!("{{{setup}let storage_value:{storage_type}={input};{invoke};{output}}}")
    }
    fn url_value(&self, ty:Type, value:&str)->String {
        if ty==Type::String {return format!("String::Owned({value})");}
        if let Type::Record(id)=ty {
            if self.program.records[id].native_type==Some(NativeTypeId::StringView) {return format!("Record{id}{{text:{value}}}");}
        }
        if let Type::Union(id)|Type::BorrowedUnion(id)=ty {
            let members=&self.program.unions[id.index()].members;
            let absent=members.iter().position(|ty|*ty==Type::Undefined).expect("optional URL value");
            let present=1-absent;let item=self.url_value(members[present],"value");
            return format!("match {value}{{Some(value)=>Union{}::V{present}({item}),None=>Union{}::V{absent}(TsrUndefined)}}",id.index(),id.index());
        }
        if ty==Type::Array(ArrayElement::String(0)) {return format!("Array::Heap(({value}).into_iter().map(String::Owned).collect())");}
        if let Type::Array(ArrayElement::Record(id,0))=ty {
            if self.program.records[id].native_type==Some(NativeTypeId::QueryEntry) {
                let item=format!("Record{}{{field0:String::Owned(name),field1:String::Owned(value){}}}",self.native_record_declaration(id),self.schema_record_order(id));
                let rows=format!("({value}).into_iter().map(|(name,value)|{item}).collect()");
                return if self.small_record(id){format!("Array::Heap({rows})")}else{rows};
            }
        }
        value.into()
    }
    fn url_result(&self,ty:Type,call:&str)->String {
        let Type::Union(id)=ty else{unreachable!("URL Result")};let members=&self.program.unions[id.index()].members;
        let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
        let tag=|record|members.iter().position(|ty|*ty==Type::Record(record)).unwrap();
        let record=|id,fields:String|format!("Record{}{{{fields}{}}}",self.native_record_declaration(id),self.schema_record_order(id).trim_start_matches(','));
        let fields=self.program.records[ok].fields.first().map(|field|format!("field0:{},",self.url_value(field.ty.unwrap(),"value"))).unwrap_or_default();
        let Type::Record(error)=self.program.records[err].fields[0].ty.unwrap()else{unreachable!()};
        let failure=record(error,"field0:String::Borrowed(error.code),field1:String::Borrowed(error.operation),field2:String::Owned(error.message),".into());
        format!("match {call}{{Ok(value)=>Union{}::V{}({}),Err(error)=>Union{}::V{}({})}}",id.index(),tag(ok),record(ok,fields),id.index(),tag(err),record(err,format!("field0:{failure},")))
    }
    pub(super) fn url_forward(&self,fun:&CheckedFunction,parameters:&[usize],span:Span,id:usize)->Option<String> {
        if crate::native_bindings::BINDINGS[id].module!="std:url" {return None;}
        let arguments=parameters.iter().map(|&id| {
            let binding=&fun.bindings[id];
            let mut argument=Expr::new(ExprKind::Name(binding.name.clone()),span);
            argument.binding=Some(id);argument.ty=Some(binding.ty);argument
        }).collect::<Vec<_>>();
        let mut result=Expr::new(ExprKind::Undefined,span);result.ty=Some(fun.result);
        self.url_native(fun,&result,id,&arguments)
    }
    pub(super) fn url_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String> {
        let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:url"{return None;}
        self.native_bindings.borrow_mut().insert(id);
        let mut setup=String::new();let mut values=Vec::new();
        for (index,parameter) in binding.parameters.iter().enumerate() {
            let Some(argument)=args.get(index) else {values.push("None".into());continue;};
            let name=format!("url_arg{index}");setup.push_str(&format!("let {name}={};",self.expr(fun,argument,parameter.access)));
            let value=if parameter.optional {let optional=self.native_optional_argument(parameter,name.clone(),argument.ty.unwrap());if parameter.ty.signature()==Signature::Optional(&Signature::String){format!("({optional}).map(|value|value.as_ref())")}else{optional}}
                else if parameter.ty.signature()==Signature::String {format!("{name}.as_ref()")}
                else {name};
            values.push(value);
        }
        let receiver=values.first().cloned().unwrap_or_default();
        let function=match binding.name {
            "parseUrl"=>"parse_url","resolveUrl"=>"resolve_url","formatUrl"=>"format_url",
            "parseQuery"=>"parse_query","formatQuery"=>"format_query","searchParams"=>"search_params","setSearchParams"=>"set_search_params",
            "pathToFileUrl"=>"path_to_file_url","fileUrlToPath"=>"file_url_to_path","encodeURIComponent"=>"encode_uri_component","decodeURIComponent"=>"decode_uri_component",
            "__setHref"=>"set_href","__setProtocol"=>"set_protocol","__setUsername"=>"set_username","__setPassword"=>"set_password","__setHostname"=>"set_hostname","__setPort"=>"set_port","__setPathname"=>"set_pathname","__setSearch"=>"set_search","__setHash"=>"set_hash",
            "__getAll"=>"get_all",name=>name.trim_start_matches('_'),
        };
        let call=if binding.parameters.first().is_some_and(|parameter|parameter.ty.signature()==Signature::Named(NativeTypeId::SearchParams)) && binding.name.starts_with("__") {
            format!("({receiver}).{function}({})",values[1..].join(","))
        } else {format!("tsrust_native_url::{function}({})",values.join(","))};
        let result=if matches!(binding.result.signature(),Signature::Result(..)) {self.url_result(expr.ty.unwrap(),&call)}else{self.url_value(expr.ty.unwrap(),&call)};
        Some(format!("{{{setup}{result}}}"))
    }
}
