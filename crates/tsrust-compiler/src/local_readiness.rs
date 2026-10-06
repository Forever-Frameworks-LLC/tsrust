//! Named capture initialization uses the existing definite-assignment flow.
use crate::local_capture_plan::{BindingId, Capture};
use crate::syntax::{ExprKind, Program};
use crate::{Diagnostic, Span};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn check(program: &mut Program) -> Result<(), Diagnostic> {
    if program.local_capture_plans.is_empty() {
        return Ok(());
    }
    let positions: BTreeMap<String, usize> = program
        .functions
        .iter()
        .enumerate()
        .map(|(position, function)| (function.name.text.clone(), position))
        .collect();
    for plan in &program.local_capture_plans {
        let captures: BTreeMap<BindingId, &Capture> = plan
            .direct
            .iter()
            .flatten()
            .map(|capture| (capture.binding, capture))
            .collect();
        let mut aliases: BTreeMap<BindingId, BTreeSet<String>> = BTreeMap::new();
        for (group, bindings) in plan.group_requirements().into_iter().enumerate() {
            for binding in bindings {
                let functions = aliases.entry(binding).or_default();
                for &member in &plan.groups[group].members {
                    functions.insert(program.local_declarations[member].emitted_name.clone());
                }
            }
        }
        for (binding, functions) in aliases {
            let capture = captures[&binding];
            let function = &mut program.functions[positions[&plan.owners[&binding.owner]]];
            let initialized = function
                .params
                .iter()
                .any(|(name, _, _)| name.span == capture.declaration);
            // Existing source closure scopes share their enclosing logical owner.
            // Find the innermost invocation body containing this binding, rather
            // than checking its local initialization at outer closure creation.
            let mut nested: Option<(Span, Vec<crate::syntax::Stmt>, bool)> = None;
            crate::source_walk::body(&mut function.body, &mut |_| Ok(()), &mut |value| {
                if let ExprKind::Closure { params, body, .. } = &value.kind {
                    if value.span.start <= capture.declaration.start
                        && capture.declaration.end <= value.span.end
                        && nested.as_ref().is_none_or(|(scope, _, _)| {
                            value.span.end - value.span.start < scope.end - scope.start
                        })
                    {
                        let initialized = params
                            .iter()
                            .any(|parameter| parameter.name.span == capture.declaration);
                        nested = Some((value.span, body.clone(), initialized));
                    }
                }
                Ok(())
            })?;
            if let Some((_, mut body, initialized)) = nested {
                crate::definite_assignment::check_named_capture(
                    &mut body,
                    capture,
                    &functions,
                    &plan.uses,
                    initialized,
                )?;
            } else {
                crate::definite_assignment::check_named_capture(
                    &mut function.body,
                    capture,
                    &functions,
                    &plan.uses,
                    initialized,
                )?;
            }
        }
    }
    Ok(())
}

/// Recheck actual invocation bodies after call failures and control targets exist.
/// Hidden projected capture slots do not count as defining declarations.
pub(crate) fn check_checked(program: &mut crate::analysis::CheckedProgram) -> Result<(), Diagnostic> {
    for plan in &program.local_capture_plans {
        let captures: BTreeMap<BindingId, &Capture> = plan.direct.iter().flatten()
            .map(|capture| (capture.binding, capture)).collect();
        let mut aliases: BTreeMap<BindingId, BTreeSet<String>> = BTreeMap::new();
        for (group, bindings) in plan.group_requirements().into_iter().enumerate() {
            for binding in bindings {
                for &member in &plan.groups[group].members {
                    aliases.entry(binding).or_default().insert(program.local_declarations[member].emitted_name.clone());
                }
            }
        }
        for (binding, functions) in aliases {
            let capture = captures[&binding];
            for function in program.functions.iter_mut()
                .chain(program.closures.iter_mut().flat_map(|closure| {
                    std::iter::once(&mut closure.function).chain(closure.methods.iter_mut().map(|method| &mut method.function))
                })) {
                let initialized = function.bindings.iter().take(function.parameter_count)
                    .any(|slot| slot.input_ty.is_some() && slot.declaration_span == Some(capture.declaration));
                if initialized || declares(&function.body, capture.declaration) {
                    crate::definite_assignment::check_named_capture(&mut function.body, capture, &functions, &plan.uses, initialized)?;
                }
            }
        }
    }
    Ok(())
}

