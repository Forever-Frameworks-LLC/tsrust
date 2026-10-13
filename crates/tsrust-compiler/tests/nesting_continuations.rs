mod support;

fn nested(value: &str, result: &str, depth: usize) -> String {
    let mut value = value.to_owned();
    for _ in 0..depth {
        value = format!("((): {result} => {{ return {value}; }})()");
    }
    value
}

fn argument_chain(callee: &str, depth: usize) -> String {
    let mut value = "7".to_owned();
    for _ in 0..depth { value = format!("{callee}({value})"); }
    value
}

#[test]
fn queued_arguments_cover_declared_bound_inline_and_stored_calls() {
    let declared = argument_chain("identity", 64);
    let bound = argument_chain("action", 64);
    let inline = argument_chain("((value: int): int => { return value; })", 64);
    let stored = argument_chain("holder.run", 64);
    let source = format!(r#"
        type Holder = {{ run: ReadOnly<Function<int, [value: int]>>; }};
        function identity(value: int): int {{ return value; }}
        function main() {{
            const action = (value: int): int => {{ return value; }};
            const holder: Holder = {{ run: (value: int): int => {{ return value; }} }};
            console.log({declared});
            console.log({bound});
            console.log({inline});
            console.log({stored});
        }}
    "#);
    support::execute(&source, "7\n7\n7\n7\n", None);
}

#[test]
fn queued_closure_bodies_preserve_owned_and_mutable_captures() {
    // This samples the original normal-stack crash, not a supported-depth limit.
    let owned = nested("text", "string", 16);
    let mutated = nested("count = count + 1", "int", 16);
    let source = format!(
        "function main() {{ {{ const text = \"payload\"; console.log({owned}); }}
        {{ let count = 0; console.log({mutated}); console.log(count); }}
        const action = (value: int = 9): int => {{ return value; }};
        console.log(action()); }}"
    );
    support::execute(&source, "payload\n1\n1\n9\n", None);
}

#[test]
fn queued_capture_transfer_keeps_diagnostic_location_and_parent_ownership() {
    let owned = nested("text", "string", 16);
    let source = support::format_round_trip(&format!(
        "function main() {{ const text = \"payload\"; console.log({owned}); console.log(text); }}"
    ));
    let error = tsrust_compiler::check(&source).err().expect("capture must transfer the owner");
    assert_eq!(error.code, "transferred");
    assert_eq!(&source[error.span.start..error.span.end], "text");
    assert_eq!(error.span.start, source.rfind("text").unwrap());
}

#[test]
fn queued_callee_completion_preserves_argument_order_and_contextual_literals() {
    let source = r#"
        type Tag = "ready" | "done";
        function mark(value: int): int { console.log(value); return value; }
        function main() {
            let count = 0;
            const tag: Tag = "ready";
            console.log(((): (left: int, right: int) => int => {
                console.log("callee");
                return (left: int, right: int): int => { return left * 10 + right; };
            })()(mark(count = count + 1), mark(count = count + 1)));
            console.log(count);
            console.log(tag);
        }
    "#;
    support::execute(source, "callee\n1\n2\n12\n2\nready\n", None);
}
