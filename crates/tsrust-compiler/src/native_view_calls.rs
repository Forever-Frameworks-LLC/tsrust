//! Checked callee origins map formal source slots to actual argument lifetimes.
use super::*;
use super::native_view_loans::{LifetimeSource,Loan};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub(super) enum Target {Function(usize),Closure(usize),Method(usize,usize)}

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) enum Source { Argument(usize), Capture(usize), BorrowedCapture(usize) }

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Returned { pub(super) source:Source, pub(super) loan:Loan }

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
                    if capture.storage!=Mode::Read || loan.access!=crate::native_return_borrows::BorrowAccess::ReadOnly {
                        return Err(Diagnostic::new("native-view-call","Returning a view of owned or mutable captured storage requires a receiver lifetime contract.",span));
                    }
                    Source::BorrowedCapture(capture.outer)
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
