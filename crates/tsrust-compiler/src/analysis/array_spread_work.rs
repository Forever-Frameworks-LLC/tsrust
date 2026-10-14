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
    /// Moves the next unchecked operand out after deriving its hints from the operand in place.
    pub(super) fn next_array_spread_value(&mut self,context:&Context,expr:&mut Expr,work:&mut Work)->Result<Option<Input>,Diagnostic>{
        let (ExprKind::ArrayOf(args,_)|ExprKind::Array(args))=&mut expr.kind else{unreachable!()};
        let Some(arg)=args.get_mut(work.next) else{return Ok(None)};
        let (value,spread)=if let ExprKind::Unary{op:UnaryOp::Spread,value,..}=&arg.kind{(value.as_ref(),true)}else{(&*arg,false)};
        let length=if spread{if let ExprKind::Array(values)=&value.kind{if values.iter().any(|value|matches!(value.kind,ExprKind::Unary{op:UnaryOp::Spread,..})){None}else{Some(values.len())}}else{None}}else{Some(1)};
        let guessed=self.expression_type_hint(context,value);
        let expected=if spread{self.array_spread_context(value,guessed,work.element)?}else{work.element};
        let copied=if spread{match guessed{Some(Type::Array(element))=>element.ty().is_copy(),Some(Type::Set(id))=>self.program.records[id].set_definition.as_ref().unwrap().key.is_copy(),Some(ty) if self.native_spread_source(ty)=>self.iterable_element(ty).is_some_and(|element|element.ty().is_copy()),_=>false}}else{guessed.is_some_and(Type::is_copy)};
        let access=if spread && guessed==Some(Type::String){Mode::Read}else if copied{Mode::Copy}else{Mode::Consume};
        let value=Self::take_array_spread_operand(arg);
        work.next+=1;
        Ok(Some(Input{value,mode:if spread&&copied{Mode::Read}else{access},expected,spread,length,access}))
    }
    fn take_array_spread_operand(arg:&mut Expr)->Box<Expr>{
        let slot=if let ExprKind::Unary{op:UnaryOp::Spread,value,..}=&mut arg.kind{value}else{
            let span=arg.span;return Box::new(std::mem::replace(arg,Expr::new(ExprKind::Undefined,span)));
        };
        let span=slot.span;std::mem::replace(slot,Box::new(Expr::new(ExprKind::Undefined,span)))
    }
    /// Returns moved operands, including a failed one, to their constructor positions.
    pub(super) fn restore_array_spread_operands(expr:&mut Expr,work:Work,failed:Option<Input>){
        let (ExprKind::ArrayOf(args,_)|ExprKind::Array(args))=&mut expr.kind else{unreachable!()};
        let failed=failed.map(|input|input.value);
        for (arg,value) in args.iter_mut().zip(work.segments.into_iter().map(|segment|segment.value).chain(failed)){
            if let ExprKind::Unary{op:UnaryOp::Spread,value:slot,..}=&mut arg.kind{*slot=value;}else{*arg=*value;}
        }
    }
    pub(super) fn finish_array_spread_value(&mut self,work:&mut Work,input:Input,actual:Type)->Result<(),(Diagnostic,Input)>{
        let element=(||{
            let actual=if input.spread{match actual{Type::Array(element)=>element.ty(),Type::String=>Type::String,Type::Set(id)=>self.program.records[id].set_definition.as_ref().unwrap().key,Type::Map(id)=>Type::Record(self.map_iteration_entry(id,input.value.span)?),ty if self.native_spread_source(ty)=>self.iterable_element(ty).unwrap().ty(),_=>return Err(Diagnostic::new("spread-source","Array spread requires an array, string, Set, Map, Deque, Heap or OrderedSet.",input.value.span))}}else{actual};
            if let Some(element)=work.element{self.require(actual,Some(element),input.value.span)?;}
            Ok(actual)
        })();
        match element{
            Ok(element)=>{
                work.element.get_or_insert(element);
                let Input{value,spread,length,access,..}=input;
                work.segments.push(SpreadSegment{value,spread,element,access,length});Ok(())
            },
            Err(error)=>Err((error,input)),
        }
    }
    pub(super) fn finish_array_spread_work(&mut self,context:&Context,expr:&mut Expr,work:Work,expected:Option<Type>)->Result<Type,(Diagnostic,Work)>{
        let span=expr.span;let slot=span.start;
        let ty=(||{
            let element=work.element.ok_or_else(||Diagnostic::new("array-type","An empty array construction needs an explicit element type or array context.",span))?;
            let ty=self.constructor_array_type(element,span)?;self.require(ty,expected,span)?;Ok(ty)
        })();
        let ty=match ty{Ok(ty)=>ty,Err(error)=>return Err((error,work))};
        let source=work.segments.first().expect("spread array source").value.as_ref();
        let mut call=self.spread_selected(context,source,slot,SpreadSelection::Rest,ty,Mode::Consume);call.span=span;
        let parameters=vec![ArgumentContract{ty,mode:Mode::Consume,omission:crate::ParameterOmission::Required,rest:true}];
        let variant=SpreadCallVariant{selection:SpreadVariantSelection::Count(0),count:0,call:Box::new(call.clone())};
        expr.kind=ExprKind::SpreadCall(Box::new(CheckedSpreadCall{callee:None,segments:work.segments,call:Box::new(call),parameters,selected_types:Vec::new(),selected_unions:Vec::new(),variants:vec![variant],slot,minimum:0,maximum:None,builtin:None}));
        expr.ty=Some(ty);expr.target=Some(CallTarget::ArrayOf);Ok(ty)
    }
}
