mod support;

#[test]
fn json_parse_without_limits_is_plain_serde_and_keeps_integers_exact() {
    let source = support::format_round_trip(r#"function code(text: string): string {
  try {
    JSON.parse(text);
    return "ok";
  } catch (error) {
    return error.code;
  }
}

function limited(text: string): string {
  try {
    JSON.parse(text, { maxBytes: 100, maxDepth: 1, maxEntries: 10, maxNodes: 10 });
    return "ok";
  } catch (error) {
    return error.code;
  }
}

function main() {
  console.log(JSON.stringify(JSON.parse("[1, 9223372036854775807, 2.5, {\"a\": [true, null]}]")));
  console.log(code("18446744073709551616"), code("[1, -9223372036854775809]"), code("{\"n\": 1.5, \"m\": 18446744073709551616}"));
  console.log(code("[1,"), code("1 x"), code("{oops}"), code("18446744073709551615"));
  console.log(code("[[1]]"), limited("[[1]]"), limited("[1]"));
}
"#);
    let rust = tsrust_compiler::check(&source).unwrap().emit_rust();
    assert!(rust.contains("tsrust_native_json::parse_plain("), "JSON.parse without limits uses plain serde_json");
    // JSON programs link serde, so run them through the CLI like other JSON tests.
    let directory = support::TempDirectory::new();
    let path = directory.0.join("main.tsr");
    std::fs::write(&path, &source).unwrap();
    let output = support::run(support::cli().arg("run").arg(&path), 90);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "[1,9223372036854775807,2.5,{\"a\":[true,null]}]\ninteger-range integer-range integer-range\nunexpected-end trailing-data invalid-json ok\nok depth-limit ok\n");
}

#[test]
fn diagnostics_outside_the_user_file_render_without_panicking() {
    let error = tsrust_compiler::Diagnostic::new("type", "generated", tsrust_compiler::Span::new(400, 418));
    let text = error.render("main.tsr", "function main() {}\n");
    assert!(text.contains("generated"));
}
