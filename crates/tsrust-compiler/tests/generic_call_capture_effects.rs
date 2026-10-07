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
