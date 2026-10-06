mod support;
#[path="../src/exception_emit.rs"] mod exception_emit;

#[test]
fn nested_regions_preserve_return_and_allow_finally_to_replace_it() {
    let inner=exception_emit::completion_region(1,"break 'exception_region1 TsrCompletion::Return(4_i64);");
    let inner=exception_emit::finally_region(&inner,"{ trace.push(5); TsrCompletion::Normal(()) }");
    let outer=exception_emit::completion_region(0,&format!("let completion:TsrCompletion<(),i64,String>={inner}; break 'exception_region0 completion;"));
    let outer=exception_emit::finally_region(&outer,"{ trace.push(6); TsrCompletion::Normal(()) }");
    let replacement=exception_emit::finally_region(&outer,"{ trace.push(7); TsrCompletion::Throw(std::string::String::from(\"replacement\")) }");
    let caught=exception_emit::catch_region("pending","caught", "{ trace.push(caught.0); TsrCompletion::Normal(()) }");
    let catch_checks=format!("struct NonClone(i64); for throwing in [false,true] {{ let pending:TsrCompletion<(),i64,NonClone>=if throwing {{TsrCompletion::Throw(NonClone(8))}}else{{TsrCompletion::Return(9)}}; let handled:TsrCompletion<(),i64,String>={caught}; if throwing {{assert!(matches!(handled,TsrCompletion::Normal(())));}}else{{assert!(matches!(handled,TsrCompletion::Return(9)));}} }} assert_eq!(trace,vec![8]);");
    let forwarded=exception_emit::propagate_completion(2,"pending");
    let forwarding_body=format!("let value:i64={forwarded}; trace.push(value);");
    let forwarding=exception_emit::completion_region(2,&forwarding_body);
    let forwarding_checks=format!("for kind in 0..5 {{ let pending:TsrCompletion<i64,i64,String>=match kind {{0=>TsrCompletion::Normal(10),1=>TsrCompletion::Return(11),2=>TsrCompletion::Throw(\"error\".into()),3=>TsrCompletion::Break(TsrControlTarget::Loop(12)),_=>TsrCompletion::Continue(TsrControlTarget::Loop(13))}}; let result:TsrCompletion<(),i64,String>={forwarding}; match kind {{0=>assert!(matches!(result,TsrCompletion::Normal(()))),1=>assert!(matches!(result,TsrCompletion::Return(11))),2=>assert!(matches!(result,TsrCompletion::Throw(error) if error==\"error\")),3=>assert!(matches!(result,TsrCompletion::Break(TsrControlTarget::Loop(12)))),_=>assert!(matches!(result,TsrCompletion::Continue(TsrControlTarget::Loop(13))))}} }} assert_eq!(trace,vec![10]);");
    let source=format!("{} fn main() {{ let mut trace=vec![]; let result:TsrCompletion<(),i64,String>={outer}; assert!(matches!(result,TsrCompletion::Return(4))); assert_eq!(trace,vec![5,6]); trace.clear(); let result:TsrCompletion<(),i64,String>={replacement}; assert!(matches!(result,TsrCompletion::Throw(error) if error==\"replacement\")); assert_eq!(trace,vec![5,6,7]); trace.clear(); {catch_checks} trace.clear(); {forwarding_checks} }}",include_str!("../src/runtime_completion.rs"));
    let directory=support::TempDirectory::new();
    let path=directory.0.join("regions.rs");
    let binary=directory.0.join("regions");
    std::fs::write(&path,source).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2021").arg(path).arg("-o").arg(&binary),30));
    support::success(&support::run(&mut std::process::Command::new(binary),30));
}

#[test]
fn inline_finally_keeps_owned_payload_until_cleanup_and_disposes_once() {
    let cleanup="{ assert_eq!(drops.get(),before); if replace {TsrCompletion::Throw(Owner(&drops))}else{TsrCompletion::Normal(())} }";
    let expression=exception_emit::finally_region("pending",cleanup);
    let source=format!(r#"{}
fn main() {{
    struct Owner<'a>(&'a std::cell::Cell<usize>);
    impl Drop for Owner<'_> {{ fn drop(&mut self) {{self.0.set(self.0.get()+1);}} }}
    let drops=std::cell::Cell::new(0);
    for replace in [false,true] {{
        let before=drops.get();
        let pending:TsrCompletion<(),Owner<'_>,Owner<'_>>=TsrCompletion::Return(Owner(&drops));
        let result:TsrCompletion<(),Owner<'_>,Owner<'_>>={expression};
        assert_eq!(drops.get(),before+usize::from(replace));
        if replace {{assert!(matches!(&result,TsrCompletion::Throw(_)));}}
        else {{assert!(matches!(&result,TsrCompletion::Return(_)));}}
        drop(result);
        assert_eq!(drops.get(),before+1+usize::from(replace));
    }}
}}
"#,include_str!("../src/runtime_completion.rs"));
    let directory=support::TempDirectory::new();
    let path=directory.0.join("owned-regions.rs");
    let binary=directory.0.join("owned-regions");
    std::fs::write(&path,source).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2021").arg(path).arg("-o").arg(&binary),30));
    support::success(&support::run(&mut std::process::Command::new(binary),30));
}

#[test]
fn ordinary_emission_preserves_labeled_for_update_routing() {
    let source="function main(){let count=0;outer:for(let i=0;i<3;i++){for(;;){count++;continue outer;}}console.log(count);}";
    let rust=tsrust_compiler::check(source).unwrap().emit_rust();
    let directory=support::TempDirectory::new();
    let path=directory.0.join("ordinary-control.rs");
    let binary=directory.0.join("ordinary-control");
    std::fs::write(&path,rust).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2024").arg(path).arg("-o").arg(&binary),30));
    let output=support::run(&mut std::process::Command::new(binary),30);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(),"3\n");
}
