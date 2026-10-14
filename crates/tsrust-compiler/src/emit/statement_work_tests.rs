//! Normal-stack witnesses for statement scheduling and borrowed declarations.
use super::*;

#[test]
fn nested_storage_branches_resume_without_retaining_rust_frames() {
    let checked = crate::check("type Holder={run:ReadOnly<Function<int>>;};function main():void{const items=[3];const handlers=new Map<string,Holder>();handlers.set('one',{run:():int=>items[0]});if(true){handlers.clear();}else{}}").unwrap();
    let emitter = emitter_for(&checked.program, "");
    let function = checked.program.functions.iter().find(|function| function.name == "main").unwrap();
    let template = function.body.last().unwrap();
    let mut nested = template.clone();
    let depth = 2_048;
    for _ in 0..depth {
        let mut parent = template.clone();
        let StmtKind::If { then_body, .. } = &mut parent.kind else { panic!("branch") };
        *then_body = vec![nested];
        nested = parent;
    }
    emitter.exception_regions.renew_storage(&["unrelated_owner".into()], 40);
    let mut output = String::new();
    emitter.control_block_epochs(&mut output, function, &[nested], 0, None, Some(Declarations::Body(&function.body)));
    assert_eq!(output.matches(" = if ").count(), depth + 1);
    assert_eq!(output.matches('{').count(), output.matches('}').count());
    assert!(emitter.exception_regions.native_storage_available("unrelated_owner", Some(crate::Span::new(20, 40))));
    assert!(!emitter.exception_regions.native_storage_available("unrelated_owner", Some(crate::Span::new(40, 41))));
    assert!(emitter.control_targets.snapshot().is_empty());
    assert_eq!(emitter.exception_regions.current_id(), None);
}

#[test]
fn generated_statement_bytes_do_not_depend_on_indentation_depth() {
    let checked = crate::check("function main():void{let total=0;for(const item of [1,2]){if(item>1){total+=item;}else{total+=3;}}console.log('  literal spacing  ',total);}").unwrap();
    let emitter = emitter_for(&checked.program, "");
    let function = checked.program.functions.iter().find(|function| function.name == "main").unwrap();
    let mut root = String::new();
    let mut nested = String::new();
    emitter.control_block(&mut root, function, &function.body, 0, None);
    emitter.control_block(&mut nested, function, &function.body, 16_384, None);
    assert_eq!(root, nested);
    assert!(root.contains("  literal spacing  "));
}

#[test]
fn mixed_statement_work_preserves_scopes_at_depth() {
    let repetitions = 512;
    let openings = (0..repetitions).map(|index| format!("{{if(true){{while(false){{for(const ch{index} of \"a\"){{do{{")).collect::<String>();
    let source = format!("function main():void{{let trace=0;{}trace=trace+1;{}}}",
        openings,
        "}while(false);}}}}".repeat(repetitions));
    let mut parsed = crate::syntax::parse_unbounded(&source).unwrap();
    crate::callables::module_state::lower(&mut parsed).unwrap();
    crate::local_functions::resolve(&mut parsed).unwrap();
    let checked = crate::analysis::analyze(parsed).unwrap();
    let emitter = emitter_for(&checked, "");
    let function = checked.functions.iter().find(|function| function.name == "main").unwrap();
    let mut output = String::new();
    emitter.control_block(&mut output, function, &function.body, 0, None);
    assert_eq!(output.matches(".as_ref().chars()").count(), repetitions);
    assert_eq!(output.matches(" else {").count(), repetitions);
    assert_eq!(output.matches("'body_").count(), repetitions);
    assert_eq!(output.matches('{').count(), output.matches('}').count());
    assert!(output.contains("1i64"));
    assert!(emitter.control_targets.snapshot().is_empty());
    assert!(emitter.conditional_temporaries.borrow().is_empty());
    assert_eq!(emitter.exception_regions.current_id(), None);
}

#[test]
fn declaration_views_preserve_source_identity_and_prefix_order() {
    let parsed = crate::syntax::parse("function main(){let first=1;{let nested=2;}let last=3;}").unwrap();
    let body = &parsed.functions[0].body;
    let StmtKind::Block(nested) = &body[1].kind else { panic!("block") };
    let mut visible = Declarations::Body(body).prefix(1).iter().collect::<Vec<_>>();
    visible.extend(nested.iter());
    let view = Declarations::Visible(&visible);
    let actual = view.iter().collect::<Vec<_>>();
    assert!(std::ptr::eq(actual[0], &body[0]));
    assert!(std::ptr::eq(actual[1], &nested[0]));
    assert!(std::ptr::eq(view.prefix(1).iter().next().unwrap(), &body[0]));
    assert!(std::ptr::eq(view.iter().rev().next().unwrap(), &nested[0]));
}

