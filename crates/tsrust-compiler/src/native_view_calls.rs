//! Checked callee origins map formal source slots to actual argument lifetimes.
use super::*;
use super::native_view_loans::{LifetimeSource,Loan};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub(super) enum Target {Function(usize),Closure(usize),Method(usize,usize)}

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) enum Source { Argument(usize), Capture(usize), BorrowedCapture(usize), Receiver(usize) }

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Returned { pub(super) source:Source, pub(super) loan:Loan }

fn validate_storage_capture(storage:Mode,invocation:Mode,loan:&Loan,span:Span)->Result<(),Diagnostic> {
    use crate::native_return_borrows::BorrowAccess;
    let message=if storage==Mode::Read && loan.access!=BorrowAccess::ReadOnly {
        Some("A shared capture cannot return exclusive access to its owner.")
    } else if storage!=Mode::Read && invocation==Mode::Consume {
        Some("A consuming invocation cannot retain a borrow of its capture storage; transferred capture borrows require separate planning.")
    } else if storage!=Mode::Read && loan.access==BorrowAccess::Exclusive && invocation!=Mode::Mutate {
        Some("Returning exclusive capture access requires a mutable invocation.")
    } else {None};
    match message {Some(message)=>Err(Diagnostic::new("native-view-call",message,span)),None=>Ok(())}
}

pub(super) fn validate_closures(closures:&[CheckedClosure])->Result<(),Diagnostic> {
    for closure in closures {
        for method in std::iter::once(None).chain((0..closure.methods.len()).map(Some)) {
            let (function,parameters,invocation)=match method {
                Some(id)=>{let method=&closure.methods[id];(&method.function,&method.parameters,method.invocation)},
                None=>(&closure.function,&closure.parameters,closure.invocation),
            };
            for loan in function.native_returns.as_deref().into_iter().flatten() {
                if !matches!(loan.source,LifetimeSource::Storage) || parameters.contains(&loan.root) {continue;}
                let position=match method {
                    Some(id)=>closure.methods[id].captures.iter().position(|root|*root==loan.root),
                    None=>closure.captures.iter().position(|capture|capture.inner==loan.root),
                }.ok_or_else(||Diagnostic::new("native-view-call","Returned view has no invocation or capture source.",closure.span))?;
                validate_storage_capture(closure.captures[position].storage,invocation,loan,closure.span)?;
            }
        }
    }
    Ok(())
}

pub(super) fn target(expression:&Expr)->Option<Target> {
    match expression.target? {
        CallTarget::Function(id)|CallTarget::ForwardFunction(id)=>Some(Target::Function(id)),
        CallTarget::Closure(_)=>expression.closure_origin().map(Target::Closure),
        CallTarget::ClosureMethod(method)=>expression.closure_origin().map(|origin|Target::Method(origin,method)),
        _=>None,
    }
}

impl Analyzer {
    pub(super) fn remember_native_function_returns(&self,context:&mut Context,id:usize,span:Span)->Result<(),Diagnostic> {
        if !self.native_borrow_types {return Ok(());}
        let Some(function)=self.checked[id].as_ref() else {
            if self.native_shape_contains_view(self.signatures[id].result) {
                return Err(Diagnostic::new("native-view-call","Recursive returned views require a stabilized source lifetime summary.",span));
            }
            return Ok(());
        };
        if let Some(origins)=&function.native_returns {
            let summaries=context.native_view_function_returns.get_or_insert_with(Default::default);
            let mapped=origins.iter().map(|loan|Returned{source:Source::Argument(loan.root),loan:loan.clone()}).collect::<Vec<_>>();
            if summaries.get(&Target::Function(id))!=Some(&mapped) {
                summaries.insert(Target::Function(id),mapped);
            }
            context.native_view_calls=true;
        } else if let Some(summaries)=context.native_view_function_returns.as_mut() {
            summaries.remove(&Target::Function(id));
        }
        Ok(())
    }
    pub(super) fn remember_native_closure_returns(&self,context:&mut Context,origin:usize,method:Option<usize>,span:Span)->Result<(),Diagnostic> {
        if !self.native_borrow_types {return Ok(());}
        let closure=&self.closures[origin];
        let (function,parameters,key)=match method {
            Some(method)=>{let checked=&closure.methods[method];(&checked.function,&checked.parameters,Target::Method(origin,method))},
            None=>(&closure.function,&closure.parameters,Target::Closure(origin)),
        };
        let Some(origins)=&function.native_returns else {
            if let Some(summaries)=context.native_view_function_returns.as_mut() {summaries.remove(&key);}
            return Ok(());
        };
        let mut mapped=Vec::with_capacity(origins.len());
        for loan in origins.iter() {
            let source=if let Some(position)=parameters.iter().position(|root|*root==loan.root) {
                Source::Argument(position)
            } else {
                let position=match method {
                    Some(method)=>closure.methods[method].captures.iter().position(|root|*root==loan.root),
                    None=>closure.captures.iter().position(|capture|capture.inner==loan.root),
                }.ok_or_else(||Diagnostic::new("native-view-call","Returned view has no invocation or capture source.",span))?;
                let capture=&closure.captures[position];
                if matches!(loan.source,LifetimeSource::Storage) {
                    let invocation=method.map_or(closure.invocation,|method|closure.methods[method].invocation);
                    validate_storage_capture(capture.storage,invocation,loan,span)?;
                    if capture.storage==Mode::Read {
                        Source::BorrowedCapture(capture.outer)
                    } else {
                        Source::Receiver(capture.inner)
                    }
                } else {Source::Capture(capture.outer)}
            };
            mapped.push(Returned{source,loan:loan.clone()});
        }
        let summaries=context.native_view_function_returns.get_or_insert_with(Default::default);
        if summaries.get(&key)!=Some(&mapped) {summaries.insert(key,mapped);}
        context.native_view_calls=true;
        Ok(())
    }

}

