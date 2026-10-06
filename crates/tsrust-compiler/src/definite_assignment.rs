//! Definite assignment of `let name: Type;` declarations.
//!
//! A deferred variable has no value until its first assignment. Every mention
//! of the name must follow an assignment on every path. The rules are a
//! subset of what the generated Rust accepts: loop bodies never count as
//! assigning, and a branch counts only when the other branch assigns or
//! leaves.
use crate::Diagnostic;
use crate::completion::CompletionKind;
#[path="exception_assignment.rs"]
mod exception_assignment;
use crate::syntax::{ControlTarget, Expr, ExprKind, Stmt, StmtKind};

/// Function values of the program: source span and the enclosing-function
/// bindings each one captures. Checked function values keep their bodies
/// outside the expression, so a capture is the only trace of a mention.
pub(crate) type Captures = [(crate::Span, Vec<usize>)];

#[derive(Clone, Copy)]
struct Variable<'a> {
    name: &'a str,
    binding: Option<usize>,
    captures: &'a Captures,
}

struct Exit { kind:CompletionKind, assigned:bool }
struct Flow {
    normal: Option<bool>,
    exits: Vec<Exit>,
}

fn unassigned(name: &str, span: crate::Span) -> Diagnostic {
    Diagnostic::new(
        "unassigned",
        format!(
            "{name:?} is used before it is assigned on every path. Assign it first, or give the declaration an initializer."
        ),
        span,
    )
}

/// Span of the first mention of the variable anywhere inside the statements,
/// function values that capture it included.
fn mention(statements: &mut [Stmt], variable: Variable) -> Option<crate::Span> {
    let mut found = None;
    let _ = crate::source_walk::body(statements, &mut |_| Ok(()), &mut |value: &mut Expr| {
        let named = matches!(&value.kind, ExprKind::Name(text) if text == variable.name) && variable.binding.is_none_or(|binding| value.binding==Some(binding));
        let captured = matches!(value.kind, ExprKind::Closure { .. })
            && variable.binding.is_some_and(|binding| {
                variable.captures.iter().any(|(span, outer)| {
                    span.start >= value.span.start
                        && span.end <= value.span.end
                        && outer.contains(&binding)
                })
            });
        if (named || captured)
            && found.is_none_or(|span: crate::Span| value.span.start < span.start)
        {
            found = Some(value.span);
        }
        Ok(())
    });
    found
}

fn expression_mention(value: &mut Expr, variable: Variable) -> Option<crate::Span> {
    let span = value.span;
    let original = std::mem::replace(value, Expr::new(ExprKind::Null, span));
    let mut holder = [Stmt { labels: Vec::new(), control_target: None, kind: StmtKind::Expr(original), span }];
    let found = mention(&mut holder, variable);
    let [Stmt { kind: StmtKind::Expr(restored), .. }] = holder else {
        unreachable!("expression holder");
    };
    *value = restored;
    found
}

fn require_clear(value: &mut Expr, variable: Variable) -> Result<(), Diagnostic> {
    expression_mention(value, variable).map_or(Ok(()), |span| Err(unassigned(variable.name, span)))
}

