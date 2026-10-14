//! Catalog excerpts compare canonical layout; native output checks stay separate.
pub mod session;
pub fn contains_formatted_snippet(fixture: &str, snippet: &str) -> bool {
    fn canonical(source: &str) -> String {
        tsrust_compiler::format(source)
            .expect("Catalog fixture and excerpt must be lexable")
            .lines()
            .map(str::trim)
            .collect::<String>()
    }
    canonical(fixture).contains(&canonical(snippet))
}

#[test]
fn canonical_excerpt_matching_preserves_literal_spaces_and_statement_content() {
    let fixture="function main(){\n    const value = 1;\n    console.log(\"a b\");\n}";
    assert!(contains_formatted_snippet(fixture,"const value=1;console.log(\"a b\");"));
    assert!(!contains_formatted_snippet(fixture,"const value=1;console.log(\"ab\");"));
    assert!(!contains_formatted_snippet(fixture,"const value=2;console.log(\"a b\");"));
}
