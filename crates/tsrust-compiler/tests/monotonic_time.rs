mod support;

#[test]
fn monotonic_clock_is_finite_ordered_and_callable_through_an_alias() {
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip("import {monotonicNow as now} from 'std:time';import {strictEqual} from 'std:testing';function main(){const clock=now;let previous:number=clock();strictEqual(Number.isFinite(previous),true);strictEqual(previous>=0,true);for(let i=0;i<1000;i++){const current=now();strictEqual(Number.isFinite(current),true);strictEqual(current>=previous,true);previous=current;}const elapsed=clock()-previous;strictEqual(elapsed>=0,true);}");
    let entry=directory.source(&source);
    for release in [false,true] {
        let mut command=support::cli();command.arg("run").arg(&entry);if release{command.arg("--release");}
        support::success(&support::run(&mut command,60));
    }
}

#[test]
fn monotonic_clock_rejects_arguments_and_integer_results() {
    let directory=support::TempDirectory::new();
    for body in ["monotonicNow(1);","const value:int=monotonicNow();"] {
        let source=support::format_round_trip(&format!("import {{monotonicNow}} from 'std:time';function main(){{{body}}}"));
        let entry=directory.source(&source);
        let result=support::run(support::cli().arg("check").arg(entry),20);
        assert!(!result.status.success(),"accepted {body}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("error["));
    }
}

#[test]
fn unused_clock_import_does_not_emit_runtime_support() {
    let directory=support::TempDirectory::new();
    let entry=directory.source(&support::format_round_trip("import {monotonicNow} from 'std:time';function main(){}"));
    let output=support::run(support::cli().arg("emit").arg(entry),20);support::success(&output);
    assert!(!String::from_utf8_lossy(&output.stdout).contains("tsrust_std_time_monotonic_now"));
}
