//! Immutable write facts let carrier planning observe nested stores without
//! retaining overlapping mutable references or cloning source subtrees.
use crate::analysis::{Analyzer, Type};
use crate::syntax::{Expr, ExprKind, Stmt, StmtKind};
use crate::Span;

pub(super) struct Write<'a> {
    pub binding: usize,
    pub path: Vec<String>,
    pub span: Span,
    pub value: &'a Expr,
    pub array_element: bool,
}

pub(super) fn collect(body: &[Stmt]) -> Vec<Write<'_>> {
    enum Work<'a> { Statement(&'a Stmt), Expression(&'a Expr) }
    fn selected(binding: usize, path: Vec<String>, value: &Expr) -> Option<Write<'_>> {
        let ty=value.ty?;
        (matches!(ty,Type::Record(_) | Type::Map(_) | Type::Array(crate::syntax::ArrayElement::Record(_, _)) | Type::Union(_) | Type::BorrowedUnion(_)) || !path.is_empty() && matches!(ty,Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)))
            .then_some(Write{binding,path,span:value.span,value,array_element:false})
    }
    fn assignment<'a>(target: &Expr, value: &'a Expr) -> Option<Write<'a>> {
        if let ExprKind::Index(base, _) = &target.kind
            && let Some(binding) = Analyzer::root_binding(base)
            && let Some((_, path)) = Analyzer::scoped_array_storage_path(base)
            && let Some(mut write) = selected(binding, path, value)
        { write.array_element = true; return Some(write); }
        let binding=Analyzer::root_binding(target)?;
        let (_,path)=Analyzer::direct_fields(target)?;
        selected(binding,path,value)
    }
    let mut result=Vec::new();
    let mut work=body.iter().rev().map(Work::Statement).collect::<Vec<_>>();
    while let Some(node)=work.pop() {
        match node {
            Work::Statement(stmt)=>match &stmt.kind {
                StmtKind::Let{binding,value,..}=>{
                    if let Some(id)=binding && !value.is_deferred() && let Some(write)=selected(*id,Vec::new(),value){result.push(write);}
                    work.push(Work::Expression(value));
                }
                StmtKind::Assign{target,value,operator}=>{
                    if operator.is_none() && let Some(write)=assignment(target,value){result.push(write);}
                    work.push(Work::Expression(value));work.push(Work::Expression(target));
                }
                StmtKind::Expr(value)|StmtKind::Throw(value)|StmtKind::Return(Some(value))=>work.push(Work::Expression(value)),
                StmtKind::Block(body)=>work.extend(body.iter().rev().map(Work::Statement)),
                StmtKind::If{condition,then_body,else_body}=>{
                    work.extend(else_body.iter().rev().map(Work::Statement));work.extend(then_body.iter().rev().map(Work::Statement));work.push(Work::Expression(condition));
                }
                StmtKind::While{condition,body}=>{work.extend(body.iter().rev().map(Work::Statement));work.push(Work::Expression(condition));}
                StmtKind::ForOf{iterable,body,..}=>{work.extend(body.iter().rev().map(Work::Statement));work.push(Work::Expression(iterable));}
                StmtKind::Loop{condition,body,update,post_test,..}=>{
                    if *post_test && let Some(value)=condition{work.push(Work::Expression(value));}
                    work.extend(update.iter().rev().map(Work::Statement));work.extend(body.iter().rev().map(Work::Statement));
                    if !*post_test && let Some(value)=condition{work.push(Work::Expression(value));}
                }
                StmtKind::Try(region)=>work.extend(region.statements().collect::<Vec<_>>().into_iter().rev().map(Work::Statement)),
                _=>{}
            },
            Work::Expression(expr)=>match &expr.kind {
                ExprKind::Assign{target,value,..}=>{
                    if let Some(write)=assignment(target,value){result.push(write);}
                    work.push(Work::Expression(value));work.push(Work::Expression(target));
                }
                ExprKind::Sequence(values)|ExprKind::Array(values)|ExprKind::ArrayOf(values,_)|ExprKind::SetConstructor(values,_)|ExprKind::MapConstructor(values,_)|ExprKind::Template(values)=>work.extend(values.iter().rev().map(Work::Expression)),
                ExprKind::Unary{value,..}|ExprKind::OptionalChain(value)|ExprKind::TypeAssertion{value,..}|ExprKind::Project{value,..}|ExprKind::Inject{value,..}|ExprKind::Field(value,_)|ExprKind::Update{target:value,..}=>work.push(Work::Expression(value)),
                ExprKind::Binary{left,right,..}|ExprKind::Index(left,right)|ExprKind::CompoundAssign{target:left,value:right,..}|ExprKind::CoalesceAssign{target:left,value:right,..}|ExprKind::LogicalAssign{target:left,value:right,..}|ExprKind::Optional{base:left,body:right,..}=>{work.push(Work::Expression(right));work.push(Work::Expression(left));}
                ExprKind::Conditional{predicate,consequent,alternate}=>{work.push(Work::Expression(alternate));work.push(Work::Expression(consequent));work.push(Work::Expression(predicate));}
                ExprKind::Call(callee,args)=>{
                    if matches!(expr.target,Some(crate::syntax::CallTarget::ArrayPush | crate::syntax::CallTarget::ArrayUnshift | crate::syntax::CallTarget::ArraySplice))
                        && let ExprKind::Field(base,_)=&callee.kind
                        && let Some(binding)=Analyzer::root_binding(base)
                        && let Some((_,path))=Analyzer::scoped_array_storage_path(base)
                    {
                        for value in args {
                            if let Some(mut write)=selected(binding,path.clone(),value){write.array_element=true;result.push(write);}
                        }
                    }
                    work.extend(args.iter().rev().map(Work::Expression));work.push(Work::Expression(callee));
                }
                ExprKind::Record(fields)=>work.extend(fields.iter().rev().map(|(_,value)|Work::Expression(value))),
                ExprKind::SpreadCall(plan)=>work.extend(plan.expressions().collect::<Vec<_>>().into_iter().rev().map(Work::Expression)),
                // Closure bodies belong to their own checked storage context.
                _=>{}
            }
        }
    }
    result
}
