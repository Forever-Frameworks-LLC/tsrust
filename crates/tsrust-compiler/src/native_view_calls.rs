//! Checked callee origins map formal source slots to actual argument lifetimes.
use super::*;
use super::native_view_loans::{LifetimeSource,Loan};

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
            if summaries.get(&id).is_none_or(|previous|previous!=origins.as_ref()) {
                summaries.insert(id,origins.as_ref().clone());
            }
            context.native_view_calls=true;
        } else if let Some(summaries)=context.native_view_function_returns.as_mut() {
            summaries.remove(&id);
        }
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
