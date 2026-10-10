//! Checked-expression transport controls; public owning storage remains guarded.
use super::*;
use super::native_view_loans::{LifetimeSource,Loan,Segment};

#[test]
fn native_view_callback_field_call_keeps_selected_environment_origin() {
    let source="type Window={};type Holder={marker:int;};function save(view:Window){return ()=>view;}function main(){}";
    let formatted=crate::formatting::format(source).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
    let mut parsed=crate::syntax::parse(&formatted).unwrap();
    parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
    let mut analyzer=Analyzer::new(parsed).unwrap();
    analyzer.function(0).unwrap();
    let capture=analyzer.closures[0].captures[0].inner;
    let mut context=Context::new("caller".into());
    let mut bind=|name:&str,ty|Analyzer::bind(&mut context,&Token{text:name.into(),kind:TokenKind::Name,span:Span::default()},ty,false).unwrap();
    let first=bind("first",Type::Record(0));
    let second=bind("second",Type::Record(0));
    let holder=bind("holder",Type::Record(1));
    assert_ne!(first,second);
    Analyzer::set_native_view_loans(&mut context,holder,vec![Loan{
        root:second,owner_fields:vec![],source:LifetimeSource::Carried(vec![]),
        access:crate::native_return_borrows::BorrowAccess::ReadOnly,creation:Span::default(),retained:true,
        path:vec![Segment::Field("read".into()),Segment::Field(format!("capture{capture}"))],guards:vec![],
    }]);
    analyzer.remember_native_closure_returns(&mut context,0,None,Span::default()).unwrap();
    let mut base=Expr::new(ExprKind::Name("holder".into()),Span::default());
    base.binding=Some(holder);base.ty=Some(Type::Record(1));
    let mut callee=Expr::new(ExprKind::Field(Box::new(base),"read".into()),Span::default());
    callee.ty=Some(Type::Closure(0));
    let mut call=Expr::new(ExprKind::Call(Box::new(callee),vec![]),Span::default());
    call.target=Some(CallTarget::ConditionalClosure(0));call.ty=Some(Type::Record(0));
    let loans=Analyzer::native_view_value_loans(&context,&call).unwrap();
    assert_eq!(loans.len(),1,"field invocation must transport its carried capture");
    assert_eq!(loans[0].root,second);
    assert!(loans[0].path.is_empty());
    assert_eq!(loans[0].source,LifetimeSource::Carried(vec![]));
    context.bindings[holder].native_view_loans.as_mut().unwrap()[0].path[1]=Segment::Field("capture999".into());
    let error=Analyzer::native_view_value_loans(&context,&call).err().expect("a missing capture slot must not silently erase a returned lifetime");
    assert_eq!(error.code,"native-view-call");
}

#[test]
fn native_view_conditional_callback_source_keeps_both_possible_origins() {
    let source="type Window={};function save(view:Window){return ()=>view;}function relay(first:Window,second:Window,flag:boolean):Window{const left=save(first);const right=save(second);return (flag?left:right)();}function main(){}";
    let formatted=crate::formatting::format(source).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
    let mut parsed=crate::syntax::parse(&formatted).unwrap();
    parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
    let checked=analyze(parsed).unwrap();
    let origins=checked.functions[1].native_returns.as_ref().expect("conditional invocation needs actual capture lifetimes");
    assert_eq!(origins.len(),2);
    assert!(origins.iter().any(|loan|loan.root==0));
    assert!(origins.iter().any(|loan|loan.root==1));
    assert!(origins.iter().all(|loan|loan.path.is_empty()));
    assert_conditional_native_execution(&checked);
}

fn assert_conditional_native_execution(checked:&CheckedProgram) {
    let mut generated=crate::emit::emit(checked);
    let main=generated.find("fn main()").unwrap();
    let opening=main+generated[main..].find('{').unwrap()+1;
    generated.insert_str(opening,"conditional_probe();");
    let probe="fn conditional_probe(){let first=vec![1u8];let second=vec![7u8,8];let left=function1(Record0{bytes:&first},Record0{bytes:&second},true);assert_eq!(left.bytes.as_ptr(),first.as_ptr());let right=function1(Record0{bytes:&first},Record0{bytes:&second},false);assert_eq!(right.bytes.as_ptr(),second.as_ptr());assert_eq!(right.bytes,&[7,8]);}";
    let directory=std::env::temp_dir().join(format!("tsrust-conditional-carried-view-{}",std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    let _cleanup=Cleanup(directory.clone());
    let input=directory.join("conditional.rs");
    std::fs::write(&input,format!("{generated}\n{probe}")).unwrap();
    for level in ["0","3"] {
        let output=directory.join(format!("conditional-{level}"));
        let compiled=std::process::Command::new("rustc").arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg(&input).arg("-o").arg(&output).output().unwrap();
        assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        assert!(std::process::Command::new(output).status().unwrap().success());
    }
    let negative="fn invalid_owner_escape(){let first=vec![1u8];let result;{let second=vec![7u8];result=function1(Record0{bytes:&first},Record0{bytes:&second},false);}assert_eq!(result.bytes.len(),1);}";
    std::fs::write(&input,format!("{generated}\n{probe}\n{negative}")).unwrap();
    let rejected=std::process::Command::new("rustc").arg("--edition=2024").arg(&input).arg("-o").arg(directory.join("rejected")).output().unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("does not live long enough"));
}