pub(super) fn transport(actual:&mut Loan,output:&Loan)->bool {
    if let LifetimeSource::VariantCarried{guards,..}=&output.source
        && guards.iter().any(|expected|actual.guards.iter().any(|actual|actual.path==expected.path && actual.success!=expected.success)) {
        return false;
    }
    match &output.source {
        LifetimeSource::Carried(path)|LifetimeSource::VariantCarried{path,..}=>{
            for segment in path {
                if !native_view_projections::select(actual,segment) {return false;}
            }
            if !actual.path.is_empty() {return false;}
        }
        LifetimeSource::Storage=>{
            for field in &output.owner_fields {
                if !native_view_projections::select(actual,&native_view_loans::Segment::Field(field.clone())) {return false;}
            }
        }
    }
    actual.path=output.path.clone();
    actual.guards=output.guards.clone();
    if output.access==crate::native_return_borrows::BorrowAccess::Exclusive {
        actual.access=output.access;
    }
    actual.retained=true;
    true
}

pub(super) fn storage(context:&Context,argument:&Expr,output:&Loan)->Result<Vec<Loan>,Diagnostic> {
    let places=native_view_places::sources(context,argument)?;
    Ok(places.into_iter().map(|place|{
        let mut loan=output.clone();
        loan.root=place.root;
        loan.owner_fields=place.fields;
        loan.owner_fields.extend(output.owner_fields.iter().cloned());
        loan.creation=argument.span;
        loan.retained=true;
        loan
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_view_from_borrowed_capture_keeps_outer_owner_loan() {
        let source="type Window={};function sample(owner:ReadOnly<string>):void{const callback=():int=>owner.length;callback();}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
        let mut analyzer=Analyzer::new(parsed).unwrap();
        analyzer.function(0).unwrap();
        let capture=&analyzer.closures[0].captures[0];
        assert_eq!(capture.storage,Mode::Read);
        let outer=capture.outer;
        let inner=capture.inner;
        analyzer.closures[0].function.native_returns=Some(Box::new(vec![Loan{
            root:inner,owner_fields:vec![],source:LifetimeSource::Storage,
            access:crate::native_return_borrows::BorrowAccess::ReadOnly,creation:Span::default(),
            retained:true,path:vec![],guards:vec![],
        }]));
        let mut context=Context::new("sample".into());
        context.bindings=analyzer.checked[0].as_ref().unwrap().bindings.clone();
        context.names=context.bindings.iter().enumerate().map(|(id,binding)|(binding.name.clone(),id)).collect();
        analyzer.remember_native_closure_returns(&mut context,0,None,Span::default()).unwrap();
        let callback=context.names["callback"];
        let mut call=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("callback".into()),Span::default())),vec![]),Span::default());
        call.target=Some(CallTarget::Closure(callback));
        call.callable_origin=Some(CallableOrigin::Closure(0));
        call.ty=Some(Type::Record(0));
        let loans=Analyzer::native_view_value_loans(&context,&call).unwrap();
        assert_eq!(loans.len(),1);
        assert_eq!(loans[0].root,outer);
        assert_eq!(loans[0].source,LifetimeSource::Storage);
        let token=Token{text:"view".into(),kind:TokenKind::Name,span:Span::default()};
        let holder=Analyzer::bind(&mut context,&token,Type::Record(0),false).unwrap();
        Analyzer::set_native_view_loans(&mut context,holder,loans);
        assert!(Analyzer::check_native_view_access(&context,outer,Mode::Read,Span::default()).is_ok());
        assert!(Analyzer::check_native_view_access(&context,outer,Mode::Mutate,Span::default()).is_err());
        assert!(Analyzer::check_native_view_access(&context,outer,Mode::Consume,Span::default()).is_err());
    }
}