/// Keep targeted exits until their enclosing statement consumes them. A
/// labeled break leaves its block, not the function's remaining statements.
#[inline(never)]
fn flow(statements: &mut [Stmt], variable: Variable, assigned: bool) -> Result<Flow, Diagnostic> {
    let mut result=Flow{normal:Some(assigned),exits:Vec::new()};
    for statement in statements {
        let Some(before)=result.normal else {break;};
        let failed_entry=crate::error_effects::statement_entry_may_fail(statement);
        let mut next=Flow{normal:Some(before),exits:Vec::new()};
        match &mut statement.kind {
            StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
            StmtKind::Try(region) => next=exception_assignment::region_flow(region,variable,before)?,
            StmtKind::Throw(value) => {
                if !before {require_clear(value,variable)?;}
                next.normal=None;next.exits.push(Exit{kind:CompletionKind::Throw,assigned:before});
            }
            StmtKind::Assign {target,value,operator:None}
                if matches!(&target.kind,ExprKind::Name(text) if text==variable.name) && variable.binding.is_none_or(|binding|target.binding==Some(binding)) => {
                if !before {require_clear(value,variable)?;}
                next.normal=Some(true);
            }
            StmtKind::Assign{target,value,..}=>{if !before {require_clear(target,variable)?;require_clear(value,variable)?;}}
            StmtKind::Let{value,..}|StmtKind::Expr(value)=>{if !before {require_clear(value,variable)?;}}
            StmtKind::Block(body)=>{
                next=flow(body,variable,before)?;
                if !statement.labels.is_empty(){
                    let target=ControlTarget::Block(statement.span.start);
                    let mut escaped=Vec::new();
                    for exit in next.exits {
                        if exit.kind.targets(target) {next.normal=Some(next.normal.map_or(exit.assigned,|normal|normal&&exit.assigned));}
                        else {escaped.push(exit);}
                    }
                    next.exits=escaped;
                }
            }
            StmtKind::If{condition,then_body,else_body}=>{
                if !before {require_clear(condition,variable)?;}
                let mut yes=flow(then_body,variable,before)?;let no=flow(else_body,variable,before)?;
                next.normal=match(yes.normal,no.normal){(Some(a),Some(b))=>Some(a&&b),(Some(a),None)|(None,Some(a))=>Some(a),(None,None)=>None};
                yes.exits.extend(no.exits);next.exits=yes.exits;
            }
            StmtKind::While{condition,body}=>{
                if !before {require_clear(condition,variable)?;}
                let inner=flow(body,variable,before)?;
                next.exits=inner.exits.into_iter().filter(|exit|!exit.kind.targets(ControlTarget::Loop(statement.span.start))).collect();
            }
            StmtKind::ForOf{iterable,body,..}=>{
                if !before {require_clear(iterable,variable)?;}
                let inner=flow(body,variable,before)?;
                next.exits=inner.exits.into_iter().filter(|exit|!exit.kind.targets(ControlTarget::Loop(statement.span.start))).collect();
            }
            StmtKind::Loop{condition,body,update,post_test,..}=>{
                if !before && !*post_test {if let Some(condition)=condition {require_clear(condition,variable)?;}}
                let inner=flow(body,variable,before)?;
                let target=ControlTarget::Tail(statement.span.start);
                let mut continuing=inner.normal;
                for exit in inner.exits {
                    if exit.kind.targets(target) {
                        if exit.kind.is_continue() {continuing=Some(continuing.map_or(exit.assigned,|normal|normal&&exit.assigned));}
                    }else{next.exits.push(exit);}
                }
                if let Some(assigned)=continuing {
                    let tail=flow(update,variable,assigned)?;
                    if !tail.normal.unwrap_or(true) && *post_test {if let Some(condition)=condition {require_clear(condition,variable)?;}}
                    if *post_test && condition.as_ref().is_some_and(|value|value.may_fail) {
                        if let Some(assigned)=tail.normal {next.exits.push(Exit{kind:CompletionKind::Throw,assigned});}
                    }
                    next.exits.extend(tail.exits);
                }
            }
            StmtKind::Return(value)=>{
                if !before {if let Some(value)=value {require_clear(value,variable)?;}}
                next.normal=None;next.exits.push(Exit{kind:CompletionKind::Return,assigned:before});
            }
            StmtKind::Break|StmtKind::Continue=>{next.normal=None;next.exits.push(Exit{kind:if matches!(statement.kind,StmtKind::Continue){CompletionKind::Continue(statement.control_target)}else{CompletionKind::Break(statement.control_target)},assigned:before});}
        }
        if failed_entry {next.exits.push(Exit{kind:CompletionKind::Throw,assigned:before});}
        result.normal=next.normal;result.exits.extend(next.exits);
    }
    Ok(result)
}

