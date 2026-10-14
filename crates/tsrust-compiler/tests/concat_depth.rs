mod support;
use tsrust_compiler::check;

fn nested(receiver: &str, leaf: &str, depth: usize) -> String {
    format!(
        "function main(){{const x={}{leaf}{};}}",
        format!("{receiver}.concat(").repeat(depth),
        ")".repeat(depth)
    )
}

#[test]
fn nested_string_and_array_concat_keep_normal_stack_and_depth_diagnostics() {
    for (receiver, leaf) in [("\"x\"", "String(7)"), ("[1]", "[2]")] {
        let source = nested(receiver, leaf, 100).replace(";}", ";console.log(x.length);}");
        let source = support::format_round_trip(&source);
        eprintln!("concat phase: check {receiver} depth 100");
        let analysis = check(&source)
            .unwrap_or_else(|error| panic!("{}", error.render("concat-depth.tsr", &source)));
        eprintln!("concat phase: emit {receiver} depth 100");
        let emitted = analysis.emit_rust();
        if std::env::var_os("TSR_SKIP_CONCAT_DEPTH_NATIVE").is_none() {
            let directory = support::TempDirectory::new();
            let rust = directory.0.join("concat-depth.rs");
            let executable = directory.0.join("concat-depth");
            std::fs::write(&rust, emitted).unwrap();
            support::success(&support::run(
                support::rustc()
                    .args(["--edition=2024", "-C", "opt-level=0"])
                    .arg(&rust)
                    .arg("-o")
                    .arg(&executable),
                60,
            ));
            let output = support::run(&mut std::process::Command::new(&executable), 10);
            support::success(&output);
            assert_eq!(output.stdout, b"101\n");
            assert!(output.stderr.is_empty());
        }
        let source = nested(receiver, leaf, 200);
        eprintln!("concat phase: check {receiver} depth 200");
        let error = check(&source).err().expect("expected diagnostic");
        assert_eq!(error.code, "unsupported");
        assert!(
            error.message.contains("nesting") && error.message.contains("128"),
            "{}",
            error.message
        );
    }
    for source in [
        "function main(){const x:int[]=[].concat(1,[2]);console.log(x.join(\"|\"));}",
        "function main(){const x:number[]=[].concat(1, [2.5]);console.log(x.join(\"|\"));}",
        "function main(){const x=[[1]].concat([[2]]);console.log(x.join(\"|\"));}",
        "function main(){const x=\"é\".concat(String(7),\"😀\");console.log(x);}",
    ] {
        assert!(!check(source).unwrap().emit_rust().is_empty());
    }
}

#[test]
fn nested_concat_retains_outer_receiver_and_prior_argument_loans() {
    for source in [
        "function main(){let text='a';function change():string{text='b';return 'c';}const result=text.concat('x'.concat(change()));}",
        "function main(){const values=[1];function change():int[]{values.push(2);return [3];}const result=values.concat([4].concat(change()));}",
        "function main(){const values=[1];function change():int[]{values.push(2);return [3];}const result=[0].concat(values,[4].concat(change()));}",
        "function main(){let text='a';function change():string{text='b';return 'c';}const result='prefix'.concat(text,'x'.concat(change()));}",
    ] {
        let formatted=support::format_round_trip(source);
        let error=check(&formatted).err().expect("overlapping mutation must reject");
        assert_eq!(error.code,"conflicting-access","{}",error.render("concat-loans.tsr",&formatted));
        assert!(!error.related.is_empty());
    }
}

#[test]
fn nested_concat_evaluates_once_in_source_order_and_releases_loans() {
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip("import {strictEqual} from 'std:testing';let order=0;function part(digit:int):int[]{order=order*10+digit;return [digit];}function text(digit:int):string{order=order*10+digit;return String(digit);}function main(){const values=part(1).concat(part(2).concat(part(3)),part(4));strictEqual(order,1234);strictEqual(values.join('|'),'1|2|3|4');order=0;const joined=text(1).concat(text(2).concat(text(3)),text(4));strictEqual(order,1234);strictEqual(joined,'1234');values.push(5);strictEqual(values.length,5);}");
    let path=directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path),90));
}
