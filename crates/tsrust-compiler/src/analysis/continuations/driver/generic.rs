//! Generic argument inference and inferred bodies share the checker work stack.
use super::*;
use crate::analysis::generic_call_work::{Arguments, Prepared, call};
use std::rc::Rc;
struct Pass { frame: Arguments, contexts: Vec<Option<Type>>, next: usize, progress: bool, deferred: Option<Diagnostic> }
enum Phase { Start, Spread, Pass(Pass), Argument { pass: Pass, index: usize }, Function { restore_name: Option<String> }, ClosureHint { pass: Pass, index: usize, key: Option<(usize, usize)>, context: ContextId, discovering: bool, effects: bool }, Done }
pub(super) struct GenericTask { expression: Box<Expr>, context: Rc<Context>, expected: Option<Type>, phase: Phase }
impl GenericTask {
    pub(super) fn new(expression: Box<Expr>, context: Rc<Context>, expected: Option<Type>) -> Self {
        Self { expression, context, expected, phase: Phase::Start }
    }
}
fn prepared<'a>(analyzer: &mut Analyzer, task: &mut GenericTask, value: Prepared) -> Result<Option<Task<'a>>, Diagnostic> {
    match value {
        Prepared::Spread(prototype) => {
            task.phase = Phase::Spread;
            return Ok(Some(Task::Generic(Box::new(GenericTask::new(prototype, task.context.clone(), task.expected)))));
        }
        Prepared::Done => task.phase = Phase::Done,
        Prepared::Function { index, restore_name } => {
            let ready = analyzer.function_request_ready(index);
            if ready.is_err() && let Some(original) = &restore_name {
                let ExprKind::Call(callee, _) = &mut call(&mut task.expression).kind else { unreachable!() };
                *Analyzer::generic_call_name(callee).expect("published generic name").0 = original.clone();
            }
            task.phase = Phase::Function { restore_name };
            if !ready? { return Ok(Some(Task::Function(Box::new(functions::FunctionTask::new(index))))); }
        }
        Prepared::Arguments(mut frame) => {
            let contexts = analyzer.generic_preparation_contexts(&task.context, &mut task.expression, task.expected, &mut frame)?;
            task.phase = Phase::Pass(Pass { frame, contexts, next: 0, progress: false, deferred: None });
        }
    }
    Ok(None)
}
// Probe only independently typed or already solved inputs; unknown inputs never
// become guessed void parameters. Real argument checking retains all effects.
fn closure_hint_key(analyzer: &mut Analyzer, expression: &mut Expr, index: usize, inputs:Option<&[(Type,Option<crate::type_syntax::ParameterAccess>)]>) -> Option<(usize, usize)> {
    let ExprKind::Call(callee, args) = &mut call(expression).kind else { return None; };
    let ExprKind::Closure { params, result: ResultAnnotation::Inferred(_), .. } = &args[index].kind else { return None; };
    for (position,parameter) in params.iter().enumerate() {
        match &parameter.ty {
            ParameterAnnotation::Named(annotation) => { analyzer.resolve_value_annotation(annotation, false).ok()?; }
            ParameterAnnotation::Callback(annotation) => { analyzer.resolve_callback(annotation).ok()?; }
            ParameterAnnotation::Inferred(_) => {let ty=inputs?.get(position)?.0;if ty==Type::Void || analyzer.has_pending_result(ty){return None;}},
        }
    }
    let (name, _) = Analyzer::generic_call_name(callee)?;
    let (_, pattern, _) = analyzer.generic_functions.template(name)?.params.get(index)?;
    crate::callable_annotations::ownership(pattern)?;
    Some((pattern.span.start, pattern.span.end))
}
pub(super) fn step<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: Box<GenericTask>, mut completion: Option<Completion<'a>>) -> Step<'a> {
    let action = (|| -> Result<Option<Task<'a>>, Diagnostic> {
        loop {
            match std::mem::replace(&mut task.phase, Phase::Start) {
                Phase::Start => {
                    let value = analyzer.start_generic_preparation(&task.context, &mut task.expression, task.expected)?;
                    if let Some(child) = prepared(analyzer, &mut task, value)? { return Ok(Some(child)); }
                }
                Phase::Spread => {
                    let Completion::Generic(prototype, result) = completion.take().expect("spread generic prototype") else { unreachable!() };
                    result?;
                    let ExprKind::Call(prepared, _) = prototype.into_kind() else { unreachable!() };
                    let ExprKind::Call(callee, _) = &mut call(&mut task.expression).kind else { unreachable!() };
                    *callee = prepared;
                    task.phase = Phase::Done;
                }
                Phase::Function { restore_name } => {
                    if let Some(value) = completion.take() {
                        let Completion::Function(result) = value else { unreachable!("generic inferred body") };
                        if let Err(error) = result {
                            if let Some(original) = restore_name {
                                let ExprKind::Call(callee, _) = &mut call(&mut task.expression).kind else { unreachable!() };
                                *Analyzer::generic_call_name(callee).expect("published generic name").0 = original;
                            }
                            return Err(error);
                        }
                    }
                    task.phase = Phase::Done;
                }
                Phase::Argument { mut pass, index } => {
                    let Completion::Generic(value, result) = completion.take().expect("generic argument preparation") else { unreachable!() };
                    let ExprKind::Call(_, args) = &mut call(&mut task.expression).kind else { unreachable!() };
                    args[index] = *value;
                    match result {
                        Ok(()) => {
                            let key=closure_hint_key(analyzer,&mut task.expression,index,pass.frame.parameter_hints[index].as_deref());
                            let ExprKind::Call(_,args)=&call(&mut task.expression).kind else{unreachable!()};
                            let array=matches!(args[index].kind,ExprKind::Array(_)) && !pass.frame.value_hints.contains_key(&index)
                                && analyzer.generic_argument_hint(&task.context,&args[index],pass.contexts[index]).is_none();
                            if key.is_some_and(|key|!pass.frame.inline_shapes.contains_key(&key)) || array {
                                let mut value=Box::new(args[index].clone());
                                if let ExprKind::Closure{params,..}=&mut value.kind {
                                    for(position,parameter)in params.iter_mut().enumerate(){
                                        if matches!(parameter.ty,ParameterAnnotation::Inferred(_)) {
                                            let (ty,access)=&pass.frame.parameter_hints[index].as_ref().expect("resolved probe inputs")[position];
                                            parameter.ty=ParameterAnnotation::Named(Box::new(analyzer.checked_value_annotation(*ty,parameter.name.span)?));
                                            parameter.access=access.as_ref().map(|access|crate::syntax::Token{kind:crate::syntax::TokenKind::Name,text:access.mode.as_str().into(),span:access.span});
                                        }
                                    }
                                }
                                let context=contexts.push(Box::new((*task.context).clone()));
                                let discovering=analyzer.discovering_results;let effects=analyzer.effects_only;
                                analyzer.discovering_results=true;analyzer.effects_only=true;
                                let child=Task::Expression(ExpressionTask::new(Expression::Child(value),context,Mode::Consume,ActiveLoans::Shared(Rc::new(Loans::new())),pass.contexts[index]));
                                task.phase=Phase::ClosureHint{pass,index,key,context,discovering,effects};
                                return Ok(Some(child));
                            }
                            pass.frame.prepared[index] = true; pass.progress = true;
                        }
                        Err(error) if error.code == "generic-inference" => { if pass.deferred.is_none() { pass.deferred = Some(error); } }
                        Err(error) => return Err(error),
                    }
                    task.phase = Phase::Pass(pass);
                }
                Phase::ClosureHint { mut pass, index, key, context, discovering, effects } => {
                    // Discovery publishes only reusable shapes. Real argument checking
                    // still owns creation effects, capture loans and validated bodies.
                    drop(contexts.take(context));
                    analyzer.discovering_results = discovering;
                    analyzer.effects_only = effects;
                    let Completion::Expression(value, result) = completion.take().expect("closure shape discovery") else { unreachable!() };
                    let span = value.into_child().span;
                    let actual = result?;
                    if analyzer.has_pending_result(actual) { return Err(Analyzer::pending_result(span)); }
                    if let Some(key)=key {
                        let shape=analyzer.checked_generic_callable_shape(actual).ok_or_else(||Analyzer::pending_result(span))?;
                        pass.frame.inline_shapes.insert(key,shape);
                    }else{pass.frame.value_hints.insert(index,actual);}
                    pass.frame.prepared[index] = true;
                    pass.progress = true;
                    pass.contexts=analyzer.generic_preparation_contexts(&task.context,&mut task.expression,task.expected,&mut pass.frame)?;
                    task.phase = Phase::Pass(pass);
                }
                Phase::Pass(mut pass) => {
                    let ExprKind::Call(_, args) = &mut call(&mut task.expression).kind else { unreachable!() };
                    let next = (pass.next..pass.contexts.len().min(args.len())).find(|&index| !pass.frame.prepared[index]);
                    if let Some(index) = next {
                        pass.next = index + 1;
                        let span = args[index].span;
                        let value = Box::new(std::mem::replace(&mut args[index], Expr::new(ExprKind::Undefined, span)));
                        let child = Task::Generic(Box::new(GenericTask::new(value, task.context.clone(), pass.contexts[index])));
                        task.phase = Phase::Argument { pass, index };
                        return Ok(Some(child));
                    }
                    if pass.frame.prepared.iter().all(|value| *value) {
                        let value = analyzer.finish_generic_preparation(&task.context, &mut task.expression, task.expected, &mut pass.frame)?;
                        if let Some(child) = prepared(analyzer, &mut task, value)? { return Ok(Some(child)); }
                    } else {
                        if !pass.progress { return Err(pass.deferred.expect("unprepared generic argument")); }
                        pass.contexts = analyzer.generic_preparation_contexts(&task.context, &mut task.expression, task.expected, &mut pass.frame)?;
                        pass.next = 0; pass.progress = false; pass.deferred = None;
                        task.phase = Phase::Pass(pass);
                    }
                }
                Phase::Done => return Ok(None),
            }
        }
    })();
    match action {
        Ok(Some(child)) => Step::Push(Task::Generic(task), child),
        result => Step::Complete(Completion::Generic(task.expression, result.map(|_| ()))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn inferred_generic_adapter_bodies_share_the_preparation_stack() {
        let count = 64;
        let mut source = "function step0<T>(value:T){return value;}".to_owned();
        for index in 1..=count {
            source.push_str(&format!("function step{index}<T>(value:T){{const action=step{};return [value].map(action)[0];}}", index-1));
        }
        source.push_str(&format!("function main(){{const action=step{count};console.log([7].map(action)[0]);}}"));
        let checked = crate::check(&source).unwrap();
        assert!(checked.program.closures.len() > count);
        drop(checked.emit_rust());
        drop(checked);
    }
    #[test]
    fn generic_argument_preparation_restores_failed_nested_call_names() {
        let source = crate::format("function fail<T>(value:T){return missing;}function outer<T>(value:T):T{return value;}function main(){outer(fail(7));}").unwrap();
        let mut program = crate::syntax::parse(&source).unwrap();
        let crate::syntax::StmtKind::Expr(mut value) = program.functions[2].body.remove(0).into_kind() else { panic!("outer call"); };
        let mut analyzer = super::Analyzer::new(program).unwrap();
        let context = super::Context::new("main".into());
        let error = analyzer.prepare_generic_call(&context, &mut value, None).expect_err("inferred nested body failure");
        assert_eq!(&source[error.span.start..error.span.end], "missing");
        let crate::syntax::ExprKind::Call(callee, args) = &value.kind else { panic!("restored outer call"); };
        assert!(matches!(&callee.kind, crate::syntax::ExprKind::Name(name) if name == "outer"));
        let crate::syntax::ExprKind::Call(callee, _) = &args[0].kind else { panic!("restored inner call"); };
        assert!(matches!(&callee.kind, crate::syntax::ExprKind::Name(name) if name == "fail"));
    }
}

#[cfg(test)]
mod chain_tests {
    #[test]
    fn inferred_direct_generic_calls_share_the_driver() {
        let count = 256;
        let mut source = "function step0<T>(value:T){return value;}".to_owned();
        for index in 1..count {
            source.push_str(&format!("function step{index}<T>(value:T){{return step{}(value);}}", index-1));
        }
        source.push_str(&format!("function main(){{console.log(step{}(7));}}", count-1));
        drop(crate::check(&source).unwrap().emit_rust());
    }
}

#[cfg(test)]
mod entry_tests {
    fn chain(kind: &str, count: usize, fail: bool) -> String {
        let leaf = if fail { "missing" } else if kind == "receiver" { "[value]" } else { "value" };
        let mut source = format!("function step0<T>(value:T){{return {leaf};}}");
        for index in 1..count {
            let previous = index-1;
            let value = match kind {
                "spread" => format!("step{previous}(...[value])"),
                "receiver" => format!("step{previous}(value).slice()"),
                "optional" => format!("step{previous}?.(value)"),
                _ => unreachable!(),
            };
            source.push_str(&format!("function step{index}<T>(value:T){{return {value};}}"));
        }
        source.push_str(&format!("function main(){{console.log(step{}(7));}}", count-1));
        source
    }
    #[test]
    fn generic_spread_optional_and_receiver_preparation_share_the_driver() {
        for (kind, count) in [("spread",128),("receiver",64),("optional",128)] {
            let source = chain(kind,count,false);
            let checked = crate::check(&source).unwrap_or_else(|error|panic!("{kind}: {}",error.render("input.tsr",&source)));
            drop(checked.emit_rust());
            drop(checked);
        }
    }
    #[test]
    fn failed_generic_entry_preparation_keeps_leaf_source_location() {
        for kind in ["spread","receiver","optional"] {
            let source = chain(kind,32,true);
            let error = crate::check(&source).err().expect("invalid leaf result");
            assert_eq!(&source[error.span.start..error.span.end],"missing","{kind}: {error:?}");
        }
    }
}

#[cfg(test)]
mod closure_hint_tests {
    use super::*;
    #[test]
    fn inferred_async_closure_results_constrain_generic_task_payloads() {
        let source = "function invoke<T>(action:Once<Function<Task<T>>>):Task<T>{return action();}async function main():Task<void>{const number:int=await invoke(async()=>{return 7;});const text:string=await invoke(async()=>{return 'ready';});await invoke(async()=>{});console.log(number,text);}";
        let checked = crate::check(source).unwrap_or_else(|error|panic!("{}",error.render("input.tsr",source)));
        drop(checked.emit_rust());
        let wrong = source.replace("const number:int", "const number:string");
        let error = crate::check(&wrong).err().expect("the inferred payload remains int");
        assert_ne!(error.code,"generic-inference");
    }
    #[test]
    fn failed_array_shape_discovery_restores_phase_flags_and_source() {
        let source=crate::format("function keep<T>(values:Owned<T[]>):T[]{return values;}function main(){keep([missing]);identity(7);}function identity<T>(value:T):T{return value;}").unwrap();
        let mut program=crate::syntax::parse(&source).unwrap();
        let StmtKind::Expr(value)=program.functions[1].body.remove(0).into_kind() else{panic!("array call")};
        let StmtKind::Expr(valid)=program.functions[1].body.remove(0).into_kind() else{panic!("valid call")};
        let mut analyzer=Analyzer::new(program).unwrap();
        for(discovering,effects)in[(false,false),(true,true)]{
            analyzer.discovering_results=discovering;analyzer.effects_only=effects;
            let mut probe=value.clone();let error=analyzer.prepare_generic_call(&Context::new("main".into()),&mut probe,None).expect_err("bad array element");
            assert_eq!(&source[error.span.start..error.span.end],"missing");
            assert_eq!((analyzer.discovering_results,analyzer.effects_only),(discovering,effects));
            let ExprKind::Call(_,arguments)=&probe.kind else{panic!("restored call")};assert!(matches!(arguments[0].kind,ExprKind::Array(_)));
            let mut next=valid.clone();analyzer.prepare_generic_call(&Context::new("main".into()),&mut next,None).unwrap();
            assert_eq!((analyzer.discovering_results,analyzer.effects_only),(discovering,effects));
        }
    }
    #[test]
    fn failed_closure_shape_discovery_restores_phase_flags_and_source() {
        let source = crate::format("function invoke<T>(action:Once<Function<Task<T>>>):Task<T>{return action();}function main(){invoke(async()=>{return missing;});identity(7);}function identity<T>(value:T):T{return value;}").unwrap();
        let mut program = crate::syntax::parse(&source).unwrap();
        let StmtKind::Expr(value) = program.functions[1].body.remove(0).into_kind() else {panic!("call")};
        let StmtKind::Expr(valid) = program.functions[1].body.remove(0).into_kind() else {panic!("valid call")};
        let mut analyzer = Analyzer::new(program).unwrap();
        for (discovering,effects) in [(false,false),(true,true)] {
            analyzer.discovering_results=discovering;
            analyzer.effects_only=effects;
            let mut probe=value.clone();
            let error=analyzer.prepare_generic_call(&Context::new("main".into()),&mut probe,None).expect_err("invalid inferred body");
            assert_eq!(&source[error.span.start..error.span.end],"missing");
            assert_eq!((analyzer.discovering_results,analyzer.effects_only),(discovering,effects));
            let ExprKind::Call(callee,args)=&probe.kind else{panic!("failed call restored")};
            assert!(matches!(&callee.kind,ExprKind::Name(name) if name=="invoke"));
            assert!(args[0].ty.is_none(),"discovery never publishes a checked argument prototype");
            assert_eq!(format!("{:?}",args[0].kind),format!("{:?}",match &value.kind{ExprKind::Call(_,args)=>&args[0].kind,_=>unreachable!()}));
            let mut valid=valid.clone();
            let mut context=Context::new("main".into());
            analyzer.prepare_generic_call(&context,&mut valid,None).expect("unrelated preparation after failure");
            assert_eq!(analyzer.expression(&mut context,&mut valid,Mode::Consume,&Loans::new(),Some(Type::Int)).unwrap(),Type::Int);
            assert_eq!((analyzer.discovering_results,analyzer.effects_only),(discovering,effects));
        }
    }
    #[test]
    fn inferred_closure_preparation_preserves_real_capture_consumption() {
        let source="function invoke<T>(action:Once<Function<Task<T>>>):Task<T>{return action();}async function main():Task<void>{const values=[1,2];const count=await invoke(async()=>{const owned=values;return owned.length;});console.log(count,values.length);}";
        let error=crate::check(source).err().expect("the actual closure still consumes its captured owner");
        assert_ne!(error.code,"generic-inference");
        assert_eq!(&source[error.span.start..error.span.end],"values");
    }
}
