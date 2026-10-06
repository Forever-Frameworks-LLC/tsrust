mod support;
use support::{execute, reject};

// Expected output was produced by Node with the "en-US" locale, which uses the
// same locale-independent Unicode mapping.
#[test]
fn string_locale_case_methods_use_the_locale_independent_mapping() {
    execute(
        r#"
function shout(text: string): string { return text.toLocaleUpperCase(); }
function main() {
    const text = "Straße ÉCOLE İstanbul ǅ ΟΔΟΣ";
    console.log(text.toLocaleLowerCase());
    console.log(text.toLocaleUpperCase());
    console.log("".toLocaleUpperCase() === "", "aBc".toLocaleUpperCase().toLocaleLowerCase(), text.toLocaleLowerCase() === text.toLowerCase(), text.toLocaleUpperCase() === text.toUpperCase());
    console.log(shout("é".repeat(2)), ["a", "b"].map((item) => item.toLocaleUpperCase()).join(""), text.length > 0);
}
"#,
        "straße école i̇stanbul ǆ οδος\nSTRASSE ÉCOLE İSTANBUL Ǆ ΟΔΟΣ\ntrue abc true true\nÉÉ AB true\n",
        None,
    );
}

#[test]
fn string_locale_case_methods_reject_locale_arguments() {
    for (source, code) in [
        ("function main() { console.log(\"a\".toLocaleUpperCase(\"tr\")); }", "unsupported"),
        ("function main() { console.log(\"a\".toLocaleLowerCase(\"en-US\")); }", "unsupported"),
        ("function main() { const value: int = \"a\".toLocaleLowerCase(); }", "type"),
    ] {
        reject(source, code);
    }
}
