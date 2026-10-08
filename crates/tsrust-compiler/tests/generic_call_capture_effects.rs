mod support;
const DECLARATIONS:&str="function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}";
fn source(body:&str)->String{format!("{DECLARATIONS}function unused<T>(flag:boolean,value:T):T{{{body}}}function main(){{}}")}

#[test]
fn invoked_capture_writes_reject_invalid_reachable_calls() {
    let cases=[
        ("named captured write","let action=one;function change(){action=two;}change();return action<T>(value);"),
        ("arrow captured write","let action=one;const change=()=>{action=two;};change();return action<T>(value);"),
        ("transitive invocation","let action=one;function change(){action=two;}function relay(){change();}relay();return action<T>(value);"),
        ("recursive invocation","let action=one;function change(remaining:int){if(remaining===0){action=two;return;}change(remaining-1);}change(2);return action<T>(value);"),
        ("conditional invocation","let action=one;function change(){action=two;}if(flag){change();}return action<T>(value);"),
        ("writer throws after replacement","let action=one;function change(){action=two;throw 1;}try{change();}catch(error){return action<T>(value);}return value;"),
        ("callback invokes writer","let action=one;function change(){action=two;}[1].forEach(()=>{change();});return action<T>(value);"),
        ("write inside call argument","let action=one;function change():boolean{action=two;return flag;}one<T>((change(),value));return action<T>(value);"),
    ];
    let failures=cases.into_iter().filter_map(|(label,body)|{
        let source=source(body);
        match tsrust_compiler::check(&source){
            Ok(_)=>Some(format!("{label}: invalid call accepted")),
            Err(error) if error.code=="arity" && !error.related.is_empty()=>None,
            Err(error)=>Some(format!("{label}: {}",error.render("input.tsr",&source))),
        }
    }).collect::<Vec<_>>();
    assert!(failures.is_empty(),"{}",failures.join("\n"));
}

#[test]
fn declaration_capture_precision_preserves_noninvocation_snapshots_and_shadowing(){
    for(label,body)in[
        ("uncalled declaration","let action=one;function change(){action=two;}return action<T>(value);"),
        ("uncalled arrow","let action=one;const change=()=>{action=two;};return action<T>(value);"),
        ("saved snapshot","let action=one;const saved=action;function change(){action=two;}change();return saved<T>(value);"),
        ("definite restoration","let action=one;function change(){action=two;}change();action=one;return action<T>(value);"),
        ("lexical shadowing","let action=one;function change(){let action=two;action<T>(value,value);}change();return action<T>(value);"),
        ("writer restores before normal exit","let action=one;function change(){action=two;action=one;}change();return action<T>(value);"),
        ("callee snapshot before writer argument","let action=one;function change():boolean{action=two;return flag;}action<T>((change(),value));return value;"),
        ("uncalled declaration after call","let action=one;const result=action<T>(value);function change(){action=two;}return result;"),
    ]{
        let source=source(body);let formatted=tsrust_compiler::format(&source).unwrap();
        assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
        if let Err(error)=tsrust_compiler::check(&formatted){panic!("{label}: {}",error.render("input.tsr",&formatted));}
    }
}

#[test]
fn nested_capturing_function_can_name_enclosing_generic_type() {
    let source = "function outer<T>(value:T):T{function take():T{return value;}return take();}function main(){}";
    let formatted = tsrust_compiler::format(source).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    if let Err(error) = tsrust_compiler::check(&formatted) { panic!("{}", error.render("input.tsr", &formatted)); }
}

#[test]
fn lexical_generic_instances_keep_explicit_arity_and_independent_types() {
    support::execute(
        "function outer<T>(value:T):T{function echo(other:T):T{return other;}return echo(value);}function main(){console.log(outer<int>(7),outer<string>(\"owned\"));}",
        "7 owned\n",
        None,
    );
}

#[test]
fn nested_explicit_generics_shadow_enclosing_variables() {
    support::execute(
        "function outer<T>(value:T):string{function identity<T>(other:T):T{return other;}const number=identity<int>(7);const text=identity<string>(\"inner\");return String(number)+\":\"+text;}function main(){console.log(outer<boolean>(true));}",
        "7:inner\n",
        None,
    );
}

