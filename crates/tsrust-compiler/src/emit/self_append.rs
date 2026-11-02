//! Reuse owned local string destinations without retaining spare output capacity.
use crate::analysis::{CheckedFunction,CheckedProgram};
use crate::fixed_storage::{visit_expression,visit_function_expressions,visit_statements};
use crate::syntax::{BinaryOp,Expr,ExprKind,StmtKind,Type};
use std::collections::{HashMap,HashSet};

#[derive(Default)]
struct Plan { assignments:HashMap<(usize,usize),usize>, bindings:HashSet<usize> }
#[derive(Default)]
pub(super) struct Index(HashMap<usize,Plan>);
impl Index {
    pub(super) fn new(program:&CheckedProgram)->Self {
        let mut index=Self::default();
        for fun in program.functions.iter().chain(program.closures.iter().flat_map(|closure|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))) {
            let Some(captured)=super::receiver_moves::captured(fun,&program.closures) else {continue;};
            let mut plan=Plan::default();
            let mut formatted=HashSet::new();
            // A proof budget falls back to ordinary copies; it never limits source.
            let mut remaining=10_000usize;
            let mut consider=|target:&Expr,value:&Expr| {
                if remaining==0 {return;}
                let Some(id)=target.binding else {return;};
                let binding=&fun.bindings[id];
                if binding.parameter || binding.capture_reference || binding.callback_reference
                    || binding.borrowed_place.is_some() || binding.borrowed_element.is_some()
                    || binding.mutable_place || captured.contains(&id) {return;}
                let Some(parts)=operands(target,value) else {return;};
                // Fused numeric reservations plus compaction lose on short builders.
                if parts.iter().any(|part|super::Emitter::appended_number(part).is_some()) {formatted.insert(id);return;}
                let mut mentions=false;
                for part in parts {
                    visit_expression(part,&mut |expr| {
                        remaining=remaining.saturating_sub(1);
                        mentions |= matches!(expr.kind,ExprKind::Name(_)) && expr.binding==Some(id);
                    });
                }
                if remaining==0 || mentions {return;}
                plan.assignments.insert((value.span.start,value.span.end),id);
            };
            visit_statements(&fun.body,&mut |statement| {
                if let StmtKind::Assign{target,value,operator:None}=&statement.kind {consider(target,value);}
            });
            visit_function_expressions(fun,&mut |expr| {
                if expr.ty==Some(Type::Void) && let ExprKind::Assign{target,value,..}=&expr.kind {consider(target,value);}
            });
            plan.assignments.retain(|_,id|!formatted.contains(&*id));
            plan.bindings=plan.assignments.values().copied().collect();
            if remaining!=0 && !plan.assignments.is_empty() {index.0.insert(fun as *const CheckedFunction as usize,plan);}
        }
        index
    }
    pub(super) fn assignment(&self,fun:&CheckedFunction,value:&Expr)->bool {
        self.0.get(&(fun as *const CheckedFunction as usize)).is_some_and(|plan|plan.assignments.contains_key(&(value.span.start,value.span.end)))
    }
    pub(super) fn binding(&self,fun:&CheckedFunction,binding:usize)->bool {
        self.0.get(&(fun as *const CheckedFunction as usize)).is_some_and(|plan|plan.bindings.contains(&binding))
    }
}

pub(super) fn operands<'a>(target:&Expr,value:&'a Expr)->Option<Vec<&'a Expr>> {
    if target.ty!=Some(Type::String) || !matches!(target.kind,ExprKind::Name(_)) {return None;}
    let mut parts=super::Emitter::string_addition_chain(value).or_else(||match &value.kind {
        ExprKind::Binary{op:BinaryOp::Add,left,right,..} if left.ty==Some(Type::String)=>Some(vec![left.as_ref(),right.as_ref()]),
        _=>None,
    })?;
    if !matches!(parts[0].kind,ExprKind::Name(_)) || parts[0].binding!=target.binding {return None;}
    parts.remove(0);Some(parts)
}
