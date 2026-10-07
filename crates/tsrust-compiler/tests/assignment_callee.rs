mod support;

#[test]
fn named_assignment_invocation_keeps_captures_failure_and_owned_results() {
    let source="type Item={id:int;};function make(item:Item){let calls=0;function identity<T>(value:T):T{calls+=1;if(item.id===1&&calls===2){throw calls;}return value;}return identity;}function main(){let action=make({id:1});console.log((action=action)<int>(7));try{(action=action)<int>(9);}catch(error){console.log(error);}console.log((action=action)<string>(\"owned\"));}";
    support::execute(source,"7\n2\nowned\n",None);
}

#[test]
fn assignment_invocation_preserves_consumption_and_argument_loans() {
    support::execute("type Item={id:int;};function make(item:Item){function take():Item{return item;}return take;}function main(){let action=make({id:7});const result=(action=action)();console.log(result.id);}","7\n",None);
    let sources = [
        "function make(){const text=\"owned\";function take():string{return text;}return take;}function main(){let action=make();console.log((action=action)());console.log(action());}",
        "function identity<T>(value:T):T{return value;}function main(){let action=identity;console.log((action=action)<int>((action=action,7)));}",
    ];
    for source in sources {
        let error=tsrust_compiler::check(source).err().expect("ownership violation must be rejected");
        assert!(matches!(error.code,"closure-consumed"|"transferred"|"borrow"|"conflicting-access"|"capture-conflict"),"{}",error.render("input.tsr",source));
    }
}

#[test]
fn assignment_replacement_drops_each_original_capture_once() {
    let source="type Item={id:int;};function make(item:Item){function identity<T>(value:T):T{if(item.id<0){throw item.id;}return value;}return identity;}function main(){let action=make({id:1});if((action=make({id:2}))<int>(7)!==7){throw 99;}if((action=action)<string>(\"owned\")!==\"owned\"){throw 99;}}";
    let formatted=support::format_round_trip(source);
    let rust=tsrust_compiler::check(&formatted).unwrap().emit_rust().replace("fn main()","fn source_main()")+r#"
static DROPS: std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN: std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 { fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);} }
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),2);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),6);}
"#;
    let directory=support::TempDirectory::new();let path=directory.0.join("assignment-disposal.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("assignment-disposal-{level}"));
        support::success(&support::rustc().arg(&path).arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg("-o").arg(&binary).output().unwrap());
        support::success(&std::process::Command::new(binary).output().unwrap());
    }
}