#[test]
fn lexical_generic_contexts_cover_captures_peers_and_nested_templates() {
    for source in [
        "function outer<T>(value:T):string{function describe():string{return String(value);}return describe();}function main(){console.log(outer<int>(7),outer<string>(\"owned\"));}",
        "function outer<T>(value:T):T{function echo(other:T):T{return other;}return echo(value);}function main(){console.log(outer(7),outer(\"owned\"));}",
        "function outer<T>(value:T):T{function relay(other:T):T{return echo(other);}function echo(other:T):T{return other;}return relay(value);}function main(){console.log(outer<int>(7),outer<string>(\"owned\"));}",
        "function outer<T>(value:T):T{function echo<U>(other:T,unused:U):T{return other;}return echo<boolean>(value,true);}function main(){console.log(outer<int>(7),outer<string>(\"owned\"));}",
        "function outer<T>(value:T):T{function echo<U>(other:T,unused:U):T{return other;}return echo(value,true);}function main(){console.log(outer<int>(7),outer<string>(\"owned\"));}",
        "function outer<T>(value:T):T{function echo(other:T):T{return other;}const saved=echo;return saved(value);}function main(){console.log(outer<int>(7),outer<string>(\"owned\"));}",
        "function outer<T>(value:T):T{function echo(other:T,remaining:int):T{if(remaining===0){return other;}return echo(other,remaining-1);}return echo(value,2);}function main(){console.log(outer<int>(7),outer<string>(\"owned\"));}",
        "function outer<T>(value:T):string{function unused():string{return String(value);}return String(value);}function main(){console.log(outer<int>(7),outer<string>(\"owned\"));}",
        "function outer<T>(value:T):T{function middle<U>(other:T,unused:U):T{function echo(final:T):T{return final;}return echo(other);}return middle<boolean>(value,true);}function main(){console.log(outer<int>(7),outer<string>(\"owned\"));}",
    ] {
        support::execute(source, "7 owned\n", None);
    }
}

#[test]
fn escaped_lexical_generic_functions_keep_independent_owned_captures() {
    for source in [
        "function factory<T>(value:T){function take():T{return value;}return take;}function main(){const number=factory<int>(7);const text=factory<string>(\"owned\");console.log(number(),text());}",
        "function factory<T>(value:T){function take<U>(unused:U):T{return value;}return take;}function main(){const number=factory<int>(7);const text=factory<string>(\"owned\");console.log(number<boolean>(true),text<int>(0));}",
    ] {
        support::execute(source, "7 owned\n", None);
    }
}

#[test]
fn escaped_lexical_generic_capture_drops_original_owner_once() {
    let source = "type Item={id:int;};function factory<T>(value:T,fail:boolean){function take():T{if(fail){throw 1;}return value;}return take;}function main(){const first=factory<Item>({id:1},false);const second=factory<Item>({id:2},false);const failed=factory<Item>({id:3},true);try{failed();}catch(error){}const one=first();const two=second();if(one.id+two.id!==3){throw 99;}}";
    let formatted = support::format_round_trip(source);
    let rust = tsrust_compiler::check(&formatted).unwrap().emit_rust().replace("fn main()", "fn source_main()") + r#"
static DROPS: std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN: std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 { fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);} }
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),3);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),14);}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.0.join("lexical-disposal.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("lexical-disposal-{level}"));
        support::success(&support::rustc().arg(&path).arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg("-o").arg(&binary).output().unwrap());
        support::success(&std::process::Command::new(binary).output().unwrap());
    }
}

#[test]
fn lexical_type_bindings_do_not_add_public_arity_or_reuse_consumed_captures() {
    for (source, expected) in [
        ("function outer<T>(value:T):T{function nested(other:T):T{return other;}return nested<T>(value);}function main(){}", "type"),
        ("function outer<T>(){function nested<U>(value:U):U{return value;}nested<int,string>(1);}function main(){}", "arity"),
        ("function factory<T>(value:T){function take():T{return value;}return take;}function main(){const action=factory<string>('owned');action();action();}", "closure-consumed"),
    ] {
        let formatted = support::format_round_trip(source);
        let error = tsrust_compiler::check(&formatted).err().expect("invalid lexical contract must be rejected");
        assert_eq!(error.code, expected, "{}", error.render("input.tsr", &formatted));
    }
}

#[test]
fn literal_guards_preserve_only_reachable_callable_contracts() {
    for body in [
        "let action=one;if(false){action=two;}return action<T>(value);",
        "let action=two;if(true){action=one;}return action<T>(value);",
    ] {
        let source=support::format_round_trip(&source(body));
        tsrust_compiler::check(&source).unwrap_or_else(|error|panic!("{}",error.render("literal-guard.tsr",&source)));

    }
}

