mod support;
use tsrust_compiler::{Mode, check};

#[test]
fn bare_callable_parameters_transfer_and_borrowing_is_explicit() {
    support::execute("function counter():()=>int{let value=0;return ():int=>{value=value+1;return value;};}function countdown(action:()=>int,remaining:int):int{if(remaining===0){return action();}const value=action();const next=action;return value+countdown(next,remaining-1);}function main(){console.log(countdown(counter(),3));}", "10\n", Some(&[("countdown.action", Mode::Consume), ("countdown.remaining", Mode::Copy)]));
    let source="function take(action:()=>int):int{return action();}function main(){const action=():int=>{return 7;};console.log(take(action));console.log(action());}";
    assert_eq!(check(source).err().unwrap().code,"closure-consumed");
    support::execute(&source.replace("action:()=>int","action:ReadOnly<Function<int>>"), "7\n7\n", Some(&[("take.action",Mode::Read)]));
}

#[test]
fn bare_and_once_callable_fields_own_environments() {
    support::execute("type Handler={run:()=>int;finish:Once<Function<string>>;};function main(){const text='Pat';const handler:Handler={run:():int=>{return 7;},finish:():string=>{return text;}};console.log(handler.run(),handler.run(),handler.finish());}","7 7 Pat\n",None);
}

#[test]
fn owned_callable_parameters_lend_mutable_borrows_without_transferring() {
    support::execute("function borrow(action:Mutable<Function<int>>):int{return action();}function forward(action:()=>int):int{const first=borrow(action);const second=borrow(action);return first+second+action();}function main(){let value=0;console.log(forward(():int=>{value=value+1;return value;}));}", "6\n", None);
    for source in [
        "function borrow(action:Mutable<Function<int>>){}function forward(action:Once<Function<int>>){borrow(action);}function main(){}",
        "function take(action:()=>int){}function forward(action:Mutable<Function<int>>){take(action);}function main(){}",
        "function borrow(action:ReadOnly<Function<int>>){}function forward(action:()=>int){borrow(action);}function main(){}",
    ] {
        assert_eq!(check(source).err().expect("invalid callback contract").code,"callback-contract");
    }
}

#[test]
fn scoped_reborrowing_keeps_exclusive_argument_loans_and_allows_later_transfer() {
    for source in [
        "function pair(first:Mutable<Function<int>>,second:Mutable<Function<int>>){first();second();}function forward(action:()=>int){pair(action,action);}function main(){}",
        "function take(first:Mutable<Function<int>>,second:()=>int){first();second();}function forward(action:()=>int){take(action,action);}function main(){}",
    ] {
        assert_eq!(check(source).err().expect("overlapping callback loan").code,"conflicting-access");
    }
    support::execute("function borrow(action:Mutable<Function<int>>):int{return action();}function retain(action:()=>int):()=>int{console.log(borrow(action));return action;}function main(){let value=0;const next=retain(():int=>{value=value+1;return value;});console.log(next(),next());}","1\n2 3\n",None);
}

#[test]
fn explicit_once_factory_results_own_and_limit_returned_environments() {
    support::execute("function make(value:int):Once<Function<int>>{return ():int=>{return value;};}function grouped():((Once<Function<string>>)){const text='Pat';return ():string=>{return text;};}function alias():Function<int>{return ():int=>{return 8;};}function main(){const first=make(7);const second=grouped();const reusable=alias();console.log(first(),second(),reusable(),reusable());}","7 Pat 8 8\n",None);
    let source="function make():Once<Function<int>>{return ():int=>{return 7;};}function main(){const action=make();action();action();}";
    assert_eq!(check(source).err().expect("second once result call").code,"closure-consumed");
}

#[test]
fn once_results_forward_owned_parameters_and_specialize_nominal_payloads() {
    let source="type User={name:string;};function retain(action:()=>int):Once<Function<int>>{return action;}function snapshot<T>(value:T):Once<Function<T>>{return ():T=>{return value;};}function main(){const action=retain(():int=>{return 7;});const original:User={name:'Pat'};const get=snapshot(original);const user:User=get();console.log(action(),user.name);}";
    let rust=support::execute(source,"7 Pat\n",None);
    let directory=support::TempDirectory::new();
    let path=directory.0.join("optimized.rs");
    std::fs::write(&path,rust).unwrap();
    let binary=directory.0.join("optimized");
    support::success(&support::run(support::rustc().arg("--edition=2024").arg("-Copt-level=3").arg(path).arg("-o").arg(&binary),60));
    let output=support::run(&mut std::process::Command::new(binary),10);
    support::success(&output);
    assert_eq!(output.stdout,b"7 Pat\n");
    assert!(output.stderr.is_empty());
    for modifier in ["ReadOnly<Function<int>>", "Mutable<Function<int>>"] {
        let source=format!("function invalid():{modifier}{{return ():int=>{{return 7;}};}}function main(){{}}");
        assert_eq!(check(&source).err().expect("borrowed factory result").code,"callback-escape");
    }
}
