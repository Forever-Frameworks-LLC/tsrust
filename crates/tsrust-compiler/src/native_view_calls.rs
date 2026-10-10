//! Checked callee origins map formal source slots to actual argument lifetimes.
use super::*;
use super::native_view_loans::{LifetimeSource,Loan};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub(super) enum Target {Function(usize),Closure(usize),Method(usize,usize)}

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) enum Source { Argument(usize), Capture(usize) }

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
                    return Err(Diagnostic::new("native-view-call","Returning a view of captured owner storage requires a receiver lifetime contract.",span));
                }
                Source::Capture(capture.outer)
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
    match &output.source {
        LifetimeSource::Carried(path)=>{
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
