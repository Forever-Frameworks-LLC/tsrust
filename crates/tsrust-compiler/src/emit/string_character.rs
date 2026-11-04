//! Borrow stack-encoded UTF-8 scalars only when every use stays read-only.
use super::*;
use crate::fixed_storage::{visit_function_expressions,visit_statements};
use std::collections::{HashMap,HashSet};

#[derive(Default)]
pub(super) struct Index(HashMap<usize,HashSet<usize>>);
impl Index {
    pub(super) fn new(program:&CheckedProgram,appends:&self_append::Index)->Self {
        let mut index=Self::default();
        for fun in program.functions.iter().chain(program.closures.iter().flat_map(|closure|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))) {
            let Some(captured)=receiver_moves::captured(fun,&program.closures) else {continue;};
            let mut candidates=HashSet::new();
            visit_statements(&fun.body,&mut |stmt|if let StmtKind::ForOf{iterable,binding:Some(id),..}=&stmt.kind {
                if iterable.ty==Some(Type::String) && !captured.contains(id){candidates.insert(*id);}
            });
            if candidates.is_empty(){continue;}
            let mut allowed=HashSet::new();
            let mut names=HashMap::<usize,HashSet<usize>>::new();
            let mut remaining=10_000usize;
            let mut allow=|expr:&Expr|if matches!(expr.kind,ExprKind::Name(_)){allowed.insert(expr as *const Expr as usize);};
            visit_statements(&fun.body,&mut |stmt|if let StmtKind::Assign{target,value,operator:None}=&stmt.kind {
                if appends.assignment(fun,value){for part in self_append::operands(target,value).expect("proved append"){allow(part);}}
            });
            visit_function_expressions(fun,&mut |expr| {
                if remaining==0{return;}
                remaining=remaining.saturating_sub(1);
                if let Some(id)=expr.binding.filter(|id|candidates.contains(id)) && matches!(expr.kind,ExprKind::Name(_)) {
                    names.entry(id).or_default().insert(expr as *const Expr as usize);
                }
                match &expr.kind {
                    ExprKind::Binary{op,left,right,..} if left.ty==Some(Type::String) && right.ty==Some(Type::String) && matches!(op,BinaryOp::Equal|BinaryOp::NotEqual|BinaryOp::Less|BinaryOp::LessEqual|BinaryOp::Greater|BinaryOp::GreaterEqual)=>{allow(left);allow(right);},
                    ExprKind::Field(base,name) if name=="length" && base.ty==Some(Type::String)=>allow(base),
                    ExprKind::Assign{target,value,..} if expr.ty==Some(Type::Void) && appends.assignment(fun,value)=>{
                        for part in self_append::operands(target,value).expect("proved append"){allow(part);}
                    },
                    _=>{},
                }
            });
            // Exhausting this proof budget keeps the ordinary owning character path.
            if remaining==0 {continue;}
            candidates.retain(|id|names.get(id).is_none_or(|uses|uses.is_subset(&allowed)));
            if !candidates.is_empty(){index.0.insert(fun as *const CheckedFunction as usize,candidates);}
        }
        index
    }
    pub(super) fn binding(&self,fun:&CheckedFunction,id:usize)->bool {
        self.0.get(&(fun as *const CheckedFunction as usize)).is_some_and(|bindings|bindings.contains(&id))
    }
    pub(super) fn read_type(&self,fun:&CheckedFunction,expr:&Expr)->&'static str {
        if matches!(expr.kind,ExprKind::Name(_)) && expr.binding.is_some_and(|id|self.binding(fun,id)){"std::borrow::Cow<'_, str>"}else{"String"}
    }
}