/// Checks every deferred declaration in the statements and in nested blocks.
pub(crate) fn check(statements: &mut [Stmt], captures: &Captures) -> Result<(), Diagnostic> {
    for index in 0..statements.len() {
        let (before, after) = statements.split_at_mut(index + 1);
        match &mut before[index].kind {
            StmtKind::Try(region) => {
                check(&mut region.body,captures)?;
                if let Some(handler)=&mut region.catch {check(&mut handler.body,captures)?;}
                if let Some(body)=&mut region.finally {check(body,captures)?;}
            }

            StmtKind::Let { name, value, binding, .. } if value.is_deferred() => {
                let name = name.text.clone();
                flow(after, Variable { name: &name, binding: *binding, captures }, false)?;
            }
            StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => {
                check(body, captures)?
            }
            StmtKind::Loop { body, update, .. } => {
                check(body, captures)?;
                check(update, captures)?;
            }
            StmtKind::If { then_body, else_body, .. } => {
                check(then_body, captures)?;
                check(else_body, captures)?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod exception_completion_tests {
    use super::*;
    fn check_source(source:&str)->Result<(),Diagnostic> {
        let mut program=crate::syntax::parse(source)?;
        check(&mut program.functions[0].body,&[])
    }
    #[test]
    fn finally_assignment_reaches_normal_and_caught_paths() {
        for source in [
            "function main(){let x:int;try{}finally{x=1;}console.log(x);}",
            "function main(){let x:int;try{throw 1;}catch{x=2;}console.log(x);}",
            "function main(){let x:int;try{if(true){throw 1;}x=2;}catch{x=3;}finally{console.log(x);}console.log(x);}",
            "function main(){let x:int;try{return;}finally{x=1;console.log(x);}}",
        ] {check_source(source).unwrap();}
    }
    #[test]
    fn finally_checks_reads_on_return_and_throw_exits() {
        for source in [
            "function main(){let x:int;try{return;}finally{console.log(x);}}",
            "function main(){let x:int;try{throw 1;}finally{console.log(x);}}",
            "function main(){let x:int;try{throw 1;x=2;}catch{console.log(x);}}",
            "function main(){let x:int;try{if(true){x=1;throw 1;}throw 2;}catch{console.log(x);}}",
        ] {assert_eq!(check_source(source).unwrap_err().code,"unassigned","{source}");}
    }
    #[test]
    fn finally_preserves_or_replaces_labeled_block_destinations() {
        fn checked_target_source(source:&str)->Result<(),Diagnostic> {
            let mut program=crate::syntax::parse(source)?;
            let body=&mut program.functions[0].body;
            let target=ControlTarget::Block(body[1].span.start);
            let mut pending=body.iter_mut().collect::<Vec<_>>();
            while let Some(statement)=pending.pop() {
                if matches!(statement.kind,StmtKind::Break) {statement.control_target=Some(target);}
                match &mut statement.kind {
                    StmtKind::Block(body)=>pending.extend(body),
                    StmtKind::Try(region)=>pending.extend(region.statements_mut()),
                    _=>{}
                }
            }
            check(body,&[])
        }
        checked_target_source("function main(){let x:int;done:{try{break done;}finally{x=1;}}console.log(x);}").unwrap();
        checked_target_source("function main(){let x:int;done:{try{break done;}finally{return;}}console.log(x);}").unwrap();
        assert_eq!(checked_target_source("function main(){let x:int;done:{try{throw 1;}finally{break done;}}console.log(x);}").unwrap_err().code,"unassigned");
    }
    #[test]
    fn catch_does_not_observe_a_failed_assignment_as_initialized() {
        fn checked_effect_source(source:&str)->Result<(),Diagnostic> {
            let mut program=crate::syntax::parse(source)?;
            let body=&mut program.functions[0].body;
            crate::source_walk::body(body,&mut |_|Ok(()),&mut |value| {
                if matches!(&value.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Name(name) if name=="risky")) {value.may_fail=true;}
                Ok(())
            })?;
            check(body,&[])
        }
        assert_eq!(checked_effect_source("function main(){let x:int;try{x=risky();}catch{console.log(x);}}").unwrap_err().code,"unassigned");
        checked_effect_source("function main(){let x:int;try{x=risky();}catch{x=2;}console.log(x);}").unwrap();
        checked_effect_source("function main(){let x:int;x=1;try{risky();}catch{console.log(x);}}").unwrap();
        checked_effect_source("function main(){let x:int;try{do{x=1;}while(risky());}catch{console.log(x);}}").unwrap();
        assert_eq!(checked_effect_source("function main(){let x:int;try{while(risky()){x=1;}}catch{console.log(x);}}").unwrap_err().code,"unassigned");
        assert_eq!(checked_effect_source("function main(){let x:int;try{x=risky();}finally{console.log(x);}}").unwrap_err().code,"unassigned");
    }
    #[test]
    fn abrupt_finally_replaces_earlier_return_or_throw() {
        for source in [
            "function main(){let x:int;try{try{return;}finally{throw 1;}}catch{x=2;}console.log(x);}",
            "function main(){let x:int;try{try{throw 1;}finally{return;}}catch{console.log(x);}}",
        ] {check_source(source).unwrap();}
    }
}
