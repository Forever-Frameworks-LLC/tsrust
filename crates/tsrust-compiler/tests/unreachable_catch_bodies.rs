mod support;

fn rejected(source: &str, code: &str) {
    let error = tsrust_compiler::check(source).err().unwrap_or_else(|| panic!("must be rejected: {source}"));
    assert_eq!(error.code, code, "{source}: {}", error.message);
}

#[test]
fn handler_statements_are_checked_when_the_try_body_cannot_fail() {
    for (source, code) in [
        ("function main(){try{const x=1;}catch(error){const bad:int=\"s\";}}", "type"),
        ("function main(){try{const x=1;}catch(error){missing(1);}}", "name"),
        ("function main(){try{const x=1;}catch{const bad:int=\"s\";}}", "type"),
        ("function main(){try{const x=1;}catch(error){console.log(error.absent);}}", "type"),
        // Inside a nested region and a generic function.
        ("function main(){try{try{const x=1;}catch(inner){const bad:boolean=1;}}finally{console.log(1);}}", "type"),
        ("function id<T>(value:T):T{try{const x=1;}catch(error){const bad:int=\"s\";}return value;}function main(){id(1);}", "type"),
    ] {
        rejected(source, code);
    }
}

#[test]
fn a_valid_unreachable_handler_compiles_and_never_runs() {
    let source = support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        function id<T>(value: T): T {
            try { const unused = 1; } catch (error) { console.log(error.message); }
            return value;
        }
        function main() {
            let caught = 0;
            let total = 0;
            try { total = 2; } catch (error) {
                caught = 1;
                console.log(error.name, error.message);
                const read = (): int => caught;
                read();
            } finally { caught += 2; }
            strictEqual(caught + total, 4);
            strictEqual(id(1), 1);
            strictEqual(id("a"), "a");
        }
    "#);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn a_handler_that_a_failure_reaches_keeps_its_real_error_type() {
    // The thrown value is a string here; the handler must not be checked as if
    // the name were a runtime error.
    let source = support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        function risky(flag: boolean): int { if (flag) { throw "big"; } return 1; }
        function main() {
            let seen = "";
            try { risky(true); } catch (error) { strictEqual(error.length, 3); seen = error; }
            strictEqual(seen, "big");
            const values = [1];
            let index = 4;
            let named = "";
            try { console.log(values[index]); } catch (error) { strictEqual(error, error); named = error.name; }
            strictEqual(named, "RangeError");
        }
    "#);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
