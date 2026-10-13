//! Definite assignment of `let name: Type;` declarations.
//!
//! A deferred variable has no value until its first assignment. Every mention
//! of the name must follow an assignment on every path. The rules are a
//! subset of what the generated Rust accepts: loop bodies never count as
//! assigning, and a branch counts only when the other branch assigns or
//! leaves.
use crate::Diagnostic;
use crate::completion::CompletionKind;
mod exception_assignment;
mod mentions;
mod flow_work;
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
    named: Option<NamedCapture<'a>>,
}

#[derive(Clone, Copy)]
struct NamedCapture<'a> {
    capture: &'a crate::local_capture_plan::Capture,
    functions: &'a std::collections::BTreeSet<String>,
    uses: &'a std::collections::BTreeMap<usize, crate::local_capture_plan::BindingId>,
}
impl Variable<'_> {
    fn matches(self, value: &Expr) -> bool {
        if let Some(named) = self.named {
            if !matches!(value.kind, ExprKind::Name(_)) { return false; }
            return named.uses.get(&value.span.start) == Some(&named.capture.binding);
        }
        matches!(&value.kind, ExprKind::Name(text) if text == self.name) && self.binding.is_none_or(|binding| value.binding == Some(binding))
    }
    fn initialized_by(self, name: &crate::syntax::Token) -> bool {
        self.named.is_some_and(|named| name.span == named.capture.declaration)
    }
}

/// Validate named-callable uses against one captured binding using the shared
/// completion/assignment flow. Checked loan and type analysis remain separate.
pub(crate) fn check_named_capture(
    body: &mut [Stmt], capture: &crate::local_capture_plan::Capture,
    functions: &std::collections::BTreeSet<String>,
    uses: &std::collections::BTreeMap<usize, crate::local_capture_plan::BindingId>,
    initialized: bool,
) -> Result<(), Diagnostic> {
    flow(body, Variable { name: &capture.name, binding: None, captures: &[], named: Some(NamedCapture { capture, functions, uses }) }, initialized).map(|_| ())
}

#[derive(Clone)]
struct Exit { kind:CompletionKind, assigned:bool }
#[derive(Clone)]
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

fn require_clear(value: &Expr, variable: Variable) -> Result<(), Diagnostic> {
    mentions::expression_mention(value, variable).map_or(Ok(()), |span| {
        let mut error = unassigned(variable.name, span);
        if let Some(named) = variable.named {
            error.message = format!("This function use requires captured binding {:?} to be initialized on every reaching path. Initialize it before calling or retaining the function.", variable.name);
            error.related.push((named.capture.declaration, "Captured binding declared here.".into()));
        }
        Err(error)
    })
}

fn flow(statements: &[Stmt], variable: Variable, assigned: bool) -> Result<Flow, Diagnostic> {
    flow_work::run(statements, variable, assigned)
}

/// Checks every deferred declaration in the statements and in nested blocks.
pub(crate) fn check(statements: &mut [Stmt], captures: &Captures) -> Result<(), Diagnostic> {
    let mut work: Vec<&[Stmt]> = vec![statements];
    while let Some(body) = work.pop() {
        let Some((statement, after)) = body.split_first() else { continue };
        work.push(after);
        match &statement.kind {
            StmtKind::Try(region) => {
                if let Some(body) = &region.finally { work.push(body); }
                if let Some(handler) = &region.catch { work.push(&handler.body); }
                work.push(&region.body);
            },
            StmtKind::Let { name, value, binding, .. } if value.is_deferred() => {
                flow(after, Variable { name: &name.text, binding: *binding, captures, named: None }, false)?;
            },
            StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => work.push(body),
            StmtKind::Loop { body, update, .. } => { work.push(update); work.push(body); },
            StmtKind::If { then_body, else_body, .. } => { work.push(else_body); work.push(then_body); },
            _ => {},
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
