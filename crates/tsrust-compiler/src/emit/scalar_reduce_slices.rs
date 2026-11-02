//! Read-only scalar reductions can borrow ranges of immutable local arrays.
use super::*;
use crate::fixed_storage::{visit_function_expressions,visit_statements};

#[derive(Default)]
pub(super) struct Index(HashMap<usize,HashSet<usize>>);
impl Index {
    pub(super) fn new(program:&CheckedProgram)->Self {
        let mut index=Self::default();
        for fun in program.functions.iter().chain(program.closures.iter().flat_map(|closure|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))) {
            if crate::analysis::has_exception_syntax(&fun.body){continue;}
            let owned_readonly=|id:usize| {
                let b=&fun.bindings[id];
                !b.parameter && !b.rebindable && !b.mutable && !b.capture_reference && !b.callback_reference
                    && b.borrowed_place.is_none() && b.borrowed_element.is_none() && !b.mutable_place
                    && b.view_roots.is_empty() && b.native_view_loans.is_none()
            };
            let mut candidates=HashMap::new();
            visit_statements(&fun.body,&mut |statement| {
                let StmtKind::Let{binding:Some(id),value,..}=&statement.kind else {return;};
                if !owned_readonly(*id) || !matches!(fun.bindings[*id].ty,Type::Array(ArrayElement::Int(0)|ArrayElement::Number(0)|ArrayElement::Boolean(0))) || value.target!=Some(CallTarget::ArraySlice){return;}
                let ExprKind::Call(callee,_) = &value.kind else {return;};
                let ExprKind::Field(base,_) = &callee.kind else {return;};
                if matches!(base.kind,ExprKind::Name(_)) && let Some(source)=base.binding.filter(|&id|owned_readonly(id)){candidates.insert(*id,source);}
            });
            if candidates.is_empty(){continue;}
            let Some(captured)=receiver_moves::captured(fun,&program.closures) else {continue;};
            candidates.retain(|id,source|!captured.contains(id) && !captured.contains(source));
            if candidates.is_empty(){continue;}
            let sources=candidates.values().copied().collect::<HashSet<_>>();
            let mut allowed=HashSet::new();let mut source_reads=HashSet::new();let mut names=HashMap::<usize,HashSet<usize>>::new();
            visit_function_expressions(fun,&mut |expr| {
                if matches!(expr.kind,ExprKind::Name(_)) && let Some(id)=expr.binding.filter(|id|candidates.contains_key(id) || sources.contains(id)) {
                    names.entry(id).or_default().insert(expr as *const Expr as usize);
                }
                if expr.target==Some(CallTarget::ArraySlice)
                    && let ExprKind::Call(callee,_)=&expr.kind
                    && let ExprKind::Field(base,_)=&callee.kind
                    && matches!(base.kind,ExprKind::Name(_))
                {source_reads.insert(base.as_ref() as *const Expr as usize);}
                if !matches!(expr.target,Some(CallTarget::ArrayReduce|CallTarget::ArrayReduceRight)){return;}
                let ExprKind::Call(callee,args)=&expr.kind else {return;};
                let ExprKind::Field(base,_) = &callee.kind else {return;};
                let Some(callback)=args.first() else {return;};
                if callback.traversal_method.is_some(){return;}
                let Some(Type::Closure(id))=callback.ty else {return;};
                let closure=&program.closures[id];
                // Rest and full-array callbacks need the ordinary owning receiver ABI.
                if closure.parameters.len()>3 || closure.parameters.iter().any(|&id|closure.function.bindings[id].rest){return;}
                if matches!(base.kind,ExprKind::Name(_)) {allowed.insert(base.as_ref() as *const Expr as usize);}
            });
            // Availability after an exited branch can hide an earlier source move.
            candidates.retain(|id,source|names.get(id).is_some_and(|uses|!uses.is_empty() && uses.is_subset(&allowed))
                && names.get(source).is_some_and(|uses|uses.is_subset(&source_reads)));
            if !candidates.is_empty(){index.0.insert(fun as *const CheckedFunction as usize,candidates.into_keys().collect());}
        }
        index
    }
    pub(super) fn binding(&self,fun:&CheckedFunction,id:usize)->bool {
        self.0.get(&(fun as *const CheckedFunction as usize)).is_some_and(|ids|ids.contains(&id))
    }
}
