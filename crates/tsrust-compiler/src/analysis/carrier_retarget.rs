//! Owned parent frames restore retargeted expressions on success and failure.
use super::*;

#[derive(Clone,Copy)]
enum State {
    Inject,
    Array{index:usize,element:Type},
    Record{index:usize,current:usize,target:usize,wanted:Type},
    Wrapper,
}
struct Parent { value:Expr,destination:Type,state:State }
type Step=Result<Option<(State,Expr,Type)>,Diagnostic>;
fn take(value:&mut Expr)->Expr {
    std::mem::replace(value,Expr::new(ExprKind::Null,value.span))
}
fn restore(value:&mut Expr,state:State,child:Expr){
    match (&mut value.kind,state){
        (ExprKind::Inject{value,..},State::Inject)|(ExprKind::Unary{value,..},State::Wrapper)=>**value=child,
        (ExprKind::Array(values),State::Array{index,..})=>values[index]=child,
        (ExprKind::Record(values),State::Record{index,..})=>values[index].1=child,
        _=>unreachable!("retarget parent kind"),
    }
}
impl Analyzer {
    pub(super) fn retarget_nullable_store(&mut self,value:&mut Expr,mut destination:Type)->Result<(),Diagnostic>{
        let mut current=take(value);
        let mut parents=Vec::<Parent>::new();
        'walk: loop {
            let step=self.prepare_carrier_retarget(&mut current,destination);
            let mut result=match step{
                Ok(Some((state,child,child_destination)))=>{
                    parents.push(Parent{value:current,destination,state});
                    current=child;destination=child_destination;continue;
                }
                Ok(None)=>Ok(()),
                Err(error)=>Err(error),
            };
            while let Some(mut parent)=parents.pop(){
                restore(&mut parent.value,parent.state,current);
                if result.is_ok(){
                    match self.resume_carrier_retarget(&mut parent.value,parent.destination,parent.state){
                        Ok(Some((state,child,child_destination)))=>{
                            parent.state=state;parents.push(parent);
                            current=child;destination=child_destination;continue 'walk;
                        }
                        Ok(None)=>{}
                        Err(error)=>result=Err(error),
                    }
                }
                current=parent.value;
            }
            *value=current;
            return result;
        }
    }
    fn prepare_carrier_retarget(&mut self,value:&mut Expr,destination:Type)->Step{
        if value.ty==Some(destination){return Ok(None);}
        match &mut value.kind{
            ExprKind::Inject{value:inner,union:tag} if destination.union().is_some()=>{
                if let(Some(Type::Record(_)),Some(payload))=(inner.ty,self.unions.payload(destination)){
                    return Ok(Some((State::Inject,take(inner),payload)));
                }
                *tag=destination.union().unwrap();value.ty=Some(destination);
            }
            ExprKind::Array(elements) if matches!(destination,Type::Array(_))=>{
                let Type::Array(element)=destination else{unreachable!()};
                if let Some(first)=elements.first_mut(){return Ok(Some((State::Array{index:0,element:element.ty()},take(first),element.ty())));}
                value.ty=Some(destination);
            }
            ExprKind::Record(_)=>{
                if let(Type::Record(target),Some(Type::Record(current)))=(destination,value.ty){
                    return self.next_carrier_record_field(value,current,target,0);
                }
                self.checked_carrier(value,destination)?;
            }
            ExprKind::Unary{op:UnaryOp::RecordCarrier,value:inner,..}=>return Ok(Some((State::Wrapper,take(inner),destination))),
            _=>self.checked_carrier(value,destination)?,
        }
        Ok(None)
    }
    fn next_carrier_record_field(&mut self,value:&mut Expr,current:usize,target:usize,start:usize)->Step{
        let ExprKind::Record(values)=&mut value.kind else{unreachable!()};
        for(index,(name,field))in values.iter_mut().enumerate().skip(start){
            let Some(wanted)=self.program.records[target].fields.iter().find(|candidate|candidate.name.text==name.text).and_then(|candidate|candidate.ty)else{continue;};
            if field.ty==Some(wanted) || !(wanted.union().is_some() || matches!(wanted,Type::Record(_)|Type::Array(_))){continue;}
            return Ok(Some((State::Record{index,current,target,wanted},take(field),wanted)));
        }
        value.ty=Some(Type::Record(current));
        if current!=target{self.checked_carrier(value,Type::Record(target))?;}
        Ok(None)
    }
    fn resume_carrier_retarget(&mut self,value:&mut Expr,destination:Type,state:State)->Step{
        match state{
            State::Inject=>{
                let ExprKind::Inject{union,..}=&mut value.kind else{unreachable!()};
                *union=destination.union().unwrap();value.ty=Some(destination);
            }
            State::Array{index,element}=>{
                let ExprKind::Array(values)=&mut value.kind else{unreachable!()};
                if let Some(next)=values.get_mut(index+1){return Ok(Some((State::Array{index:index+1,element},take(next),element)));}
                value.ty=Some(destination);
            }
            State::Record{index,mut current,target,wanted}=>{
                let ExprKind::Record(values)=&value.kind else{unreachable!()};
                let(name,field)=&values[index];
                if field.ty==Some(wanted) && let Some(next)=self.record_type_with_field(current,std::slice::from_ref(&name.text),wanted){current=next;}
                return self.next_carrier_record_field(value,current,target,index+1);
            }
            State::Wrapper=>{
                let ExprKind::Unary{value:inner,..}=&value.kind else{unreachable!()};
                if inner.ty==Some(destination){
                    let ExprKind::Unary{value:inner,..}=take(value).into_kind() else{unreachable!()};*value=*inner;
                }else{value.ty=Some(destination);}
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn analyzer()->Analyzer{
        Analyzer::new(crate::syntax::parse("type A={value:int;};type B={value:boolean;};type C={value:int;};function main(){}").unwrap()).unwrap()
    }
    fn name(ty:Type,span:Span)->Expr{
        let mut value=Expr::new(ExprKind::Name("item".into()),span);value.ty=Some(ty);value
    }
    fn wrappers(mut value:Expr,count:usize)->Expr{
        for _ in 0..count{
            value=Expr::new(ExprKind::Unary{op:UnaryOp::RecordCarrier,operator:Span::default(),value:Box::new(value)},Span::default());
            value.ty=Some(Type::Null);
        }
        value
    }
    #[test]
    fn deep_success_removes_completed_wrappers_on_normal_stack(){
        let mut analyzer=analyzer();let span=Span::new(12,16);
        let mut value=wrappers(name(Type::Int,span),16384);
        analyzer.retarget_nullable_store(&mut value,Type::Int).unwrap();
        assert!(matches!(value.kind,ExprKind::Name(_)));assert_eq!(value.ty,Some(Type::Int));assert_eq!(value.span,span);
    }
    #[test]
    fn deep_failure_restores_every_parent_and_original_diagnostic_span(){
        let mut analyzer=analyzer();let span=Span::new(12,16);
        let mut value=wrappers(name(Type::Int,span),16384);
        let error=analyzer.retarget_nullable_store(&mut value,Type::Boolean).unwrap_err();
        assert_eq!(error.code,"unsupported");assert_eq!(error.span,span);
        let mut current=&value;let mut count=0;
        while let ExprKind::Unary{op:UnaryOp::RecordCarrier,value:inner,..}=&current.kind{
            assert_eq!(current.ty,Some(Type::Null));count+=1;current=inner;
        }
        assert_eq!(count,16384);assert!(matches!(current.kind,ExprKind::Name(_)));assert_eq!(current.ty,Some(Type::Int));
        drop(value);
    }
    #[test]
    fn array_failure_retains_completed_prefix_and_restores_failed_child(){
        let mut analyzer=analyzer();let span=Span::new(22,26);
        let original=Type::Array(ArrayElement::Record(2,0));
        let mut value=Expr::new(ExprKind::Array(vec![name(Type::Record(2),Span::default()),name(Type::Record(1),span)]),Span::default());value.ty=Some(original);
        let error=analyzer.retarget_nullable_store(&mut value,Type::Array(ArrayElement::Record(0,0))).unwrap_err();
        assert_eq!(error.span,span);assert_eq!(value.ty,Some(original));
        let ExprKind::Array(values)=&value.kind else{panic!("restored array")};
        assert_eq!(values[0].ty,Some(Type::Record(0)));assert!(matches!(values[0].kind,ExprKind::Unary{op:UnaryOp::RecordCarrier,..}));
        assert_eq!(values[1].ty,Some(Type::Record(1)));assert!(matches!(values[1].kind,ExprKind::Name(_)));
    }
    #[test]
    fn failed_injection_keeps_original_tag_and_payload(){
        let mut analyzer=analyzer();let span=Span::new(30,34);
        let source=analyzer.unions.intern(vec![Type::Record(1),Type::Undefined],span).unwrap();
        let destination=analyzer.unions.intern(vec![Type::Record(0),Type::Undefined],span).unwrap();
        let mut value=Expr::new(ExprKind::Inject{value:Box::new(name(Type::Record(1),span)),union:source.union().unwrap()},Span::default());value.ty=Some(source);
        assert_eq!(analyzer.retarget_nullable_store(&mut value,destination).unwrap_err().span,span);
        assert_eq!(value.ty,Some(source));
        let ExprKind::Inject{value:inner,union}=&value.kind else{panic!("restored injection")};
        assert_eq!(*union,source.union().unwrap());assert_eq!(inner.ty,Some(Type::Record(1)));assert!(matches!(inner.kind,ExprKind::Name(_)));
    }
}
