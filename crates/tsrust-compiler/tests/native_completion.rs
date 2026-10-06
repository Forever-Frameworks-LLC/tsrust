mod support;

#[test]
fn typed_completion_cleanup_preserves_targets_and_disposes_replaced_payload() {
    let directory = support::TempDirectory::new();
    let source = directory.0.join("completion.rs");
    let binary = directory.0.join("completion");
    std::fs::write(&source, concat!(
        include_str!("../src/runtime_completion.rs"),
        r#"
fn main() {
    use std::cell::Cell;
    struct Owner<'a>(&'a Cell<usize>);
    impl Drop for Owner<'_> { fn drop(&mut self) { self.0.set(self.0.get()+1); } }
    let drops=Cell::new(0);
    let pending:TsrCompletion<(),Owner<'_>,String>=TsrCompletion::Return(Owner(&drops));
    let replaced=tsr_finally(pending,|| {
        assert_eq!(drops.get(),0);
        TsrCompletion::Throw("replacement".to_owned())
    });
    assert_eq!(drops.get(),1);
    assert!(matches!(replaced,TsrCompletion::Throw(value) if value=="replacement"));
    let thrown:TsrCompletion<i64,(),Owner<'_>>=TsrCompletion::Throw(Owner(&drops));
    let caught:TsrCompletion<i64,(),String>=tsr_catch(thrown,|error| {
        assert_eq!(drops.get(),1);
        drop(error);
        TsrCompletion::Normal(7)
    });
    assert!(matches!(caught,TsrCompletion::Normal(7)));
    assert_eq!(drops.get(),2);
    let calls=Cell::new(0);
    let returning:TsrCompletion<(),Owner<'_>,String>=TsrCompletion::Return(Owner(&drops));
    let kept:TsrCompletion<(),Owner<'_>,bool>=tsr_catch(returning,|_| {
        calls.set(calls.get()+1);
        TsrCompletion::Throw(false)
    });
    assert_eq!(calls.get(),0);
    assert_eq!(drops.get(),2);
    drop(kept);
    assert_eq!(drops.get(),3);
    let error:TsrCompletion<(),i64,String>=TsrCompletion::Throw("old".to_owned());
    let rethrown:TsrCompletion<(),i64,i64>=tsr_catch(error,|_|TsrCompletion::Throw(17));
    assert!(matches!(rethrown,TsrCompletion::Throw(17)));
    fn fallible<T>(value:T,succeed:bool)->Result<T,T> {
        if succeed {Ok(value)}else{Err(value)}
    }
    for succeed in [false,true] {
        let before=drops.get();
        let called:TsrCompletion<Owner<'_>,(),Owner<'_>>=tsr_from_result(fallible(Owner(&drops),succeed));
        assert_eq!(drops.get(),before);
        let handled:TsrCompletion<Owner<'_>,(),String>=tsr_catch(called,|error|TsrCompletion::Normal(error));
        let handled=tsr_finally(handled,|| {
            assert_eq!(drops.get(),before);
            TsrCompletion::Normal(())
        });
        assert!(matches!(&handled,TsrCompletion::Normal(_)));
        drop(handled);
        assert_eq!(drops.get(),before+1);
    }
    let factories=Cell::new(0);
    for checked in [i64::MAX.checked_add(1),1_i64.checked_div(0),i64::MIN.checked_div(-1)] {
        let result=tsr_checked(checked,|| {
            factories.set(factories.get()+1);
            "runtime-failure".to_owned()
        });
        let completion:TsrCompletion<i64,(),String>=tsr_from_result(result);
        let caught:TsrCompletion<i64,(),String>=tsr_catch(completion,|error| {
            assert_eq!(error,"runtime-failure");
            TsrCompletion::Normal(23)
        });
        assert!(matches!(caught,TsrCompletion::Normal(23)));
    }
    assert_eq!(factories.get(),3);
    let success=tsr_checked(20_i64.checked_add(3),|| {
        factories.set(factories.get()+1);
        "unexpected".to_owned()
    });
    assert_eq!(success,Ok(23));
    assert_eq!(factories.get(),3);
    for continuing in [false,true] {
        let target=TsrControlTarget::Loop(17);
        let pending:TsrCompletion<(),i64,String>=if continuing {TsrCompletion::Continue(target)}else{TsrCompletion::Break(target)};
        let kept=tsr_finally(pending,||TsrCompletion::Normal(()));
        let kept:TsrCompletion<(),i64,bool>=tsr_catch(kept,|_| {
            calls.set(calls.get()+1);
            TsrCompletion::Throw(false)
        });
        assert_eq!(calls.get(),0);
        assert!(if continuing {matches!(kept,TsrCompletion::Continue(actual) if actual==target)}else{matches!(kept,TsrCompletion::Break(actual) if actual==target)});
    }
    let block:TsrCompletion<(),i64,String>=TsrCompletion::Break(TsrControlTarget::Block(21));
    let block:TsrCompletion<(),i64,bool>=tsr_catch(block,|_|TsrCompletion::Throw(false));
    assert!(matches!(block,TsrCompletion::Break(TsrControlTarget::Block(21))));
    let pending:TsrCompletion<i64,i64,String>=TsrCompletion::Normal(4);
    assert!(matches!(tsr_finally(pending,||TsrCompletion::Normal(())),TsrCompletion::Normal(4)));
    let pending:TsrCompletion<(),i64,String>=TsrCompletion::Throw("old".to_owned());
    assert!(matches!(tsr_finally(pending,||TsrCompletion::Return(9)),TsrCompletion::Return(9)));
}
"#,
    )).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2021").arg(&source).arg("-o").arg(&binary),30));
    support::success(&support::run(&mut std::process::Command::new(binary),10));
}
