//! Returned views follow the selected owning-family environment, not factory roots.
use super::*;
use super::native_view_calls::{Returned,Source,Target};
use super::native_view_loans::LifetimeSource;

impl Analyzer {
    pub(super) fn remember_native_family_returns(&self,context:&mut Context,family:usize,span:Span)->Result<(),Diagnostic> {
        if !self.native_borrow_types {return Ok(());}
        let Some(layout)=self.callables.storage.scoped_native_family(family) else {
            // A lifetime-free family holds no view. Its result can only be a view
            // that one of its callbacks received as an argument.
            let mut returned=Vec::new();
            for &origin in self.callables.storage.origins(family).unwrap_or_default() {
                let closure=&self.closures[origin];
                let closure=match closure.forward {Some(ClosureForward::Once(inner))=>&self.closures[inner],_=>closure};
                for loan in closure.function.native_returns.as_deref().into_iter().flatten() {
                    let Some(position)=closure.parameters.iter().position(|root|*root==loan.root) else {
                        return Err(Diagnostic::new("native-view-call","A returned view of this callback has no checked argument source.",span));
                    };
                    returned.push(Returned{source:Source::Argument(position),loan:loan.clone()});
                }
            }
            if !returned.is_empty() {
                context.native_view_function_returns.get_or_insert_with(Default::default).insert(Target::Family(family),returned);
                context.native_view_calls=true;
            }
            return Ok(());
        };
        let mut returned=Vec::new();
        for alternative in &layout.alternatives {
            let closure=&self.closures[alternative.origin];
            for loan in closure.function.native_returns.as_deref().into_iter().flatten() {
                let source=if let Some(position)=closure.parameters.iter().position(|root|*root==loan.root) {
                    Source::Argument(position)
                }else {
                    let capture=closure.captures.iter().find(|capture|capture.inner==loan.root).ok_or_else(||Diagnostic::new("native-view-call","A returned family view has no checked invocation or capture source.",span))?;
                    if matches!(loan.source,LifetimeSource::Storage) {
                        return Err(Diagnostic::new("native-view-call","Returning a borrow of scoped callable receiver storage requires receiver lifetime planning.",span));
                    }
                    Source::FamilyCapture(alternative.origin,capture.inner)
                };
                returned.push(Returned{source,loan:loan.clone()});
            }
        }
        context.native_view_function_returns.get_or_insert_with(Default::default).insert(Target::Family(family),returned);
        context.native_view_calls=true;
        Ok(())
    }
}