#[test]
fn literal_guard_native_callable_replacement_has_common_carrier() {
    for body in [
        "let action=one;if(false){action=two;}return action<T>(value);",
        "let action=two;if(true){action=one;}return action<T>(value);",
    ] {
        let executable=format!("{DECLARATIONS}function unused<T>(flag:boolean,value:T):T{{{body}}}function main(){{console.log(unused<int>(false,7));}}");
        support::execute(&executable,"7\n",None);
    }
}

#[test]
fn generic_callable_replacement_keeps_saved_values_and_runtime_selection() {
    let executable = r#"
        function identity<T>(value: T): T { return value; }
        function second<T>(first: T, next: T): T { return next; }
        function select(flag: boolean): int {
            let action = identity;
            const saved = action;
            action = identity;
            if (flag) { action = second; }
            if (flag) { return saved<int>(3) + action<int>(4, 7); }
            return saved<int>(3) + action<int>(4);
        }
        function main() { console.log(select(false), select(true)); }
    "#;
    support::execute(executable, "7 10\n", None);
}

#[test]
fn generic_callable_replacement_rejects_mutation_of_the_borrowed_callee() {
    let executable = r#"
        function identity<T>(value: T): T { return value; }
        function other<T>(value: T): T { return value; }
        function main() {
            let action = identity;
            let evaluations = 0;
            const value = action<int>((evaluations += 1, action = other, 7));
            console.log(value, evaluations, action<string>("ready"));
        }
    "#;
    let formatted = support::format_round_trip(executable);
    let error = tsrust_compiler::check(&formatted)
        .err()
        .expect("argument evaluation must not replace the actively borrowed callee");
    assert_eq!(error.code, "conflicting-access");
    assert_eq!(&formatted[error.span.start..error.span.end], "action");
    assert!(!error.related.is_empty());
}

#[test]
fn collection_callbacks_preserve_empty_regions_element_guards_and_completion_order() {
    for (body, invalid) in [
        ("let action=one;[].forEach(()=>{action=two;});return action<T>(value);", false),
        ("let action=one;[false].forEach((enabled:boolean)=>{if(enabled){action=two;}});return action<T>(value);", false),
        ("let action=one;[false,true].forEach((enabled:boolean)=>{if(enabled){action=two;}});return action<T>(value);", true),
        ("let action=one;[true].forEach(()=>{action=two;action=one;});return action<T>(value);", false),
        ("let action=one;try{[true].forEach(()=>{action=two;throw 1;});}catch(error){return action<T>(value);}return value;", true),
    ] {
        let source=support::format_round_trip(&source(body));
        let result=tsrust_compiler::check(&source);
        if invalid {
            let error=result.err().expect("invoked write must reject invalid arity");
            assert_eq!(error.code,"arity","{}",error.render("collection-effect.tsr",&source));
            assert!(!error.related.is_empty());
        } else if let Err(error)=result { panic!("{}",error.render("collection-effect.tsr",&source)); }
    }
}

#[test]
fn console_calls_preserve_known_capture_writes_and_restoration() {
    for (body, invalid) in [
        ("let action=one;function change(){action=two;console.log(1);}change();return action<T>(value);",true),
        ("let action=one;function change(){console.log(1);action=two;}change();return action<T>(value);",true),
        ("let action=one;function change(){action=two;console.log(1);action=one;}change();return action<T>(value);",false),
        ("let action=one;function change(){action=two;console.log(1);}return action<T>(value);",false),
        ("let action=one;[1].forEach(()=>{action=two;console.log(1);});return action<T>(value);",true),
    ] {
        let source=support::format_round_trip(&source(body));
        let result=tsrust_compiler::check(&source);
        if invalid { let error=result.err().expect("console output cannot erase captured write diagnostics");assert_eq!(error.code,"arity");assert!(!error.related.is_empty()); }
        else if let Err(error)=result {panic!("{}",error.render("console-effects.tsr",&source));}
    }
}

#[test]
fn returned_callable_contracts_preserve_snapshots_and_finally_overrides() {
    for (body, invalid) in [
        ("function pick(){return two;}const chosen=pick();return chosen<T>(value);",true),
        ("function pick(){return [one,two];}const chosen=pick()[1];return chosen<T>(value);",true),
        ("function pick(){return [one,two];}const chosen=pick()[0];return chosen<T>(value);",false),
        ("function pick(){let action=one;try{return action;}finally{action=two;}}const chosen=pick();return chosen<T>(value);",false),
        ("function pick(){try{return one;}finally{return two;}}const chosen=pick();return chosen<T>(value);",true),
    ] {
        let source=support::format_round_trip(&source(body));let result=tsrust_compiler::check(&source);
        if invalid {let error=result.err().expect("returned callable must retain its actual contract");assert_eq!(error.code,"arity");assert!(!error.related.is_empty());}
        else if let Err(error)=result {panic!("{}",error.render("return-snapshot.tsr",&source));}
    }
}