#[test]
fn deep_clear_discovery_preserves_reset_order_and_rejects_effects() {
    let checked = crate::check("type Holder={run:ReadOnly<Function<int>>;};function main():void{const items=[3];const handlers=new Map<string,Holder>();handlers.set('one',{run:():int=>items[0]});handlers.clear();}").unwrap();
    let emitter = emitter_for(&checked.program, "");
    let function = checked.program.functions.iter().find(|function| function.name == "main").unwrap();
    let declarations = Declarations::Body(&function.body);
    let clear = function.body.last().unwrap();
    let expected = emitter.local_map_clear_epoch(function, declarations, clear).expect("scoped reset");
    let mut nested = clear.clone();
    for _ in 0..16_384 {
        nested = Stmt { span: nested.span, labels: Vec::new(), control_target: None, kind: StmtKind::Block(vec![nested]) };
    }
    assert_eq!(emitter.map_clear_block_epoch(function, declarations, &nested), Some(expected));
    let StmtKind::Block(body) = &mut nested.kind else { unreachable!() };
    body.push(function.body[2].clone());
    assert!(emitter.map_clear_block_epoch(function, declarations, &nested).is_none());
}


#[test]
fn nested_exception_bodies_resume_and_restore_transport_scopes() {
    let mut checked=crate::check("function probe():void{try{console.log(7);}catch(error){}finally{}}function main(){probe();}").unwrap();
    let runtime=crate::error_effects::materialize_runtime_record(&mut checked.program);
    let function_id=checked.program.functions.iter().position(|function|function.name=="probe").unwrap();
    let function=&checked.program.functions[function_id];
    let template=&function.body[0];
    let mut nested=template.clone();
    let depth=512;
    for _ in 1..depth {
        let mut parent=template.clone();
        let StmtKind::Try(region)=&mut parent.kind else {panic!("exception region")};
        region.body=vec![nested];
        nested=parent;
    }
    let emitter=emitter_for(&checked.program,"");
    let contract=checked.program.error_effects.function_errors[function_id].clone();
    let (_,name)=emitter.completion_error_layout("DeepError",&contract,Some(runtime),template.span);
    let output=emitter.callable_result_body(function,&[nested],0,&name,contract,Some(runtime));
    assert_eq!(output.matches("7i64").count(),1);
    assert_eq!(output.matches("let pending_completion").count(),depth);
    assert_eq!(output.matches('{').count(),output.matches('}').count());
    assert_eq!(emitter.exception_regions.current_id(),None);
    assert!(emitter.control_targets.snapshot().is_empty());
    assert!(emitter.inline_return.borrow().is_none());
    assert!(emitter.inline_abrupt.borrow().is_none());
    assert!(!emitter.cleanup_replaces_exits.get());
}


#[test]
fn nested_field_completion_bodies_use_shared_scheduler() {
    let mut checked=crate::check("type Row={name:string;other:string;};function pick(row:Row,early:boolean):string{try{if(early){return row.name;}}finally{console.log(row.other);}return row.name;}function main(){}").unwrap();
    let runtime=crate::error_effects::materialize_runtime_record(&mut checked.program);
    let function_id=checked.program.functions.iter().position(|function|function.name=="pick").unwrap();
    let function=&checked.program.functions[function_id];
    let template=&function.body[0];let mut nested=template.clone();let depth=512;
    for _ in 1..depth {
        let mut parent=template.clone();let StmtKind::Try(region)=&mut parent.kind else{panic!("exception region")};
        region.body=vec![nested];nested=parent;
    }
    let emitter=emitter_for(&checked.program,"");
    let contract=checked.program.error_effects.function_errors[function_id].clone();
    let (_,name)=emitter.completion_error_layout("DeepFieldError",&contract,Some(runtime),template.span);
    let output=emitter.callable_result_body(function,&[nested],0,&name,contract,Some(runtime));
    assert_eq!(output.matches("checked field cleanup control").count()+output.matches("checked field completion\"").count(),depth,"every nested region must exercise field transport");
    assert_eq!(output.matches('{').count(),output.matches('}').count());
    assert_eq!(emitter.exception_regions.current_id(),None);
    assert!(emitter.exception_regions.field_error_fields().is_none());
    assert!(emitter.control_targets.snapshot().is_empty());
}