fn declares(body: &[crate::syntax::Stmt], declaration: Span) -> bool {
    use crate::syntax::StmtKind;
    let mut pending: Vec<_> = body.iter().collect();
    while let Some(statement) = pending.pop() {
        match &statement.kind {
            StmtKind::Let { name, .. } if name.span == declaration => return true,
            StmtKind::ForOf { name, body, .. } => {
                if name.span == declaration { return true; }
                pending.extend(body);
            }
            StmtKind::Block(body) | StmtKind::While { body, .. } => pending.extend(body),
            StmtKind::Loop { body, update, .. } => { pending.extend(body); pending.extend(update); }
            StmtKind::If { then_body, else_body, .. } => { pending.extend(then_body); pending.extend(else_body); }
            StmtKind::Try(region) => {
                if region.catch.as_ref().and_then(|handler| handler.name.as_ref()).is_some_and(|name| name.span == declaration) { return true; }
                pending.extend(region.statements());
            }
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check(source: &str) -> Result<(), Diagnostic> {
        super::check(&mut crate::syntax::parse(source)?)
    }
    #[test]
    fn initialized_paths_parameters_and_lexical_iteration_bindings_are_ready() {
        for source in [
            "function test(flag:boolean){let value:int;function read():int{return value;}if(flag){value=1;}else{value=2;}console.log(read());}function main(){}",
            "function test(flag:boolean){let value:int;function read():int{return value;}if(!flag){return;}value=1;console.log(read());}function main(){}",
            "function main(){let value:int;function read():int{return value;}try{}finally{value=2;}console.log(read());}",
            "function main(){let value:int;function read():int{return value;}try{throw 1;}catch{value=2;}console.log(read());}",
            "function main(){for(const value of [1,2]){function read():int{return value;}console.log(read());}}",
            "function main(){try{throw 1;}catch(value){function read():int{return value;}console.log(read());}}",
            "function test(value:int){function read():int{return value;}console.log(read());}function main(){}",
            "function main(){const callback=(value:int):int=>{function read():int{return value;}return read();};}",
            "function main(){const callback=():int=>{const value=3;function read():int{return value;}return read();};}",
        ] {
            check(source).unwrap_or_else(|error| panic!("{}: {source}", error.message));
        }
    }
    #[test]
    fn conditional_loop_and_initializer_uses_do_not_prove_readiness() {
        for source in [
            "function test(flag:boolean){let value:int;function read():int{return value;}if(flag){value=1;}console.log(read());}function main(){}",
            "function test(flag:boolean){let value:int;function read():int{return value;}while(flag){value=1;}console.log(read());}function main(){}",
            "function main(){function read():int{return value;}const value=read();}",
            "function main(){function read():int{return value;}const callback=read;const value=3;}",
            "function main(){let value:int;function read():int{return value;}{let value=0;value=3;}console.log(read());}",
            "function main(){let value:int[];function read():int{return value[0];}value[0]=3;console.log(read());}",
        ] {
            let error = check(source).unwrap_err();
            assert_eq!(error.code, "unassigned", "{source}");
            assert_eq!(
                &source[error.span.start..error.span.end],
                "read",
                "{source}"
            );
            assert_eq!(error.related.len(), 1);
        }
    }
}

#[cfg(test)]
mod typed_tests {
    #[test]
    fn checked_exception_paths_initialize_captures_before_use() {
        for source in [
            "function fail():int{throw 1;}function main(){let value:int;function read():int{return value;}try{fail();}catch{value=3;console.log(read());}}",
            "function fail():int{throw 1;}function main(){let value:int;function read():int{return value;}try{fail();}finally{value=3;}console.log(read());}",
        ] {
            // Assignment on the catch or finally path makes the capture ready.
            assert!(crate::check(source).is_ok(), "{source}");
        }
    }
}