#[test]
fn independent_factory_environments_reject_the_wrong_returned_contract() {
    for factory in [
        "function factory<A>(action:A){function select(){return action;}return select;}",
        "function factory<A>(action:A){return ()=>action;}",
    ] {
        let source=tsrust_compiler::format(&format!("{DECLARATIONS}{factory}function unused<T>(value:T):T{{const first=factory(one);const second=factory(two);const saved=first();const wrong=second();saved<T>(value);wrong<T>(value);return value;}}function main(){{}}")).unwrap();
        assert_eq!(tsrust_compiler::format(&source).unwrap(),source);
        let error=tsrust_compiler::check(&source).err().expect("second instance requires two arguments");
        assert_eq!(error.code,"arity","{}",error.render("factory.tsr",&source));
        assert!(source[error.span.start..error.span.end].contains("wrong"),"{}",error.render("factory.tsr",&source));
        assert!(!error.related.is_empty());
        tsrust_compiler::check(&source.replace("wrong<T>(value)","wrong<T>(value, value)")).unwrap();
    }
}

#[test]
fn returned_factory_contracts_execute_from_distinct_native_environments() {
    let directory=support::TempDirectory::new();
    let source=include_str!("fixtures/callable-environments/retained.tsr");
    for source in [source.to_owned(),source.replace("    function select() {\n        return action;\n    }\n\n    return select;","    return () => action;")] {
        let formatted=tsrust_compiler::format(&source).unwrap();
        assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
        let path=directory.source(&formatted);
        let output=support::run(support::cli().arg("run").arg(path),60);
        support::success(&output);
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn transported_polymorphic_factory_owners_drop_once_on_success_failure_and_abandonment() {
    let source=tsrust_compiler::format("type Item={id:int;fail:boolean;};function make(item:Item){function run<T>(value:T):T{if(item.fail){throw item.id;}return value;}return run;}function factory<A>(action:A){function select(){return action;}return select;}function main(){{const action=make({id:1,fail:false});const first=factory(action);const saved=first();if(saved<int>(7)!==7||saved<string>('x')!=='x'){throw 99;}}{const action=make({id:2,fail:false});const second=factory(action);const saved=second();if(saved<int>(9)!==9){throw 99;}}{const action=make({id:3,fail:true});const third=factory(action);const saved=third();let caught=false;try{saved<int>(0);}catch(error){caught=error===3;}if(!caught){throw 99;}}{const action=make({id:4,fail:false});const abandoned=factory(action);}}").unwrap();
    assert_eq!(tsrust_compiler::format(&source).unwrap(),source);
    let rust=tsrust_compiler::check(&source).unwrap().emit_rust().replace("fn main()","fn source_main()")+r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),4);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),30);}
"#;
    let directory=support::TempDirectory::new();let path=directory.0.join("transported-environments.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("transported-environments-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary),60));
        support::success(&support::run(&mut std::process::Command::new(binary),10));
    }
}

#[test]
fn transported_polymorphic_arguments_preserve_borrow_and_consumption_rejections() {
    let factory="function factory<A>(action:A){function select(){return action;}return select;}";
    for (body,code) in [
        ("function view(item:ReadOnly<Item>){function step<T>(value:T):T{console.log(item.id);return value;}const action=step;const selected=factory(action);}function main(){}","callback-escape"),
        ("function main(){const action=make({id:1});const selected=factory(action);action<int>(7);}","closure-consumed"),
        ("function main(){const action=make({id:1});const selected=factory(action);const saved=selected();const first=saved<int>(7);const second=saved<int>(9);}","closure-consumed"),
    ] {
        let source=tsrust_compiler::format(&format!("type Item={{id:int;}};function make(item:Item){{function take<T>(value:T):Item{{return item;}}return take;}}{factory}{body}")).unwrap();
        let error=tsrust_compiler::check(&source).err().unwrap_or_else(||panic!("borrow or transferred owner must be rejected: {body}"));
        assert_eq!(error.code,code,"{}",error.render("input.tsr",&source));
    }
}