#[cfg(test)]
mod receiver_tests {
    use super::*;
    #[test]
    fn native_view_unused_consuming_closure_is_rejected_before_emission() {
        let source="function factory(owner:string,token:string):Once<Function<int>>{return ():int=>{const moved=token;return owner.length;};}function main(){const unused=factory('bytes','token');}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut checked=analyze(crate::syntax::parse(&formatted).unwrap()).unwrap();
        let closure=checked.closures.iter_mut().find(|closure|closure.invocation==Mode::Consume && closure.captures.iter().any(|capture|closure.function.bindings[capture.inner].name=="owner")).unwrap();
        let capture=closure.captures.iter().find(|capture|closure.function.bindings[capture.inner].name=="owner").unwrap();
        assert_eq!(capture.storage,Mode::Consume);
        closure.function.native_returns=Some(Box::new(vec![Loan{
            root:capture.inner,owner_fields:vec![],source:LifetimeSource::Storage,
            access:crate::native_return_borrows::BorrowAccess::ReadOnly,creation:closure.span,
            retained:true,path:vec![],guards:vec![],
        }]));
        closure.invocation=Mode::Read;
        validate_closures(std::slice::from_ref(closure)).unwrap();
        closure.invocation=Mode::Consume;
        closure.function.native_returns.as_mut().unwrap()[0].source=LifetimeSource::Carried(vec![]);
        validate_closures(std::slice::from_ref(closure)).unwrap();
        closure.function.native_returns.as_mut().unwrap()[0].source=LifetimeSource::Storage;
        let error=finish_analysis(checked).err().expect("unused invalid closure must fail before emission");
        assert_eq!(error.code,"native-view-call");
    }
    #[test]
    fn native_view_from_owned_capture_borrows_receiver_storage() {
        let source="type Window={};function factory(owner:string):Function<int>{return ():int=>owner.length;}function main(){const unrelated=7;const callback=factory('bytes');callback();}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
        let mut analyzer=Analyzer::new(parsed).unwrap();
        analyzer.function(1).unwrap();
        let mut context=Context::new("main".into());
        context.bindings=analyzer.checked[1].as_ref().unwrap().bindings.clone();
        context.names=context.bindings.iter().enumerate().map(|(id,binding)|(binding.name.clone(),id)).collect();
        let callback=context.names["callback"];
        let Type::Closure(origin)=context.bindings[callback].ty else {panic!("concrete factory closure expected")};
        let capture=&analyzer.closures[origin].captures[0];
        assert_eq!(capture.storage,Mode::Consume);
        let inner=capture.inner;
        analyzer.closures[origin].function.native_returns=Some(Box::new(vec![Loan{
            root:inner,owner_fields:vec![],source:LifetimeSource::Storage,
            access:crate::native_return_borrows::BorrowAccess::ReadOnly,creation:Span::default(),
            retained:true,path:vec![],guards:vec![],
        }]));
        analyzer.remember_native_closure_returns(&mut context,origin,None,Span::default()).unwrap();
        let mut call=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("callback".into()),Span::default())),vec![]),Span::default());
        call.target=Some(CallTarget::Closure(callback));
        call.callable_origin=Some(CallableOrigin::Closure(origin));call.ty=Some(Type::Record(0));
        let loans=Analyzer::native_view_value_loans(&context,&call).unwrap();
        assert_eq!(loans.len(),1);assert_eq!(loans[0].root,callback);
        assert_ne!(callback,context.names["unrelated"]);
        let token=Token{text:"view".into(),kind:TokenKind::Name,span:Span::default()};
        let holder=Analyzer::bind(&mut context,&token,Type::Record(0),false).unwrap();
        Analyzer::set_native_view_loans(&mut context,holder,loans);
        assert!(Analyzer::check_native_view_access(&context,callback,Mode::Read,Span::default()).is_ok());
        assert!(Analyzer::check_native_view_access(&context,callback,Mode::Mutate,Span::default()).is_err());
        assert!(Analyzer::check_native_view_access(&context,callback,Mode::Consume,Span::default()).is_err());
        let mut returned=Expr::new(ExprKind::Name("view".into()),Span::default());
        returned.binding=Some(holder);returned.ty=Some(Type::Record(0));
        assert_eq!(analyzer.record_native_return(&mut context,&returned).unwrap_err().code,"native-view-escape");
        context.bindings[holder].native_view_loans=None;
        analyzer.closures[origin].invocation=Mode::Mutate;
        analyzer.closures[origin].function.native_returns.as_mut().unwrap()[0].access=crate::native_return_borrows::BorrowAccess::Exclusive;
        analyzer.remember_native_closure_returns(&mut context,origin,None,Span::default()).unwrap();
        let exclusive=Analyzer::native_view_value_loans(&context,&call).unwrap();
        Analyzer::set_native_view_loans(&mut context,holder,exclusive);
        for mode in [Mode::Read,Mode::Mutate,Mode::Consume] {
            assert!(Analyzer::check_native_view_access(&context,callback,mode,Span::default()).is_err());
        }
        context.bindings[holder].native_view_loans=None;
        assert!(Analyzer::check_native_view_access(&context,callback,Mode::Mutate,Span::default()).is_ok());
        analyzer.closures[origin].invocation=Mode::Consume;
        assert_eq!(analyzer.remember_native_closure_returns(&mut context,origin,None,Span::default()).unwrap_err().code,"native-view-call");
    }
}
