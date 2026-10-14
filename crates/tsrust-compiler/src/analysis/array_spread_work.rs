//! Constructor source checks suspend between left-to-right inference steps.
use super::*;
pub(super) struct Work { pub next:usize, element:Option<Type>, segments:Vec<SpreadSegment> }
pub(super) struct Input { pub value:Box<Expr>, pub mode:Mode, pub expected:Option<Type>, spread:bool, length:Option<usize>, access:Mode }
impl Analyzer {
    pub(super) fn start_array_spread_work(&mut self,context:&Context,expr:&Expr,expected:Option<Type>)->Result<Work,Diagnostic>{
        let (args,annotation)=match &expr.kind{ExprKind::ArrayOf(args,annotation)=>(args,annotation.as_ref()),ExprKind::Array(args)=>(args,None),_=>unreachable!()};
        let annotated=annotation.map(|annotation|self.resolve_value_annotation(annotation,false)).transpose()?;
        let element=annotated.or_else(||if let Some(Type::Array(element))=expected{Some(element.ty())}else{None}).or_else(||{
            args.iter().filter_map(|arg|{
                if let ExprKind::Unary{op:UnaryOp::Spread,value,..}=&arg.kind{
                    match self.expression_type_hint(context,value)?{Type::Array(element)=>Some(element.ty()),_=>None}
                }else{self.expression_type_hint(context,arg)}
            }).find(|ty|matches!(ty,Type::Number|Type::Array(ArrayElement::Number(_))))
        });
        Ok(Work{next:0,element,segments:Vec::new()})
    }
    pub(super) fn next_array_spread_value(&mut self,context:&Context,expr:&Expr,work:&mut Work)->Result<Option<Input>,Diagnostic>{
        let (ExprKind::ArrayOf(args,_)|ExprKind::Array(args))=&expr.kind else{unreachable!()};
        let Some(arg)=args.get(work.next) else{return Ok(None)};
        let (value,spread)=if let ExprKind::Unary{op:UnaryOp::Spread,value,..}=&arg.kind{(value.clone(),true)}else{(Box::new(arg.clone()),false)};
        let length=if spread{if let ExprKind::Array(values)=&value.kind{if values.iter().any(|value|matches!(value.kind,ExprKind::Unary{op:UnaryOp::Spread,..})){None}else{Some(values.len())}}else{None}}else{Some(1)};
        let guessed=self.expression_type_hint(context,&value);
        let expected=if spread{self.array_spread_context(&value,guessed,work.element)?}else{work.element};
        let copied=if spread{match guessed{Some(Type::Array(element))=>element.ty().is_copy(),Some(Type::Set(id))=>self.program.records[id].set_definition.as_ref().unwrap().key.is_copy(),Some(ty) if self.native_spread_source(ty)=>self.iterable_element(ty).is_some_and(|element|element.ty().is_copy()),_=>false}}else{guessed.is_some_and(Type::is_copy)};
        let access=if spread && guessed==Some(Type::String){Mode::Read}else if copied{Mode::Copy}else{Mode::Consume};
        work.next+=1;
        Ok(Some(Input{value,mode:if spread&&copied{Mode::Read}else{access},expected,spread,length,access}))
    }
    pub(super) fn finish_array_spread_value(&mut self,work:&mut Work,input:Input,actual:Type)->Result<(),Diagnostic>{
        let Input{value,spread,length,access,..}=input;
        let actual=if spread{match actual{Type::Array(element)=>element.ty(),Type::String=>Type::String,Type::Set(id)=>self.program.records[id].set_definition.as_ref().unwrap().key,Type::Map(id)=>Type::Record(self.map_iteration_entry(id,value.span)?),ty if self.native_spread_source(ty)=>self.iterable_element(ty).unwrap().ty(),_=>return Err(Diagnostic::new("spread-source","Array spread requires an array, string, Set, Map, Deque, Heap or OrderedSet.",value.span))}}else{actual};
        if let Some(element)=work.element{self.require(actual,Some(element),value.span)?;}else{work.element=Some(actual);}
        work.segments.push(SpreadSegment{value,spread,element:actual,access,length});Ok(())
    }
    pub(super) fn finish_array_spread_work(&mut self,context:&Context,expr:&mut Expr,work:Work,expected:Option<Type>)->Result<Type,Diagnostic>{
        let span=expr.span;let slot=span.start;
        let element=work.element.ok_or_else(||Diagnostic::new("array-type","An empty array construction needs an explicit element type or array context.",span))?;
        let ty=self.constructor_array_type(element,span)?;self.require(ty,expected,span)?;
        let source=work.segments.first().expect("spread array source").value.as_ref();
        let mut call=self.spread_selected(context,source,slot,SpreadSelection::Rest,ty,Mode::Consume);call.span=span;
        let parameters=vec![ArgumentContract{ty,mode:Mode::Consume,omission:crate::ParameterOmission::Required,rest:true}];
        let variant=SpreadCallVariant{selection:SpreadVariantSelection::Count(0),count:0,call:Box::new(call.clone())};
        expr.kind=ExprKind::SpreadCall(Box::new(CheckedSpreadCall{callee:None,segments:work.segments,call:Box::new(call),parameters,selected_types:Vec::new(),selected_unions:Vec::new(),variants:vec![variant],slot,minimum:0,maximum:None,builtin:None}));
        expr.ty=Some(ty);expr.target=Some(CallTarget::ArrayOf);Ok(ty)
    }
}
