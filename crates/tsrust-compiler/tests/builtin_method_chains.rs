mod support;

#[test]
fn builtin_method_chains_check_and_emit_on_shared_work_stacks() {
    for (seed, link, tail, expected) in [
        ("' a '", ".trim().toUpperCase()", "", "A\n"),
        ("'abc'", ".slice(1)", ".length", "0\n"),
        ("[1, 2, 3]", ".slice()", ".length", "3\n"),
        ("7", ".toString()", "", "7\n"),
        ("'x'", ".padStart(1)", "", "x\n"),
        ("['a', 'b']", ".join('').split('')", ".length", "2\n"),
    ] {
        let source = format!("function main(){{console.log({seed}{}{tail});}}", link.repeat(24));
        support::execute(&source, expected, None);
    }
}

#[test]
fn rejected_chain_links_report_their_own_arity_and_receiver_errors() {
    support::reject(&format!("function main(){{console.log(' a '{}.trim(1));}}", ".trim()".repeat(40)), "arity");
    support::reject(&format!("function main(){{console.log(7{}.join(''));}}", ".toString()".repeat(40)), "type");
    support::reject("function main(){console.log(missing().trim().trim());}", "name");
}
