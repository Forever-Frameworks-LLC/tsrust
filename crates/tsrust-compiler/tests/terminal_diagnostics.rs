mod support;
use tsrust_compiler::{Diagnostic, Span};
#[test]
fn excerpts_preserve_plain_mode_and_color_related_locations() {
    let source = "\tconst text=\"😀\"; const bad=true;\n";
    let start = source.find("true").unwrap();
    let error = Diagnostic::new("type", "Expected int.", Span::new(start, start + 4))
        .related(Span::new(1, 6), "Declared here.");
    let plain = error.render("example.tsr", source);
    assert!(!plain.contains('\x1b'));
    assert!(plain.contains("1 |     const text"));
    assert!(plain.contains("^^^^ Invalid use here."));
    let colored = error.render_terminal("example.tsr", source, true);
    assert!(colored.contains("\x1b[1;31m"));
    assert!(colored.contains("\x1b[1;36m"));
    assert!(colored.contains("Declared here."));
}
#[test]
fn cli_color_policy_keeps_json_and_redirected_output_clean() {
    let dir = support::TempDirectory::new();
    let path = dir.source("function main(){const x:int=true;}");
    let plain = support::run(support::cli().arg("check").arg(&path), 10);
    assert_eq!(plain.status.code(), Some(1));
    assert!(!plain.stderr.contains(&27));
    let color = support::run(
        support::cli().arg("check").arg(&path).arg("--color=always"),
        10,
    );
    assert!(color.stderr.contains(&27));
    let disabled = support::run(
        support::cli().arg("check").arg(&path).env("NO_COLOR", "1"),
        10,
    );
    assert!(!disabled.stderr.contains(&27));
    let json = support::run(
        support::cli()
            .arg("check")
            .arg(&path)
            .args(["--json", "--color", "always"]),
        10,
    );
    assert!(!json.stdout.contains(&27));
    assert!(json.stderr.is_empty());
}
