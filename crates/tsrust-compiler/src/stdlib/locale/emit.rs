//! Convert owned backend ranges into checked source records without copying text.
use super::*;
use crate::{Span,native_signatures::{NativeTypeId,Signature}};
impl Emitter<'_> {
    fn locale_record(&self,id:usize,fields:Vec<String>)->String {
        let fields=fields.into_iter().enumerate().map(|(index,value)|format!("field{index}:{value}")).collect::<Vec<_>>().join(",");
        format!("Record{}{{{fields}{}}}",self.native_record_declaration(id),self.schema_record_order(id))
    }
    fn locale_value(&self,ty:Type,value:&str)->String {
        match ty {
            Type::String=>format!("String::Owned({value})"),
            Type::Array(element)=>{
                let item=self.locale_value(element.ty(),"item");
                format!("({value}).into_iter().map(|item|{item}).collect::<{}>()",self.native_type(ty))
            },
            Type::Record(id)=>match self.program.records[id].native_type {
                Some(NativeTypeId::SegmentRange)=>format!("{{let range={value};{}}}",self.locale_record(id,vec!["range.start".into(),"range.end".into()])),
                _=>value.into(),
            },
            _=>value.into(),
        }
    }
    fn locale_result(&self,ty:Type,call:&str)->String {
        let Type::Union(id)=ty else{unreachable!("locale Result")};let members=&self.program.unions[id.index()].members;
        let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
        let tag=|record|members.iter().position(|ty|*ty==Type::Record(record)).unwrap();
        let success=self.locale_record(ok,vec![self.locale_value(self.program.records[ok].fields[0].ty.unwrap(),"value")]);
        let Type::Record(error)=self.program.records[err].fields[0].ty.unwrap()else{unreachable!()};
        let payload=self.locale_record(error,vec!["String::Borrowed(error.code)".into(),"String::Owned(error.message)".into()]);
        let failure=self.locale_record(err,vec![payload]);
        format!("match {call}{{Ok(value)=>Union{}::V{}({success}),Err(error)=>Union{}::V{}({failure})}}",id.index(),tag(ok),id.index(),tag(err))
    }
    fn locale_options(&self,ty:Type,value:&str)->String{
        let Type::Record(id)=ty else{unreachable!("locale options")};
        let fields=self.program.records[id].fields.iter().enumerate().map(|(field,metadata)|{
            let Type::Union(union)=metadata.ty.unwrap()else{unreachable!()};
            let arms=self.program.unions[union.index()].members.iter().enumerate().map(|(tag,ty)|{
                let converted:String=match ty{Type::Undefined=>"None".into(),Type::Int|Type::Boolean=>"Some(*value)".into(),_=>"Some(value.as_ref().to_owned())".into()};
                format!("Union{}::V{tag}(value)=>{converted}",union.index())
            }).collect::<Vec<_>>().join(",");
            let name=match metadata.name.text.as_str(){"alternateHandling"=>"alternate_handling","maxVariable"=>"max_variable","caseLevel"=>"case_level","fractionDigits"=>"fraction_digits",name=>name};
            format!("{name}:match &({value}).field{field}{{{arms}}}")
        }).collect::<Vec<_>>().join(",");
        format!("tsrust_native_locale::{}{{{fields}}}",if self.program.records[id].native_type==Some(NativeTypeId::CollatorOptions){"CollatorOptions"}else{"NumberFormatOptions"})
    }
    fn locale_number(&self,ty:Type,value:&str,formatter:&str)->String{
        match ty{
            Type::Int=>format!("tsrust_native_locale::format_int({formatter},{value})"),
            Type::Record(id)if self.program.records[id].native_type==Some(NativeTypeId::Decimal)=>format!("tsrust_native_locale::format_decimal({formatter},&tsrust_native_decimal::format({value}))"),
            Type::Union(union)|Type::BorrowedUnion(union)=>{
                let arms=self.program.unions[union.index()].members.iter().enumerate().map(|(tag,ty)|format!("ViewUnion{}::V{tag}(input)=>{}",union.index(),self.locale_number(*ty,if *ty==Type::Int{"*input"}else{"input"},formatter))).collect::<Vec<_>>().join(",");
                format!("match {value}{{{arms}}}")
            },_=>unreachable!("checked int or Decimal")
        }
    }
    pub(super) fn locale_forward(&self,fun:&CheckedFunction,parameters:&[usize],span:Span,id:usize)->Option<String>{
        if crate::native_bindings::BINDINGS[id].module!="std:locale"{return None;}
        let arguments=parameters.iter().map(|&id|{let binding=&fun.bindings[id];let mut value=Expr::new(ExprKind::Name(binding.name.clone()),span);value.binding=Some(id);value.ty=Some(binding.ty);value}).collect::<Vec<_>>();
        let mut result=Expr::new(ExprKind::Undefined,span);result.ty=Some(fun.result);self.locale_native(fun,&result,id,&arguments)
    }
    pub(super) fn locale_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
        let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:locale"{return None;}
        self.native_bindings.borrow_mut().insert(id);
        let mut setup=String::new();let mut values=Vec::new();
        for(index,parameter)in binding.parameters.iter().enumerate(){
            let Some(argument)=args.get(index)else{values.push("None".into());continue;};
            let name=format!("locale_arg{index}");
            let value=if parameter.ty.signature()==Signature::String{self.string_observation(fun,argument)}else{self.expr(fun,argument,parameter.access)};
            if parameter.optional {
                let optional=self.native_optional_argument(parameter,value,argument.ty.unwrap());
                let signature=crate::native_binding_types::resolve(&self.program.records,&parameter.ty.signature()).expect("prepared optional locale options");
                let ty=self.program.unions[signature.union().expect("optional locale options").index()].members.iter().copied().find(|ty|*ty!=Type::Undefined).expect("locale options payload");
                let options=format!("({optional}).map(|value:&{}|{})",self.native_type(ty),self.locale_options(ty,"value"));
                setup.push_str(&format!("let locale_options={options};"));values.push("locale_options.as_ref()".into());
            }else{
                setup.push_str(&format!("let {name}={value};"));
                if parameter.ty.signature()==Signature::String{values.push(format!("{name}.as_ref()"));}else{values.push(name);}
            }
        }
        let function=match binding.name{"parseLocale"=>"parse_locale","formatLocale"=>"format_locale","createCollator"=>"create_collator","__compare"=>"compare","__stringCompare"=>"string_compare","createSegmenter"=>"create_segmenter","__segment"=>"segment","createNumberFormat"=>"create_number_format","__formatNumber"=>"format_int",_=>unreachable!()};
        let call=if binding.name=="__formatNumber"{self.locale_number(args[1].ty.unwrap(),&values[1],&values[0])}else{format!("tsrust_native_locale::{function}({})",values.join(","))};
        let result=if matches!(binding.result.signature(),Signature::Result(..)){self.locale_result(expr.ty.unwrap(),&call)}else{self.locale_value(expr.ty.unwrap(),&call)};
        Some(format!("{{{setup}{result}}}"))
    }
}
