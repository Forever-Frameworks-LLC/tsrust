mod support;

#[test]
fn readable_match_examples_execute_after_formatting() {
    for (source, output) in [
        (include_str!("../../../examples/match-values/main.tsr"), "Hello, Ada!\nName unavailable\n"),
        (include_str!("../../../examples/match-literals/main.tsr"), "Ready Not found Try again\nOn\n"),
        (include_str!("../../../examples/match-variants/main.tsr"), "Welcome, Ada\nInvitation: Friday dinner\n"),
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(source));
        let executed = support::run(support::cli().arg("run").arg(path), 60);
        support::success(&executed);
        assert_eq!(String::from_utf8(executed.stdout).unwrap(), output);
    }
}

#[test]
fn matches_preserve_ownership_exhaustiveness_and_arm_types() {
    for source in [
        "function main(){const x:Result<string,int>=Ok(\"x\");const y=match(x){Ok(v)=>v,Err(_)=>\"e\",};console.log(x,y);}",
        "function get(x:ReadOnly<Result<string,int>>):string{return match(x){Ok(v)=>v,Err(_)=>\"e\",};}",
        "function main(){const x=3;const y=match(x){1=>1,2=>2,};}",
        "function main(){const x=true;const y=match(x){true=>1,true=>2,false=>0,};}",
        "function main(){const x=3;const y=match(x){_=>1,2=>2,};}",
        "function main(){const x=true;const y=match(x){true=>1,false=>\"no\",};}",
        "function main(){const x=true;const y=match(x){true=>{},false=>1,};}",
        "type A={a:int;};type B={b:int;};function get(x:A|B):int{return match(x){A(v)=>v.a,};}",
    ] {
        assert!(tsrust_compiler::check(source).is_err(), "accepted invalid source: {source}");
    }
}

#[test]
fn matches_evaluate_once_and_keep_expression_order_and_enclosing_returns() {
    let source = r#"
import {strictEqual} from "std:testing";
let calls = 0;
function next(): Result<int,string> { calls += 1; return Ok(calls); }
function selected(flag:boolean):int {
    const value=match(flag){true=>{return 9;},false=>4,};
    return value;
}
function main(){
    const sum=(a:int,b:int):int=>a+b;
    const value=sum(match(next()){Ok(n)=>n,Err(_)=>0,},match(next()){Ok(n)=>n,Err(_)=>0,});
    strictEqual(value,3); strictEqual(calls,2);
    const skipped=false && match(next()){Ok(_)=>true,Err(_)=>false,};
    strictEqual(skipped,false); strictEqual(calls,2);
    strictEqual(selected(true),9); strictEqual(selected(false),4);
    strictEqual(match("hello"){"bye"=>1,"hello"=>2,_=>3,},2);
    strictEqual(match(-2){-2=>7,_=>0,},7);
    strictEqual(match(1.5){1.5=>8,_=>0,},8);
    const nested=match(true){true=>match(false){true=>0,false=>11,},false=>0,};
    strictEqual(nested,11);
    strictEqual(match(true){true=>"hello",false=>"no",}.length,5);
    strictEqual(match(true){true=>"hello",false=>"no",},"hello");
}
"#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let graph=tsrust_compiler::project::load_project(&tsrust_compiler::project::FileSourceProvider,path.to_str().unwrap()).unwrap();
    tsrust_compiler::project::check_graph(graph).unwrap();
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn enclosing_return_checker_witness() {
    tsrust_compiler::check("function selected(flag:boolean):int{const value=match(flag){true=>{return 9;},false=>4,};return value;}function main(){console.log(selected(false));}").unwrap();
}

#[test]
fn matches_check_finite_unions_borrows_callbacks_and_argument_loans() {
    for source in [
        "function get(value:'a'|'b'):int{return match(value){'a'=>1,'b'=>2,};}function main(){console.log(get('a'));}",
        "function get(value:Result<int,string>):int{return match(value){Ok(n)=>n,Err(_)=>0,};}function main(){const apply=(x:Result<int,string>):int=>match(x){Ok(n)=>n+1,Err(_)=>0,};console.log(apply(Ok(2)));}",
        "type A={value:int;};type B={value:int;};function read(x:ReadOnly<A|B>):int{return match(x){A(a)=>a.value,B(b)=>b.value,};}function main(){const member:A={value:3};const a:A|B=member;console.log(read(a),read(a));}",
        "function main(){const result:Result<int[],string>=Ok([1,2]);const values=match(result){Ok(v)=>v,Err(_)=>[0],};values.push(3);console.log(values.length);}",
        "type User={name:string;};function read(user:ReadOnly<User>):int{return match(user){User(value)=>value.name.length,};}function main(){const user:User={name:'Ada'};console.log(read(user),read(user));}",
    ] {
        let directory=support::TempDirectory::new();
        let path=directory.source(&support::format_round_trip(source));
        support::success(&support::run(support::cli().arg("run").arg(path),60));
    }
    let source="function use(values:ReadOnly<int[]>,length:int):int{return values.length+length;}function main(){const values=[1];console.log(use(values,match(true){true=>values.push(2),false=>0,}));}";
    let error=tsrust_compiler::check(source).err().expect("conflicting match argument");
    assert!(matches!(error.code,"conflicting-access"|"type"),"{error:?}");
}

#[test]
fn expression_match_keeps_typed_errors_finally_and_owned_callable_results() {
    let directory=support::TempDirectory::new();
    let source=r#"
import {strictEqual} from 'std:testing';
type Fault={message:string;};
let cleaned=0;
function failure():Result<int,Fault>{return Err({message:'broken'});}
function load():int{try{return match(failure()){Ok(value)=>value,Err(error)=>{throw error;},};}finally{cleaned+=1;}}
function returning():int{try{const x=match(true){true=>{return 9;},false=>0,};return x;}finally{cleaned+=1;}}
function callback():Function<int>{const text='owned';return match(true){true=>():int=>text.length,false=>{throw 'unreachable';},};}
function main(){
 try{load();}catch(error){strictEqual(error.message,'broken');}
 strictEqual(cleaned,1);strictEqual(returning(),9);strictEqual(cleaned,2);
 const action=callback();strictEqual(action(),5);
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn nested_match_values_use_the_normal_checker_stack() {
    let mut value="1".to_string();
    for _ in 0..64 {value=format!("match(true){{true=>{value},false=>0,}}");}
    tsrust_compiler::check(&format!("function main(){{console.log({value});}}")).unwrap();
}
