//! Named capture initialization uses the existing definite-assignment flow.
use crate::local_capture_plan::{BindingId, Capture, Plan};
use crate::syntax::{ExprKind, Program};
use crate::{Diagnostic, Span};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

// The condensed graph is acyclic. Propagate binding identities once in reverse
// dependency order; avoid repeated graph walks for each callable use.
fn requirements(plan: &Plan) -> Vec<BTreeSet<BindingId>> {
    let mut values: Vec<BTreeSet<BindingId>> = plan
        .groups
        .iter()
        .map(|group| {
            group
                .captures
                .iter()
                .map(|capture| capture.binding)
                .collect()
        })
        .collect();
    let mut remaining: Vec<usize> = plan
        .groups
        .iter()
        .map(|group| group.dependencies.len())
        .collect();
    let mut parents = vec![Vec::new(); plan.groups.len()];
    for (group, data) in plan.groups.iter().enumerate() {
        for &dependency in &data.dependencies {
            parents[dependency].push(group);
        }
    }
    let mut ready: VecDeque<usize> = remaining
        .iter()
        .enumerate()
        .filter_map(|(group, &count)| (count == 0).then_some(group))
        .collect();
    while let Some(group) = ready.pop_front() {
        for &parent in &parents[group] {
            let inherited = values[group].clone();
            values[parent].extend(inherited);
            remaining[parent] -= 1;
            if remaining[parent] == 0 {
                ready.push_back(parent);
            }
        }
    }
    debug_assert!(remaining.iter().all(|&count| count == 0));
    values
}

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
        for (group, bindings) in requirements(plan).into_iter().enumerate() {
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
