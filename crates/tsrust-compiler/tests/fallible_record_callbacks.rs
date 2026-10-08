mod support;

fn native(source: &str) {
    let source = support::format_round_trip(source);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn callback_fields_of_record_parameters_propagate_typed_failures() {
    let source = include_str!("fixtures/fallible-record-callbacks/values.tsr");
    assert_eq!(tsrust_compiler::format(source).unwrap(), source);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn runtime_failures_of_field_callbacks_reach_callers_and_callee_handlers() {
    // An array index outside its bounds is a runtime error natively.
    native(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function call(holder: ReadOnly<Holder>): int { return holder.run(); }
        function guarded(holder: ReadOnly<Holder>): string {
            try { holder.run(); return "ok"; } catch (error) { return error.name; }
        }
        function outer(holder: ReadOnly<Holder>): string {
            try { return String(call(holder) + holder.run()); } catch (error) { return error.message.length > 0 ? error.name : ""; }
        }
        function main() {
            const values = [1];
            let index = 3;
            const holder: Holder = { run: (): int => values[index] };
            strictEqual(guarded(holder), "RangeError");
            strictEqual(outer(holder), "RangeError");
            let seen = "";
            try { call(holder); } catch (error) { seen = error.name; }
            strictEqual(seen, "RangeError");
        }
    "#);
}

#[test]
fn a_handler_declared_before_its_records_is_checked_with_their_failures() {
    // The function with the handler comes first in the file; the records that
    // decide its error type are constructed later, in two functions.
    native(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function guarded(holder: ReadOnly<Holder>): int {
            try { return holder.run(); } catch (error) { return typeof error === "string" ? error.length : typeof error === "number" ? error : 0; }
        }
        function first(): int { const holder: Holder = { run: (): int => { throw "four"; } }; return guarded(holder); }
        function second(): int { const holder: Holder = { run: (): int => { throw 9; } }; return guarded(holder); }
        function main() { strictEqual(first(), 4); strictEqual(second(), 9); }
    "#);
}

#[test]
fn a_field_callback_without_any_constructed_record_has_no_known_failures() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function use(holder:ReadOnly<Holder>):int{try{return holder.run();}catch(error){console.log(error.name);return 0;}}function main(){}";
    let error = tsrust_compiler::check(source).err().expect("no record supplies the callback");
    assert_eq!(error.code, "error-effect", "{}", error.message);
    assert!(error.message.contains("construct a record"), "{}", error.message);
    // A wrong use of the caught value is still a type error.
    let source = "type Holder={run:ReadOnly<Function<int>>;};function use(holder:ReadOnly<Holder>):int{try{return holder.run();}catch(error){return error.length;}}function main(){const holder:Holder={run:():int=>{throw 7;}};use(holder);}";
    let error = tsrust_compiler::check(source).err().expect("number has no length");
    assert_eq!(error.code, "type", "{}", error.message);
}
