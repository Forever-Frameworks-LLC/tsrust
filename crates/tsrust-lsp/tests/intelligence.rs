use lsp_types::Position;
use serde_json::{Value, json};
use tsrust_lsp::intelligence::SourceIndex;
fn at(source: &str, needle: &str, n: usize) -> Position {
    let offset = source.match_indices(needle).nth(n).unwrap().0;
    let prefix = &source[..offset];
    Position::new(
        prefix.bytes().filter(|&c| c == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
}
fn ask(source: &str, method: &str, needle: &str, n: usize) -> Value {
    SourceIndex::new(source)
        .request(
            method,
            &json!({"position":at(source,needle,n),"context":{"includeDeclaration":true}}),
            "file:///test.tsr",
        )
        .unwrap()
}
#[test]
fn documented_function_hover_signature_definition_and_outline() {
    let source = "/** Add two values.\n * @param left - First value.\n * @param right - Second value.\n * @returns Their sum.\n */\nfunction sum(left: int, right: int): int { return left + right; }\nfunction main() { const value = sum(1, 2); }";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "sum(", 1);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("function sum(left: int, right: int): int")
    );
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Add two values.")
    );
    let signature = ask(source, "textDocument/signatureHelp", "2);", 0);
    assert_eq!(signature["activeParameter"], 1);
    assert_eq!(
        signature["signatures"][0]["parameters"][0]["documentation"]["value"],
        "First value."
    );
    assert_eq!(
        ask(source, "textDocument/definition", "sum(", 1)["range"]["start"],
        json!(at(source, "sum(", 0))
    );
    let symbols = SourceIndex::new(source)
        .request(
            "textDocument/documentSymbol",
            &json!({}),
            "file:///test.tsr",
        )
        .unwrap();
    assert_eq!(symbols.as_array().unwrap().len(), 2);
}
#[test]
fn shadowing_and_comments_do_not_create_false_references() {
    let source = "function main() { const text: string = \"outer\"; { const text: string = \"inner\"; console.log(text); } console.log(text); /* text */ }";
    // The source index remains useful even where current strict checking rejects shadowing.
    let inside = ask(source, "textDocument/hover", "text);", 0);
    assert!(
        inside["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("const text: string")
    );
    let inner = ask(source, "textDocument/definition", "text);", 0);
    let outer = ask(source, "textDocument/definition", "text);", 1);
    assert_ne!(inner, outer);
    let refs = ask(source, "textDocument/references", "text);", 0);
    assert_eq!(refs.as_array().unwrap().len(), 2);
    assert!(ask(source, "textDocument/hover", "text */", 0).is_null());
}
#[test]
fn incomplete_members_receiver_types_and_unsupported_methods() {
    let source = "type User = { name: string; age: int; }; function main() { const user: User = { name: \"😀\", age: 2 }; user.na";
    // Position at the end of the typed prefix, not its beginning.
    let mut pos = at(source, "user.na", 0);
    pos.character += 7;
    let members = SourceIndex::new(source)
        .request(
            "textDocument/completion",
            &json!({"position":pos}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        members
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["label"] == "name")
    );
    let _ = members;
    let array = "function main() { const values: int[] = [1,2]; values.";
    let mut pos = at(array, "values.", 0);
    pos.character += 7;
    let items = SourceIndex::new(array)
        .request(
            "textDocument/completion",
            &json!({"position":pos}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["label"] == "map")
    );
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["label"] == "flatMap")
    );
    assert!(items.as_array().unwrap().iter().any(|m| m["label"] == "sort"));
    assert!(items.as_array().unwrap().iter().any(|m| m["label"] == "at"));
}
#[test]
fn forward_function_completion_docs_and_literal_member_hover() {
    for (method, arguments, signature, parameter) in [
        (
            "substring",
            "0, 2",
            "substring(start: int, end?: int): string",
            1,
        ),
        (
            "concat",
            "\"!\", \"?\"",
            "concat(...values: string[]): string",
            0,
        ),
        (
            "padStart",
            "8, \"é\"",
            "padStart(target: int, filler?: string): string",
            1,
        ),
        (
            "padEnd",
            "8, \"é\"",
            "padEnd(target: int, filler?: string): string",
            1,
        ),
    ] {
        let source = format!(
            "function main() {{ const word: string = \"hi\"; const result = word.{method}({arguments}); }}"
        );
        assert!(tsrust_compiler::check(&source).is_ok());
        assert!(
            ask(&source, "textDocument/hover", method, 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(signature)
        );
        let help = ask(&source, "textDocument/signatureHelp", ");", 0);
        assert_eq!(help["signatures"][0]["label"], signature);
        assert_eq!(help["activeParameter"], parameter);
        let incomplete = format!("function main() {{ const word: string = \"hi\"; word.{method}");
        let mut position = at(&incomplete, &format!("word.{method}"), 0);
        position.character += 5 + method.len() as u32;
        let items = SourceIndex::new(&incomplete)
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == method && item["detail"] == signature)
        );
    }
    let source = "// Reads a name.\nfunction nameOf(value: string): string { return value; }\nfunction main() { const x = nameOf(\"😀\"); x.trim(); na }";
    let mut pos = at(source, "na }", 0);
    pos.character += 2;
    let items = SourceIndex::new(source)
        .request(
            "textDocument/completion",
            &json!({"position":pos}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["label"] == "nameOf" && m["documentation"]["value"] == "Reads a name.")
    );
    assert!(
        ask(source, "textDocument/hover", "trim", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("trim(): string")
    );
    assert!(
        SourceIndex::new(".x")
            .request(
                "textDocument/hover",
                &json!({"position":{"line":0,"character":1}}),
                "file:///test.tsr"
            )
            .unwrap()
            .is_null()
    );
}
#[test]
fn nested_call_commas_and_utf16_positions() {
    let source = "function add(a: int, b: int): int { return a + b; } function main() { const emoji = \"😀\"; add(add(1, 2), 3); }";
    let help = ask(source, "textDocument/signatureHelp", "3);", 0);
    assert_eq!(help["activeParameter"], 1);
    let help = ask(source, "textDocument/signatureHelp", "2),", 0);
    assert_eq!(help["activeParameter"], 1);
    let source = "function main() { const word: string = \"a,b\"; word.replace(\"a,b\", \"c\"); }";
    assert_eq!(
        ask(source, "textDocument/signatureHelp", "\"c\"", 0)["activeParameter"],
        1
    );
}

#[test]
fn every_catalog_entry_has_a_current_compiler_witness() {
    for (ty, literal, method, result) in [
        ("string", "\"x\"", "valueOf", "string"),
        ("string", "\"x\"", "toString", "string"),
        ("boolean", "true", "valueOf", "boolean"),
        ("boolean", "false", "toString", "string"),
        ("number", "1.0", "valueOf", "number"),
        ("int", "7", "valueOf", "int"),
    ] {
        let source = format!(
            "function main(){{const value:{ty}={literal};const result=value.{method}();result.valueOf();}}"
        );
        assert!(tsrust_compiler::check(&source).is_ok());
        let hover = ask(&source, "textDocument/hover", &format!("{method}("), 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("{method}(): {result}"))
        );
        let help = ask(&source, "textDocument/signatureHelp", ");", 0);
        assert!(
            help["signatures"][0]["parameters"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let result_hover = ask(&source, "textDocument/hover", "result.valueOf", 0);
        assert!(
            result_hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("result: {result}"))
        );
        let incomplete = format!("function main(){{const value:{ty}={literal};value.");
        let mut position = at(&incomplete, "value.", 0);
        position.character += 6;
        let items = SourceIndex::new(&incomplete)
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == method)
        );
    }
    for (expression, result) in [
        ("\"x\".valueOf()", "string"),
        ("(true)", "string"),
        ("(false).valueOf()", "string"),
    ] {
        let source =
            format!("function main(){{const text={expression}.toString();text.valueOf();}}");
        assert!(tsrust_compiler::check(&source).is_ok());
        assert!(
            ask(&source, "textDocument/hover", "toString", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("toString(): string")
        );
        assert!(
            ask(&source, "textDocument/signatureHelp", ");text", 0)["signatures"][0]["parameters"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            ask(&source, "textDocument/hover", "text.valueOf", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("text: {result}"))
        );
        let source = format!("function main(){{{expression}.");
        let mut position = at(&source, &format!("{expression}."), 0);
        position.character += format!("{expression}.").encode_utf16().count() as u32;
        let items = SourceIndex::new(&source)
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "toString")
        );
    }
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    for (family, entries) in catalog.as_object().unwrap() {
        for entry in entries.as_array().unwrap() {
            let method = entry["name"].as_str().unwrap();
            if family.starts_with("std:") {
                // A native binding fixes its parameter count; a source module
                // such as std:testing is witnessed by its examples below.
                if let Ok(binding) = tsrust_compiler::native_bindings::lookup(family, method) {
                    assert_eq!(
                        binding.parameters.len(),
                        entry["parameters"].as_array().unwrap().len()
                    );
                }
                for example in entry["examples"].as_array().unwrap() {
                    checked_std_example(example["source"].as_str().unwrap());
                }
                continue;
            }
            // Matcher families of std:testing are reached through expect();
            // their complete-program examples are the compiler witnesses.
            if matches!(family.as_str(), "Expectation" | "NegatedExpectation") {
                let examples = entry["examples"].as_array().unwrap();
                assert!(!examples.is_empty(), "{family}.{method} needs a witness example");
                for example in examples {
                    checked_std_example(example["source"].as_str().unwrap());
                }
                continue;
            }
            if family == "globals" {
                // Built-in function and access types are witnessed in type position.
                let type_witness = match method {
                    "Function" => Some("Function<int, [int]>"),
                    "Mutable" => Some("Mutable<Function<int, [int]>>"),
                    "ReadOnly" => Some("ReadOnly<Function<int, [ReadOnly<string>]>>"),
                    "Once" => Some("Once<Function<int, [int]>>"),
                    "Owned" => Some("ReadOnly<Function<int, [Owned<string>]>>"),
                    _ => None,
                };
                if let Some(annotation) = type_witness {
                    assert!(
                        tsrust_compiler::check(&format!(
                            "function use(action:{annotation}):int{{return 1;}}function main(){{}}"
                        ))
                        .is_ok(),
                        "{method}"
                    );
                    continue;
                }
                let expression = match method {
                    "null" => "null",
                    "undefined" => "undefined",
                    "typeof" => "typeof null",
                    "void" => "void 0",
                    "String" => "String(null)",
                    "Boolean" => "Boolean(undefined)",
                    "Array" => "Array.isArray([])",
                    "Object" => "Object.is(1, 1)",
                    "parseInt" => "parseInt(\"4\", 10)",
                    "parseFloat" => "parseFloat(\"4.5\")",
                    "isNaN" => "isNaN(1.5)",
                    "isFinite" => "isFinite(1.5)",
                    "NaN" => "NaN",
                    "Infinity" => "Infinity",
                    _ => panic!("unverified global {method}"),
                };
                assert!(
                    tsrust_compiler::check(&format!(
                        "function main(){{const result={expression};console.log(result);}}"
                    ))
                    .is_ok()
                );
                continue;
            }

            if family == "StringConstructor" && method == "raw" {
                // The template record type must be declared outside main.
                let source = "type Template = { raw: string[]; };function main() { const template: Template = { raw: [\"a\"] }; const result: string = String.raw(template); }";
                if let Err(error) = tsrust_compiler::check(source) {
                    panic!("Catalog drift for String.raw: {}", error.message);
                }
                continue;
            }
            let (setup, receiver, args, result) = match family.as_str() {
                "string" => (
                    "const value: string = \"hello\";",
                    "value",
                    match method {
                        "slice" | "substring" => "0, 2",
                        "charAt" | "at" | "codePointAt" | "charCodeAt" => "0",
                        "repeat" => "2",
                        "concat" => "\"!\", \"?\"",
                        "padStart" | "padEnd" => "8, \"é\"",
                        "replace" | "replaceAll" => "\"l\", \"x\"",
                        "split" => "\"l\", 2",
                        "anchor" | "fontcolor" | "fontsize" | "link" => "\"target\"",
                        "includes" | "startsWith" | "endsWith" => "\"l\"",
                        "indexOf" | "lastIndexOf" => "\"l\", 0",
                        _ => "",
                    },
                    if method == "split" {
                        "string[]"
                    } else if method == "at" {
                        "string | undefined"
                    } else if method == "codePointAt" {
                        "int | undefined"
                    } else if method == "charCodeAt" {
                        "number"
                    } else if method == "length"
                        || method.ends_with("IndexOf")
                        || method == "indexOf"
                        || method == "lastIndexOf"
                    {
                        "int"
                    } else if matches!(
                        method,
                        "includes" | "startsWith" | "endsWith" | "isWellFormed"
                    ) {
                        "boolean"
                    } else {
                        "string"
                    },
                ),
                "Array" => (
                    "let value: int[] = [1, 2, 3];",
                    "value",
                    match method {
                        "push" | "unshift" => "4",
                        "remove" | "at" => "0",
                        "includes" | "indexOf" | "lastIndexOf" => "2, 0",
                        "slice" => "0, 2",
                        "splice" => "1, 1, 4",
                        "map" | "flatMap" => "(x: int): int => { return x + 1; }",
                        "filter" | "some" | "every" | "findIndex" | "findLastIndex" | "find"
                        | "findLast" => "(x: int): boolean => { return x > 0; }",
                        "forEach" => "(x: int): int => { return x; }",
                        "reduce" | "reduceRight" => {
                            "(acc: int, x: int): int => { return acc + x; }, 0"
                        }
                        "concat" => "4",
                        "fill" => "4, 0, 2",
                        "copyWithin" => "0, 1, 2",
                        "with" => "0, 4",
                        "toSpliced" => "0, 1, 4",
                        _ => "",
                    },
                    match method {
                        "length" | "remove" | "unshift" | "indexOf" | "lastIndexOf" | "findIndex"
                        | "findLastIndex" | "reduce" | "reduceRight" => "int",
                        "push" | "reverse" | "sort" | "forEach" | "fill" | "copyWithin" => "void",
                        "join" | "toString" | "toLocaleString" => "string",
                        "pop" | "shift" | "at" | "find" | "findLast" => "int | undefined",
                        "includes" | "some" | "every" => "boolean",
                        "entries" => "[int, int][]",
                        _ => "int[]",
                    },
                ),
                "Math" => (
                    "",
                    "Math",
                    if matches!(method, "atan2" | "pow" | "imul" | "min" | "max" | "hypot") {
                        "1.0, 2.0"
                    } else if method == "random" {
                        ""
                    } else {
                        "1.0"
                    },
                    "number",
                ),
                "Number" => (
                    "",
                    "Number",
                    if method == "parseFloat" {
                        "\"1.25suffix\""
                    } else if method == "parseInt" {
                        "\"ff\", 16"
                    } else {
                        "1.0"
                    },
                    if matches!(method, "parseFloat" | "parseInt") || entry["parameters"].is_null() {
                        "number"
                    } else {
                        "boolean"
                    },
                ),
                "int" => (
                    "const value:int=7;",
                    "value",
                    if method == "toString" { "16" } else { "" },
                    if matches!(method, "toString" | "toLocaleString") {
                        "string"
                    } else {
                        "int"
                    },
                ),
                "number" => (
                    "const value:number=1.0;",
                    "value",
                    "",
                    if method == "valueOf" {
                        "number"
                    } else {
                        "string"
                    },
                ),
                "boolean" => (
                    "const value:boolean=true;",
                    "value",
                    "",
                    if method == "toString" {
                        "string"
                    } else {
                        "boolean"
                    },
                ),
                "console" => ("", "console", "1", "void"),
                "ArrayConstructor" if method == "of" => ("", "Array", "1, 2", "int[]"),
                "ArrayConstructor" if method == "from" => ("", "Array", "[1, 2]", "int[]"),
                "ArrayConstructor" => ("", "Array", "[]", "boolean"),
                "Object" if method == "keys" => ("", "Object", "[1]", "string[]"),
                "Object" if method == "values" => ("", "Object", "[1]", "int[]"),
                "Object" if method == "entries" => ("const pair: [int, int] = [1, 2];", "Object", "pair", "[string, int][]"),
                "Object" => ("", "Object", "1, 1", "boolean"),
                "StringConstructor" => ("", "String", "72, 105", "string"),
                "Tuple" => ("const value: [int, string] = [2, \"cat\"];", "value", "", "int"),
                _ => panic!("unverified family {family}"),
            };
            let operation = if entry["parameters"].is_null() {
                format!("{receiver}.{method}")
            } else {
                format!("{receiver}.{method}({args})")
            };
            let statement = if result == "void" {
                format!("{operation};")
            } else {
                format!("const result: {result} = {operation};")
            };
            let source = format!("function main() {{ {setup} {statement} }}");
            if let Err(error) = tsrust_compiler::check(&source) {
                panic!(
                    "Catalog drift for {family}.{method}: {}\n{source}",
                    error.message
                );
            }
            assert!(
                entry["signature"].as_str().unwrap().ends_with(&format!(
                    ": {}",
                    if family == "Array" {
                        match result {
                            "int[]" if method == "map" => "Result[]",
                            "int[]" if matches!(method, "flat" | "flatMap") => "Flattened[]",
                            "int[]" if method == "keys" => "int[]",
                            "[int, int][]" => "[int,Element][]",
                            "int[]" => "Element[]",
                            "int" if method == "remove" => "Element",
                            "int | undefined"
                                if matches!(
                                    method,
                                    "pop" | "shift" | "at" | "find" | "findLast"
                                ) =>
                            {
                                "Element | undefined"
                            }
                            "int" if matches!(method, "reduce" | "reduceRight") => "Accumulator",
                            _ => result,
                        }
                    } else if family == "Object" && method == "values" {
                        "Value[]"
                    } else if family == "Object" && method == "entries" {
                        "[string, Value][]"
                    } else if family == "ArrayConstructor" && method == "from" {
                        "U[]"
                    } else if family == "ArrayConstructor" && method == "of" {
                        "T[]"
                    } else {
                        result
                    }
                )),
                "Catalog result drift: {family}.{method}"
            );
        }
    }
}

#[test]
fn callback_parameter_signatures_and_folding_survive_incomplete_bodies() {
    let source = "/** Calls an operation. */\nfunction apply(callback: (n: int) => int, value: int): int {\n return callback(value);\n}\nfunction main() {\n const increment = (n: int): int => { return n + 1; };\n apply(increment, 2);\n}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "callback(value)", 0);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("callback: Function<int, [n: int]>")
    );
    let help = ask(source, "textDocument/signatureHelp", "value);", 0);
    assert_eq!(help["signatures"][0]["parameters"][0]["label"], "n: int");
    let help = ask(source, "textDocument/signatureHelp", "2);", 0);
    assert_eq!(
        help["signatures"][0]["parameters"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let folds = SourceIndex::new(source)
        .request("textDocument/foldingRange", &json!({}), "file:///test.tsr")
        .unwrap();
    assert!(folds.as_array().unwrap().len() >= 2);
    let broken = "function main() { const value = 1 > 2; value.";
    let mut position = at(broken, "value.", 0);
    position.character += 6;
    let items = SourceIndex::new(broken)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        items.as_array().unwrap().is_empty(),
        "must not infer compound expression from its first literal"
    );
}

#[test]
fn loop_bindings_have_element_types_and_do_not_escape() {
    let source = "function main() { const values: string[] = [\"hello\"]; for (const item of values) { item.trim(); } it }";
    let mut position = at(source, "it }", 0);
    position.character += 2;
    let outside = SourceIndex::new(source)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        !outside
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "item")
    );
    let hover = ask(source, "textDocument/hover", "trim", 0);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("trim(): string")
    );
}

#[test]
fn integer_formatting_discovery_and_chains_keep_string_types() {
    for expression in [
        "value.toString()",
        "value.toString(16)",
        "(value).toString(2)",
        "value.valueOf().toString(36)",
    ] {
        let source = format!(
            "function main(){{const value:int=31;const text={expression};text.valueOf();}}"
        );
        assert!(tsrust_compiler::check(&source).is_ok());
        assert!(
            ask(&source, "textDocument/hover", "text.valueOf", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("text: string")
        );
        let source = format!("function main(){{const value:int=31;{expression}.valueOf();}}");
        assert!(
            ask(&source, "textDocument/hover", "valueOf();", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("valueOf(): string")
        );
    }
    let source = "function main(){const value:int=31;const radix:int=16;value.toString(radix);}";
    assert!(
        ask(source, "textDocument/hover", "toString", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("toString(radix?: int): string")
    );
    let help = ask(source, "textDocument/signatureHelp", "radix);", 0);
    assert_eq!(
        help["signatures"][0]["parameters"][0]["label"],
        "radix?: int"
    );
}

#[test]
fn number_formatting_discovery_optional_arguments_and_chains() {
    for (method, argument) in [
        ("toFixed", "fractionDigits"),
        ("toExponential", "fractionDigits"),
        ("toPrecision", "precision"),
    ] {
        for expression in [
            format!("value.{method}()"),
            format!("value.{method}(2)"),
            format!("(value).{method}(2)"),
            format!("value.valueOf().{method}(2)"),
        ] {
            let source = format!(
                "function main(){{const value:number=1.25;const text={expression};text.valueOf();}}"
            );
            assert!(tsrust_compiler::check(&source).is_ok());
            assert!(
                ask(&source, "textDocument/hover", "text.valueOf", 0)["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains("text: string")
            );
            let source =
                format!("function main(){{const value:number=1.25;{expression}.valueOf();}}");
            assert!(
                ask(&source, "textDocument/hover", "valueOf();", 0)["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains("valueOf(): string")
            );
        }
        let source = format!(
            "function main(){{const value:number=1.25;const digits:int=2;value.{method}(digits);}}"
        );
        let hover = ask(&source, "textDocument/hover", method, 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("{method}({argument}?: int): string"))
        );
        let help = ask(&source, "textDocument/signatureHelp", "digits);", 0);
        assert_eq!(
            help["signatures"][0]["parameters"][0]["label"],
            format!("{argument}?: int")
        );
        let source = format!("function main(){{const value:number=1.25;value.{method}();}}");
        assert!(
            ask(&source, "textDocument/completion", method, 0)
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == method)
        );
    }
}

#[test]
fn number_parse_float_discovery_signature_and_number_result_chains() {
    let source = "function main(){const parsed=Number.parseFloat(\"1.25tail\");parsed.valueOf();}";
    assert!(tsrust_compiler::check(source).is_ok());
    assert!(
        ask(source, "textDocument/hover", "parseFloat", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("parseFloat(text: string): number")
    );
    assert!(
        ask(source, "textDocument/hover", "parsed.valueOf", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("parsed: number")
    );
    let help = ask(source, "textDocument/signatureHelp", "\"1.25tail\"", 0);
    assert_eq!(
        help["signatures"][0]["parameters"][0]["label"],
        "text: string"
    );
    let incomplete = "function main(){Number.";
    let mut position = at(incomplete, "Number.", 0);
    position.character += 7;
    let items = SourceIndex::new(incomplete)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "parseFloat")
    );
    for expression in [
        "Number.parseFloat(\"1.25tail\")",
        "(Number.parseFloat(\"1.25tail\"))",
    ] {
        let source =
            format!("function main(){{const parsed={expression}.valueOf();parsed.toFixed(1);}}");
        assert!(tsrust_compiler::check(&source).is_ok());
        assert!(
            ask(&source, "textDocument/hover", "parsed.toFixed", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("parsed: number")
        );
        assert!(
            ask(&source, "textDocument/hover", "valueOf", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("valueOf(): number")
        );
    }
}

#[test]
fn string_well_formed_discovery_zero_arity_and_chain_results() {
    for (method, result) in [("isWellFormed", "boolean"), ("toWellFormed", "string")] {
        let source = format!(
            "function main(){{const text:string=\"é😀\";const output=text.{method}();output.valueOf();}}"
        );
        assert!(tsrust_compiler::check(&source).is_ok());
        assert!(
            ask(&source, "textDocument/hover", method, 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("{method}(): {result}"))
        );
        assert!(
            ask(&source, "textDocument/hover", "output.valueOf", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("output: {result}"))
        );
        let help = ask(&source, "textDocument/signatureHelp", ");output", 0);
        assert!(
            help["signatures"][0]["parameters"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let incomplete = "function main(){const text:string=\"é😀\";text.";
        let mut position = at(incomplete, "text.", 0);
        position.character += 5;
        let items = SourceIndex::new(incomplete)
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == method)
        );
        for receiver in ["text", "(text)", "text.toWellFormed()"] {
            let source = format!(
                "function main(){{const text:string=\"é😀\";const output={receiver}.{method}();output.valueOf();}}"
            );
            assert!(tsrust_compiler::check(&source).is_ok());
            assert!(
                ask(&source, "textDocument/hover", "output.valueOf", 0)["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("output: {result}"))
            );
            let direct = format!(
                "function main(){{const text:string=\"é😀\";{receiver}.{method}().valueOf();}}"
            );
            assert!(
                ask(&direct, "textDocument/hover", "valueOf", 0)["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("valueOf(): {result}"))
            );
        }
    }
}

#[test]
fn multiline_string_method_chain_retains_hover_and_completion() {
    let source = "type User = { name: string };\nfunction normalize(user: User): string {\nreturn user.name\n.trim()\n.toLowerCase();\n}";
    let hover = ask(source, "textDocument/hover", "toLowerCase", 0);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap_or("")
            .contains("toLowerCase(): string")
    );
    let incomplete = source.replace("toLowerCase();", "toLo");
    let completion = ask(&incomplete, "textDocument/completion", "toLo", 0);
    assert!(
        completion
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "toLowerCase")
    );
}

#[test]
fn both_quote_delimiters_escape_forms_and_string_contexts_have_editor_types() {
    for literal in [
        r#"'é😀\x41\u0042\u{1F600}\0\v\q'"#,
        r#""é😀\x41\u0042\u{1F600}\0\v\q""#,
        r#"'a\'b"c\\'"#,
        r#""a\"b'c\\""#,
    ] {
        let source = format!(
            "function main(){{const text={literal};const copy=({literal}).toWellFormed();const texts=[{literal}];const alias=clone({literal});text.valueOf();copy.valueOf();texts[0].valueOf();alias.valueOf();}}"
        );
        assert!(tsrust_compiler::check(&source).is_ok());
        for (needle, expected) in [
            ("text.valueOf", "text: string"),
            ("copy.valueOf", "copy: string"),
            ("texts[0]", "texts: string[]"),
            ("alias.valueOf", "alias: string"),
        ] {
            assert!(
                ask(&source, "textDocument/hover", needle, 0)["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains(expected)
            );
        }
        for receiver in [
            literal.to_owned(),
            format!("({literal})"),
            format!("{literal}.toWellFormed()"),
        ] {
            let direct = format!("function main(){{{receiver}.valueOf();}}");
            assert!(
                ask(&direct, "textDocument/hover", "valueOf", 0)["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains("valueOf(): string")
            );
            let incomplete = format!("function main(){{{receiver}.");
            let last_dot = incomplete.match_indices('.').count() - 1;
            let mut position = at(&incomplete, ".", last_dot);
            position.character += 1;
            let items = SourceIndex::new(&incomplete)
                .request(
                    "textDocument/completion",
                    &json!({"position":position}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert!(
                items
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|item| item["label"] == "trim")
            );
        }
    }
}

#[test]
fn quoted_syntax_and_comment_text_do_not_leak_symbols_or_signature_commas() {
    for quote in ['\'', '"'] {
        let literal = format!(
            "{quote}// /* {{ const phantom:int=7; }} function fake() {{}} (a,b) \\{quote} é😀{quote}"
        );
        let source = format!(
            "function pair(text:string,value:int):int{{return value;}}function main(){{const text={literal};pair({literal}, 7);text.valueOf();}}"
        );
        assert!(tsrust_compiler::check(&source).is_ok());
        for method in [
            "textDocument/hover",
            "textDocument/definition",
            "textDocument/references",
        ] {
            assert!(ask(&source, method, "phantom", 0).is_null());
        }
        assert_eq!(
            ask(&source, "textDocument/completion", "phantom", 0),
            json!([])
        );
        let help = ask(&source, "textDocument/signatureHelp", "phantom", 1);
        assert_eq!(
            help["signatures"][0]["label"],
            "function pair(text:string,value:int):int"
        );
        assert_eq!(help["activeParameter"], 0);
        assert_eq!(
            ask(&source, "textDocument/signatureHelp", "7);text", 0)["activeParameter"],
            1
        );
        let outline = SourceIndex::new(&source)
            .request(
                "textDocument/documentSymbol",
                &json!({}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(outline.as_array().unwrap().len(), 2);
        assert!(
            outline
                .as_array()
                .unwrap()
                .iter()
                .all(|symbol| symbol["name"] != "fake")
        );
        assert!(
            ask(&source, "textDocument/hover", "text.valueOf", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("text: string")
        );
    }
}

#[test]
fn string_continuations_and_unfinished_literals_keep_editor_token_boundaries() {
    for quote in ['\'', '"'] {
        for continuation in ["\n", "\r\n", "\r", "\u{2028}", "\u{2029}"] {
            let literal =
                format!("{quote}A\\{continuation}é\\{quote} // /* {{scope}} (x,y) B{quote}");
            let source = format!(
                "function pair(text:string,value:int):int{{return value;}}function main(){{const text={literal};pair({literal}, 7);text.valueOf();}}"
            );
            assert!(tsrust_compiler::check(&source).is_ok());
            assert!(ask(&source, "textDocument/hover", "scope", 0).is_null());
            assert_eq!(
                ask(&source, "textDocument/completion", "scope", 0),
                json!([])
            );
            assert!(
                ask(&source, "textDocument/hover", "text.valueOf", 0)["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains("text: string")
            );
            assert_eq!(
                ask(&source, "textDocument/signatureHelp", "7);text", 0)["activeParameter"],
                1
            );
        }
        let escaped_quote = format!("\\{quote}");
        for tail in ["", "\\", escaped_quote.as_str()] {
            let source = format!(
                "function main(){{const text={quote}unfinished /* }} function ghost() {{}} (a,b){tail}"
            );
            assert!(ask(&source, "textDocument/hover", "ghost", 0).is_null());
            assert_eq!(
                ask(&source, "textDocument/completion", "ghost", 0),
                json!([])
            );
            let mut position = at(&source, "(a,b)", 0);
            position.character +=
                ("(a,b)".encode_utf16().count() + tail.encode_utf16().count()) as u32;
            let items = SourceIndex::new(&source)
                .request(
                    "textDocument/completion",
                    &json!({"position":position}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert_eq!(items, json!([]));
        }
    }
}

// Catalog snippets may begin with named type declarations used by their example body.
// Both the full catalog fixture and focused method fixtures use this same wrapper.
#[derive(Default)]
struct CatalogExampleSource {
    declarations: Vec<String>,
    body: String,
}
impl CatalogExampleSource {
    fn push(&mut self, mut snippet: &str) -> String {
        let mut prelude = String::new();
        snippet = snippet.trim_start();
        while snippet.starts_with("type ") {
            let mut depth = 0_usize;
            let end = snippet
                .char_indices()
                .find_map(|(index, ch)| {
                    match ch {
                        '{' | '[' | '(' => depth += 1,
                        '}' | ']' | ')' => {
                            depth = depth.checked_sub(1).expect("balanced example type")
                        }
                        ';' if depth == 0 => return Some(index + 1),
                        _ => {}
                    }
                    None
                })
                .expect("terminated example type declaration");
            let declaration = &snippet[..end];
            prelude.push_str(declaration);
            if !self.declarations.iter().any(|saved| saved == declaration) {
                self.declarations.push(declaration.to_owned());
            }
            snippet = snippet[end..].trim_start();
        }
        self.body.push_str(&format!("{{{snippet}}}\n"));
        format!("{prelude}function main(){{{snippet}}}")
    }
    fn finish(self) -> String {
        format!(
            "{}function main(){{\n{}}}",
            self.declarations.join("\n"),
            self.body
        )
    }
}

#[test]
fn documented_builtin_examples_match_native_output() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let mut source = CatalogExampleSource::default();
    let mut expected = String::new();
    let mut expected_stderr = String::new();
    for (family, entries) in catalog.as_object().unwrap() {
        for entry in entries.as_array().unwrap() {
            let docs = entry["documentation"].as_str().unwrap();
            assert!(
                docs.contains("```tsrust"),
                "{family}.{} lacks an example",
                entry["name"]
            );
            for example in entry["examples"].as_array().unwrap() {
                let snippet = example["source"].as_str().unwrap();
                // Standard-library and matcher examples are complete programs,
                // as is any example that declares its own main function.
                if family.starts_with("std:")
                    || matches!(family.as_str(), "Expectation" | "NegatedExpectation")
                    || snippet.contains("function main(")
                {
                    let analysis = checked_std_example(snippet);
                    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../target/editor/doc-examples");
                    std::fs::create_dir_all(&dir).unwrap();
                    let prefix = if family.starts_with("std:") { "std".to_owned() } else { family.to_lowercase() };
                    let input = dir.join(format!("{prefix}-{}-{}.rs", entry["name"].as_str().unwrap(), snippet.len()));
                    let binary = input.with_extension("bin");
                    std::fs::write(&input, analysis.emit_rust()).unwrap();
                    let compilation = std::process::Command::new("rustc")
                        .args(["--edition=2024", "-Awarnings"])
                        .arg(&input)
                        .arg("-o")
                        .arg(&binary)
                        .output()
                        .unwrap();
                    assert!(
                        compilation.status.success(),
                        "{}",
                        String::from_utf8_lossy(&compilation.stderr)
                    );
                    let output = std::process::Command::new(&binary).output().unwrap();
                    assert!(output.status.success());
                    assert_eq!(
                        String::from_utf8(output.stdout).unwrap(),
                        example["output"].as_str().unwrap()
                    );
                    assert!(output.stderr.is_empty());
                    continue;
                }
                let witness = source.push(snippet);
                assert!(
                    tsrust_compiler::check(&witness).is_ok(),
                    "{family}.{} example rejected: {:?}",
                    entry["name"],
                    tsrust_compiler::check(&witness).err()
                );
                expected.push_str(example["output"].as_str().unwrap());
                expected_stderr.push_str(example["stderr"].as_str().unwrap_or(""));
            }
        }
    }
    let analysis = tsrust_compiler::check(&source.finish()).unwrap();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("examples.rs");
    let binary = dir.join("examples");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    let compilation = std::process::Command::new("rustc")
        .args(["--edition=2024", "-Awarnings"])
        .arg(&input)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
    let output = std::process::Command::new(binary).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), expected_stderr);
}

#[test]
fn builtin_hover_and_optional_parameter_help_include_behavior_and_examples() {
    let source = "function main(){const value:number=1.25;value.toFixed(2);}";
    let hover = ask(source, "textDocument/hover", "toFixed", 0);
    let docs = hover["contents"]["value"].as_str().unwrap();
    assert!(docs.contains("console.log(value.toFixed(2))"));
    assert!(docs.contains("half to even"));
    let help = ask(source, "textDocument/signatureHelp", "2);", 0);
    assert!(
        help["signatures"][0]["parameters"][0]["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("from 0 through 100")
    );
}

#[test]
fn builtin_namespaces_globals_and_types_have_contextual_hover() {
    let source = "function main(){const value:int=3;const text:string=\"é\";const ready:boolean=true;const ratio:number=Number(value);console.log(Math.sqrt(ratio));const copy=clone(text);}";
    assert!(tsrust_compiler::check(source).is_ok());
    for (needle, description) in [
        ("console", "Supported members: log"),
        ("Math", "radians"),
        ("Number(value)", "Number(value?:"),
        ("clone(text)", "independent copy"),
        ("int=", "Signed 64-bit"),
        ("string=", "UTF-8"),
        ("boolean=", "true and false"),
        ("number=", "IEEE 754"),
    ] {
        let hover = ask(source, "textDocument/hover", needle, 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap_or("")
                .contains(description),
            "{needle}: {hover}"
        );
    }
    for (needle, parameter) in [
        ("value);", "Optional primitive"),
        ("text);", "Required string"),
    ] {
        let help = ask(source, "textDocument/signatureHelp", needle, 0);
        assert!(
            help["signatures"][0]["parameters"][0]["documentation"]["value"]
                .as_str()
                .unwrap_or("")
                .contains(parameter)
        );
    }
    let shadow = "function console():int{return 1;}function main(){console();}";
    assert!(
        ask(shadow, "textDocument/hover", "console();", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("function console")
    );
    assert!(
        ask(
            "function main(){const text=\"console\";}",
            "textDocument/hover",
            "console",
            0
        )
        .is_null()
    );
    assert!(ask("type User={console:string};function main(){const user:User={console:\"x\"};console.log(user.console);}", "textDocument/hover", "console);", 0)["contents"]["value"].as_str().unwrap().contains("console: string"));
}

#[test]
fn template_chunks_hide_text_while_nested_interpolations_keep_references() {
    let source = r#"function main(){const value:int=7;const text:string="v";const result=`fake value ${value} ${`inner value ${value} ${text.concat("}", /* } ` */ "x")}`} \${value} \` tail`;console.log(value);}"#;
    let references = ask(source, "textDocument/references", "value:int", 0);
    assert_eq!(references.as_array().unwrap().len(), 4);
    let hover = ask(source, "textDocument/hover", "value}", 0);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("const value: int")
    );
    assert_eq!(
        ask(source, "textDocument/definition", "value}", 1)["range"]["start"],
        json!(at(source, "value:int", 0))
    );
    for needle in ["fake value", "inner value", "tail", "} ` */"] {
        assert!(ask(source, "textDocument/hover", needle, 0).is_null());
        assert_eq!(ask(source, "textDocument/completion", needle, 0), json!([]));
    }
    let chunks = r#"function main(){const value:int=7;const text=`${value}value${value}const${value},${value}{${value}`;}"#;
    assert_eq!(
        ask(chunks, "textDocument/references", "value:int", 0)
            .as_array()
            .unwrap()
            .len(),
        6
    );
    let escaped = r#"function main(){const value:int=7;const text=`\${value}`;}"#;
    assert_eq!(
        ask(escaped, "textDocument/references", "value:int", 0)
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let source = r#"function main(){const value:int=7;const text=`${({part:value}).part} ${value /* } */} ${value // } `
}`;console.log(value);}"#;
    assert_eq!(
        ask(source, "textDocument/references", "value:int", 0)
            .as_array()
            .unwrap()
            .len(),
        5
    );
}

#[test]
fn template_results_infer_string_for_initializers_and_member_receivers() {
    let source = r#"function main(){const value:int=7;const direct=`value ${value}`;const wrapped=(`value ${value}`);const chained=`${value}`.trim().valueOf();direct.substring(0);wrapped.trim();chained.toWellFormed();`raw ${value}`.substring(0,1);(`raw ${value}`).toWellFormed();`raw ${value}`.trim().substring(0,1);}"#;
    for needle in ["direct.substring", "wrapped.trim", "chained.toWellFormed"] {
        assert!(
            ask(source, "textDocument/hover", needle, 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(": string")
        );
    }
    for (needle, n) in [("substring", 1), ("toWellFormed", 1), ("substring", 2)] {
        assert!(
            ask(source, "textDocument/hover", needle, n)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(": string")
        );
    }
    for expression in [
        "`plain`.",
        "`raw ${value}`.",
        "(`raw ${value}`).",
        "`raw ${value}`.trim().",
    ] {
        let source = format!("function main(){{const value:int=7;{expression}");
        let position = Position::new(0, source.encode_utf16().count() as u32);
        let items = SourceIndex::new(&source)
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "substring")
        );
        assert!(
            !items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "toFixed")
        );
    }
}

#[test]
fn unfinished_templates_preserve_interpolation_completion_and_argument_counting() {
    // EOF uses the active scope, without leaking a closed function's locals.
    let closed = "function main(){const value:int=7;}";
    let position = Position::new(0, closed.encode_utf16().count() as u32);
    let items = SourceIndex::new(closed)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        !items
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "value")
    );
    for tail in [
        "`literal ghost",
        "`raw ${value} ghost",
        "`escaped \\${ghost",
        "`nested ${`inner ghost",
    ] {
        let source = format!("function main(){{const value:int=7;const text={tail}");
        let position = Position::new(0, source.encode_utf16().count() as u32);
        assert_eq!(
            SourceIndex::new(&source)
                .request(
                    "textDocument/completion",
                    &json!({"position":position}),
                    "file:///test.tsr"
                )
                .unwrap(),
            json!([])
        );
    }
    for tail in ["`${val", "`outer ${`inner ${val", "`${({part:val"] {
        let source = format!("function main(){{const value:int=7;const text={tail}");
        let position = Position::new(0, source.encode_utf16().count() as u32);
        let items = SourceIndex::new(&source)
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "value"),
            "{tail}: {items}"
        );
    }
    for newline in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
        for escape in ["", "\\"] {
            let source =
                format!("function main(){{const value:int=7;const text=`raw{escape}{newline}ghost");
            assert!(ask(&source, "textDocument/hover", "ghost", 0).is_null());
            assert_eq!(
                ask(&source, "textDocument/completion", "ghost", 0),
                json!([])
            );
        }
    }
    let source = r#"function outer(left:string,right:int):int{return right;}function pair(left:int,right:int):int{return left;}function main(){const value:int=1;outer(`literal , ${pair(value,2)},${`nested , ${pair(value,3)}`}`, value);}"#;
    assert_eq!(
        ask(source, "textDocument/signatureHelp", "value);", 0)["activeParameter"],
        1
    );
    assert_eq!(
        ask(source, "textDocument/signatureHelp", "2)}", 0)["activeParameter"],
        1
    );
    let source = r#"function outer(left:string,right:int):int{return right;}function main(){const value:int=1;outer(`literal , ${({left:value,right:value}).left}`, value);}"#;
    assert_eq!(
        ask(source, "textDocument/signatureHelp", "value);", 0)["activeParameter"],
        1
    );
}

#[test]
fn expanded_console_catalog_discovery_and_variadic_argument_help() {
    let source = "function main(){console.log(\"user\", 7);console.table([1,2],[\"Value\"]);}";
    for method in [
        "log",
        "info",
        "debug",
        "warn",
        "error",
        "table",
        "assert",
        "dir",
        "dirxml",
        "group",
        "groupCollapsed",
        "groupEnd",
        "count",
        "countReset",
        "time",
        "timeLog",
        "timeEnd",
        "clear",
        "trace",
        "profile",
        "profileEnd",
        "timeStamp",
    ] {
        let text = format!("function main(){{console.{method}(1);}}");
        assert!(tsrust_compiler::check(&text).is_ok());
        assert!(
            ask(&text, "textDocument/hover", method, 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("Returns void")
                || method == "table"
        );
    }
    let help = ask(source, "textDocument/signatureHelp", "7);", 0);
    assert!(
        help["signatures"][0]["parameters"][0]["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("Zero or more")
    );
    let source = "function main(){console.";
    let completion = SourceIndex::new(source)
        .request(
            "textDocument/completion",
            &json!({"position":{"line":0,"character":source.len()}}),
            "file:///test.tsr",
        )
        .unwrap();
    for method in [
        "info",
        "debug",
        "warn",
        "error",
        "table",
        "assert",
        "dir",
        "dirxml",
        "group",
        "groupCollapsed",
        "groupEnd",
        "count",
        "countReset",
        "time",
        "timeLog",
        "timeEnd",
        "clear",
        "trace",
        "profile",
        "profileEnd",
        "timeStamp",
    ] {
        assert!(
            completion
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["label"] == method)
        );
    }
}

#[test]
fn string_split_optional_docs_examples_and_native_results_are_consistent() {
    let source = "function main(){const text:string=\"a,b,c\";const parts=text.split(\",\",2);console.log(parts.length);}";
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["string"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "split")
        .unwrap();
    let signature = "split(separator?: string, limit?: int): string[]";
    assert_eq!(entry["signature"], signature);
    let hover = ask(source, "textDocument/hover", "split", 0);
    let docs = hover["contents"]["value"].as_str().unwrap();
    for detail in [
        signature,
        "defaults to 4294967295",
        "modulo 2^32",
        "discard the rest",
        "Unicode scalars",
        "static slices",
        "copied storage",
        "console.log(parts[1])",
    ] {
        assert!(docs.contains(detail), "{detail}: {docs}");
    }
    let help = ask(source, "textDocument/signatureHelp", "2);", 0);
    assert_eq!(help["signatures"][0]["label"], signature);
    assert_eq!(help["activeParameter"], 1);
    let parameters = help["signatures"][0]["parameters"].as_array().unwrap();
    assert_eq!(parameters[0]["label"], "separator?: string");
    assert_eq!(parameters[1]["label"], "limit?: int");
    assert!(
        parameters[0]["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("Omission")
    );
    assert!(
        parameters[1]["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("4294967296 converts to zero")
    );
    let partial = "function main(){const text:string=\"a,b,c\";text.spl";
    let position = Position::new(0, partial.encode_utf16().count() as u32);
    let items = SourceIndex::new(partial)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            "file:///test.tsr",
        )
        .unwrap();
    let completion = items
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["label"] == "split")
        .unwrap();
    assert_eq!(completion["detail"], signature);
    assert!(
        completion["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("defaults to 4294967295")
    );
    assert!(
        ask(source, "textDocument/hover", "parts.length", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("parts: string[]")
    );

    // Only this entry's executable examples are exercised; unrelated catalog checks stay separate.
    let mut examples = String::from("function main(){\n");
    let mut expected = String::new();
    for example in entry["examples"].as_array().unwrap() {
        let snippet = example["source"].as_str().unwrap();
        let witness = format!("function main(){{{snippet}}}");
        let _ = tsrust_compiler::check(&witness).unwrap();
        examples.push_str(&format!("{{{snippet}}}\n"));
        expected.push_str(example["output"].as_str().unwrap());
    }
    examples.push('}');
    let analysis = tsrust_compiler::check(&examples).unwrap();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/string-split-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = dir.join(format!("examples-o{level}"));
        let compilation = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = std::process::Command::new(binary).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn string_substr_optional_docs_examples_and_native_results_are_consistent() {
    let source = "function main(){const text:string=\"hello\";const piece=text.substr(1,3);console.log(piece.length);}";
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["string"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "substr")
        .unwrap();
    let signature = "substr(start?: int, length?: int): string";
    assert_eq!(entry["signature"], signature);
    let hover = ask(source, "textDocument/hover", "substr", 0);
    let docs = hover["contents"]["value"].as_str().unwrap();
    for detail in [
        signature,
        "defaults to zero",
        "defaults to the remaining bytes",
        "length <= 0",
        "without overflowing",
        "UTF-8 bytes",
        "UTF-16 code units",
        "boundary diagnostic",
        "-9223372036854775808",
        "independently copied",
        "console.log(\"hello\".substr(1, 3))",
    ] {
        assert!(docs.contains(detail), "{detail}: {docs}");
    }
    let help = ask(source, "textDocument/signatureHelp", "3);", 0);
    assert_eq!(help["signatures"][0]["label"], signature);
    assert_eq!(help["activeParameter"], 1);
    let parameters = help["signatures"][0]["parameters"].as_array().unwrap();
    assert_eq!(parameters[0]["label"], "start?: int");
    assert_eq!(parameters[1]["label"], "length?: int");
    assert!(
        parameters[0]["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("Negative values count from the end")
    );
    assert!(
        parameters[1]["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("Values <= 0")
    );
    let partial = "function main(){const text:string=\"hello\";text.subs";
    let position = Position::new(0, partial.encode_utf16().count() as u32);
    let items = SourceIndex::new(partial)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            "file:///test.tsr",
        )
        .unwrap();
    let completion = items
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["label"] == "substr")
        .unwrap();
    assert_eq!(completion["detail"], signature);
    assert!(
        completion["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("length defaults to the remaining bytes")
    );
    assert!(
        ask(source, "textDocument/hover", "piece.length", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("piece: string")
    );
    let chain = "function main(){const piece=\"hello\".substr().toWellFormed();piece.substr();}";
    assert!(
        ask(chain, "textDocument/hover", "piece.substr", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("piece: string")
    );
    assert!(
        ask(chain, "textDocument/hover", "substr", 1)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains(signature)
    );

    // Only this entry's executable examples are exercised; unrelated catalog checks stay separate.
    let mut examples = String::from("function main(){\n");
    let mut expected = String::new();
    for example in entry["examples"].as_array().unwrap() {
        let snippet = example["source"].as_str().unwrap();
        let witness = format!("function main(){{{snippet}}}");
        let _ = tsrust_compiler::check(&witness).unwrap();
        examples.push_str(&format!("{{{snippet}}}\n"));
        expected.push_str(example["output"].as_str().unwrap());
    }
    examples.push('}');
    let analysis = tsrust_compiler::check(&examples).unwrap();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/string-substr-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = dir.join(format!("examples-o{level}"));
        let compilation = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = std::process::Command::new(binary).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn nullish_literals_types_reflection_and_shadowing_have_editor_support() {
    let source = "function main(){const nothing=null;const missing=undefined;const kind=typeof nothing;const ignored=void 0;console.log(nothing,missing,kind,ignored);}";
    for (needle, expected) in [
        ("nothing,", "nothing: null"),
        ("missing,", "missing: undefined"),
        ("kind,", "kind: string"),
        ("ignored);", "ignored: undefined"),
    ] {
        assert!(
            ask(source, "textDocument/hover", needle, 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(expected)
        );
    }
    for word in ["null", "undefined", "typeof", "void"] {
        let text = format!(
            "function main(){{console.log({word}{});}}",
            if matches!(word, "typeof" | "void") {
                " 0"
            } else {
                ""
            }
        );
        assert!(
            ask(&text, "textDocument/hover", word, 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("```tsrust")
        );
    }
    let shadow = "function main(){const local=(undefined:int):int=>{return undefined;};console.log(local(7));}";
    assert!(
        ask(shadow, "textDocument/hover", "undefined;", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .replace(" ", "")
            .contains("undefined:int")
    );
    let source = "function main(){const values=[null];values.";
    let items = SourceIndex::new(source)
        .request(
            "textDocument/completion",
            &json!({"position":{"line":0,"character":source.len()}}),
            "file:///values.tsr",
        )
        .unwrap();
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "includes")
    );
}

#[test]
fn legacy_html_wrapper_docs_protocol_and_native_examples_are_consistent() {
    let methods = [
        "anchor",
        "big",
        "blink",
        "bold",
        "fixed",
        "fontcolor",
        "fontsize",
        "italics",
        "link",
        "small",
        "strike",
        "sub",
        "sup",
    ];
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let mut examples = String::from("function main(){\n");
    let mut expected = String::new();
    for method in methods {
        let entry = catalog["string"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["name"] == method)
            .unwrap();
        let parameters = entry["parameters"].as_array().unwrap();
        let attributed = matches!(method, "anchor" | "fontcolor" | "fontsize" | "link");
        assert_eq!(parameters.len(), usize::from(attributed));
        let argument = if attributed { "\"value\"" } else { "" };
        let source = format!(
            "function main(){{const text:string=\"é😀\";const result=text.{method}({argument});console.log(result.length);}}"
        );
        let signature = entry["signature"].as_str().unwrap();
        let hover = ask(&source, "textDocument/hover", method, 0);
        let docs = hover["contents"]["value"].as_str().unwrap();
        for detail in [
            signature,
            "receiver remains unchanged",
            "without escaping",
            "not an HTML sanitizer",
            "legacy, deprecated",
            "UTF-8",
            "evaluated once",
            "independently owned",
            "```tsrust",
        ] {
            assert!(docs.contains(detail), "{method}: missing {detail}: {docs}");
        }
        let help = ask(&source, "textDocument/signatureHelp", ");console", 0);
        assert_eq!(help["signatures"][0]["label"], signature);
        let actual = help["signatures"][0]["parameters"].as_array().unwrap();
        assert_eq!(actual.len(), parameters.len());
        if attributed {
            let label = match method {
                "anchor" => "name: string",
                "fontcolor" => "color: string",
                "fontsize" => "size: string",
                "link" => "url: string",
                _ => unreachable!(),
            };
            assert_eq!(parameters[0], label);
            assert_eq!(actual[0]["label"], label);
            let parameter_docs = actual[0]["documentation"]["value"].as_str().unwrap();
            assert!(parameter_docs.contains("Required string"));
            assert!(parameter_docs.contains("only ASCII double quotes become &quot;"));
            assert!(docs.contains("ampersands, angle brackets and single quotes are unchanged"));
            assert!(docs.contains("No implicit coercion or default \"undefined\" attribute"));
        } else {
            assert!(docs.contains("Takes no arguments"));
        }
        let partial = format!("function main(){{const text:string=\"x\";text.{method}");
        let position = Position::new(0, partial.encode_utf16().count() as u32);
        let items = SourceIndex::new(&partial)
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        let completion = items
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["label"] == method)
            .unwrap();
        assert_eq!(completion["detail"], signature);
        assert!(
            completion["documentation"]["value"]
                .as_str()
                .unwrap()
                .contains("independently owned")
        );
        assert!(
            ask(&source, "textDocument/hover", "result.length", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("result: string")
        );
        let chain = format!(
            "function main(){{const result=\"x\".{method}({argument}).valueOf();result.toWellFormed();}}"
        );
        assert!(
            ask(&chain, "textDocument/hover", "result.toWellFormed", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("result: string")
        );
        for example in entry["examples"].as_array().unwrap() {
            let snippet = example["source"].as_str().unwrap();
            let witness = format!("function main(){{{snippet}}}");
            let _ = tsrust_compiler::check(&witness).unwrap();
            examples.push_str(&format!("{{{snippet}}}\n"));
            expected.push_str(example["output"].as_str().unwrap());
        }
    }
    examples.push('}');
    let analysis = tsrust_compiler::check(&examples).unwrap();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/legacy-html-wrapper-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = dir.join(format!("examples-o{level}"));
        let compilation = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = std::process::Command::new(binary).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn array_static_and_conversion_globals_have_complete_editor_support() {
    let source = "function main(){const text=String(null);const ready=Boolean(undefined);const array=Array.isArray([]);console.log(text,ready,array);}";
    for (needle, expected) in [
        ("text,", "text: string"),
        ("ready,", "ready: boolean"),
        ("array);", "array: boolean"),
    ] {
        assert!(
            ask(source, "textDocument/hover", needle, 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(expected)
        );
    }
    for word in ["String", "Boolean", "isArray"] {
        assert!(
            ask(source, "textDocument/hover", word, 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("@param value")
        );
    }
    assert!(
        ask(source, "textDocument/signatureHelp", "null);", 0)["signatures"][0]["label"]
            .as_str()
            .unwrap()
            .starts_with("String(")
    );
    let incomplete = "function main(){Array.";
    let items = SourceIndex::new(incomplete)
        .request(
            "textDocument/completion",
            &json!({"position":{"line":0,"character":incomplete.len()}}),
            "file:///a.tsr",
        )
        .unwrap();
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "isArray")
    );
    assert!(
        !items
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "push")
    );
    let instance = "function main(){const values=[1];values.";
    let items = SourceIndex::new(instance)
        .request(
            "textDocument/completion",
            &json!({"position":{"line":0,"character":instance.len()}}),
            "file:///a.tsr",
        )
        .unwrap();
    assert!(
        !items
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "isArray")
    );
}

#[test]
fn array_join_docs_inference_and_native_examples_are_consistent() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["Array"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "join")
        .unwrap();
    let signature = "join(separator?: string): string";
    assert_eq!(entry["signature"], signature);
    assert_eq!(entry["parameters"], json!(["separator?: string"]));
    let source = "function main(){const values:int[]=[1,2];const text=values.join(\"|\");console.log(text.length);}";
    let hover = ask(source, "textDocument/hover", "join", 0);
    let docs = hover["contents"]["value"].as_str().unwrap();
    for detail in [
        signature,
        "array is unchanged",
        "defaults to a comma",
        "empty array",
        "More than one argument",
        "explicit undefined",
        "Nested arrays use commas",
        "[object Object]",
        "negative zero becomes 0",
        "UTF-8",
        "left to right",
        "independently owned",
        "```tsrust",
    ] {
        assert!(docs.contains(detail), "missing {detail}: {docs}");
    }
    let help = ask(source, "textDocument/signatureHelp", ");console", 0);
    assert_eq!(help["signatures"][0]["label"], signature);
    assert_eq!(
        help["signatures"][0]["parameters"][0]["label"],
        "separator?: string"
    );
    assert!(
        help["signatures"][0]["parameters"][0]["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("defaults to")
    );
    assert!(
        ask(source, "textDocument/hover", "text.length", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("text: string")
    );
    let partial = "function main(){const values:int[]=[1,2];values.jo";
    let position = Position::new(0, partial.encode_utf16().count() as u32);
    let items = SourceIndex::new(partial)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            "file:///test.tsr",
        )
        .unwrap();
    let completion = items
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["label"] == "join")
        .unwrap();
    assert_eq!(completion["detail"], signature);
    assert!(
        completion["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("Nested arrays use commas")
    );
    for receiver in ["[1,2]", "([1,2])", "values", "nested[0]"] {
        let source = format!(
            "function main(){{const values:int[]=[1,2];const nested:int[][]=[[1,2]];const text={receiver}.join(\"|\").valueOf();text.toWellFormed();}}"
        );
        assert!(tsrust_compiler::check(&source).is_ok());
        assert!(
            ask(&source, "textDocument/hover", "join", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(signature),
            "{receiver}"
        );
        assert!(
            ask(&source, "textDocument/hover", "text.toWellFormed", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("text: string"),
            "{receiver}"
        );
    }
    // Closing-bracket inference must retain named indexed values and existing properties.
    let indexed = "function main(){const values:string[]=[\"é\"];console.log(values[0].length);values[0].valueOf();}";
    assert!(tsrust_compiler::check(indexed).is_ok());
    assert!(
        ask(indexed, "textDocument/hover", "length", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("length: int")
    );
    let record = "type Item={label:string;};function main(){const values:Item[]=[{label:\"x\"}];values[0].la";
    let position = Position::new(0, record.encode_utf16().count() as u32);
    let fields = SourceIndex::new(record)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        fields
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "label")
    );
    let existing = "function main(){const nested:int[][]=[[1,2]];console.log(nested[0].length);nested[0].slice(0,1);}";
    assert!(tsrust_compiler::check(existing).is_ok());
    assert!(
        ask(existing, "textDocument/hover", "slice", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("slice(")
    );
    for call in ["join(1)", "join(undefined)", "join(\"a\",\"b\")"] {
        assert!(
            tsrust_compiler::check(&format!(
                "function main(){{const values:int[]=[1];values.{call};}}"
            ))
            .is_err()
        );
    }
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for example in entry["examples"].as_array().unwrap() {
        let snippet = example["source"].as_str().unwrap();
        let witness = examples.push(snippet);
        let _ = tsrust_compiler::check(&witness).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }
    let analysis = tsrust_compiler::check(&examples.finish()).unwrap();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/array-join-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = dir.join(format!("examples-o{level}"));
        let compilation = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = std::process::Command::new(binary).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn array_to_string_and_global_string_docs_preserve_primitive_inference() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["Array"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "toString")
        .unwrap();
    let signature = "toString(): string";
    assert_eq!(entry["signature"], signature);
    assert_eq!(entry["parameters"], json!([]));
    let source = "function main(){const values:int[]=[1,2];const text=values.toString();console.log(text.length);}";
    let hover = ask(source, "textDocument/hover", "toString", 0);
    let docs = hover["contents"]["value"].as_str().unwrap();
    for detail in [
        signature,
        "array element text",
        "array is unchanged",
        "Takes no arguments",
        "including undefined",
        "empty array",
        "Nested arrays",
        "[object Object]",
        "negative zero becomes 0",
        "UTF-8",
        "rounding boundaries",
        "prototype hooks",
        "borrowed",
        "evaluated once",
        "independently owned",
        "```tsrust",
    ] {
        assert!(docs.contains(detail), "missing {detail}: {docs}");
    }
    let help = ask(source, "textDocument/signatureHelp", ");console", 0);
    assert_eq!(help["signatures"][0]["label"], signature);
    assert!(
        help["signatures"][0]["parameters"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let partial = "function main(){const values:int[]=[1,2];values.toS";
    let position = Position::new(0, partial.encode_utf16().count() as u32);
    let items = SourceIndex::new(partial)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            "file:///test.tsr",
        )
        .unwrap();
    let method = items
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["label"] == "toString")
        .unwrap();
    assert_eq!(method["detail"], signature);
    assert!(
        method["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("array element text")
    );
    for receiver in ["values", "[1,2]", "([1,2])", "nested[0]", "[]", "[[],[]]"] {
        let source = format!(
            "function main(){{const values:int[]=[1,2];const nested:int[][]=[[1,2]];const text={receiver}.toString().valueOf();text.toWellFormed();}}"
        );
        let _ = tsrust_compiler::check(&source).unwrap();
        assert!(
            ask(&source, "textDocument/hover", "toString", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("array element text"),
            "{receiver}"
        );
        assert!(
            ask(&source, "textDocument/hover", "text.toWellFormed", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("text: string"),
            "{receiver}"
        );
    }
    // Empty literals remain recognizable arrays for join, including nested empty literals.
    for receiver in ["[]", "[[],[]]"] {
        let source =
            format!("function main(){{const text={receiver}.join().valueOf();text.length;}}");
        let _ = tsrust_compiler::check(&source).unwrap();
        assert!(
            ask(&source, "textDocument/hover", "text.length", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("text: string")
        );
    }
    for (ty, value, label) in [
        ("string", "\"x\"", "toString(): string"),
        ("boolean", "true", "toString(): string"),
        ("int", "7", "toString(radix?: int): string"),
    ] {
        let source = format!(
            "function main(){{const value:{ty}={value};const text=value.toString();text.length;}}"
        );
        let _ = tsrust_compiler::check(&source).unwrap();
        assert!(
            ask(&source, "textDocument/hover", "toString", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(label)
        );
        assert!(
            ask(&source, "textDocument/hover", "text.length", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("text: string")
        );
    }
    // number.toString is supported and documented like the other conversions.
    let number = "function main(){const value:number=1.0;value.toString();}";
    assert!(tsrust_compiler::check(number).is_ok());
    assert!(!ask(number, "textDocument/hover", "toString", 0).is_null());
    let partial = "function main(){const value:number=1.0;value.toS";
    let position = Position::new(0, partial.encode_utf16().count() as u32);
    let methods = SourceIndex::new(partial)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        methods
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "toString")
    );
    for call in ["toString(\",\")", "toString(undefined)"] {
        assert!(
            tsrust_compiler::check(&format!(
                "function main(){{const values:int[]=[1];values.{call};}}"
            ))
            .is_err()
        );
    }
    let global = catalog["globals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "String")
        .unwrap();
    let global_source = "function main(){const values:int[][]=[[1,2],[],[3]];const text=String(values).valueOf();text.length;}";
    let _ = tsrust_compiler::check(global_source).unwrap();
    let hover = ask(global_source, "textDocument/hover", "String(values)", 0);
    let docs = hover["contents"]["value"].as_str().unwrap();
    assert!(!docs.contains("128"));
    assert!(docs.contains("all supported nested-array text"));
    assert!(
        ask(global_source, "textDocument/hover", "text.length", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("text: string")
    );
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for entry in [entry, global] {
        for example in entry["examples"].as_array().unwrap() {
            let witness = examples.push(example["source"].as_str().unwrap());
            let _ = tsrust_compiler::check(&witness).unwrap();
            expected.push_str(example["output"].as_str().unwrap());
        }
    }
    let analysis = tsrust_compiler::check(&examples.finish()).unwrap();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/array-to-string-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = dir.join(format!("examples-o{level}"));
        let compilation = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = std::process::Command::new(binary).output().unwrap();
        std::fs::write(
            dir.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            dir.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn nullable_union_help_and_observations_preserve_whole_value_types() {
    // Declaration help is canonical, without narrowing from the initializer.
    for annotation in [
        "int[] | null | undefined",
        "undefined | null | int[] | null",
        "(null | (int[] | undefined))",
    ] {
        let source = format!("function main(){{const value:{annotation}=[1];String(value);}}");
        tsrust_compiler::check(&source).unwrap();
        let hover = ask(&source, "textDocument/hover", "value);", 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("value: int[] | null | undefined")
        );
        let partial = format!("function main(){{const value:{annotation}=[1];value.jo");
        let items = SourceIndex::new(&partial)
            .request(
                "textDocument/completion",
                &json!({"position":Position::new(0,partial.encode_utf16().count() as u32)}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            items.as_array().unwrap().is_empty(),
            "{annotation}: {items}"
        );
    }
    // Even rejected projections must not invent an array from the index syntax.
    for annotation in [
        "int[][] | null",
        "null | int[][]",
        "(undefined | int[][] | null)",
    ] {
        let source = format!("function main(){{const value:{annotation}=[[1]];value[0].join();}}");
        assert!(tsrust_compiler::check(&source).is_err());
        assert!(ask(&source, "textDocument/hover", "join", 0).is_null());
        assert!(ask(&source, "textDocument/signatureHelp", ");}", 0).is_null());
    }
    // A null-only guard still cannot project a payload when undefined remains.
    for (annotation, value, member) in [
        ("string | null | undefined", "\"x\"", "valueOf()"),
        ("null | int[] | undefined", "[1]", "join()"),
        ("Row | undefined", "{name:\"x\"}", "name"),
    ] {
        let source = format!(
            "type Row={{name:string;}};function main(){{const value:{annotation}={value};if(value!==null){{value.{member};}}}}"
        );
        assert!(tsrust_compiler::check(&source).is_err());
        assert!(
            ask(
                &source,
                "textDocument/hover",
                member.split('(').next().unwrap(),
                if member == "name" { 2 } else { 0 }
            )
            .is_null()
        );
    }
    let source = "type Row={value:null|string;};function forward(input:null|int[]):int[]|null{return input;}function main(){const row:Row={value:null};const result=forward(null);String(row.value);String(result);}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "value);", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("value: string | null")
    );
    assert!(
        ask(source, "textDocument/hover", "result);", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("result: int[] | null")
    );
    let help = ask(source, "textDocument/signatureHelp", "null);", 0);
    assert!(
        help["signatures"][0]["label"]
            .as_str()
            .unwrap()
            .contains("input:null|int[]")
    );

    // Ordinary arrays and callbacks retain their existing editor paths.
    for source in [
        "function main(){const value:int[][]=[[1]];value[0].join();}",
        "function main(){const value:int[]|int[]=[1];value.join();}",
    ] {
        tsrust_compiler::check(source).unwrap();
        assert!(
            ask(source, "textDocument/hover", "join", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("join(separator?: string): string")
        );
    }
    let callable = "function main(){const action=(value:int):int=>{return value;};action(3);}";
    tsrust_compiler::check(callable).unwrap();
    assert_eq!(
        ask(callable, "textDocument/signatureHelp", "3);", 0)["signatures"][0]["parameters"][0]["label"],
        "value:int"
    );
    for prefix in ["Function<int, [value:int]>", "Once<Function<int, [value:int]>>", "Owned<Function<int, [value:int]>>"] {
        let source = format!(
            "function invoke(action:{prefix}):int{{return action(3);}}function main(){{console.log(invoke((value:int):int=>{{return value;}}));}}"
        );
        tsrust_compiler::check(&source).unwrap();
        let help = ask(&source, "textDocument/signatureHelp", "3);", 0);
        assert_eq!(
            help["signatures"][0]["parameters"][0]["label"], "value: int",
            "{prefix}"
        );
    }
    let nullable_result = "function apply(action:(value:int)=>int|null):string{const result=action(3);return String(result);}function main(){console.log(apply((value:int):int|null=>{return value>0?value:null;}));}";
    tsrust_compiler::check(nullable_result).unwrap();
    assert_eq!(
        ask(nullable_result, "textDocument/signatureHelp", "3);", 0)["signatures"][0]["parameters"]
            [0]["label"],
        "value: int"
    );
    assert!(
        ask(nullable_result, "textDocument/hover", "result);", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("result: int | null")
    );
    let factory = "function factory():(value:int)=>int{return (value:int):int=>{return value;};}function main(){const callback=factory();callback(3);}";
    tsrust_compiler::check(factory).unwrap();
    assert_eq!(
        ask(factory, "textDocument/signatureHelp", "3);", 0)["signatures"][0]["parameters"][0]["label"],
        "value:int"
    );
    let rejected = "function main(){const action:null|((value:int)=>int)=null;action(3);}";
    assert!(tsrust_compiler::check(rejected).is_err());
    assert!(ask(rejected, "textDocument/signatureHelp", "3);", 0).is_null());

    let unsupported_conversion = "function main(){const value:number|null=1.0;Number(value);}";
    assert_eq!(tsrust_compiler::check(unsupported_conversion).err().unwrap().code, "unsupported");
    for source in [
        "function main(){const value:string|null=null;console.log(value);}",
        "type Box={value:string|null;};function main(){const value:Box={value:null};console.log(value);}",
    ] {
        tsrust_compiler::check(source).unwrap();
        let hover = ask(source, "textDocument/hover", "log", 0);
        assert!(hover["contents"]["value"].as_str().unwrap().contains("console.log"));
    }
    let number = "function main(){Number(1);}";
    assert!(
        ask(number, "textDocument/hover", "Number", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Nullable unions cannot be passed directly")
    );
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for (family, name) in [
        ("globals", "Boolean"),
        ("globals", "String"),
        ("ArrayConstructor", "isArray"),
    ] {
        let entry = catalog[family]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["name"] == name)
            .unwrap();
        assert!(
            entry["documentation"]
                .as_str()
                .unwrap()
                .contains("union")
        );
        let example = entry["examples"].as_array().unwrap().last().unwrap();
        tsrust_compiler::check(&examples.push(example["source"].as_str().unwrap())).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }
    let mut program = examples.finish();
    program.push_str("\nfunction transport(value:string|null|undefined):string|null|undefined{return value;}\nfunction pick(flag:boolean):int[]|null{return flag?[1,2]:null;}\n");
    // Separate language examples cover owned transport, conditional joins and reusable observations.
    program=program.replacen("function main(){", "function main(){ {const value=transport(clone(\"é\"));console.log(String(value),Boolean(value),typeof value);console.log(String(value));const values=pick(true);console.log(String(values),Boolean(values),typeof values);}",1);
    expected.insert_str(0, "é true string\né\n1,2 true object\n");
    program=program.replacen("function main(){", "function main(){ {let source:int[]|null|undefined=[3];const copy=clone(source);source=null;console.log(String(copy),String(source));const missing:int[]|null=null;console.log(String(clone(missing)));const absent:int[]|undefined=undefined;console.log(String(clone(absent)));}",1);
    expected.insert_str(0, "3 null\nnull\nundefined\n");
    program.push_str("\ntype HiddenNullable={data:int[]|null;};type Plain={count:int;};\n");
    program = program.replacen(
        "function main(){",
        "function main(){ {const plain:Plain={count:7};console.log(plain);}",
        1,
    );
    expected.insert_str(0, "{ count: 7 }\n");
    let clone_docs = ask(&program, "textDocument/hover", "clone(source)", 0);
    assert!(
        clone_docs["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("preserving its type and active variant")
    );
    let rejected_copy = "function main(){const value:int|null=1;clone(value);}";
    assert!(tsrust_compiler::check(rejected_copy).is_err());
    let analysis = tsrust_compiler::check(&program).unwrap();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/nullable-union-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("examples.rs");
    std::fs::write(dir.join("examples.tsr"), &program).unwrap();
    std::fs::write(dir.join("expected.stdout"), &expected).unwrap();
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = dir.join(format!("examples-o{level}"));
        let compilation = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = std::process::Command::new(binary).output().unwrap();
        std::fs::write(
            dir.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            dir.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn top_level_main_explains_automatic_entry_without_overwriting_user_docs() {
    let source = "/** Starts the demo. */\nfunction main() { console.log(7); }\n";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "main", 0);
    let docs = hover["contents"]["value"].as_str().unwrap();
    assert!(docs.contains("Starts the demo."));
    assert!(docs.contains("automatically calls main()"));
    assert!(docs.contains("VS Code play button"));
    assert!(docs.contains("tsrust run"));
    let source = "function main() {}\nma";
    let completion = ask(source, "textDocument/completion", "ma", 1);
    assert!(
        completion
            .to_string()
            .contains("automatically calls main()")
    );
    let field = "type User={main:int;};function main(){const user:User={main:1};user.main;}";
    let hover = ask(field, "textDocument/hover", "main;", 0);
    assert!(!hover.to_string().contains("Program entry point"));
    let local = "function helper(){const main=():int=>{return 1;};main();}function main(){}";
    let hover = ask(local, "textDocument/hover", "main();", 0);
    assert!(!hover.to_string().contains("Program entry point"));
}

#[test]
fn nullable_narrowing_editor_views_follow_checked_places_and_effects() {
    let positives = [
        (
            "function main(){const value:string|null=clone(\" x \" );if(value!==null){const text=value.trim();console.log(text);}}",
            "trim",
            "trim(): string",
            "string",
        ),
        (
            "function main(){const value:int[]|null|undefined=[1];if(null!==value&&undefined!==value){const text=value.join();console.log(text);}}",
            "join",
            "join(separator?: string): string",
            "int[]",
        ),
        (
            "type Holder={value:string|null;};function main(){const holder:Holder={value:clone(\"x\")};if(holder.value!==null){const text=holder.value.trim();console.log(text);}}",
            "trim",
            "trim(): string",
            "string",
        ),
        (
            "function observe(value:string|null):string{if(value===null){return \"missing\";}return value.trim();}function main(){console.log(observe(null));}",
            "trim",
            "trim(): string",
            "string",
        ),
        (
            "function main(){const value:int[]|null|undefined=[1];if(!(value===null||value===undefined)){const text=value.join();console.log(text);}}",
            "join",
            "join(separator?: string): string",
            "int[]",
        ),
        (
            "function main(){let value:int[]|null=[1];while(value!==null){const text=value.join();console.log(text);value=null;}}",
            "join",
            "join(separator?: string): string",
            "int[]",
        ),
    ];
    for (source, method, signature, ty) in positives {
        tsrust_compiler::check(source).unwrap();
        let index = SourceIndex::new(source);
        if source.contains("const value:") {
            let declaration = index
                .request(
                    "textDocument/hover",
                    &json!({"position": at(source, "value:", 0)}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert!(
                declaration["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains(" | null")
            );
            assert!(
                !declaration["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains("Narrowed here")
            );
        }
        let request = |method_name: &str, needle: &str| {
            index
                .request(
                    method_name,
                    &json!({"position":at(source,needle,0)}),
                    "file:///test.tsr",
                )
                .unwrap()
        };
        let hover = request("textDocument/hover", method);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(signature),
            "{source}: {hover}"
        );
        let receiver = if source.contains("holder.value.trim") || method == "trim" {
            "value.trim"
        } else {
            "value.join"
        };
        let view = request("textDocument/hover", receiver);
        assert!(
            view["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("Narrowed here to `{ty}`")),
            "{source}: {view}"
        );
        // Move the request into the call, after its opening parenthesis.
        let mut position = at(source, &format!("{method}();"), 0);
        position.character += method.len() as u32 + 1;
        let help = index
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(help["signatures"][0]["label"], signature);
        let mut position = at(source, method, 0);
        position.character += 2;
        let completion = index
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            completion
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == method),
            "{source}: {completion}"
        );
        if source.contains("const text=") {
            let text = request("textDocument/hover", "text);");
            assert!(
                text["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains("text: string"),
                "{source}: {text}"
            );
        }
    }
    // Only a successful checked receiver probe can enable incomplete member help.
    for (source, method) in [
        (
            "function main(){const value:string|null=\"x\";if(value!==null){value.tr",
            "trim",
        ),
        (
            "function main(){const value:int[]|null|undefined=[1];if(value!==null&&value!==undefined){value.jo",
            "join",
        ),
        (
            "function main(){let value:int[]|null=[1];while(value!==null){value.jo",
            "join",
        ),
        (
            "type Holder={value:string|null;};function main(){const holder:Holder={value:\"x\"};if(holder.value!==null){holder.value.tr",
            "trim",
        ),
    ] {
        let index = SourceIndex::new(source);
        let position = Position::new(0, source.encode_utf16().count() as u32);
        let first = index
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            first
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == method),
            "{source}: {first}"
        );
        assert_eq!(
            first,
            index
                .request(
                    "textDocument/completion",
                    &json!({"position":position}),
                    "file:///test.tsr"
                )
                .unwrap()
        );
    }
    for source in [
        "type Holder={value:int[]|null;};function clear(holder:Holder):boolean{holder.value=null;return true;}function main(){const holder:Holder={value:[1]};if(holder.value!==null){clear(holder);holder.value.jo",
        "function main(){const value:int[]|null=[1];if(Array.isArray(value)){value.jo",
        "function main(){const value:int[]|null|undefined=[1];if(value!==null){value.jo",
        "function main(){let value:int[]|null=[1];if(value!==null){value=null;value.jo",
        "function main(){const value:int[]|null=[1];if(value!==null){const moved=value;value.jo",
        "function main(){const value:int[]|null=[1];if(value!==null){const action=():string=>{return value.jo",
        "type Holder={value:string|null;};function main(){const values:Holder[]=[{value:\"x\"}];if(values[0].value!==null){values[0].value.tr",
    ] {
        let index = SourceIndex::new(source);
        let result = index
            .request(
                "textDocument/completion",
                &json!({"position":Position::new(0,source.encode_utf16().count() as u32)}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(result.as_array().unwrap().is_empty(), "{source}: {result}");
    }
    // A whole-record projection supplies fields; the declaration stays nullable.
    let source = "type Item={label:string;};function main(){const value:Item|null={label:\"x\"};if(value!==null){value.la";
    let index = SourceIndex::new(source);
    let members = index
        .request(
            "textDocument/completion",
            &json!({"position": Position::new(0, source.encode_utf16().count() as u32)}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        members
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "label")
    );
    // Inferred whole-payload transport must not retain the old nullable annotation.
    let source = "function main(){const value:int[]|null=[1];if(value!==null){const selected=value;console.log(selected.join());}}";
    tsrust_compiler::check(source).unwrap();
    let index = SourceIndex::new(source);
    let hover = index
        .request(
            "textDocument/hover",
            &json!({"position":at(source,"selected.join",0)}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("selected: int[]")
    );
    let method = index
        .request(
            "textDocument/hover",
            &json!({"position":at(source,"join",0)}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        method["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("join(separator?: string): string")
    );
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let array_is_array = catalog["ArrayConstructor"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "isArray")
        .unwrap();
    assert!(
        array_is_array["documentation"]
            .as_str()
            .unwrap()
            .contains("by itself does not narrow")
    );
    let source = r#"type Box={text:string|null;values:int[]|undefined;};
function label(value:string|null|undefined):string{if(value===null||value===undefined){return "missing";}return value.trim();}
function main(){
console.log(label(null),label(undefined),label(clone(" é ")));
let values:int[]|null|undefined=[1,2];
if(values!==null&&values!==undefined){console.log(values.join("|"));values.push(3);console.log(values.join());}
const box:Box={text:clone("x"),values:[4]};
if(box.text!==null){console.log(box.text.toWellFormed());}
if(box.values!==undefined){box.values.push(5);console.log(box.values.join("|"));}
let loopValue:int[]|null=[6];while(loopValue!==null){console.log(loopValue.join());loopValue=null;}
const owned:string|null=clone("owned");if(owned!==null){const moved=owned;console.log(moved);}
}"#;
    let expected = "missing missing é\n1|2\n1,2,3\nx\n4|5\n6\nowned\n";
    let analysis = tsrust_compiler::check(source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/nullable-narrowing-language-examples");
    std::fs::create_dir_all(&directory).unwrap();
    let input = directory.join("examples.rs");
    std::fs::write(directory.join("examples.tsr"), source).unwrap();
    std::fs::write(directory.join("expected.stdout"), expected).unwrap();
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("examples-o{level}"));
        let built = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let output = std::process::Command::new(binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn array_removal_docs_and_nullable_result_views_are_consistent() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let cases = [
        ("int[]", "[1]", "int | undefined"),
        ("number[]", "[-0.0]", "number | undefined"),
        ("boolean[]", "[true]", "boolean | undefined"),
        ("string[]", "[clone(\"é\")]", "string | undefined"),
        ("null[]", "[null]", "null | undefined"),
        ("undefined[]", "[undefined]", "undefined"),
        ("int[][]", "[[1]]", "int[] | undefined"),
        ("Item[]", "[{label:\"x\"}]", "Item | undefined"),
        ("unknownBox[]", "[{label:\"x\"}]", "unknownBox | undefined"),
    ];
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for method in ["pop", "shift"] {
        let entry = catalog["Array"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["name"] == method)
            .unwrap();
        assert_eq!(
            entry["signature"],
            format!("{method}(): Element | undefined")
        );
        assert_eq!(entry["parameters"], json!([]));
        let docs = entry["documentation"].as_str().unwrap();
        for detail in [
            "Takes no arguments",
            "arity error",
            "empty array returns undefined",
            "length decreases by one",
            "dense homogeneous",
            "Sparse arrays",
            "without cloning",
            "mutable access",
            "```tsrust",
        ] {
            assert!(docs.contains(detail), "{method}: {detail}");
        }
        for (ty, initial, result) in cases {
            let source = format!(
                "type Item={{label:string;}};type unknownBox={{label:string;}};function main(){{const values:{ty}={initial};const result=values.{method}();String(result);}}"
            );
            tsrust_compiler::check(&source).unwrap();
            let index = SourceIndex::new(&source);
            let hover = index
                .request(
                    "textDocument/hover",
                    &json!({"position":at(&source,&format!("{method}();"),0)}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert!(
                hover["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("{method}(): {result}")),
                "{source}: {hover}"
            );
            let inferred = index
                .request(
                    "textDocument/hover",
                    &json!({"position":at(&source,"result);",0)}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert!(
                inferred["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("result: {result}")),
                "{source}: {inferred}"
            );
            let mut position = at(&source, &format!("{method}();"), 0);
            position.character += method.len() as u32 + 1;
            let help = index
                .request(
                    "textDocument/signatureHelp",
                    &json!({"position":position}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert_eq!(
                help["signatures"][0]["label"],
                format!("{method}(): {result}")
            );
            assert_eq!(help["signatures"][0]["parameters"], json!([]));
            let mut position = at(&source, &format!("{method}();"), 0);
            position.character += 2;
            let completion = index
                .request(
                    "textDocument/completion",
                    &json!({"position":position}),
                    "file:///test.tsr",
                )
                .unwrap();
            let item = completion
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["label"] == method)
                .unwrap();
            assert_eq!(item["detail"], format!("{method}(): {result}"));
        }
        let source = format!(
            "function main(){{const values:string[]|undefined=[\"x\"];if(values!==undefined){{const result=values.{method}();if(result!==undefined){{console.log(result.trim());}}}}}}"
        );
        tsrust_compiler::check(&source).unwrap();
        assert!(
            ask(&source, "textDocument/hover", "trim", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("trim(): string")
        );
        let partial = format!(
            "function main(){{const values:int[][]=[[1]];const result=values.{method}();if(result!==undefined){{result.jo"
        );
        let members = SourceIndex::new(&partial)
            .request(
                "textDocument/completion",
                &json!({"position":Position::new(0,partial.encode_utf16().count() as u32)}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            members
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "join")
        );
        for source in [
            format!(
                "function main(){{const values:string[]=[\"x\"];const result=values.{method}();result.tr"
            ),
            format!(
                "function main(){{const values:int[]|undefined=[1];values.{}",
                &method[..2]
            ),
        ] {
            let members = SourceIndex::new(&source)
                .request(
                    "textDocument/completion",
                    &json!({"position":Position::new(0,source.encode_utf16().count() as u32)}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert!(
                members.as_array().unwrap().is_empty(),
                "{source}: {members}"
            );
        }
        for argument in ["1", "undefined", "\"x\""] {
            let source =
                format!("function main(){{const values:int[]=[1];values.{method}({argument});}}");
            assert!(tsrust_compiler::check(&source).is_err());
        }
        let source = format!("function main(){{const text:string=\"x\";text.{method}();}}");
        assert!(tsrust_compiler::check(&source).is_err());
        assert!(ask(&source, "textDocument/hover", method, 0).is_null());
        for example in entry["examples"].as_array().unwrap() {
            let witness = examples.push(example["source"].as_str().unwrap());
            tsrust_compiler::check(&witness).unwrap();
            expected.push_str(example["output"].as_str().unwrap());
        }
    }
    // Field-chain and existing indexed receiver inference remain intact.
    let source = "type Holder={values:int[];};function main(){const holder:Holder={values:[1]};const result=holder.values.pop();String(result);console.log(holder.values.length);}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "result);", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("result: int | undefined")
    );
    let indexed = "function main(){const nested:int[][]=[[1]];nested[0].shift();console.log(nested[0].join());}";
    tsrust_compiler::check(indexed).unwrap();
    assert!(
        ask(indexed, "textDocument/hover", "shift", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("shift(): int | undefined")
    );
    let source = examples.finish();
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/array-removal-doc-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    let input = directory.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

fn checked_std_example(source: &str) -> tsrust_compiler::project::ProjectAnalysis {
    use tsrust_compiler::project::{SourceId, SourceProvider, check_graph, load_project};
    struct Input<'a>(&'a str);
    impl SourceProvider for Input<'_> {
        fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
            Ok(SourceId(path.into()))
        }
        fn read(&self, _: &SourceId) -> Result<String, String> {
            Ok(self.0.into())
        }
    }
    check_graph(load_project(&Input(source), "/std-example.tsr").unwrap()).unwrap()
}

#[test]
fn array_at_docs_and_scoped_optional_views_are_consistent() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["Array"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "at")
        .unwrap();
    assert_eq!(entry["signature"], "at(index: int): Element | undefined");
    assert_eq!(entry["parameters"], json!(["index: int"]));
    for detail in [
        "exactly one int",
        "negative index",
        "out-of-range",
        "without changing",
        "dense homogeneous",
        "scoped read-only",
        "clone(view)",
        "Temporary owned receivers",
        "```tsrust",
    ] {
        assert!(
            entry["documentation"].as_str().unwrap().contains(detail),
            "{detail}"
        );
    }
    for (ty, initial, result) in [
        ("int[]", "[7]", "int | undefined"),
        ("number[]", "[-0.0]", "number | undefined"),
        ("boolean[]", "[true]", "boolean | undefined"),
        ("string[]", "[clone(\"x\")]", "string | undefined"),
        ("null[]", "[null]", "null | undefined"),
        ("undefined[]", "[undefined]", "undefined"),
        ("int[][]", "[[7]]", "int[] | undefined"),
        ("Item[]", "[{label:\"x\"}]", "Item | undefined"),
        ("unknownBox[]", "[{label:\"x\"}]", "unknownBox | undefined"),
    ] {
        let source = format!(
            "type Item={{label:string;}};type unknownBox={{label:string;}};function main(){{const values:{ty}={initial};const selected=values.at(-1);String(selected);console.log(values.length);}}"
        );
        tsrust_compiler::check(&source).unwrap();
        let hover = ask(&source, "textDocument/hover", "at(-1)", 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("at(index: int): {result}")),
            "{source}: {hover}"
        );
        let hover = ask(&source, "textDocument/hover", "selected);", 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("selected: {result}")),
            "{source}: {hover}"
        );
        let index = SourceIndex::new(&source);
        let mut position = at(&source, "at(-1)", 0);
        position.character += 3;
        let help = index
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(
            help["signatures"][0]["label"],
            format!("at(index: int): {result}")
        );
        assert_eq!(
            help["signatures"][0]["parameters"][0]["label"],
            "index: int"
        );
        let mut position = at(&source, "at(-1)", 0);
        position.character += 1;
        let items = index
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        let item = items
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["label"] == "at")
            .unwrap();
        assert_eq!(item["detail"], format!("at(index: int): {result}"));
    }
    let source = "function main(){const values:string[]|undefined=[clone(\"x\")];if(values!==undefined){const selected=values.at(0);if(selected!==undefined){console.log(selected.trim(),values.join());}}}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "trim", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("trim(): string")
    );
    let partial = "function main(){const values:int[][]=[[7]];const selected=values.at(0);if(selected!==undefined){selected.jo";
    let items = SourceIndex::new(partial)
        .request(
            "textDocument/completion",
            &json!({"position":Position::new(0,partial.encode_utf16().count() as u32)}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "join")
    );
    for source in [
        "function main(){const values:string[]=[\"x\"];const selected=values.at(0);selected.tr",
        "function main(){const values:int[]|undefined=[7];values.at",
    ] {
        let items = SourceIndex::new(source)
            .request(
                "textDocument/completion",
                &json!({"position":Position::new(0,source.encode_utf16().count() as u32)}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(items.as_array().unwrap().is_empty(), "{source}: {items}");
    }
    for arguments in ["", "1.0", "undefined", "\"0\"", "0,1"] {
        let source = format!("function main(){{const values:int[]=[7];values.at({arguments});}}");
        assert!(tsrust_compiler::check(&source).is_err(), "{source}");
    }
    // Earlier indexed receiver, field-chain and primitive method results remain distinct.
    let source = "type Holder={values:int[];};function main(){const holder:Holder={values:[7]};const selected=holder.values.at(0);String(selected);const nested:int[][]=[[8]];String(nested[0].at(-1));console.log((7).toString());}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "selected);", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("selected: int | undefined")
    );
    assert!(
        ask(source, "textDocument/hover", "at(-1)", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("at(index: int): int | undefined")
    );
    assert!(
        ask(source, "textDocument/hover", "toString", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("toString(radix?: int): string")
    );
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for example in entry["examples"].as_array().unwrap() {
        tsrust_compiler::check(&examples.push(example["source"].as_str().unwrap())).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }
    let source = examples.finish();
    assert_eq!(expected.lines().count(), 5);
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/array-at-doc-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    let input = directory.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn array_find_docs_predicates_and_scoped_optional_views_are_consistent() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for method in ["find", "findLast"] {
        let entry = catalog["Array"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["name"] == method)
            .unwrap();
        let callback = "callback: (value: Element, index: int, source: Element[]) => boolean";
        assert_eq!(
            entry["signature"],
            format!("{method}({callback}): Element | undefined")
        );
        assert_eq!(entry["parameters"], json!([callback]));
        for detail in [
            "exactly one reusable callback",
            "no default",
            "must be boolean",
            "callback may declare fewer parameters",
            "original indices",
            "not changed",
            "empty array invokes no callbacks",
            "read-only",
            "clone(view)",
            "Temporary owned receivers",
            "```tsrust",
        ] {
            assert!(
                entry["documentation"].as_str().unwrap().contains(detail),
                "{method}: {detail}"
            );
        }
        for (ty, initial, result) in [
            ("int[]", "[7]", "int | undefined"),
            ("number[]", "[-0.0]", "number | undefined"),
            ("boolean[]", "[true]", "boolean | undefined"),
            ("string[]", "[clone(\"x\")]", "string | undefined"),
            ("null[]", "[null]", "null | undefined"),
            ("undefined[]", "[undefined]", "undefined"),
            ("int[][]", "[[7]]", "int[] | undefined"),
            ("Item[]", "[{label:\"x\"}]", "Item | undefined"),
            ("unknownBox[]", "[{label:\"x\"}]", "unknownBox | undefined"),
        ] {
            let source = format!(
                "type Item={{label:string;}};type unknownBox={{label:string;}};function main(){{const values:{ty}={initial};const selected=values.{method}(():boolean=>{{return true;}});String(selected);console.log(values.length);}}"
            );
            tsrust_compiler::check(&source).unwrap();
            let label = format!(
                "{method}(callback: (value: {}, index: int, source: {ty}) => boolean): {result}",
                ty.strip_suffix("[]").unwrap()
            );
            let marker = format!("{method}(()");
            let hover = ask(&source, "textDocument/hover", &marker, 0);
            assert!(
                hover["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains(&label),
                "{source}: {hover}"
            );
            let hover = ask(&source, "textDocument/hover", "selected);", 0);
            assert!(
                hover["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("selected: {result}")),
                "{source}: {hover}"
            );
            let index = SourceIndex::new(&source);
            let mut position = at(&source, &marker, 0);
            position.character += method.len() as u32 + 1;
            let help = index
                .request(
                    "textDocument/signatureHelp",
                    &json!({"position":position}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert_eq!(help["signatures"][0]["label"], label);
            assert!(
                help["signatures"][0]["parameters"][0]["documentation"]["value"]
                    .as_str()
                    .unwrap()
                    .contains("strict-boolean")
            );
            let mut position = at(&source, &marker, 0);
            position.character += 2;
            let members = index
                .request(
                    "textDocument/completion",
                    &json!({"position":position}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert_eq!(
                members
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|item| item["label"] == method)
                    .unwrap()["detail"],
                label
            );
        }
        let source = format!(
            "function main(){{const values:string[]|undefined=[clone(\"x\")];if(values!==undefined){{const selected=values.{method}((value:string):boolean=>{{return value.length>0;}});if(selected!==undefined){{console.log(selected.trim(),values.join());}}}}}}"
        );
        tsrust_compiler::check(&source).unwrap();
        assert!(
            ask(&source, "textDocument/hover", "trim", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("trim(): string")
        );
        let partial = format!(
            "function main(){{const values:int[][]=[[7]];const selected=values.{method}(():boolean=>{{return true;}});if(selected!==undefined){{selected.jo"
        );
        let members = SourceIndex::new(&partial)
            .request(
                "textDocument/completion",
                &json!({"position":Position::new(0,partial.encode_utf16().count() as u32)}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            members
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "join")
        );
        for source in [
            format!(
                "function main(){{const values:string[]=[\"x\"];const selected=values.{method}(():boolean=>{{return true;}});selected.tr"
            ),
            format!(
                "function main(){{const values:int[]|undefined=[7];values.{}",
                &method[..2]
            ),
        ] {
            let members = SourceIndex::new(&source)
                .request(
                    "textDocument/completion",
                    &json!({"position":Position::new(0,source.encode_utf16().count() as u32)}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert!(
                members.as_array().unwrap().is_empty(),
                "{source}: {members}"
            );
        }
        for argument in [
            "",
            "true",
            "undefined",
            "():int=>{return 1;}",
            "(value:number):boolean=>{return true;}",
            "():boolean=>{return true;},():boolean=>{return true;}",
        ] {
            let source =
                format!("function main(){{const values:int[]=[7];values.{method}({argument});}}");
            assert!(tsrust_compiler::check(&source).is_err(), "{source}");
        }
        for example in entry["examples"].as_array().unwrap() {
            tsrust_compiler::check(&examples.push(example["source"].as_str().unwrap())).unwrap();
            expected.push_str(example["output"].as_str().unwrap());
        }
    }
    // Existing index-result and scalar-method inference must stay distinct.
    let source = "function main(){const values:int[]=[7];const index:int=values.findIndex((value:int):boolean=>{return true;});console.log(index.toString());const selected=values.at(0);String(selected);}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "index.toString", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("index: int")
    );
    assert!(
        ask(source, "textDocument/hover", "at(0)", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("at(index: int): int | undefined")
    );
    let source = examples.finish();
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/array-find-doc-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    let input = directory.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn array_unshift_docs_variadic_arguments_and_length_views_are_consistent() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["Array"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "unshift")
        .unwrap();
    assert_eq!(entry["signature"], "unshift(...values: Element[]): int");
    assert_eq!(entry["parameters"], json!(["...values: Element[]"]));
    for detail in [
        "zero or more",
        "new int length",
        "no arguments",
        "left to right",
        "before the outer insertion",
        "exactly",
        "not flattened",
        "must fit int",
        "read access",
        "mutable receiver",
        "without cloning",
        "```tsrust",
    ] {
        assert!(
            entry["documentation"].as_str().unwrap().contains(detail),
            "{detail}"
        );
    }
    for (ty, initial, argument) in [
        ("int[]", "[3]", "1"),
        ("number[]", "[-0.0]", "1.5"),
        ("boolean[]", "[false]", "true"),
        ("string[]", "[clone(\"x\")]", "clone(\"y\")"),
        ("null[]", "[null]", "null"),
        ("undefined[]", "[undefined]", "undefined"),
        ("int[][]", "[[3]]", "[1,2]"),
        ("Item[]", "[{label:\"x\"}]", "{label:clone(\"y\")}"),
        ("unknownBox[]", "[{label:\"x\"}]", "{label:clone(\"y\")}"),
    ] {
        let source = format!(
            "type Item={{label:string;}};type unknownBox={{label:string;}};function main(){{const values:{ty}={initial};const length=values.unshift({argument},{argument});console.log(length,values.length);}}"
        );
        tsrust_compiler::check(&source).unwrap();
        let label = format!("unshift(...values: {ty}): int");
        let hover = ask(&source, "textDocument/hover", "unshift(", 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&label),
            "{source}: {hover}"
        );
        assert!(
            ask(&source, "textDocument/hover", "length,values", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("length: int")
        );
        let index = SourceIndex::new(&source);
        let mut position = at(&source, "unshift(", 0);
        position.character += 8;
        let help = index
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(help["signatures"][0]["label"], label);
        assert_eq!(
            help["signatures"][0]["parameters"][0]["label"],
            format!("...values: {ty}")
        );
        assert!(
            help["signatures"][0]["parameters"][0]["documentation"]["value"]
                .as_str()
                .unwrap()
                .contains("not flattened")
        );
        let mut position = at(&source, "unshift(", 0);
        position.character += 2;
        let members = index
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(
            members
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["label"] == "unshift")
                .unwrap()["detail"],
            label
        );
        let source = format!(
            "function size(values:{ty}):int{{return values.unshift();}}type Item={{label:string;}};type unknownBox={{label:string;}};function main(){{const values:{ty}={initial};console.log(size(values),values.length);}}"
        );
        tsrust_compiler::check(&source).unwrap();
    }
    let source = "function main(){const values:int[]|undefined=[3];if(values!==undefined){const length=values.unshift(1,2);console.log(length,values.length);}}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "unshift(", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("unshift(...values: int[]): int")
    );
    let partial = "function main(){const values:int[]|undefined=[3];values.un";
    let members = SourceIndex::new(partial)
        .request(
            "textDocument/completion",
            &json!({"position":Position::new(0,partial.encode_utf16().count() as u32)}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(members.as_array().unwrap().is_empty());
    for argument in ["\"x\"", "1.5", "null", "undefined", "[1]", "true"] {
        let source =
            format!("function main(){{const values:int[]=[3];values.unshift({argument});}}");
        assert!(tsrust_compiler::check(&source).is_err(), "{source}");
    }
    let source = "function main(){const nested:int[][]=[[1]];{const selected=nested.at(0);if(selected!==undefined){console.log(selected.unshift());}}console.log(nested[0].unshift(2));const values:int[]=[3];values.push(4);console.log(values.unshift());}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "unshift()", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("unshift(...values: int[]): int")
    );
    assert!(
        ask(source, "textDocument/hover", "push(4)", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("push(value: int): void")
    );
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for example in entry["examples"].as_array().unwrap() {
        tsrust_compiler::check(&examples.push(example["source"].as_str().unwrap())).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }
    let source = examples.finish();
    assert_eq!(expected.lines().count(), 6);
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/array-unshift-doc-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    let input = directory.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn array_splice_docs_defaults_rest_parameters_and_removed_result_types_are_consistent() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["Array"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "splice")
        .unwrap();
    assert_eq!(
        entry["signature"],
        "splice(start?: int, deleteCount?: int, ...items: Element[]): Element[]"
    );
    assert_eq!(
        entry["parameters"],
        json!(["start?: int", "deleteCount?: int", "...items: Element[]"])
    );
    for detail in [
        "independent, mutable array",
        "no arguments",
        "only start",
        "zero removes nothing",
        "post-argument current length",
        "signed 64-bit",
        "Explicit undefined",
        "without flattening",
        "Read access",
        "mutable receiver",
        "without implicit cloning",
        "```tsrust",
    ] {
        assert!(
            entry["documentation"].as_str().unwrap().contains(detail),
            "{detail}"
        );
    }
    for (ty, initial, argument) in [
        ("int[]", "[3]", "1"),
        ("number[]", "[-0.0]", "1.5"),
        ("boolean[]", "[false]", "true"),
        ("string[]", "[clone(\"x\")]", "clone(\"y\")"),
        ("null[]", "[null]", "null"),
        ("undefined[]", "[undefined]", "undefined"),
        ("int[][]", "[[3]]", "[1,2]"),
        ("Item[]", "[{label:\"x\"}]", "{label:clone(\"y\")}"),
        ("unknownBox[]", "[{label:\"x\"}]", "{label:clone(\"y\")}"),
    ] {
        let source = format!(
            "type Item={{label:string;}};type unknownBox={{label:string;}};function main(){{const values:{ty}={initial};const removed=values.splice(0,1,{argument});console.log(removed.length,values.length);}}"
        );
        tsrust_compiler::check(&source).unwrap();
        let label = format!("splice(start?: int, deleteCount?: int, ...items: {ty}): {ty}");
        assert!(
            ask(&source, "textDocument/hover", "splice(", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&label)
        );
        assert!(
            ask(&source, "textDocument/hover", "removed.length", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("removed: {ty}"))
        );
        let index = SourceIndex::new(&source);
        for (offset, active, parameter) in [
            (7, 0, "start?: int".to_owned()),
            (9, 1, "deleteCount?: int".to_owned()),
            (11, 2, format!("...items: {ty}")),
        ] {
            let mut position = at(&source, "splice(", 0);
            position.character += offset;
            let help = index
                .request(
                    "textDocument/signatureHelp",
                    &json!({"position":position}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert_eq!(help["signatures"][0]["label"], label);
            assert_eq!(help["activeParameter"], active);
            assert_eq!(
                help["signatures"][0]["parameters"][active as usize]["label"],
                parameter
            );
        }
        let mut position = at(&source, "splice(", 0);
        position.character += 2;
        let members = index
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(
            members
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["label"] == "splice")
                .unwrap()["detail"],
            label
        );
        let source = format!(
            "type Item={{label:string;}};type unknownBox={{label:string;}};function size(values:{ty}):int{{const removed=values.splice();return removed.length;}}function main(){{const values:{ty}={initial};console.log(size(values),values.length);}}"
        );
        tsrust_compiler::check(&source).unwrap();
    }
    let source = "function main(){const values:int[]|undefined=[3];if(values!==undefined){const removed=values.splice(0);console.log(removed.join());}}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "join()", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("join(separator?: string): string")
    );
    let partial = "function main(){const values:int[]|undefined=[3];values.sp";
    let members = SourceIndex::new(partial)
        .request(
            "textDocument/completion",
            &json!({"position":Position::new(0,partial.encode_utf16().count() as u32)}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(members.as_array().unwrap().is_empty());
    for arguments in [
        "undefined",
        "0,undefined",
        "1.0",
        "0,1.0",
        "0,0,\"x\"",
        "0,0,[1]",
        "0,0,null",
    ] {
        let source =
            format!("function main(){{const values:int[]=[3];values.splice({arguments});}}");
        assert!(tsrust_compiler::check(&source).is_err(), "{source}");
    }
    let source = "function main(){const nested:int[][]=[[1]];{const selected=nested.at(0);if(selected!==undefined){const removed=selected.splice();removed.push(9);console.log(selected.length,removed.length);}}const values:int[]=[3];const removed=values.splice();console.log(values.unshift(),removed.length);}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "splice()", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("splice(start?: int, deleteCount?: int, ...items: int[]): int[]")
    );
    assert!(
        ask(source, "textDocument/hover", "unshift()", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("unshift(...values: int[]): int")
    );
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for example in entry["examples"].as_array().unwrap() {
        tsrust_compiler::check(&examples.push(example["source"].as_str().unwrap())).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }
    let source = examples.finish();
    assert_eq!(expected.lines().count(), 10);
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/array-splice-doc-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    let input = directory.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn array_sorting_docs_defaults_comparators_and_owned_results_are_consistent() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for method in ["sort", "toSorted"] {
        let entry = catalog["Array"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["name"] == method)
            .unwrap();
        let result = if method == "sort" {
            "void"
        } else {
            "Element[]"
        };
        assert_eq!(
            entry["signature"],
            format!(
                "{method}(compare?: (left: Element, right: Element) => int | number): {result}"
            )
        );
        for detail in [
            "stable",
            "may declare fewer parameters",
            "int or number",
            "NaN",
            "UTF-8",
            "even when empty",
            "invocation order and count",
            "```tsrust",
        ] {
            assert!(
                entry["documentation"].as_str().unwrap().contains(detail),
                "{method}: {detail}"
            );
        }
        for (ty, initial) in [
            ("int[]", "[2,1]"),
            ("number[]", "[2.0,-0.0]"),
            ("boolean[]", "[true,false]"),
            ("string[]", "[clone(\"x\")]"),
            ("null[]", "[null]"),
            ("undefined[]", "[undefined]"),
            ("int[][]", "[[2]]"),
            ("Item[]", "[{key:2}]"),
            ("unknownBox[]", "[{key:2}]"),
        ] {
            let element = ty.strip_suffix("[]").unwrap();
            let source = format!(
                "type Item={{key:int;}};type unknownBox={{key:int;}};function main(){{const values:{ty}={initial};values.{method}((left:{element},right:{element}):int=>{{return 0;}});}}"
            );
            tsrust_compiler::check(&source).unwrap();
            let optional = if matches!(
                element,
                "int" | "number" | "boolean" | "string" | "null" | "undefined"
            ) {
                "?"
            } else {
                ""
            };
            let result = if method == "sort" { "void" } else { ty };
            let label = format!(
                "{method}(compare{optional}: (left: {element}, right: {element}) => int | number): {result}"
            );
            assert!(
                ask(&source, "textDocument/hover", &format!("{method}("), 0)["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .contains(&label),
                "{source}"
            );
            let mut position = at(&source, &format!("{method}("), 0);
            position.character += method.len() as u32 + 1;
            let help = SourceIndex::new(&source)
                .request(
                    "textDocument/signatureHelp",
                    &json!({"position":position}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert_eq!(help["signatures"][0]["label"], label);
            assert_eq!(help["activeParameter"], 0);
            assert_eq!(
                help["signatures"][0]["parameters"][0]["label"],
                format!("compare{optional}: (left: {element}, right: {element}) => int | number")
            );
            let source = format!(
                "type Item={{key:int;}};type unknownBox={{key:int;}};function main(){{const values:{ty}={initial};values.{method}();}}"
            );
            if optional.is_empty() {
                assert!(tsrust_compiler::check(&source).is_err());
            } else {
                tsrust_compiler::check(&source).unwrap();
            }
        }
        for example in entry["examples"].as_array().unwrap() {
            tsrust_compiler::check(&examples.push(example["source"].as_str().unwrap())).unwrap();
            expected.push_str(example["output"].as_str().unwrap());
        }
        for arguments in [
            "undefined",
            "0",
            "():boolean=>{return true;}",
            "():int|null=>{return null;}",
            "():string=>{return \"x\";}",
            "():int=>{return 0;},():int=>{return 0;}",
            "(left:int,right:int,extra:int):int=>{return 0;}",
        ] {
            let source = format!(
                "function main(){{const values:int[]=[2,1];values.{method}({arguments});}}"
            );
            assert!(tsrust_compiler::check(&source).is_err(), "{source}");
        }
        for callback in [
            "():int=>{return 0;}",
            "(left:int):number=>{return Number.NaN;}",
            "(left:int,right:int):number=>{return -0.0;}",
        ] {
            tsrust_compiler::check(&format!(
                "function main(){{const values:int[]=[2,1];values.{method}({callback});}}"
            ))
            .unwrap();
        }
    }
    let source = "function main(){const values:int[]=[2,1];const sorted=values.toSorted((left:int,right:int):int=>{return values.length*(left-right);});console.log(sorted.join(),values.join());}";
    tsrust_compiler::check(source).unwrap();
    let source = "function main(){const values:int[]=[2,1];values.sort((left:int,right:int):int=>{return values.length*(left-right);});}";
    assert!(tsrust_compiler::check(source).is_err());
    let source = "function ordered(values:string[]):string[]{return values.toSorted();}function main(){const values:string[]=[clone(\"b\"),clone(\"a\")];const sorted=ordered(values);console.log(sorted.join(),values.join());}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "sorted.join", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("sorted: string[]")
    );
    let source = "function main(){const values:int[]|undefined=[2,1];if(values!==undefined){const sorted=values.toSorted();console.log(sorted.join());values.sort();}}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "join()", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("join(separator?: string): string")
    );
    let partial = "function main(){const values:int[]|undefined=[2,1];values.toS";
    let members = SourceIndex::new(partial)
        .request(
            "textDocument/completion",
            &json!({"position":Position::new(0,partial.encode_utf16().count() as u32)}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(members.as_array().unwrap().is_empty());
    let source = "function main(){const values:int[]=[2,1];values.sort();console.log(values.unshift(),values.splice().length,values.join());}";
    tsrust_compiler::check(source).unwrap();
    let source = examples.finish();
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/array-sorting-doc-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    let input = directory.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn generic_declarations_and_nested_argument_annotations_remain_discoverable() {
    let source = "type Pair<A,B>={left:A;right:B;};function inspect(value:Pair<int,Pair<string,int>>):int{return value.left;}function main(){const value:Pair<int,Pair<string,int>>={left:7,right:{left:'Pat',right:8}};console.log(inspect(value));}";
    assert!(tsrust_compiler::check(source).is_ok());
    let declaration = ask(source, "textDocument/hover", "Pair<", 0);
    assert!(declaration["contents"]["value"].as_str().unwrap().contains("type Pair<A,B>"));
    let parameter = ask(source, "textDocument/hover", "value:", 0);
    assert!(parameter["contents"]["value"].as_str().unwrap().contains("Pair<int,Pair<string,int>>"));
    let local = ask(source, "textDocument/hover", "value:", 1);
    assert!(local["contents"]["value"].as_str().unwrap().contains("Pair<int,Pair<string,int>>"));
    let incomplete = "type Box<T>={value:T;};function identity<T>(value:T):T{return value;}function main(){";
    let outline = SourceIndex::new(incomplete).request("textDocument/documentSymbol", &json!({}), "file:///test.tsr").unwrap();
    assert!(outline.as_array().unwrap().iter().any(|symbol| symbol["name"] == "identity"));
}

#[test]
fn concrete_generic_fields_supply_hover_completion_and_chained_methods() {
    let source = "type Box<T>={value:T;};function main(){const text:Box<string>={value:'Pat'};const nested:Box<Box<string>>={value:{value:'Sam'}};console.log(text.value.trim(),nested.value.value.toLowerCase());}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source,"textDocument/hover","value.trim",0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("value: string"));
    let chained = ask(source,"textDocument/hover","toLowerCase",0);
    assert!(chained["contents"]["value"].as_str().unwrap().contains("toLowerCase"));
    let incomplete = "type Box<T>={value:T;};function main(){const text:Box<string>={value:'Pat'};text.";
    let mut position = at(incomplete,"text.",0); position.character += 5;
    let members = SourceIndex::new(incomplete).request("textDocument/completion",&json!({"position":position}),"file:///test.tsr").unwrap();
    assert!(members.as_array().unwrap().iter().any(|member| member["label"] == "value" && member["detail"] == "value: string"));
}

#[test]
fn grouped_and_reordered_generic_arguments_keep_concrete_members() {
    let source = "type Box<T>={value:T;};function main(){const text:Box<(string)>={value:'Pat'};const maybe:Box<null|(int)>={value:7};console.log(text.value.trim(),String(maybe.value));}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source,"textDocument/hover","value.trim",0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("value: string"),"{hover}");
    let hover = ask(source,"textDocument/hover","value));",0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("value: int | null"),"{hover}");
}

#[test]
fn array_flattening_docs_and_checked_result_types_are_consistent() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for method in ["flat", "flatMap"] {
        let entry = catalog["Array"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["name"] == method)
            .unwrap();
        assert!(
            entry["signature"]
                .as_str()
                .unwrap()
                .ends_with(": Flattened[]")
        );
        assert_eq!(entry["parameters"].as_array().unwrap().len(), 1);
        for detail in [
            "independently mutable",
            "dense homogeneous",
            "once",
            "```tsrust",
        ] {
            assert!(
                entry["documentation"].as_str().unwrap().contains(detail),
                "{method}: {detail}"
            );
        }
        for example in entry["examples"].as_array().unwrap() {
            tsrust_compiler::check(&examples.push(example["source"].as_str().unwrap())).unwrap();
            expected.push_str(example["output"].as_str().unwrap());
        }
    }
    for (operation, result) in [
        ("flat()", "int[][]"),
        ("flat(0)", "int[][][]"),
        ("flat(-1)", "int[][][]"),
        ("flat(2)", "int[]"),
        ("flat(9223372036854775807)", "int[]"),
    ] {
        let source = format!(
            "function main(){{const values:int[][][]=[[[1]],[[2]]];const flattened=values.{operation};console.log(flattened.length);}}"
        );
        let analysis = tsrust_compiler::check(&source).unwrap();
        let facts = analysis.checked_type_facts();
        assert_eq!(facts.projections, analysis.projections());
        assert!(facts.method_results.iter().any(|fact| fact.ty == result
            && source[fact.span.start..fact.span.end] == format!("values.{operation}")));
        let label = format!("flat(depth?: int): {result}");
        assert!(
            ask(&source, "textDocument/hover", operation, 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&label)
        );
        assert!(
            ask(&source, "textDocument/hover", "flattened.length", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("flattened: {result}"))
        );
        let mut position = at(&source, operation, 0);
        position.character += 5;
        let help = SourceIndex::new(&source)
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(help["signatures"][0]["label"], label);
        assert_eq!(
            help["signatures"][0]["parameters"][0]["label"],
            "depth?: int"
        );
        assert!(
            help["signatures"][0]["parameters"][0]["documentation"]["value"]
                .as_str()
                .unwrap()
                .contains("signed integer literal")
        );
    }
    for (callback, result) in [
        ("(value:int):number=>{return Number(value);}", "number[]"),
        ("(value:int):int[]=>{return [value,value];}", "int[]"),
        ("(value:int):int[][]=>{return [[value]];}", "int[][]"),
        ("(value:int):Item=>{return {text:clone(\"é\")};}", "Item[]"),
    ] {
        let source = format!(
            "type Item={{text:string;}};function main(){{const values:int[]=[1,2];const mapped=values.flatMap({callback});console.log(mapped.length);}}"
        );
        let analysis = tsrust_compiler::check(&source).unwrap();
        assert!(
            analysis
                .checked_type_facts()
                .method_results
                .iter()
                .any(|fact| fact.ty == result)
        );
        let label = format!(
            "flatMap(callback: (value: int, index: int, source: int[]) => Result): {result}"
        );
        assert!(
            ask(&source, "textDocument/hover", "flatMap(", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&label)
        );
        assert!(
            ask(&source, "textDocument/hover", "mapped.length", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("mapped: {result}"))
        );
        let mut position = at(&source, "flatMap(", 0);
        position.character += 8;
        let help = SourceIndex::new(&source)
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(help["signatures"][0]["label"], label);
        assert!(
            help["signatures"][0]["parameters"][0]["documentation"]["value"]
                .as_str()
                .unwrap()
                .contains("one fixed concrete")
        );
    }
    let source = "function flatten<T>(values:T[]):T[]{return values.flat();}function main(){const integers=flatten<int>([1]);const strings=flatten<string>([clone(\"x\")]);console.log(integers.length,strings.length);}";
    let analysis = tsrust_compiler::check(source).unwrap();
    let span = analysis
        .checked_type_facts()
        .method_results
        .iter()
        .find(|fact| fact.ty == "int[]")
        .unwrap()
        .span;
    assert!(
        analysis
            .checked_type_facts()
            .method_results
            .iter()
            .any(|fact| fact.span == span && fact.ty == "string[]")
    );
    assert!(
        ask(source, "textDocument/hover", "flat()", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Flattened[]")
    );
    let source = "function main(){const values:int[][]=[[1]];console.log(values.flat().join());}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "join()", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("join(separator?: string): string")
    );
    let source = "function main(){const values:int[][]|null=[[1]];if(values!==null){const flattened=values.flat();console.log(flattened.join());}}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "flattened.join", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("flattened: int[]")
    );
    for (source, available) in [
        ("function main(){const values:int[][]=[[1]];values.fl", true),
        (
            "function main(){const values:int[][]|null=[[1]];values.fl",
            false,
        ),
        (
            "function main(){const values:int[][]|null=[[1]];if(values!==null){values.fl",
            true,
        ),
    ] {
        let items = SourceIndex::new(source)
            .request(
                "textDocument/completion",
                &json!({"position":Position::new(0,source.encode_utf16().count() as u32)}),
                "file:///test.tsr",
            )
            .unwrap();
        for method in ["flat", "flatMap"] {
            assert_eq!(
                items
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|item| item["label"] == method),
                available,
                "{source}"
            );
        }
    }
    for arguments in ["1.0", "undefined", "1,2", "1+1", "depth"] {
        assert!(tsrust_compiler::check(&format!("function main(){{const values:int[][]=[[1]];const depth:int=1;values.flat({arguments});}}")).is_err(),"{arguments}");
    }
    for arguments in [
        "",
        "undefined",
        "(value:int):void=>{}",
        "(value:int,index:int,source:int[],extra:int):int=>{return value;}",
        "():int=>{return 1;},():int=>{return 2;}",
    ] {
        assert!(
            tsrust_compiler::check(&format!(
                "function main(){{const values:int[]=[1];values.flatMap({arguments});}}"
            ))
            .is_err(),
            "{arguments}"
        );
    }
    let source = "function main(){const values:int[]=[1,2];const depth:int=1;const copied=values.flat(depth);console.log(copied.join(),String(values.at(0)),values.toSorted().length);}";
    tsrust_compiler::check(source).unwrap();
    let source = examples.finish();
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/array-flattening-doc-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    let input = directory.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn array_of_docs_generic_calls_and_checked_constructor_results_are_consistent() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["ArrayConstructor"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "of")
        .unwrap();
    assert_eq!(entry["signature"], "of<T>(...items: T[]): T[]");
    assert_eq!(entry["parameters"], json!(["...items: T[]"]));
    for detail in [
        "original order",
        "zero or more",
        "empty call without",
        "left to right",
        "rank is at most 128",
        "Homogeneous null[]",
        "explicit clone",
        "```tsrust",
    ] {
        assert!(
            entry["documentation"].as_str().unwrap().contains(detail),
            "{detail}"
        );
    }
    for (element, items) in [
        ("int", "1,2"),
        ("number", "1,2.5"),
        ("boolean", "true,false"),
        ("string", "clone(\"é\")"),
        ("int[]", "[1],[2]"),
        ("Item", "{text:clone(\"é\")}"),
        ("null", "null"),
        ("undefined", "undefined"),
        ("unknownBox", "{value:7}"),
        ("Box<int>", "{value:7}"),
    ] {
        let result = format!("{element}[]");
        let operation = format!("Array.of<{element}>({items})");
        let source = format!(
            "type Item={{text:string;}};type unknownBox={{value:int;}};type Box<T>={{value:T;}};function main(){{const values={operation};console.log(values.length);}}"
        );
        let analysis = tsrust_compiler::check(&source).unwrap();
        assert!(
            analysis
                .checked_type_facts()
                .method_results
                .iter()
                .any(|fact| fact.ty == result
                    && source[fact.span.start..fact.span.end] == operation)
        );
        let label = format!("of<{element}>(...items: {result}): {result}");
        assert!(
            ask(&source, "textDocument/hover", "of<", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&label),
            "{source}"
        );
        assert!(
            ask(&source, "textDocument/hover", "values.length", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("values: {result}")),
            "{source}"
        );
        let mut position = at(&source, &format!(">({items}"), 0);
        position.character += 2;
        let help = SourceIndex::new(&source)
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(help["signatures"][0]["label"], label);
        assert_eq!(
            help["signatures"][0]["parameters"][0]["label"],
            format!("...items: {result}")
        );
        assert!(
            help["signatures"][0]["parameters"][0]["documentation"]["value"]
                .as_str()
                .unwrap()
                .contains("one exact supported element type")
        );
    }
    for (expression, result) in [
        ("Array.of(1,2)", "int[]"),
        ("Array.of<int>()", "int[]"),
        ("Array.of<int[]>()", "int[][]"),
        ("Array.of<int[][]>([[1]])", "int[][][]"),
    ] {
        let source =
            format!("function main(){{const values={expression};console.log(values.length);}}");
        tsrust_compiler::check(&source).unwrap();
        assert!(
            ask(&source, "textDocument/hover", "values.length", 0)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("values: {result}"))
        );
    }
    let source = "type Box<T>={value:T;};function main(){const values=Array.of<Box<int>>({value:7});console.log(values[0].value);}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "value);", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("value: int")
    );
    let source = r#"type Box<T>={value:T;};function main(){const explicit:Box<int>={value:9};const integers=Array.of<Box<int>>({value:7});const texts=Array.of<Box<string>>({value:clone("é")});console.log(integers[0].value,texts[0].value,explicit.value);}"#;
    tsrust_compiler::check(source).unwrap();
    let index = SourceIndex::new(source);
    for (needle, expected) in [
        ("integers[0].value", "value: int"),
        ("texts[0].value", "value: string"),
    ] {
        let mut position = at(source, needle, 0);
        position.character += needle.find(".value").unwrap() as u32 + 2;
        let items = index
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        let fields = items
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["label"] == "value")
            .collect::<Vec<_>>();
        assert_eq!(
            fields.len(),
            1,
            "repeated discovery must not duplicate {needle}"
        );
        assert_eq!(fields[0]["detail"], expected);
    }
    let source = "function main(){const values:number[]=Array.of(1,2.5);console.log(Array.of<int>(1,2).join());}";
    tsrust_compiler::check(source).unwrap();
    assert!(
        ask(source, "textDocument/hover", "of(1", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("of<number>(...items: number[]): number[]")
    );
    assert!(
        ask(source, "textDocument/hover", "join()", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("join(separator?: string): string")
    );
    let source = "function pack<T>(value:T):T[]{return Array.of<T>(value);}function main(){const integers=pack<int>(1);const strings=pack<string>(clone(\"x\"));console.log(integers.length,strings.length);}";
    let analysis = tsrust_compiler::check(source).unwrap();
    let facts = analysis.checked_type_facts();
    let span = facts
        .method_results
        .iter()
        .find(|fact| fact.ty == "int[]")
        .unwrap()
        .span;
    assert!(
        facts
            .method_results
            .iter()
            .any(|fact| fact.span == span && fact.ty == "string[]")
    );
    assert!(
        ask(source, "textDocument/hover", "of<T>", 0)["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("of<T>(...items: T[]): T[]")
    );
    for partial in ["function main(){Array.", "function main(){Array.o"] {
        let items = SourceIndex::new(partial)
            .request(
                "textDocument/completion",
                &json!({"position":Position::new(0,partial.encode_utf16().count() as u32)}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "of" && item["detail"] == "of<T>(...items: T[]): T[]")
        );
    }
    for partial in [
        "function main(){Array.of(",
        "function main(){Array.of<int[]>(",
    ] {
        let help = SourceIndex::new(partial)
            .request(
                "textDocument/signatureHelp",
                &json!({"position":Position::new(0,partial.encode_utf16().count() as u32)}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(help["signatures"][0]["label"], "of<T>(...items: T[]): T[]");
    }
    for body in [
        "Array.of();",
        "Array.of().flat();",
        "Array.isArray(Array.of());",
        "Array.of<int,number>(1);",
        "Array.of<int>(1.0);",
        "const integer:int=1;Array.of<number>(integer);",
        "const source:string[]=[clone(\"x\")];Array.of(source[0]);",
    ] {
        assert!(
            tsrust_compiler::check(&format!("function main(){{{body}}}")).is_err(),
            "{body}"
        );
    }
    let source = "function main(){const values=Array.of<int>(1,2);console.log(values.flat().join(),Array.isArray(values),values.at(0)===undefined,values.toSorted().length);}";
    tsrust_compiler::check(source).unwrap();
    let mut examples = CatalogExampleSource::default();
    let mut expected = String::new();
    for example in entry["examples"].as_array().unwrap() {
        tsrust_compiler::check(&examples.push(example["source"].as_str().unwrap())).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }
    let source = examples.finish();
    assert_eq!(entry["examples"].as_array().unwrap().len(), 7);
    assert_eq!(expected.lines().count(), 7);
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/array-of-doc-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    let input = directory.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn checked_generic_calls_show_concrete_hover_help_and_result_chains() {
    let source="/** Return the supplied value. */ function identity<T>(value:T):T{return value;}function main(){const n=identity(7);const text=identity<string>('Pat');console.log(identity('Sam').trim(),text.toLowerCase());}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover=ask(source,"textDocument/hover","identity(7)",0);
    let docs=hover["contents"]["value"].as_str().unwrap();
    assert!(docs.contains("function identity(value: int): int"),"{hover}");
    assert!(docs.contains("Return the supplied value."));
    assert!(!docs.contains("specialization#"));
    let help=ask(source,"textDocument/signatureHelp","7)",0);
    assert_eq!(help["signatures"][0]["label"],"function identity(value: int): int");
    let explicit=ask(source,"textDocument/hover","identity<string>",0);
    assert!(explicit["contents"]["value"].as_str().unwrap().contains("function identity(value: string): string"));
    let explicit_help=ask(source,"textDocument/signatureHelp","'Pat')",0);
    assert_eq!(explicit_help["signatures"][0]["label"],"function identity(value: string): string");
    assert!(!ask(source,"textDocument/hover","trim()",0).is_null());
    assert!(!ask(source,"textDocument/hover","toLowerCase()",0).is_null());
}

#[test]
fn shared_generic_source_spans_keep_template_help_instead_of_picking_a_specialization() {
    let source="function identity<T>(value:T):T{return value;}function forward<T>(value:T):T{return identity(value);}function main(){forward(7);forward('Pat');}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover=ask(source,"textDocument/hover","identity(value)",0);
    let value=hover["contents"]["value"].as_str().unwrap();
    assert!(value.contains("T"),"{hover}");
    assert!(!value.contains("value: int"));
    assert!(!value.contains("value: string"));
}

#[test]
fn generic_callback_hover_preserves_declared_names_and_read_access() {
    let source = "type User={name:string;};function invoke<T>(action:(person:ReadOnly<T>)=>int,value:T):int{return action(value);}function main(){const action=(user:User):int=>{return user.name.length;};const saved=invoke(action,{name:'Pat'});console.log(saved);}";
    tsrust_compiler::check(source).unwrap();
    let hover = ask(source, "textDocument/hover", "invoke(action", 0);
    let value = hover["contents"]["value"].as_str().unwrap();
    assert!(value.contains("person: ReadOnly<User>"), "{hover}");
    assert!(!value.contains("arg0"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "action,{name", 0);
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("person: ReadOnly<User>"), "{help}");
}

#[test]
fn generic_owned_callback_hover_preserves_declared_contract() {
    let source = "type User={name:string;};function invoke<T>(action:Function<T, [person:Owned<T>]>,value:T):T{return action(value);}function main(){const action=(user:User):User=>{return user;};const saved=invoke(action,{name:'Pat'});console.log(saved.name);}";
    tsrust_compiler::check(source).unwrap();
    let hover = ask(source, "textDocument/hover", "invoke(action", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Function<User, [person: Owned<User>]>"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "action,{name", 0);
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("Function<User, [person: Owned<User>]>"), "{help}");
}

#[test]
fn nullish_coalescing_checked_results_drive_editor_views_conservatively() {
    let examples: &[(&str, &str)] = &[
        (
            r#"const zero:int|null=0;const flag:boolean|undefined=false;const text:string|null="";console.log(zero??9,flag??true,(text??"fallback").length);"#,
            "0 false 0\n",
        ),
        (
            r#"const input:string|null="é".repeat(1);const selected=input??"fallback";const independent=clone(selected);console.log(selected,independent);"#,
            "é é\n",
        ),
        (
            r#"const input:int[]|undefined=undefined;const selected=input??[1,2];console.log(selected.join("|"),selected.length);"#,
            "1|2 2\n",
        ),
        (
            r#"type Box<T>={value:T;};const input:Box<int>|null={value:7};const selected=input??{value:0};console.log(selected.value);"#,
            "7\n",
        ),
        (
            r#"const selected=null??((value:int):int=>{return value+1;});console.log(selected(4));"#,
            "5\n",
        ),
        (
            r#"const input="once".repeat(1);const selected=null??(():string=>{return input;});console.log(selected());"#,
            "once\n",
        ),
    ];
    let mut aggregated = CatalogExampleSource::default();
    let mut expected = String::new();
    for (source, output) in examples {
        tsrust_compiler::check(&aggregated.push(source)).unwrap();
        expected.push_str(output);
    }
    let source = aggregated.finish();
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/nullish-coalescing-language-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    let input = directory.join("examples.rs");
    std::fs::write(&input, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "-Awarnings",
                "-C",
                &format!("opt-level={level}"),
            ])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, expected.as_bytes());
    }
    assert_eq!(examples.len(), 6);
    assert_eq!(expected.lines().count(), 6);
    for (source, needle, ty) in [
        (
            r#"function main(){const input:int|null=null;const selected=input??7;console.log(selected.toString());}"#,
            "selected.toString",
            "selected: int",
        ),
        (
            r#"function main(){const input:number|null=2.5;const selected:number=input??0;console.log(selected.toFixed(1));}"#,
            "selected.toFixed",
            "selected: number",
        ),
        (
            r#"function main(){const input:string|null="é";const selected=input??"fallback";console.log(selected.repeat(1));}"#,
            "selected.repeat",
            "selected: string",
        ),
        (
            r#"function main(){const input:int[]|undefined=undefined;const selected=input??[1,2];console.log(selected.join());}"#,
            "selected.join",
            "selected: int[]",
        ),
        (
            r#"function main(){const input:int|null|undefined=undefined;const selected=input??null;console.log(typeof selected);}"#,
            "selected);",
            "selected: int | null",
        ),
        (
            r#"function main(){const selected=2.5??"unused";console.log(selected.toFixed(1));}"#,
            "selected.toFixed",
            "selected: number",
        ),
    ] {
        tsrust_compiler::check(source).unwrap();
        let hover = ask(source, "textDocument/hover", needle, 0);
        assert!(
            hover["contents"]["value"].as_str().unwrap().contains(ty),
            "{source}: {hover}"
        );
    }
    for (source, method, signature) in [
        (
            r#"function main(){const input:string|null=null;console.log((input??"fallback").repeat(2));}"#,
            "repeat(",
            "repeat(count: int): string",
        ),
        (
            r#"function main(){const input:int[]|null=null;console.log((input??[1,2]).join("|"));}"#,
            "join(",
            "join(separator?: string): string",
        ),
        (
            r#"type Box<T>={value:T;};function main(){const input:Box<int>|null={value:7};const selected=input??{value:0};console.log(selected.value);}"#,
            "value);",
            "value: int",
        ),
        (
            r#"type Box<T>={value:T;};function main(){const input:Box<int>|null={value:7};console.log((input??{value:0}).value);}"#,
            "value);",
            "value: int",
        ),
        (
            r#"type Holder={value:string|null;};function main(){const holder:Holder={value:null};console.log((holder.value??"x").repeat(2));}"#,
            "repeat(",
            "repeat(count: int): string",
        ),
    ] {
        tsrust_compiler::check(source).unwrap();
        let hover = ask(source, "textDocument/hover", method, 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(signature),
            "{source}: {hover}"
        );
        if method != "value);" {
            let items = ask(source, "textDocument/completion", method, 0);
            assert!(
                items
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|item| item["label"] == method.trim_end_matches('('))
            );
            let mut position = at(source, method, 0);
            position.character += method.len() as u32;
            let help = SourceIndex::new(source)
                .request(
                    "textDocument/signatureHelp",
                    &json!({"position":position}),
                    "file:///test.tsr",
                )
                .unwrap();
            assert_eq!(help["signatures"][0]["label"], signature);
        }
    }
    for (source, shape) in [
        (
            r#"function main(){const callable=(value:int):int=>{return value+1;};const selected=callable??"unused";console.log(selected(4));}"#,
            "Function<int, [value: int]>",
        ),
        (
            r#"function main(){const selected=null??((value:int):int=>{return value+1;});console.log(selected(4));}"#,
            "Function<int, [value: int]>",
        ),
        (
            r#"function main(){const input="once".repeat(1);const selected=null??(():string=>{return input;});console.log(selected());}"#,
            "Once<Function<string>>",
        ),
    ] {
        let analysis = tsrust_compiler::check(source).unwrap();
        assert!(
            analysis
                .checked_type_facts()
                .method_results
                .iter()
                .any(|fact| fact.ty == shape)
        );
        let expression_start = source.find("=null??").map(|start| start + 1);
        if let Some(start) = expression_start {
            let end = source.find(";console.log").unwrap();
            assert!(
                analysis
                    .checked_type_facts()
                    .method_results
                    .iter()
                    .any(|fact| {
                        fact.span.start == start && fact.span.end == end && fact.ty == shape
                    }),
                "The selected callable fact must include the grouped RHS."
            );
        }
        let hover = ask(source, "textDocument/hover", "selected(", 0);
        assert!(
            hover["contents"]["value"].as_str().unwrap().contains(shape),
            "{hover}"
        );
        let items = ask(source, "textDocument/completion", "selected(", 0);
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "selected")
        );
    }
    let source = r#"function select<T>(value:T):T{const selected=value??null;return selected;}function main(){console.log(select<int>(1),select<string>("x".repeat(1)));}"#;
    let analysis = tsrust_compiler::check(source).unwrap();
    let results = analysis.checked_type_facts().method_results;
    let span = results.iter().find(|fact| fact.ty == "int").unwrap().span;
    assert!(
        results
            .iter()
            .any(|fact| fact.span == span && fact.ty == "string")
    );
    let hover = ask(source, "textDocument/hover", "selected;", 0);
    assert!(
        !hover["contents"]["value"]
            .as_str()
            .unwrap_or("")
            .contains("selected: int")
    );
    for source in [
        r#"function main(){const selected=missing??[1];selected.join();}"#,
        r#"function main(){const input:int|null=null;const selected=input??"bad";selected.repeat(1);}"#,
        r#"function main(){const input:int[]|null=[1];const selected=input??[2];console.log(String(input));selected.join();}"#,
    ] {
        assert!(tsrust_compiler::check(source).is_err(), "{source}");
        let method = if source.contains(".join") {
            "join("
        } else {
            "repeat("
        };
        assert!(
            ask(source, "textDocument/hover", method, 0).is_null(),
            "{source}"
        );
    }
    let source = r#"function main(){let input:int[]|null=[1];if(input!==null){input=null;console.log(input??[2]);}}"#;
    assert!(tsrust_compiler::check(source).is_ok());
    let index = SourceIndex::new(source);
    let hover = index
        .request(
            "textDocument/hover",
            &json!({"position":at(source,"input??",0)}),
            "file:///test.tsr",
        )
        .unwrap();
    assert!(
        !hover["contents"]["value"]
            .as_str()
            .unwrap_or("")
            .contains("Narrowed here to `int[]`")
    );
}

#[test]
fn contextual_generic_array_calls_keep_checked_parameter_and_result_help() {
    let source="function identity<T>(value:T):T{return value;}function first<T>(left:T,right:T):T{return left;}function main(){const values:number[]=identity([1,2]);const selected=first(identity([]),identity([7]));console.log(values.length,selected.length);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover=ask(source,"textDocument/hover","identity([1,2])",0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("identity(value: number[]): number[]"),"{hover}");
    let deferred=ask(source,"textDocument/hover","identity([])",0);
    assert!(deferred["contents"]["value"].as_str().unwrap().contains("identity(value: int[]): int[]"),"{deferred}");
    let help=ask(source,"textDocument/signatureHelp","identity([])",0);
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("first(left: int[], right: int[]): int[]"),"{help}");
}

#[test]
fn nullish_rhs_absence_views_are_checked_scoped_and_invalidated() {
    for (source, marker, shape) in [
        (
            r#"function takeNull(value:null):string{return "fallback";}function pick(value:string|null):string{return value??takeNull(value);}function main(){console.log(pick(null));}"#,
            "value);",
            "value: null",
        ),
        (
            r#"type Holder={value:string|null;};function takeNull(value:null):string{return "fallback";}function read(value:string):int{return value.length;}function pick(holder:Holder):int{return read(holder.value??takeNull(holder.value));}function main(){const holder:Holder={value:null};console.log(pick(holder));}"#,
            "value));",
            "value: null",
        ),
        (
            r#"function takeUndefined(value:undefined):int{return 7;}function pick(value:int|undefined):int{return value??takeUndefined(value);}function main(){console.log(pick(undefined));}"#,
            "value);",
            "value: undefined",
        ),
        (
            r#"function takeAbsent(value:null|undefined):int{return 7;}function pick(value:int|null|undefined):int{return value??takeAbsent(value);}function main(){console.log(pick(null));}"#,
            "value);",
            "value: null | undefined",
        ),
    ] {
        let analysis = tsrust_compiler::check(source).unwrap();
        let start = source.find(marker).unwrap();
        assert!(
            analysis
                .projections()
                .iter()
                .any(|fact| fact.span.start <= start
                    && fact.span.end == start + "value".len()
                    && shape.ends_with(&fact.ty)),
            "missing checked RHS projection: {source}"
        );
        let hover = ask(source, "textDocument/hover", marker, 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!(
                    "Narrowed here to `{}`.",
                    shape.split_once(": ").unwrap().1
                )),
            "{hover}"
        );
    }
    let source = r#"function takeNull(value:null):int{return 9;}function pick(value:int|null):int{const selected=value??takeNull(value);console.log(String(value));return selected;}function main(){console.log(pick(null));}"#;
    tsrust_compiler::check(source).unwrap();
    let hover = ask(source, "textDocument/hover", "value));", 0);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .replace(' ', "")
            .contains("value:int|null"),
        "{hover}"
    );
    let invalid = r#"type Holder={value:int|null;};function replace(holder:Holder):int{holder.value=2;return 0;}function takeNull(value:null):int{return 9;}function pick(holder:Holder):int{return holder.value??(replace(holder)+takeNull(holder.value));}function main(){const holder:Holder={value:null};console.log(pick(holder));}"#;
    assert!(tsrust_compiler::check(invalid).is_err());
    let hover = ask(invalid, "textDocument/hover", "value));", 0);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .replace(' ', "")
            .contains("int|null"),
        "{hover}"
    );
    assert!(
        !hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Narrowed here to"),
        "{hover}"
    );
}

#[test]
fn nullish_assignment_checked_aliases_and_flow_views_match_language_examples() {
    let examples = [
        (
            r#""#,
            r#"let count:int|null=0;let flag:boolean|undefined=false;let text:string|null="";const kept=(text??="fallback");console.log(count??=9,flag??=true,kept.length);"#,
            "0 false 0\n",
        ),
        (
            r#"function makeText(value:null):string{return "é".repeat(1);}"#,
            r#"let value:string|null=null;const selected=(value??=makeText(value));const independent=clone(selected);console.log(selected,independent,value.length);"#,
            "é é 2\n",
        ),
        (
            r#""#,
            r#"let values:int[]|undefined=undefined;const selected=(values??=[1,2]);console.log(selected.join("|"),values.length);"#,
            "1|2 2\n",
        ),
        (
            r#"type Row={n:int;};"#,
            r#"let value:Row|null=null;const selected=(value??={n:7});console.log(selected.n,value.n);"#,
            "7 7\n",
        ),
        (
            r#""#,
            r#"let value:int|null|undefined=undefined;const selected=(value??=null);value=4;console.log(String(selected),String(value));"#,
            "null 4\n",
        ),
        (
            r#"type Holder={text:string|null;};function makeFieldText(value:null):string{return "filled".repeat(1);}"#,
            r#"const holder:Holder={text:null};const selected=(holder.text??=makeFieldText(holder.text));console.log(selected,holder.text.length);"#,
            "filled 6\n",
        ),
    ];
    let mut prelude = String::new();
    let mut body = String::new();
    let mut expected = String::new();
    for (declarations, snippet, output) in examples {
        tsrust_compiler::check(&format!("{declarations}function main(){{{snippet}}}")).unwrap();
        prelude.push_str(declarations);
        body.push_str(&format!("{{{snippet}}}\n"));
        expected.push_str(output);
    }
    let source = format!("{prelude}function main(){{{body}}}");
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/nullish-assignment-language-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("examples.rs"), analysis.emit_rust()).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    for level in [0, 3] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustup")
            .args(["run", "1.96.1", "rustc", "--edition=2024", "-C"])
            .arg(format!("opt-level={level}"))
            .arg(directory.join("examples.rs"))
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    assert_eq!(expected.lines().count(), 6);
    for (source, marker, ty) in [
        (
            r#"function main(){let value:string|null=null;const selected=(value??="x");console.log(selected.length,value.length);}"#,
            "selected.length",
            "string",
        ),
        (
            r#"function main(){let values:int[]|undefined=undefined;const selected=(values??=[1,2]);console.log(selected.join(),values.length);}"#,
            "selected.join",
            "int[]",
        ),
        (
            r#"type Row={n:int;};function main(){let value:Row|null=null;const selected=(value??={n:7});console.log(selected.n,value.n);}"#,
            "selected.n",
            "Row",
        ),
        (
            r#"function read(value:string|null):int{return Boolean(value)?1:0;}function main(){let value:string|null|undefined=undefined;const selected=(value??=null);console.log(read(selected));}"#,
            "selected));",
            "string | null",
        ),
        (
            r#"function main(){let value:number|null=null;const selected:number=(value??=2.5);console.log(selected.toFixed(1));}"#,
            "selected.toFixed",
            "number",
        ),
    ] {
        let analysis = tsrust_compiler::check(source).unwrap();
        assert!(
            analysis
                .checked_type_facts()
                .method_results
                .iter()
                .any(|fact| {
                    fact.ty == ty && source[fact.span.start..fact.span.end].contains("??=")
                }),
            "missing checked assignment result: {source}"
        );
        let hover = ask(source, "textDocument/hover", marker, 0);
        let text = hover["contents"]["value"].as_str().unwrap();
        assert!(
            text.replace(' ', "")
                .contains(&format!("selected:{}", ty.replace(' ', ""))),
            "{hover}"
        );
    }
    for (source, method, label) in [
        (
            r#"function main(){let value:string|null=null;console.log((value??="x").repeat(2));}"#,
            "repeat(",
            "repeat(count: int): string",
        ),
        (
            r#"function main(){let values:int[]|undefined=undefined;console.log((values??=[1]).join("|"));}"#,
            "join(",
            "join(separator?: string): string",
        ),
    ] {
        tsrust_compiler::check(source).unwrap();
        let hover = ask(source, "textDocument/hover", method, 0);
        assert!(
            hover["contents"]["value"].as_str().unwrap().contains(label),
            "{hover}"
        );
        let items = ask(source, "textDocument/completion", method, 0);
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == method.trim_end_matches('('))
        );
        let mut position = at(source, method, 0);
        position.character += method.len() as u32;
        let help = SourceIndex::new(source)
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(help["signatures"][0]["label"], label);
    }
    let source = r#"function makeText(value:null):string{return "x";}function main(){let value:string|null=null;const selected=(value??=makeText(value));console.log(value.length,selected.length);}"#;
    tsrust_compiler::check(source).unwrap();
    let rhs = ask(source, "textDocument/hover", "value));", 0);
    assert!(
        rhs["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Narrowed here to `null`."),
        "{rhs}"
    );
    let after = ask(source, "textDocument/hover", "value.length", 0);
    assert!(
        after["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Narrowed here to `string`."),
        "{after}"
    );
    let source = r#"function makeUndefined(value:undefined):int{return 7;}function main(){let value:int|undefined=undefined;const selected=(value??=makeUndefined(value));console.log(value,selected);}"#;
    tsrust_compiler::check(source).unwrap();
    let rhs = ask(source, "textDocument/hover", "value));", 0);
    assert!(
        rhs["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Narrowed here to `undefined`."),
        "{rhs}"
    );
    let source = r#"function maybe(value:null|undefined):string|null{return null;}function main(){let value:string|null|undefined=undefined;const selected=(value??=maybe(value));console.log(String(value),String(selected));}"#;
    tsrust_compiler::check(source).unwrap();
    let rhs = ask(source, "textDocument/hover", "value));", 0);
    assert!(
        rhs["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Narrowed here to `null | undefined`."),
        "{rhs}"
    );
    let after = ask(source, "textDocument/hover", "value),String", 0);
    assert!(
        after["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Narrowed here to `string | null`."),
        "{after}"
    );
    let source = r#"function read(value:string):int{return value.length;}function main(){let value:string|null=null;const selected=value??="x";console.log(read(selected));}"#;
    tsrust_compiler::check(source).unwrap();
    let read_help = ask(source, "textDocument/hover", "read(selected)", 0);
    assert!(
        read_help["contents"]["value"]
            .as_str()
            .unwrap()
            .replace(' ', "")
            .contains("read(value:string):int"),
        "{read_help}"
    );
    for invalid in [
        r#"function main(){let value:string="x";value??=7;}"#,
        r#"function main(){let value:string|null=null;const selected=value??="x";value="changed";console.log(selected);}"#,
        r#"function escape():string{let value:string|null=null;const selected=value??="x";return selected;}function main(){console.log(escape());}"#,
        r#"function main(){let values:int[]|undefined=undefined;const selected=values??=[1];selected.push(2);console.log(selected.length);}"#,
    ] {
        assert!(tsrust_compiler::check(invalid).is_err(), "{invalid}");
    }
    let invalid = r#"function main(){let value:string|null=null;const selected=value??="x";value=null;console.log(selected.length);}"#;
    assert!(tsrust_compiler::check(invalid).is_err());
    let hover = ask(invalid, "textDocument/hover", "selected.length", 0);
    assert!(
        !hover["contents"]["value"]
            .as_str()
            .unwrap()
            .replace(' ', "")
            .contains("selected:string"),
        "invalid borrowed code must not retain checked result facts: {hover}"
    );
}

#[test]
fn grouped_generic_callback_help_preserves_checked_access_and_source_names() {
    let source = "type User={name:string;};function invoke<T>(action:((Function<T, [person:Owned<T>]>)),value:T):T{return action(value);}function main(){const action=(user:User):User=>{return user;};const saved=invoke(action,{name:'Pat'});console.log(saved.name);}";
    tsrust_compiler::check(source).unwrap();
    let hover = ask(source, "textDocument/hover", "invoke(action", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Function<User, [person: Owned<User>]>"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "action,{name", 0);
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("Function<User, [person: Owned<User>]>"), "{help}");
}

#[test]
fn annotated_local_callables_show_source_parameter_help_and_result_chains() {
    let source = "function main(){const normalize:Function<string, [input:ReadOnly<string>]>=(text:string):string=>{return text.trim();};const saved=normalize(' Pat ');console.log(saved.toLowerCase());}";
    tsrust_compiler::check(source).unwrap();
    let hover = ask(source, "textDocument/hover", "normalize(' Pat '", 0);
    let docs = hover["contents"]["value"].as_str().expect("callable hover");
    assert!(docs.contains("input:ReadOnly<string>"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "' Pat ')", 0);
    let label = help["signatures"][0]["label"].as_str().expect("callable help");
    assert!(label.contains("input:ReadOnly<string>"), "{help}");
    assert!(!ask(source, "textDocument/hover", "toLowerCase()", 0).is_null());
    let source = "function main(){const single:((Once<Function<int, [value:int]>>))=(value:int):int=>{return value+1;};console.log(single(4));}";
    tsrust_compiler::check(source).unwrap();
    let hover = ask(source, "textDocument/hover", "single(4)", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Once<Function<int"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "4)", 0);
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("value"), "{help}");
}
#[test]
fn optional_chain_editor_views_and_language_examples_follow_checked_hops() {
    let examples = [
        (
            r#"type OptionalRow={n:int;};"#,
            r#"const absent:OptionalRow|null=null;const present:OptionalRow|null={n:7};console.log(String(absent?.n),present?.n??0);"#,
            "undefined 7\n",
        ),
        (
            r#"function optionalIndex(trace:int[]):int{trace.push(1);return 0;}"#,
            r#"const trace:int[]=[];const absent:int[]|undefined=undefined;const present:int[]|undefined=[5];console.log(String(absent?.[optionalIndex(trace)]),trace.length,present?.[optionalIndex(trace)]??0,trace.length);"#,
            "undefined 0 5 1\n",
        ),
        (
            r#"function optionalCount(trace:int[]):int{trace.push(1);return 1;}"#,
            r#"const trace:int[]=[];const absent:string|null=null;const present:string|null="é".repeat(1);console.log(String(absent?.repeat(optionalCount(trace))),trace.length,String(present?.repeat(optionalCount(trace))),trace.length);"#,
            "undefined 0 é 1\n",
        ),
        (
            r#"type OptionalTextRow={text:string;};"#,
            r#"const absent:OptionalTextRow|null=null;const present:OptionalTextRow|null={text:"é"};console.log(String(absent?.text.length),present?.text.length??0);"#,
            "undefined 2\n",
        ),
        (
            r#""#,
            r#"const values:string[]|null=["é".repeat(1)];const selected=values?.[0];const independent=clone(selected);console.log(String(selected),String(independent),values?.length??0);"#,
            "é é 1\n",
        ),
        (
            r#"type OptionalHolder={text:string|null;};"#,
            r#"const holder:OptionalHolder|undefined={text:null};const selected=holder?.text;console.log(String(clone(selected)??"empty"),Boolean(selected));"#,
            "empty false\n",
        ),
    ];
    let mut prelude = String::new();
    let mut body = String::new();
    let mut expected = String::new();
    for (declarations, snippet, output) in examples {
        tsrust_compiler::check(&format!("{declarations}function main(){{{snippet}}}")).unwrap();
        prelude.push_str(declarations);
        body.push_str(&format!("{{{snippet}}}\n"));
        expected.push_str(output);
    }
    let source = format!("{prelude}function main(){{{body}}}");
    let analysis = tsrust_compiler::check(&source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/optional-chaining-language-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), &source).unwrap();
    std::fs::write(directory.join("examples.rs"), analysis.emit_rust()).unwrap();
    std::fs::write(directory.join("expected.stdout"), &expected).unwrap();
    for level in [0, 3] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustup")
            .args(["run", "1.96.1", "rustc", "--edition=2024", "-C"])
            .arg(format!("opt-level={level}"))
            .arg(directory.join("examples.rs"))
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(&binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    assert_eq!(expected.lines().count(), 6);
    for (source, marker, ty) in [
        (
            r#"function main(){const value:string|null=null;const selected=value?.length;console.log(String(selected));}"#,
            "selected=value",
            "int | undefined",
        ),
        (
            r#"function main(){const value:string="x";const selected=value?.length;console.log(selected);}"#,
            "selected=value",
            "int",
        ),
        (
            r#"function main(){const values:int[]|undefined=undefined;const selected=values?.join("|");console.log(String(selected));}"#,
            "selected=values",
            "string | undefined",
        ),
        (
            r#"type Row={text:string|null;};function main(){const value:Row|null=null;const selected=value?.text;console.log(String(selected));}"#,
            "selected=value",
            "string | null | undefined",
        ),
    ] {
        tsrust_compiler::check(source).unwrap();
        let hover = ask(source, "textDocument/hover", marker, 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .replace(' ', "")
                .contains(&format!("selected:{}", ty.replace(' ', ""))),
            "{hover}"
        );
    }
    for (source, marker, label) in [
        (
            r#"function main(){const text:string|null=null;console.log(String(text?.repeat(2)));}"#,
            "repeat(",
            "repeat(count: int): string",
        ),
        (
            r#"function main(){const text:string="x";console.log(text.toUpperCase?.());}"#,
            "toUpperCase?.(",
            "toUpperCase(): string",
        ),
        (
            r#"type Row={text:string;};function main(){const row:Row|null=null;console.log(String(row?.text.repeat(2)));}"#,
            "repeat(",
            "repeat(count: int): string",
        ),
        (
            r#"function main(){console.log(Array?.isArray([1]));}"#,
            "isArray(",
            "isArray(value?: unknown): boolean",
        ),
    ] {
        tsrust_compiler::check(source).unwrap();
        let hover = ask(source, "textDocument/hover", marker, 0);
        assert!(
            hover["contents"]["value"].as_str().unwrap().contains(label),
            "{hover}"
        );
        let mut position = at(source, marker, 0);
        position.character += marker.len() as u32;
        let help = SourceIndex::new(source)
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(help["signatures"][0]["label"], label, "{help}");
    }
    for (source, label) in [
        (
            "function main(){const value:string|null=null;value?.",
            "repeat",
        ),
        (
            "type Row={text:string;};function main(){const value:Row|undefined=undefined;value?.",
            "text",
        ),
        (
            "type Row={value:int[]|null;};function main(){const row:Row={value:null};row.value?.",
            "join",
        ),
    ] {
        let items = SourceIndex::new(source)
            .request(
                "textDocument/completion",
                &json!({"position":Position::new(0,source.encode_utf16().count() as u32)}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == label),
            "{items}"
        );
    }
    for source in [
        "function main(){const value:string|null=null;value.",
        "function main(){const value:null=null;value?.",
        "function main(){missing();const value:string|null=null;value?.",
    ] {
        let position = Position::new(0, source.encode_utf16().count() as u32);
        let items = SourceIndex::new(source)
            .request(
                "textDocument/completion",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert!(
            !items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "repeat"),
            "{items}"
        );
    }
    for (source, marker) in [
        (
            r#"function main(){const selected=Array?.of<int>(1);console.log(selected.join("|"));}"#,
            "of<int>",
        ),
        (
            r#"function main(){const selected=Array.of?.<int>(1);console.log(selected.join("|"));}"#,
            "of?.<int>",
        ),
        (
            r#"function identity<T>(value:T):T{return value;}function main(){const selected=identity?.<int>(1);console.log(selected);}"#,
            "identity?.<int>",
        ),
    ] {
        tsrust_compiler::check(source).unwrap();
        let hover = ask(source, "textDocument/hover", marker, 0);
        let text = hover["contents"]["value"].as_str().unwrap();
        assert!(text.contains("int"), "{hover}");
        let help = ask(source, "textDocument/signatureHelp", "1)", 0);
        assert!(
            help["signatures"][0]["label"]
                .as_str()
                .unwrap()
                .contains("int"),
            "{help}"
        );
        let hover = ask(source, "textDocument/hover", "selected=", 0);
        let expected = if marker.starts_with("of") {
            "selected:int[]"
        } else {
            "selected:int"
        };
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .replace(' ', "")
                .contains(expected),
            "{hover}"
        );
    }
    let callable = r#"function main(){const action=(value:int):int=>{return value+1;};const selected=action?.(4);console.log(selected);}"#;
    tsrust_compiler::check(callable).unwrap();
    let hover = ask(callable, "textDocument/hover", "selected=", 0);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .replace(' ', "")
            .contains("selected:int"),
        "{hover}"
    );
    let help = ask(callable, "textDocument/signatureHelp", "4)", 0);
    assert!(
        help["signatures"][0]["label"]
            .as_str()
            .unwrap()
            .replace(' ', "")
            .contains("value:int"),
        "{help}"
    );
    let conflict = r#"function observe<T>(value:T):void{console.log(String(value?.valueOf()));}function main(){observe("x");observe(1);}"#;
    let analysis = tsrust_compiler::check(conflict).unwrap();
    let facts = analysis.checked_type_facts();
    assert!(facts.projections.iter().any(|fact| fact.ty == "string"));
    assert!(facts.projections.iter().any(|fact| fact.ty == "int"));
    let hover = ask(conflict, "textDocument/hover", "valueOf(", 0);
    assert!(
        hover.is_null(),
        "conflicting specializations must not select one method: {hover}"
    );
    for invalid in [
        r#"type Row={text:string;};function main(){const row:Row|null=null;console.log((row?.text).length);}"#,
        r#"function main(){const values:int[]|null=null;values?.[0]=1;}"#,
        r#"function main(){const values:string[]|null=["x".repeat(1)];const selected=values?.[0];values=null;console.log(String(selected));}"#,
    ] {
        assert!(tsrust_compiler::check(invalid).is_err(), "{invalid}");
    }
}
#[test]
fn optional_chain_static_receivers_and_mutators_keep_checked_protocol_facts() {
    for (source, marker, label, result) in [
        (
            r#"function main(){const text:string="x";const selected=text?.repeat(2);console.log(selected);}"#,
            "repeat(",
            "repeat(count: int): string",
            "string",
        ),
        (
            r#"function main(){const xs:int[]=[1];const selected=xs?.unshift(xs.length);console.log(selected);}"#,
            "unshift(",
            "unshift(...values: int[]): int",
            "int",
        ),
        (
            r#"function main(){const xs:int[]=[1,2];const selected=xs?.splice(0,xs.length);console.log(selected.join("|"));}"#,
            "splice(",
            "splice(start?: int, deleteCount?: int, ...items: int[]): int[]",
            "int[]",
        ),
        (
            r#"function main(){const xs:int[]|null=null;const selected=xs?.unshift(2);console.log(String(selected));}"#,
            "unshift(",
            "unshift(...values: int[]): int",
            "int | undefined",
        ),
        (
            r#"function main(){const xs:int[]|null=null;const selected=xs?.splice(0,1);console.log(String(selected));}"#,
            "splice(",
            "splice(start?: int, deleteCount?: int, ...items: int[]): int[]",
            "int[] | undefined",
        ),
    ] {
        let analysis = tsrust_compiler::check(source).unwrap();
        assert!(
            analysis
                .checked_type_facts()
                .method_results
                .iter()
                .any(|fact| fact.ty == result),
            "{source}"
        );
        let hover = ask(source, "textDocument/hover", marker, 0);
        assert!(
            hover["contents"]["value"].as_str().unwrap().contains(label),
            "{hover}"
        );
        let completion = ask(source, "textDocument/completion", marker, 0);
        assert!(
            completion
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == marker.trim_end_matches('(')),
            "{completion}"
        );
        let mut position = at(source, marker, 0);
        position.character += marker.len() as u32;
        let help = SourceIndex::new(source)
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position}),
                "file:///test.tsr",
            )
            .unwrap();
        assert_eq!(help["signatures"][0]["label"], label, "{help}");
        let hover = ask(source, "textDocument/hover", "selected=", 0);
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .replace(' ', "")
                .contains(&format!("selected:{}", result.replace(' ', ""))),
            "{hover}"
        );
    }
    let conflict = r#"function observe<T>(value:T):void{console.log(String(value?.valueOf()));}function main(){observe("x");observe(1);}"#;
    tsrust_compiler::check(conflict).unwrap();
    let hover = ask(conflict, "textDocument/hover", "valueOf(", 0);
    assert!(
        hover.is_null(),
        "conflicting successful-hop facts must stay conservative: {hover}"
    );
}

#[test]
fn owned_callable_fields_keep_declaration_labels_in_hover_and_parameter_help() {
    let source = "type Handler={first:Function<string, [input:ReadOnly<string>]>;second:Function<string, [other:ReadOnly<string>]>;};function main(){const handler:Handler={first:(text:string):string=>{return text.trim();},second:(text:string):string=>{return text.toLowerCase();}};const result=handler.first(' Pat ');console.log(handler.second('SAM').trim());}";
    tsrust_compiler::check(source).unwrap();
    for (field, parameter, argument) in [("first", "input", "' Pat ')"), ("second", "other", "'SAM')")] {
        let hover = ask(source, "textDocument/hover", &format!("{field}('"), 0);
        let docs = hover["contents"]["value"].as_str().expect("field hover");
        assert!(docs.contains(parameter), "{hover}");
        assert!(docs.contains("Function<string"), "{hover}");
        let help = ask(source, "textDocument/signatureHelp", argument, 0);
        let label = help["signatures"][0]["label"].as_str().expect("field help");
        assert!(label.contains(parameter), "{help}");
    }
    assert!(!ask(source, "textDocument/hover", "trim());", 0).is_null());
    let result = ask(source, "textDocument/hover", "result=", 0);
    assert!(result["contents"]["value"].as_str().unwrap().contains("string"), "{result}");
    let completion = ask(source, "textDocument/completion", "first('", 0);
    let first = completion.as_array().unwrap().iter().find(|item| item["label"] == "first").unwrap();
    assert!(first["detail"].as_str().unwrap().contains("input"), "{first}");
}

#[test]
fn single_use_callable_field_help_preserves_source_documentation() {
    let source = "type Handler={/** Normalize supplied text.\n * @param value - Text whose surrounding spaces are removed.\n */ run:Once<Function<string, [value:ReadOnly<string>]>>;};function main(){const handler:Handler={run:(text:string):string=>{return text.trim();}};console.log(handler.run(' Pat '));}";
    tsrust_compiler::check(source).unwrap();
    let hover = ask(source, "textDocument/hover", "run('", 0);
    let docs = hover["contents"]["value"].as_str().unwrap();
    assert!(docs.contains("Once<Function<string"), "{hover}");
    assert!(docs.contains("Normalize supplied text"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "' Pat ')", 0);
    assert!(help["signatures"][0]["parameters"][0]["documentation"]["value"].as_str().unwrap().contains("surrounding spaces"), "{help}");
}

#[test]
fn generic_callable_field_results_keep_chained_builtin_help() {
    let source = "type Handler<T>={run:Function<T, [value:ReadOnly<T>]>;};function main(){const handler:Handler<string>={run:(text:string):string=>{return text.trim();}};console.log(handler.run(' Pat ').toLowerCase());}";
    tsrust_compiler::check(source).unwrap();
    let hover = ask(source, "textDocument/hover", "toLowerCase", 0);
    assert!(!hover.is_null(), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "' Pat ')", 0);
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("string"), "{help}");
}

#[test]
fn factory_parameter_help_uses_inputs_instead_of_returned_callable_parameters() {
    let source = "function make(start:int):(delta:int)=>int{return (delta:int):int=>{return start+delta;};}function main(){const action=make(7);console.log(action(2));}";
    tsrust_compiler::check(source).unwrap();
    let help = ask(source, "textDocument/signatureHelp", "7)", 0);
    assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(), 1);
    let parameter = help["signatures"][0]["parameters"][0]["label"].as_str().unwrap();
    assert!(parameter.contains("start"), "{help}");
    assert!(!parameter.contains("delta"), "{help}");
}

#[test]
fn conditional_callable_fields_keep_declared_help_and_result_chains() {
    let source = "type Handler={run:Function<string, [input:ReadOnly<string>]>;};function main(){const handler:Handler={run:true?(text:string):string=>{return text.trim();}:(text:string):string=>{return text.toUpperCase();}};console.log(handler.run('Pat').toLowerCase());}";
    assert!(tsrust_compiler::check(source).is_ok());
    let help = ask(source, "textDocument/signatureHelp", "'Pat')", 0);
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("input"), "{help}");
    assert!(!ask(source, "textDocument/hover", "toLowerCase()", 0).is_null());
}

#[test]
fn explicit_mutate_callback_annotations_preserve_help_and_chained_results() {
    let source = "function apply(action:Mutable<Function<string, [text:ReadOnly<string>]>>):string{return action('Pat');}function main(){const action=(text:string):string=>{return text.trim();};console.log(apply(action).toLowerCase());}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "action('Pat')", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Mutable<Function<string, [text: ReadOnly<string>]>>"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "'Pat')", 0);
    assert!(help["signatures"][0]["parameters"][0]["label"].as_str().unwrap().contains("text"), "{help}");
    assert!(!ask(source, "textDocument/hover", "toLowerCase()", 0).is_null());
}

#[test]
fn explicit_read_callable_help_keeps_borrowing_and_argument_names() {
    let source = "function apply(action:ReadOnly<Function<int, [text:ReadOnly<string>]>>):int{return action('Pat');}function main(){const action=(text:string):int=>{return text.length;};console.log(apply(action));}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "action('Pat')", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("ReadOnly<Function<int, [text: ReadOnly<string>]>>"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "'Pat')", 0);
    assert!(help["signatures"][0]["parameters"][0]["label"].as_str().unwrap().contains("text"), "{help}");
}

#[test]
fn owned_parameter_reborrowing_keeps_both_signature_contracts_visible() {
    let source="function borrow(action:Mutable<Function<int, [value:int]>>):int{return action(3);}function forward(action:(value:int)=>int):int{return borrow(action)+action(4);}function main(){console.log(forward((value:int):int=>{return value;}));}";
    assert!(tsrust_compiler::check(source).is_ok());
    let borrowed=ask(source,"textDocument/signatureHelp","action)+",0);
    assert!(borrowed["signatures"][0]["label"].as_str().unwrap().contains("Mutable<Function<"),"{borrowed}");
    let owned=ask(source,"textDocument/signatureHelp","(value:int):int=>",0);
    let label=owned["signatures"][0]["label"].as_str().unwrap();
    assert!(label.contains("forward"),"{owned}");
    assert!(!label.contains("mutate"),"{owned}");
}

#[test]
fn explicit_once_factory_results_keep_callable_help_and_result_chains() {
    let source="function make():Once<Function<string, [input:ReadOnly<string>]>>{return (text:string):string=>{return text.trim();};}function main(){const action=make();console.log(action('Pat').toLowerCase());}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover=ask(source,"textDocument/hover","make();",0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Once<Function<string"),"{hover}");
    let help=ask(source,"textDocument/signatureHelp","'Pat')",0);
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("input"),"{help}");
    assert!(!ask(source,"textDocument/hover","toLowerCase()",0).is_null());
}

#[test]
fn indexed_callable_arrays_keep_declared_parameter_help() {
    let source = "function main(){const actions:((input:ReadOnly<string>)=>string)[]=[(text:string):string=>{return text.trim();}];console.log(actions[0](' Pat '));}";
    tsrust_compiler::check(source).unwrap();
    let help = ask(source, "textDocument/signatureHelp", "' Pat '", 0);
    let label = help["signatures"][0]["label"].as_str().expect("indexed callable help");
    assert!(label.contains("input"), "{help}");
    assert!(label.contains("ReadOnly<string>"), "{help}");
    assert!(label.starts_with("actions[0]:"), "{help}");
}

#[test]
fn indexed_callable_array_help_preserves_result_arrays_and_nested_indices() {
    for (source, needle, prefix, parameter) in [
        ("function main(){const actions:((input:int)=>int[])[]=[(value:int):int[]=>{return [value];}];console.log(actions[0](7));}", "7)", "actions[0]:", "input: int"),
        ("function main(){const indices=[0];const actions:((input:ReadOnly<string>)=>string)[][]=[[(text:string):string=>{return text.trim();}]];console.log(actions[indices[0]][0](' Pat '));}", "' Pat '", "actions[indices[0]][0]:", "input: ReadOnly<string>"),
    ] {
        tsrust_compiler::check(source).unwrap();
        let help = ask(source, "textDocument/signatureHelp", needle, 0);
        let signature = &help["signatures"][0];
        assert!(signature["label"].as_str().unwrap().starts_with(prefix), "{help}");
        assert_eq!(signature["parameters"][0]["label"].as_str().unwrap().replace(' ', ""), parameter.replace(' ', ""), "{help}");
        if source.contains("=>int[]") {
            assert!(signature["label"].as_str().unwrap().ends_with("int[]"), "{help}");
        }
    }
}

#[test]
fn callable_array_fields_keep_documented_parameters_and_result_chains() {
    let source = "type Collection={/** Normalize supplied text.\n * @param input - Text with surrounding spaces.\n */ actions:((input:ReadOnly<string>)=>string)[];};function main(){const collection:Collection={actions:[(text:string):string=>{return text.trim();}]};console.log(collection.actions[0](' Pat ').toLowerCase());}";
    tsrust_compiler::check(source).unwrap();
    let help = ask(source, "textDocument/signatureHelp", "' Pat '", 0);
    assert!(help["signatures"][0]["label"].as_str().expect("field array help").contains("input"), "{help}");
    assert!(help["signatures"][0]["parameters"][0]["documentation"]["value"].as_str().unwrap().contains("surrounding spaces"), "{help}");
    assert!(!ask(source, "textDocument/hover", "toLowerCase", 0).is_null());
}

#[test]
fn generic_callable_array_fields_keep_nominal_parameter_help() {
    let source = "type User={name:string;};type Collection<T>={actions:((input:ReadOnly<T>)=>int)[];};function main(){const collection:Collection<User>={actions:[(value:User):int=>{return value.name.length;}]};const user:User={name:'Pat'};console.log(collection.actions[0](user));}";
    tsrust_compiler::check(source).unwrap();
    let help = ask(source, "textDocument/signatureHelp", "user));", 0);
    let label = help["signatures"][0]["label"].as_str().expect("generic field array help");
    assert!(label.contains("input"), "{help}");
    assert!(label.contains("User"), "{help}");
    assert!(!label.contains("read T"), "{help}");
}

#[test]
fn stored_callable_reborrowing_keeps_function_parameter_help() {
    let source = "function invoke(action:Mutable<Function<int, [input:int]>>,value:int):int{return action(value);}function main(){const actions:((input:int)=>int)[]=[(value:int):int=>{return value+1;}];console.log(invoke(actions[0],7),actions[0](8));}";
    tsrust_compiler::check(source).unwrap();
    let help = ask(source, "textDocument/signatureHelp", "actions[0],7", 0);
    let label = help["signatures"][0]["parameters"][0]["label"].as_str().expect("borrowed callback help");
    assert!(label.contains("Mutable<Function<"), "{help}");
    assert!(label.contains("input"), "{help}");
    let element = ask(source, "textDocument/signatureHelp", "8)", 0);
    assert!(element["signatures"][0]["label"].as_str().unwrap().contains("input"), "{element}");
}

#[test]
fn mutable_callable_iteration_help_keeps_borrowed_parameters_and_mapped_results() {
    let source = "function main(){const actions:(()=>string)[]=[():string=>{return 'Pat';}];const results=actions.map((action:Mutable<Function<string>>):string=>{return action();});console.log(results.join('|').trim());}";
    tsrust_compiler::check(source).unwrap();
    let hover = ask(source, "textDocument/hover", "action();", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Mutable<Function<"), "{hover}");
    assert!(!ask(source, "textDocument/hover", "trim()", 0).is_null());
}

#[test]
fn contextual_arrow_parameters_and_expression_results_have_editor_types() {
    let source="function main(){const users=['Alice','Bob'];const values=users.map(user=>user.toUpperCase());console.log(values.join('|'));}";
    tsrust_compiler::check(source).unwrap();
    assert!(!ask(source,"textDocument/hover","toUpperCase()",0).is_null());
    let result=ask(source,"textDocument/hover","values=",0);
    assert!(result["contents"]["value"].as_str().unwrap().contains("string[]"),"{result}");
    let parameter=ask(source,"textDocument/hover","user.toUpperCase",0);
    assert!(parameter["contents"]["value"].as_str().unwrap().contains("string"),"{parameter}");
    let invalid="function main(){const add=(value:int)=>value+1;console.log(value);}";
    assert!(ask(invalid,"textDocument/hover","value);",0).is_null());
}

#[test]
fn mutable_callable_reduction_help_preserves_borrowing_and_result() {
    for method in ["reduce", "reduceRight"] {
        let source = format!("function main(){{const actions:(()=>int)[]=[():int=>{{return 7;}}];const total=actions.{method}((sum:int,action:Mutable<Function<int>>):int=>{{return sum+action();}},0);console.log(total);}}");
        tsrust_compiler::check(&source).unwrap();
        let hover = ask(&source, "textDocument/hover", "action()", 0);
        assert!(hover["contents"]["value"].as_str().unwrap().contains("Mutable<Function<"), "{hover}");
        let total = ask(&source, "textDocument/hover", "total=", 0);
        assert!(total["contents"]["value"].as_str().unwrap().contains("int"), "{total}");
    }
}

#[test]
fn mutable_callable_flat_map_help_preserves_flattened_result_types() {
    let source = "function main(){const actions:(()=>string)[]=[():string=>{return 'Pat';}];const results=actions.flatMap((action:Mutable<Function<string>>):string[]=>{return [action()];});console.log(results.join('|').trim());}";
    tsrust_compiler::check(source).unwrap();
    let hover = ask(source, "textDocument/hover", "action()", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Mutable<Function<"), "{hover}");
    let result = ask(source, "textDocument/hover", "results=", 0);
    assert!(result["contents"]["value"].as_str().unwrap().contains("string[]"), "{result}");
    assert!(!ask(source, "textDocument/hover", "trim()", 0).is_null());
}

#[test]
fn mutable_callable_predicate_help_keeps_parameters_and_builtin_documentation() {
    for method in ["some", "every", "findIndex", "findLastIndex"] {
        let source = format!("function main(){{const actions:(()=>int)[]=[():int=>{{return 7;}}];console.log(actions.{method}((action:Mutable<Function<int>>,index:int):boolean=>{{return action()>index;}}));}}");
        tsrust_compiler::check(&source).unwrap();
        let hover = ask(&source, "textDocument/hover", &format!("{method}("), 0);
        assert!(hover["contents"]["value"].as_str().unwrap().contains("exclusive receiver access"), "{hover}");
        assert!(hover["contents"]["value"].as_str().unwrap().contains("=> boolean"), "{hover}");
        let action = ask(&source, "textDocument/hover", "action()>index", 0);
        assert!(action["contents"]["value"].as_str().unwrap().contains("Mutable<Function<"), "{action}");
    }
}

#[test]
fn popped_nullable_values_can_be_logged_and_keep_their_editor_type() {
    let source = "function main(){const users=['Alice','Bob','Charlie'];const lower=users.map(user=>user.toLowerCase());const upper=users.map((user:string):string=>{return user.toUpperCase();});const lastUser=users.pop();console.log(lastUser);}";
    assert!(tsrust_compiler::check(source).is_ok());
    assert!(tsrust_lsp::diagnostics(source, "file:///test.tsr").unwrap().is_empty());
    let hover = ask(source, "textDocument/hover", "lastUser);", 0);
    let text = hover["contents"]["value"].as_str().unwrap();
    assert!(text.contains("string | undefined"), "{text}");
    let log = ask(source, "textDocument/hover", "log(lastUser)", 0);
    assert!(log["contents"]["value"].as_str().unwrap().contains("Nullable values print their present payload"));
}

#[test]
fn non_null_asserted_pop_keeps_a_nonnullable_editor_type() {
    let source="function main(){const users=['Alice','Bob'];const lastUser=users.pop()!;console.log(lastUser.toUpperCase());}";
    assert!(tsrust_lsp::diagnostics(source,"file:///test.tsr").unwrap().is_empty());
    let hover=ask(source,"textDocument/hover","lastUser.toUpperCase",0);
    let text=hover["contents"]["value"].as_str().unwrap();
    assert!(text.contains("const lastUser: string"),"{text}");
    assert!(!text.contains("undefined"),"{text}");
    let method=ask(source,"textDocument/hover","toUpperCase",0);
    assert!(method["contents"]["value"].as_str().unwrap().contains("toUpperCase"));
}

#[test]
fn incomplete_non_null_chains_offer_payload_members() {
    for source in [
        "function main(){const users=['Alice'];users.pop()!.to",
        "function main(){const users=['Alice'];const last=users.pop()!;last.to",
    ] {
        let completion=SourceIndex::new(source).request("textDocument/completion",&json!({"position":Position::new(0,source.encode_utf16().count() as u32)}),"file:///test.tsr").unwrap();
        assert!(completion.as_array().unwrap().iter().any(|entry| entry["label"]=="toUpperCase"),"{source}: {completion}");
    }
}

#[test]
fn logical_value_results_have_payload_hover_and_chained_methods() {
    let source="function main(){const users=['Alice','Bob','Charlie'];const userOne=users.shift();const alice=userOne||'Alice';console.log(alice.toUpperCase());const value:string|undefined=undefined;const selected=value&&'yes';console.log(selected);}";
    assert!(tsrust_lsp::diagnostics(source,"file:///test.tsr").unwrap().is_empty());
    let hover=ask(source,"textDocument/hover","alice.toUpperCase",0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("const alice: string"),"{hover}");
    let hover=ask(source,"textDocument/hover","selected);",0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("string | undefined"),"{hover}");
    let upper=ask(source,"textDocument/hover","toUpperCase",0);
    assert!(upper["contents"]["value"].as_str().unwrap().contains("toUpperCase"));
}

#[test]
fn function_and_access_types_have_hover_completion_and_documented_examples() {
    let source = "function apply(action: Mutable<Function<int, [value: int]>>, reader: ReadOnly<Function<int, [ReadOnly<string>]>>, last: Once<Function<string>>, sink: ReadOnly<Function<string, [Owned<string>]>>): int { return action(1); } function main() {}";
    assert!(tsrust_compiler::check(source).is_ok());
    for (needle, signature, description) in [
        ("Function<int, [value", "type Function<Result, [Arguments]>", "type the function returns"),
        ("Mutable<Function<int, [value", "type Mutable<T>", "allows it to be changed"),
        ("ReadOnly<Function<int, [ReadOnly", "type ReadOnly<T>", "without allowing it to be changed"),
        ("Once<Function<string>>", "type Once<T>", "exactly one time"),
        ("Owned<string>", "type Owned<T>", "takes over"),
    ] {
        let hover = ask(source, "textDocument/hover", needle, 0);
        let text = hover["contents"]["value"].as_str().unwrap_or("");
        assert!(text.contains(signature), "{needle}: {hover}");
        assert!(text.contains(description), "{needle}: {hover}");
        assert!(text.contains("```tsrust"), "{needle}: {hover}");
        assert!(text.contains("Ownership:"), "{needle}: {hover}");
    }
    for (prefix, label) in [("Fun", "Function"), ("Mut", "Mutable"), ("Read", "ReadOnly"), ("Onc", "Once"), ("Own", "Owned")] {
        let text = format!("function use(action: {prefix}) {{}} function main() {{}}");
        let completion = ask(&text, "textDocument/completion", ") {} function main", 0);
        let item = completion
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["label"] == label)
            .unwrap_or_else(|| panic!("{label}: {completion}"));
        assert!(item["documentation"]["value"].as_str().unwrap().contains("```tsrust"), "{item}");
        assert!(item["detail"].as_str().unwrap().starts_with("type "), "{item}");
    }
}

#[test]
fn function_typed_values_supply_call_help_hover_and_results() {
    let source = "function apply(action: Mutable<Function<string, [count: int, label?: ReadOnly<string>]>>, plain: Function<int, [int]>): int { const text = action(1); const size = plain(2); return text.length + size; } function main() {}";
    assert!(tsrust_compiler::check(source).is_ok());
    let help = ask(source, "textDocument/signatureHelp", "1); const size", 0);
    let signature = &help["signatures"][0];
    assert!(signature["label"].as_str().unwrap().split_whitespace().collect::<String>().contains("count:int"), "{help}");
    assert_eq!(signature["parameters"].as_array().unwrap().len(), 2, "{help}");
    let help = ask(source, "textDocument/signatureHelp", "2); return", 0);
    assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(), 1, "{help}");
    let hover = ask(source, "textDocument/hover", "text = action", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("string"), "{hover}");
    let hover = ask(source, "textDocument/hover", "action(1)", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Mutable<Function<string"), "{hover}");
}

#[test]
fn retired_access_keywords_are_not_completion_suggestions() {
    for prefix in ["rea", "mut", "consu", "owne", "onc"] {
        let source = format!("function use(action: {prefix}) {{}} function main() {{}}");
        let completion = ask(&source, "textDocument/completion", ") {} function main", 0);
        for item in completion.as_array().unwrap() {
            assert!(!matches!(item["label"].as_str(), Some("read" | "mutate" | "consume" | "owned" | "once")), "{completion}");
        }
    }
}

#[test]
fn array_literal_spread_editor_types_and_catalog_follow_checked_sources() {
    let source = "function main(){const source=[1,2];const values=[0,...source,3];console.log(values.join(\"|\"),source.length);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "source,3", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("int[]"), "{hover}");
    let hover = ask(source, "textDocument/hover", "values.join", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("int[]"), "{hover}");
    let source = "function main(){const source=[1,2];const values=[...source];console.log(Array.of(...values).join());}";
    let hover = ask(source, "textDocument/hover", "of(...", 0);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("spread arguments are accepted"), "{hover}");
    let partial = "function main(){const source=[1,2];const values=[...source";
    let hover = ask(partial, "textDocument/hover", "source", 1);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("int[]"), "{hover}");
}

#[test]
fn array_from_has_hover_completion_parameter_help_and_checked_results() {
    let source = "function main(){const source=[3,1,2];const copy=Array.from(source);const labels=Array.from(source,(value:int,index:int):string=>{return String(value);});const letters=Array.from(\"ab\");console.log(copy.length,labels.join(\",\"),letters.join(\"\"));}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "from(source);", 0);
    let text = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(text.contains("from<T, U>(source: T[] | string"), "{hover}");
    assert!(text.contains("characters of a string"), "{hover}");
    assert!(text.contains("Ownership:"), "{hover}");
    let constructor = ask(source, "textDocument/hover", "Array.from", 0);
    let docs = constructor["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("Array.from creates"), "{constructor}");
    assert!(!docs.contains("Array.from are unavailable"), "{constructor}");
    let help = ask(source, "textDocument/signatureHelp", "source);const labels", 0);
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("from<T, U>"), "{help}");
    assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(), 2, "{help}");
    for (needle, expected) in [("copy=Array", "int[]"), ("labels=Array", "string[]"), ("letters=Array", "string[]")] {
        let hover = ask(source, "textDocument/hover", needle, 0);
        assert!(hover["contents"]["value"].as_str().unwrap_or("").contains(expected), "{needle}: {hover}");
    }
    let partial = "function main(){const values=Array.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "from"), "{completion}");
    assert!(!ask(source, "textDocument/hover", "join(\",\")", 0).is_null());
}

#[test]
fn array_keys_and_values_have_hover_completion_help_and_result_types() {
    let source = "function main(){const words=[\"a\",\"b\"];const keys=words.keys();const copy=words.values();console.log(keys.join(\",\"),copy.join(\",\"),words.keys().length);}";
    assert!(tsrust_compiler::check(source).is_ok());
    for (needle, signature, description) in [
        ("keys();const copy", "keys(): int[]", "index of every element"),
        ("values();console", "values(): string[]", "elements in order as a new array"),
    ] {
        let hover = ask(source, "textDocument/hover", needle, 0);
        let text = hover["contents"]["value"].as_str().unwrap_or("");
        assert!(text.contains(signature), "{needle}: {hover}");
        assert!(text.contains(description), "{needle}: {hover}");
        assert!(text.contains("```tsrust"), "{needle}: {hover}");
    }
    for (needle, expected) in [("keys=words", "int[]"), ("copy=words", "string[]")] {
        let hover = ask(source, "textDocument/hover", needle, 0);
        assert!(hover["contents"]["value"].as_str().unwrap_or("").contains(expected), "{needle}: {hover}");
    }
    assert!(!ask(source, "textDocument/hover", "join(\",\"),copy", 0).is_null());
    assert!(!ask(source, "textDocument/hover", "length);}", 0).is_null());
    let partial = "function main(){const words=[\"a\"];words.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    for label in ["keys", "values"] {
        assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == label), "{label}: {completion}");
    }
}

#[test]
fn string_at_has_hover_help_completion_and_nullable_result() {
    let source = "function main(){const text=\"aé\";const last=text.at(-1);console.log(String(last),text.at(0)??\"none\");}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "at(-1)", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("at(index: int): string | undefined"), "{hover}");
    assert!(docs.contains("counted from the start"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "-1);", 0);
    assert_eq!(help["signatures"][0]["parameters"][0]["label"], "index: int", "{help}");
    let result = ask(source, "textDocument/hover", "last=text", 0);
    assert!(result["contents"]["value"].as_str().unwrap_or("").contains("string | undefined"), "{result}");
    let partial = "function main(){const text=\"a\";text.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "at"), "{completion}");
}

#[test]
fn number_parse_int_has_hover_help_completion_and_number_result() {
    let source = "function main(){const value=Number.parseInt(\"ff\",16);console.log(value.toFixed(1),Number.parseInt(\"7\"));}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "parseInt(\"ff\"", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("parseInt(text: string, radix?: int): number"), "{hover}");
    assert!(docs.contains("Reads a whole number from the start of a string"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "16);", 0);
    assert_eq!(help["activeParameter"], 1, "{help}");
    assert_eq!(help["signatures"][0]["parameters"][1]["label"], "radix?: int", "{help}");
    let result = ask(source, "textDocument/hover", "value=Number", 0);
    assert!(result["contents"]["value"].as_str().unwrap_or("").contains("number"), "{result}");
    assert!(!ask(source, "textDocument/hover", "toFixed(1)", 0).is_null());
    let partial = "function main(){Number.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "parseInt"), "{completion}");
}

#[test]
fn math_f16round_has_hover_help_completion_and_number_result() {
    let source = "function main(){const value=Math.f16round(1.337);console.log(value.toFixed(1));}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "f16round(1.337)", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("f16round(value: number): number"), "{hover}");
    assert!(docs.contains("16-bit (half precision) floating point"), "{hover}");
    assert!(docs.contains("65520"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "1.337);", 0);
    assert_eq!(help["signatures"][0]["parameters"][0]["label"], "value: number", "{help}");
    let result = ask(source, "textDocument/hover", "value=Math", 0);
    assert!(result["contents"]["value"].as_str().unwrap_or("").contains("number"), "{result}");
    // Every Math call and constant shows its number result.
    let math = "function main(){const single=Math.fround(1.5);const pi=Math.PI;const top=Math.max(1.0,2.0).toFixed(1);console.log(single,pi,top);}";
    assert!(tsrust_compiler::check(math).is_ok());
    for (needle, expected) in [("single=Math", "const single: number"), ("pi=Math", "const pi: number"), ("top=Math", "const top: string")] {
        let hover = ask(math, "textDocument/hover", needle, 0);
        assert!(hover["contents"]["value"].as_str().unwrap_or("").contains(expected), "{hover}");
    }
    let partial = "function main(){Math.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "f16round"), "{completion}");
}

#[test]
fn math_random_has_hover_help_completion_and_number_result() {
    let source = "function main(){const value=Math.random();console.log(value.toFixed(1).length);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "random()", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("random(): number"), "{hover}");
    assert!(docs.contains("at least 0 and less than 1"), "{hover}");
    assert!(docs.contains("must be unpredictable"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", ");console", 0);
    assert_eq!(help["signatures"][0]["label"], "random(): number", "{help}");
    let result = ask(source, "textDocument/hover", "value=Math", 0);
    assert!(result["contents"]["value"].as_str().unwrap_or("").contains("const value: number"), "{result}");
    let partial = "function main(){Math.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "random"), "{completion}");
}

#[test]
fn string_from_codes_have_namespace_hover_help_completion_and_string_result() {
    let source = "function main(){const codes=[72,105];const text=String.fromCodePoint(65,...codes);const units=String.fromCharCode(72,105);console.log(text.toUpperCase(),units.trim());}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "fromCodePoint(65", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("fromCodePoint(...codes: int[]): string"), "{hover}");
    assert!(docs.contains("Builds a string from Unicode code points"), "{hover}");
    assert!(docs.contains("0 through 1114111"), "{hover}");
    let hover = ask(source, "textDocument/hover", "fromCharCode(72", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("fromCharCode(...codes: int[]): string"), "{hover}");
    assert!(docs.contains("low 16 bits"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "105);console", 0);
    assert_eq!(help["signatures"][0]["parameters"][0]["label"], "...codes: int[]", "{help}");
    for needle in ["text=String", "units=String"] {
        let result = ask(source, "textDocument/hover", needle, 0);
        assert!(result["contents"]["value"].as_str().unwrap_or("").contains("string"), "{result}");
    }
    assert!(!ask(source, "textDocument/hover", "toUpperCase()", 0).is_null());
    let namespace = ask(source, "textDocument/hover", "String.fromCodePoint", 0);
    assert!(namespace["contents"]["value"].as_str().unwrap_or("").contains("Static members: fromCodePoint, fromCharCode"), "{namespace}");
    let object = "function main(){console.log(Object.is(1,1),Array.isArray([1]));}";
    assert!(ask(object, "textDocument/hover", "Object.is", 0)["contents"]["value"].as_str().unwrap_or("").contains("Static members: is"));
    assert!(ask(object, "textDocument/hover", "Array.isArray", 0)["contents"]["value"].as_str().unwrap_or("").contains("Static members: isArray, of, from."));
    let partial = "function main(){String.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    let labels = completion.as_array().unwrap().iter().map(|item| item["label"].as_str().unwrap_or("")).collect::<Vec<_>>();
    assert_eq!(labels[..2], ["fromCodePoint", "fromCharCode"], "{completion}");
}

#[test]
fn string_code_at_methods_have_hover_help_completion_and_result_types() {
    let source = "function main(){const text=\"aé\";const point=text.codePointAt(1);const unit=text.charCodeAt(1);console.log(point??0,unit.toFixed(0));}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "codePointAt(1)", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("codePointAt(position?: int): int | undefined"), "{hover}");
    assert!(docs.contains("Returns the Unicode code point of the character at a position"), "{hover}");
    assert!(docs.contains("positions do not count from the end"), "{hover}");
    let hover = ask(source, "textDocument/hover", "charCodeAt(1)", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("charCodeAt(position?: int): number"), "{hover}");
    assert!(docs.contains("or `NaN` when the position names no character"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "1);const unit", 0);
    assert_eq!(help["signatures"][0]["parameters"][0]["label"], "position?: int", "{help}");
    let point = ask(source, "textDocument/hover", "point=text", 0);
    assert!(point["contents"]["value"].as_str().unwrap_or("").contains("int | undefined"), "{point}");
    let unit = ask(source, "textDocument/hover", "unit=text", 0);
    assert!(unit["contents"]["value"].as_str().unwrap_or("").contains("const unit: number"), "{unit}");
    assert!(!ask(source, "textDocument/hover", "toFixed(0)", 0).is_null());
    let partial = "function main(){const text=\"a\";text.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    for label in ["codePointAt", "charCodeAt", "charAt"] {
        assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == label), "{label}: {completion}");
    }
}

#[test]
fn string_locale_case_methods_have_hover_completion_and_string_result() {
    let source = "function main(){const text=\"Ab\";const low=text.toLocaleLowerCase();const up=text.toLocaleUpperCase();console.log(low.trim(),up.trim());}";
    assert!(tsrust_compiler::check(source).is_ok());
    for (needle, signature, word) in [("toLocaleLowerCase()", "toLocaleLowerCase(): string", "converted to lowercase"), ("toLocaleUpperCase()", "toLocaleUpperCase(): string", "converted to uppercase")] {
        let hover = ask(source, "textDocument/hover", needle, 0);
        let docs = hover["contents"]["value"].as_str().unwrap_or("");
        assert!(docs.contains(signature), "{hover}");
        assert!(docs.contains(word), "{hover}");
        assert!(docs.contains("locale-specific rules are not provided"), "{hover}");
    }
    for needle in ["low=text", "up=text"] {
        let result = ask(source, "textDocument/hover", needle, 0);
        assert!(result["contents"]["value"].as_str().unwrap_or("").contains("string"), "{result}");
    }
    assert!(!ask(source, "textDocument/hover", "trim(),up", 0).is_null());
    let partial = "function main(){const text=\"a\";text.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    for label in ["toLocaleLowerCase", "toLocaleUpperCase"] {
        assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == label), "{label}: {completion}");
    }
}

#[test]
fn union_array_elements_show_types_members_and_search_help() {
    let source = "function main(){const mixed:(int|string)[]=[1,\"a\"];const first=mixed.at(0);const copy=mixed.slice(0);const text=mixed.join(\"-\");const found=mixed.includes(\"a\");for(const item of mixed){console.log(typeof item);}const lengths=mixed.map((entry)=>typeof entry===\"string\"?entry.length:entry);console.log(String(first),copy.length,text,found,lengths.length);}";
    assert!(tsrust_compiler::check(source).is_ok());
    // Spacing inside a displayed union is not significant here.
    let text = |needle: &str| ask(source, "textDocument/hover", needle, 0)["contents"]["value"].as_str().unwrap_or("").replace(" | ", "|");
    assert!(text("mixed:(").contains("const mixed: (int|string)[]"), "{}", text("mixed:("));
    assert!(text("first=mixed").contains("const first: int|string|undefined"), "{}", text("first=mixed"));
    assert!(text("copy=mixed").contains("(int|string)[]"), "{}", text("copy=mixed"));
    assert!(text("text=mixed").contains("string"), "{}", text("text=mixed"));
    assert!(text("found=mixed").contains("boolean"), "{}", text("found=mixed"));
    assert!(text("item of mixed").contains("int|string"), "{}", text("item of mixed"));
    assert!(text("entry)=>").contains("int|string"), "{}", text("entry)=>"));
    assert!(text("lengths=mixed").contains("int[]"), "{}", text("lengths=mixed"));
    let docs = text("includes(\"a\")");
    assert!(docs.contains("includes(value: int|string, fromIndex?: int): boolean"), "{docs}");
    assert!(docs.contains("array of a primitive union such as `(int|string)[]`"), "{docs}");
    let partial = "function main(){const mixed:(int|string)[]=[1,\"a\"];mixed.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    for label in ["push", "includes", "map", "join", "at"] {
        assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == label), "{label}: {completion}");
    }
}

#[test]
fn destructured_names_have_hover_types_completion_and_references() {
    let source = "type Row={id:int;name:string;};\nfunction pair():[int,string]{return [1,\"a\"];}\nfunction main(){const [count,word]=pair();const row:Row={id:1,name:\"n\"};const {id,name:label}=clone(row);for(const [key,value] of Object.entries(row)){console.log(key.length,String(value));}console.log(count+id,word.length,label.length);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let text = |needle: &str, n: usize| ask(source, "textDocument/hover", needle, n)["contents"]["value"].as_str().unwrap_or("").replace(" | ", "|");
    assert!(text("count,word", 0).contains("const count: int"), "{}", text("count,word", 0));
    assert!(text("word]", 0).contains("const word: string"), "{}", text("word]", 0));
    assert!(text("id,name:label", 0).contains("const id: int"), "{}", text("id,name:label", 0));
    assert!(text("label}", 0).contains("const label: string"), "{}", text("label}", 0));
    assert!(text("key,value", 0).contains("const key: string"), "{}", text("key,value", 0));
    assert!(text("value]", 0).contains("int|string"), "{}", text("value]", 0));
    // A use resolves to its pattern declaration.
    assert!(text("word.length", 0).contains("const word: string"), "{}", text("word.length", 0));
    assert!(text("key.length", 0).contains("const key: string"), "{}", text("key.length", 0));
    let partial = "function pair():[int,string]{return [1,\"a\"];}\nfunction main(){const [count,word]=pair();word.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 1);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "toUpperCase"), "{completion}");
}

#[test]
fn destructured_parameters_and_indexed_borrows_have_hover_types() {
    let source = "type Point={x:int;label:string;};\nfunction describe({x,label}:Point,scale:int):string{return label.repeat(x*scale);}\nfunction main(){const rows:[string,int][]=[[\"a\",1]];const lengths=rows.map(([name,amount])=>name.length+amount);const words=[\"w\"];{const first=words[0];console.log(first.length);}console.log(describe({x:1,label:\"p\"},2),lengths.length);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let text = |needle: &str, n: usize| ask(source, "textDocument/hover", needle, n)["contents"]["value"].as_str().unwrap_or("").replace(" | ", "|");
    assert!(text("x,label}", 0).contains("const x: int"), "{}", text("x,label}", 0));
    assert!(text("label}", 0).contains("const label: string"), "{}", text("label}", 0));
    assert!(text("label.repeat", 0).contains("const label: string"), "{}", text("label.repeat", 0));
    assert!(text("name,amount", 0).contains("const name: string"), "{}", text("name,amount", 0));
    assert!(text("amount);", 0).contains("const amount: int"), "{}", text("amount);", 0));
    assert!(text("first=words", 0).contains("const first: string"), "{}", text("first=words", 0));
    let help = ask(source, "textDocument/signatureHelp", "{x:1,label", 0);
    assert_eq!(help["signatures"][0]["parameters"][0]["label"], "{x,label}:Point", "{help}");
    assert_eq!(help["signatures"][0]["parameters"][1]["label"], "scale:int", "{help}");
}

#[test]
fn number_to_string_has_hover_help_completion_and_string_result() {
    let source = "function main(){const value=255.5;const hex=value.toString(16);console.log(hex.length,value.toString());}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "toString(16)", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("toString(radix?: int): string"), "{hover}");
    assert!(docs.contains("Returns the number as text"), "{hover}");
    let help = ask(source, "textDocument/signatureHelp", "16);", 0);
    assert_eq!(help["signatures"][0]["parameters"][0]["label"], "radix?: int", "{help}");
    let result = ask(source, "textDocument/hover", "hex=value", 0);
    assert!(result["contents"]["value"].as_str().unwrap_or("").contains("string"), "{result}");
    assert!(!ask(source, "textDocument/hover", "length,value", 0).is_null());
    let partial = "function main(){const value=1.5;value.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "toString"), "{completion}");
}

#[test]
fn object_is_has_namespace_hover_completion_help_and_boolean_result() {
    let source = "function main(){const same=Object.is(1.5,1.5);console.log(same,Object.is(\"a\",\"b\"));}";
    assert!(tsrust_compiler::check(source).is_ok());
    let hover = ask(source, "textDocument/hover", "is(1.5", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("is(left: unknown, right: unknown): boolean"), "{hover}");
    assert!(docs.contains("same value"), "{hover}");
    let namespace = ask(source, "textDocument/hover", "Object.is(1.5", 0);
    let text = namespace["contents"]["value"].as_str().unwrap_or("");
    assert!(text.contains("namespace Object"), "{namespace}");
    assert!(text.contains("`Object.is` tests whether two values are the same value"), "{namespace}");
    let help = ask(source, "textDocument/signatureHelp", "1.5);", 0);
    assert_eq!(help["activeParameter"], 1, "{help}");
    assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(), 2, "{help}");
    let result = ask(source, "textDocument/hover", "same=Object", 0);
    assert!(result["contents"]["value"].as_str().unwrap_or("").contains("boolean"), "{result}");
    let partial = "function main(){Object.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "is"), "{completion}");
    let global = "function main(){Obj;}";
    let completion = ask(global, "textDocument/completion", ";}", 0);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "Object"), "{completion}");
}

#[test]
fn string_raw_has_hover_completion_parameter_help_and_string_result() {
    let source = "type Template={raw:string[];};function main(){const template:Template={raw:[\"a\",\"b\"]};const result=String.raw(template,7);console.log(result);}";
    assert!(tsrust_compiler::check(source).is_ok(), "{:?}",tsrust_compiler::check(source).err());
    let hover = ask(source, "textDocument/hover", "raw(template", 0);
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("raw(template:"), "{hover}");
    assert!(docs.contains("missing substitutions"), "{hover}");
    assert!(docs.contains("Ownership:"), "{hover}");
    let help = ask(source,"textDocument/signatureHelp","7);",0);
    assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),2,"{help}");
    let hover = ask(source,"textDocument/hover","result=String",0);
    assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("string"),"{hover}");
    let partial = "function main(){String.;}";
    let completion=ask(partial,"textDocument/completion",";}",0);
    assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="raw"),"{completion}");
}

#[test]
fn string_raw_catalog_examples_match_native_output() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["StringConstructor"].as_array().unwrap().iter().find(|entry|entry["name"]=="raw").unwrap();
    let mut source = CatalogExampleSource::default();
    let mut expected = String::new();
    for example in entry["examples"].as_array().unwrap() {
        let witness = source.push(example["source"].as_str().unwrap());
        tsrust_compiler::check(&witness).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }
    let analysis=tsrust_compiler::check(&source.finish()).unwrap();
    let dir=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/editor/string-raw-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input=dir.join("examples.rs");let binary=dir.join("examples");
    std::fs::write(&input,analysis.emit_rust()).unwrap();
    let compilation=std::process::Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"rustc","--edition=2024","-Awarnings"]).arg(&input).arg("-o").arg(&binary).output().unwrap();
    assert!(compilation.status.success(),"{}",String::from_utf8_lossy(&compilation.stderr));
    let output=std::process::Command::new(binary).output().unwrap();
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
    assert!(output.stderr.is_empty());
}

#[test]
fn number_locale_string_has_hover_completion_help_and_string_results() {
 let source="function main(){const value:number=1234.5675;const count:int=-1234;const display=value.toLocaleString();const grouped=count.toLocaleString();console.log(display,grouped);}";
 assert!(tsrust_compiler::check(source).is_ok());
 for needle in ["toLocaleString();const grouped","toLocaleString();console"] {
  let hover=ask(source,"textDocument/hover",needle,0);
  let docs=hover["contents"]["value"].as_str().unwrap_or("");
  assert!(docs.contains("toLocaleString(): string"),"{hover}");
  assert!(docs.contains("comma"),"{hover}");assert!(docs.contains("Ownership:"),"{hover}");
 }
 for needle in ["display=value","grouped=count"] {
  let hover=ask(source,"textDocument/hover",needle,0);
  assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("string"),"{hover}");
 }
 let help=ask(source,"textDocument/signatureHelp", ");const grouped",0);
 assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),0,"{help}");
 for ty in ["int","number"] {
  let partial=format!("function main(){{const value:{ty}=1;value.;}}");
  let completion=ask(&partial,"textDocument/completion",";}",0);
  assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="toLocaleString"),"{completion}");
 }
}

#[test]
fn number_locale_string_catalog_examples_match_native_output() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entries:Vec<_>=["number","int"].into_iter().map(|family|catalog[family].as_array().unwrap().iter().find(|entry|entry["name"]=="toLocaleString").unwrap()).collect();
    let mut source = CatalogExampleSource::default();
    let mut expected = String::new();
    for example in entries.iter().flat_map(|entry|entry["examples"].as_array().unwrap()) {
        let witness = source.push(example["source"].as_str().unwrap());
        tsrust_compiler::check(&witness).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }
    let analysis=tsrust_compiler::check(&source.finish()).unwrap();
    let dir=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/editor/number-locale-string-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input=dir.join("examples.rs");let binary=dir.join("examples");
    std::fs::write(&input,analysis.emit_rust()).unwrap();
    let compilation=std::process::Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"rustc","--edition=2024","-Awarnings"]).arg(&input).arg("-o").arg(&binary).output().unwrap();
    assert!(compilation.status.success(),"{}",String::from_utf8_lossy(&compilation.stderr));
    let output=std::process::Command::new(binary).output().unwrap();
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
    assert!(output.stderr.is_empty());
}


#[test]
fn array_locale_string_has_documented_hover_completion_help_and_string_result() {
 let source="function main(){const values:number[]=[1000.5,-0.0];const display=values.toLocaleString();console.log(display,values.length);}";
 assert!(tsrust_compiler::check(source).is_ok());
 let hover=ask(source,"textDocument/hover","toLocaleString();",0);
 let docs=hover["contents"]["value"].as_str().unwrap_or("");
 assert!(docs.contains("toLocaleString(): string"),"{hover}");
 assert!(docs.contains("Nested arrays"),"{hover}");assert!(docs.contains("Ownership:"),"{hover}");
 let hover=ask(source,"textDocument/hover","display=values",0);
 assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("string"),"{hover}");
 let help=ask(source,"textDocument/signatureHelp", ");console",0);
 assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),0,"{help}");
 let partial="function main(){const values=[1];values.;}";
 let completion=ask(partial,"textDocument/completion",";}",0);
 assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="toLocaleString"),"{completion}");
}

#[test]
fn array_locale_string_catalog_examples_match_native_output() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["Array"].as_array().unwrap().iter().find(|entry|entry["name"]=="toLocaleString").unwrap();
    let mut source = CatalogExampleSource::default();
    let mut expected = String::new();
    for example in entry["examples"].as_array().unwrap() {
        let witness = source.push(example["source"].as_str().unwrap());
        tsrust_compiler::check(&witness).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }
    let analysis=tsrust_compiler::check(&source.finish()).unwrap();
    let dir=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/editor/array-locale-string-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input=dir.join("examples.rs");let binary=dir.join("examples");
    std::fs::write(&input,analysis.emit_rust()).unwrap();
    let compilation=std::process::Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"rustc","--edition=2024","-Awarnings"]).arg(&input).arg("-o").arg(&binary).output().unwrap();
    assert!(compilation.status.success(),"{}",String::from_utf8_lossy(&compilation.stderr));
    let output=std::process::Command::new(binary).output().unwrap();
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
    assert!(output.stderr.is_empty());
}


#[test]
fn string_normalize_has_hover_completion_help_and_string_result() {
 let source="function main(){const text=\"é\";const normalized=text.normalize(\"NFKC\");console.log(normalized);}";
 assert!(tsrust_compiler::check(source).is_ok());
 let hover=ask(source,"textDocument/hover","normalize(",0);
 let docs=hover["contents"]["value"].as_str().unwrap_or("");
 assert!(docs.contains("normalize(form?: string): string"),"{hover}");
 assert!(docs.contains("NFKD"),"{hover}");assert!(docs.contains("Ownership:"),"{hover}");
 let hover=ask(source,"textDocument/hover","normalized=text",0);
 assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("string"),"{hover}");
 let help=ask(source,"textDocument/signatureHelp", "\"NFKC\");",0);
 assert_eq!(help["signatures"][0]["parameters"][0]["label"],"form?: string","{help}");
 let partial="function main(){const text=\"é\";text.;}";
 let completion=ask(partial,"textDocument/completion",";}",0);
 assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="normalize"),"{completion}");
}

#[test]
fn string_normalize_catalog_examples_match_native_output() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entry = catalog["string"].as_array().unwrap().iter().find(|entry|entry["name"]=="normalize").unwrap();
    let mut source = CatalogExampleSource::default();
    let mut expected = String::new();
    for example in entry["examples"].as_array().unwrap() {
        let witness = source.push(example["source"].as_str().unwrap());
        tsrust_compiler::check(&witness).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }
    let analysis=tsrust_compiler::check(&source.finish()).unwrap();
    let dir=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/editor/string-normalize-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input=dir.join("examples.rs");let binary=dir.join("examples");
    std::fs::write(&input,analysis.emit_rust()).unwrap();
    let compilation=std::process::Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"rustc","--edition=2024","-Awarnings"]).arg(&input).arg("-o").arg(&binary).output().unwrap();
    assert!(compilation.status.success(),"{}",String::from_utf8_lossy(&compilation.stderr));
    let output=std::process::Command::new(binary).output().unwrap();
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
    assert!(output.stderr.is_empty());
}


#[test]
fn tuple_types_have_positional_hover_results_length_and_completion() {
 let source="function main(){const item:[int,string]=[2,\"cat\"];const first=item[0];const text=item[1].slice();console.log(first,text,item.length);}";
 tsrust_compiler::check(source).unwrap();
 let item=ask(source,"textDocument/hover","item:[",0);
 assert!(item["contents"]["value"].as_str().unwrap_or("").contains("[int,string]"),"{item}");
 let first=ask(source,"textDocument/hover","first=item",0);
 assert!(first["contents"]["value"].as_str().unwrap_or("").contains(": int"),"{first}");
 let text=ask(source,"textDocument/hover","text=item",0);
 assert!(text["contents"]["value"].as_str().unwrap_or("").contains(": string"),"{text}");
 let length=ask(source,"textDocument/hover","length);",0);
 assert!(length["contents"]["value"].as_str().unwrap_or("").contains("declared positions"),"{length}");
 let partial="function main(){const item:[int,string]=[2,\"cat\"];item.;}";
 let completion=ask(partial,"textDocument/completion",";}",0);
 assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="length"),"{completion}");
}

#[test]
fn tuple_types_catalog_example_matches_native_output() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let example=&catalog["Tuple"][0]["examples"][0];
    let source=format!("function main(){{{}}}",example["source"].as_str().unwrap());
    let analysis=tsrust_compiler::check(&source).unwrap();
    let dir=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/editor/tuple-types-doc-examples");
    std::fs::create_dir_all(&dir).unwrap();
    let input=dir.join("examples.rs");let binary=dir.join("examples");
    std::fs::write(&input,analysis.emit_rust()).unwrap();
    let compilation=std::process::Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"rustc","--edition=2024","-Awarnings"]).arg(&input).arg("-o").arg(&binary).output().unwrap();
    assert!(compilation.status.success(),"{}",String::from_utf8_lossy(&compilation.stderr));
    let output=std::process::Command::new(binary).output().unwrap();
    assert!(output.status.success());assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap());
}

#[test]
fn tuple_types_signature_help_keeps_positions_and_parameter_access() {
 let source="function pair(value:ReadOnly<[int,string]>):[int,string]{return [value[0],value[1].slice()];}function main(){const item:[int,string]=[1,\"x\"];pair(item);}";
 tsrust_compiler::check(source).unwrap();
 let help=ask(source,"textDocument/signatureHelp","item);}",0);
 let label=help["signatures"][0]["label"].as_str().unwrap_or("");
 assert!(label.contains("[int,string]"),"{help}");
 assert!(help["signatures"][0]["parameters"][0]["label"].as_str().unwrap_or("").contains("ReadOnly<[int,string]>"),"{help}");
}

#[test]
fn array_entries_has_typed_pairs_hover_completion_and_signature_help() {
 let source="function main(){const words=[\"cat\",\"dog\"];const entries=words.entries();const index=entries[0][0];const text=entries[1][1].slice();console.log(index,text);}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","entries();",0);
 let docs=hover["contents"]["value"].as_str().unwrap_or("");
 assert!(docs.contains("entries(): [int,string][]"),"{hover}");
 assert!(docs.contains("eagerly"),"{hover}");assert!(docs.contains("Ownership:"),"{hover}");
 let entries=ask(source,"textDocument/hover","entries=words",0);
 assert!(entries["contents"]["value"].as_str().unwrap_or("").contains("[int,string][]"),"{entries}");
 let index=ask(source,"textDocument/hover","index=entries",0);
 assert!(index["contents"]["value"].as_str().unwrap_or("").contains(": int"),"{index}");
 let help=ask(source,"textDocument/signatureHelp",");const index",0);
 assert_eq!(help["signatures"][0]["parameters"],serde_json::json!([]),"{help}");
 let partial="function main(){const words=[\"cat\"];words.;}";
 let completion=ask(partial,"textDocument/completion",";}",0);
 assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="entries"),"{completion}");
}

#[test]
fn array_entries_catalog_examples_match_native_output() {
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let entry=catalog["Array"].as_array().unwrap().iter().find(|entry|entry["name"]=="entries").unwrap();
 let mut source=CatalogExampleSource::default();let mut expected=String::new();
 for example in entry["examples"].as_array().unwrap(){tsrust_compiler::check(&source.push(example["source"].as_str().unwrap())).unwrap();expected.push_str(example["output"].as_str().unwrap());}
 let analysis=tsrust_compiler::check(&source.finish()).unwrap();
 let dir=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/editor/array-entries-doc-examples");std::fs::create_dir_all(&dir).unwrap();
 let input=dir.join("examples.rs");let binary=dir.join("examples");std::fs::write(&input,analysis.emit_rust()).unwrap();
 let compilation=std::process::Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"rustc","--edition=2024","-Awarnings"]).arg(&input).arg("-o").arg(&binary).output().unwrap();assert!(compilation.status.success(),"{}",String::from_utf8_lossy(&compilation.stderr));
 let output=std::process::Command::new(binary).output().unwrap();assert!(output.status.success());assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
}

#[test]
fn array_entries_keeps_nullable_tuple_members_inside_array_element_types() {
 let source="function main(){const rows:[int,string|null][]=[[4,null]];const pairs=rows.entries();console.log(pairs.length);}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","entries();",0);
 assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("entries(): [int,[int,string|null]][]"),"{hover}");
 let partial="function main(){const rows:[int,string|null][]=[[4,null]];rows.;}";
 let completion=ask(partial,"textDocument/completion",";}",0);
 assert!(completion.as_array().unwrap().iter().any(|entry|entry["label"]=="entries"),"{completion}");
}

#[test]
fn object_keys_has_static_docs_result_completion_and_parameter_help() {
 let source="type Row={a:int;b:string;};function main(){const row:Row={b:\"x\",a:1};const names=Object.keys(row);console.log(names.length);}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","keys(row)",0);
 let text=hover["contents"]["value"].as_str().unwrap_or("");
 assert!(text.contains("string[]")&&text.contains("constructor")&&text.contains("Ownership"),"{hover}");
 let result=ask(source,"textDocument/hover","names=",0);
 assert!(result["contents"]["value"].as_str().unwrap_or("").contains("string[]"),"{result}");
 let help=ask(source,"textDocument/signatureHelp","row);console",0);
 assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),1,"{help}");
 let partial="function main(){Object.;}";
 let completion=ask(partial,"textDocument/completion",";}",0);
 assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="keys"),"{completion}");
}

#[test]
fn object_values_and_entries_have_docs_checked_results_completion_and_help() {
    let source = "type Row={id:int;name:string;};type Size={w:int;h:int;};function main(){const row:Row={name:\"a\",id:1};const size:Size={w:1,h:2};const mixed=Object.values(row);const sides=Object.values(size);const pairs=Object.entries(row);console.log(mixed.length,sides.length,pairs.length);}";
    tsrust_compiler::check(source).unwrap();
    let text = |needle: &str| ask(source, "textDocument/hover", needle, 0)["contents"]["value"].as_str().unwrap_or("").replace(" | ", "|").replace(", ", ",");
    let docs = text("values(row)");
    assert!(docs.contains("values(value: unknown): Value[]") && docs.contains("same order as `Object.keys`") && docs.contains("Ownership"), "{docs}");
    let docs = text("entries(row)");
    assert!(docs.contains("entries(value: unknown): [string,Value][]") && docs.contains("`[name,value]` pairs") && docs.contains("array.entries()"), "{docs}");
    assert!(text("mixed=").contains("const mixed: (int|string)[]"), "{}", text("mixed="));
    assert!(text("sides=").contains("const sides: int[]"), "{}", text("sides="));
    assert!(text("pairs=").contains("[string,int|string][]"), "{}", text("pairs="));
    let help = ask(source, "textDocument/signatureHelp", "row);const sides", 0);
    assert_eq!(help["signatures"][0]["parameters"][0]["label"], "value: unknown", "{help}");
    let partial = "function main(){Object.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    for label in ["is", "keys", "values", "entries"] {
        assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == label), "{label}: {completion}");
    }
}

#[test]
fn object_keys_catalog_examples_match_native_output() {
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let entry=catalog["Object"].as_array().unwrap().iter().find(|entry|entry["name"]=="keys").unwrap();
 let mut source=CatalogExampleSource::default();let mut expected=String::new();
 for example in entry["examples"].as_array().unwrap(){tsrust_compiler::check(&source.push(example["source"].as_str().unwrap())).unwrap();expected.push_str(example["output"].as_str().unwrap());}
 let analysis=tsrust_compiler::check(&source.finish()).unwrap();
 let dir=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/editor/object-keys-doc-examples");std::fs::create_dir_all(&dir).unwrap();
 let input=dir.join("examples.rs");let binary=dir.join("examples");std::fs::write(&input,analysis.emit_rust()).unwrap();
 let compilation=std::process::Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"rustc","--edition=2024","-Awarnings"]).arg(&input).arg("-o").arg(&binary).output().unwrap();assert!(compilation.status.success(),"{}",String::from_utf8_lossy(&compilation.stderr));
 let output=std::process::Command::new(binary).output().unwrap();assert!(output.status.success());assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
}


#[test]
fn data_unions_typeof_guards_provide_checked_string_members(){
 let source="function main(){const value:int|string|boolean=\"text\";if(typeof value===\"string\"){console.log(value.toUpperCase());}}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","toUpperCase",0);
 assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("toUpperCase"),"{hover}");
 let partial="function main(){const value:int|string|boolean=\"text\";if(typeof value===\"string\"){value.";
 let completion=ask(partial,"textDocument/completion","",partial.len());
 assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="toUpperCase"),"{completion}");
}

#[test]
fn data_unions_catalog_examples_match_native_output() {
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let mut source=CatalogExampleSource::default();let mut expected=String::new();
 let entries=catalog["globals"].as_array().unwrap().iter().filter(|entry|matches!(entry["name"].as_str(),Some("typeof"|"null"|"undefined"|"void"|"String"|"Boolean"))).chain(catalog["ArrayConstructor"].as_array().unwrap().iter().filter(|entry|entry["name"]=="isArray"));
 for entry in entries {for example in entry["examples"].as_array().unwrap(){tsrust_compiler::check(&source.push(example["source"].as_str().unwrap())).unwrap();expected.push_str(example["output"].as_str().unwrap());}}
 let analysis=tsrust_compiler::check(&source.finish()).unwrap();
 let dir=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/editor/data-unions-doc-examples");std::fs::create_dir_all(&dir).unwrap();
 let input=dir.join("examples.rs");let binary=dir.join("examples");std::fs::write(&input,analysis.emit_rust()).unwrap();
 let compilation=std::process::Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"rustc","--edition=2024","-Awarnings"]).arg(&input).arg("-o").arg(&binary).output().unwrap();assert!(compilation.status.success(),"{}",String::from_utf8_lossy(&compilation.stderr));
 let output=std::process::Command::new(binary).output().unwrap();assert!(output.status.success());assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
}

#[test]
fn borrowed_optional_fallbacks_show_string_type_and_members(){
 let source="type Row={label:string;};function main(){const rows:(Row|null)[]=[{label:\"x\"}];const label=rows[0]?.label??\"none\";console.log(label.toUpperCase());}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","label.to",0);
 assert!(hover.to_string().contains("string"),"{hover}");
 let member=ask(source,"textDocument/hover","toUpperCase",0);
 assert!(member.to_string().contains("toUpperCase"),"{member}");
}
#[test]
fn destructuring_defaults_show_selected_types_and_parameter_members(){
 let source="type Row={label:string|undefined;};function describe({label=\"none\"}:Row):string{return label.toUpperCase();}function main(){const values:int[]=[];const [value=7]=values;console.log(value+1,describe({label:undefined}));}";
 tsrust_compiler::check(source).unwrap();
 let value=ask(source,"textDocument/hover","value+1",0);
 assert!(value.to_string().contains("int"),"{value}");
 let label=ask(source,"textDocument/hover","label.toUpper",0);
 assert!(label.to_string().contains("string"),"{label}");
 let member=ask(source,"textDocument/hover","toUpperCase",0);
 assert!(member.to_string().contains("toUpperCase"),"{member}");
}
#[test]
fn destructuring_default_expressions_do_not_declare_fallback_references(){
 let source="function main(){const fallback=4;const values:int[]=[];const [value=fallback]=values;console.log(value,fallback);}";
 tsrust_compiler::check(source).unwrap();
 let definition=ask(source,"textDocument/definition","fallback);",0);
 assert_eq!(definition["range"]["start"],json!(at(source,"fallback=4",0)),"{definition}");
 let references=ask(source,"textDocument/references","fallback);",0);
 assert_eq!(references.as_array().unwrap().len(),3,"{references}");
 let hover=ask(source,"textDocument/hover","fallback);",0);
 assert!(hover.to_string().contains("int"),"{hover}");
}

#[test]
fn destructuring_defaults_resolve_earlier_parameter_names(){
 let source="function sum([first=3]:int[],last:int=first+1):int{return first+last;}function main(){console.log(sum([]));}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","first+1",0);
 assert!(hover.to_string().contains("int"),"{hover}");
 let definition=ask(source,"textDocument/definition","first+1",0);
 assert_eq!(definition["range"]["start"],json!(at(source,"first=3",0)),"{definition}");
}

#[test]
fn object_has_own_docs_and_boolean_results(){
 let source="function main(){const values=[1];const present=Object.hasOwn(values,0);console.log(present);}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","hasOwn",0);assert!(hover.to_string().contains("UTF-16"),"{hover}");
 let result=ask(source,"textDocument/hover","present);",0);assert!(result.to_string().contains("boolean"),"{result}");
 let help=ask(source,"textDocument/signatureHelp","0);",0);assert_eq!(help["activeParameter"],1);assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),2);assert!(help.to_string().contains("boolean"),"{help}");
 let completion=ask("function main(){Object.;}","textDocument/completion",";}",0);assert!(completion.to_string().contains("hasOwn"),"{completion}");
}

#[test]
fn destructuring_rest_names_show_array_types_and_definitions(){
 let source="function tail([,...rest]:string[]):string[]{return rest;}function main(){const words=[\"a\",\"b\"];const [first,...remaining]=words;remaining.push(\"c\");}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","remaining.push",0);assert!(hover.to_string().contains("string[]"),"{hover}");
 let definition=ask(source,"textDocument/definition","remaining.push",0);assert_eq!(definition["range"]["start"],json!(at(source,"remaining]=",0)),"{definition}");
 let parameter=ask(source,"textDocument/hover","rest;",0);assert!(parameter.to_string().contains("string[]"),"{parameter}");
}
#[test]
fn tuple_rest_keeps_editor_position_types(){
 let source="function main(){const values:[int,string,int[]]=[1,\"a\",[2]];const [first,...rest]=values;const text=rest[0];text.toUpperCase();}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","text.toUpper",0);assert!(hover.to_string().contains("string"),"{hover}");
 let method=ask(source,"textDocument/hover","toUpperCase",0);assert!(method.to_string().contains("toUpperCase"),"{method}");
}
#[test]
fn record_rest_has_typed_members_and_parameter_definitions(){
 let source="type Row={name:string;count:int;};function count({name,...rest}:ReadOnly<Row>):int{return rest.count;}function main(){const row:Row={name:\"a\",count:2};const {name,...remaining}=row;const count=remaining.count;}";
 tsrust_compiler::check(source).unwrap();
 let member=ask(source,"textDocument/hover","count;}",0);assert!(member.to_string().contains("int"),"{member}");
 let definition=ask(source,"textDocument/definition","rest.count",0);assert_eq!(definition["range"]["start"],json!(at(source,"rest}:",0)),"{definition}");
 let remaining=ask(source,"textDocument/hover","remaining.count",0);assert!(remaining.to_string().contains("count"),"{remaining}");
 let completion=ask(source,"textDocument/completion","count;}",1);assert!(completion.to_string().contains("\"label\":\"count\""),"{completion}");assert!(!completion.to_string().contains("\"label\":\"name\""),"{completion}");
}
#[test]
fn object_spread_inferred_fields_have_hover_and_completion(){
 let source="type Row={name:string;count:int;};function main(){const row:Row={name:\"a\",count:2};const result={...row,active:true,count:4};const count=result.count;const flag=result.active;}";
 tsrust_compiler::check(source).unwrap();
 let count=ask(source,"textDocument/hover","count=result",0);assert!(count.to_string().contains("int"),"{count}");
 let flag=ask(source,"textDocument/hover","flag=result",0);assert!(flag.to_string().contains("boolean"),"{flag}");
 let completion=ask(source,"textDocument/completion","active;",0);assert!(completion.to_string().contains("\"label\":\"active\""),"{completion}");
 let hover=ask(source,"textDocument/hover","result.active",0);assert!(hover.to_string().contains("active: boolean"),"{hover}");assert!(!hover.to_string().contains("rest#"),"{hover}");
}

#[test]
fn nested_rest_names_have_types_and_definitions(){
 let source="function main(){const values=[\"a\",\"b\"];const [,...[second]]=values;second.toUpperCase();}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","second.toUpper",0);assert!(hover.to_string().contains("string"),"{hover}");
 let definition=ask(source,"textDocument/definition","second.toUpper",0);assert_eq!(definition["range"]["start"],json!(at(source,"second]]=",0)),"{definition}");
}

#[test]
fn for_initializer_patterns_have_types_and_loop_scoped_definitions(){
 let source="function main(){for(let [i,end]=[0,3];i<end;i++){const value=i+1;}}";
 tsrust_compiler::check(source).unwrap();
 let hover=ask(source,"textDocument/hover","i<end",0);assert!(hover.to_string().contains("int"),"{hover}");
 let definition=ask(source,"textDocument/definition","i+1",0);assert_eq!(definition["range"]["start"],json!(at(source,"i,end",0)),"{definition}");
}

#[test]
fn declaration_lists_declare_every_name_with_its_type() {
    let source = "function pair():[int,string]{return [1,\"a\"];}\nfunction main(){let count=1,label=\"x\".repeat(2),[left,right]=pair(),total:number=2.5;for(let i=0,j=3;i<j;i++,j--){count+=i+j;}console.log(count,label.length,left,right.length,total);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let text = |needle: &str, n: usize| ask(source, "textDocument/hover", needle, n)["contents"]["value"].as_str().unwrap_or("").to_owned();
    assert!(text("count=1", 0).contains("let count: int"), "{}", text("count=1", 0));
    assert!(text("label=", 0).contains("let label: string"), "{}", text("label=", 0));
    assert!(text("left,right", 0).contains("let left: int"), "{}", text("left,right", 0));
    assert!(text("right]", 0).contains("let right: string"), "{}", text("right]", 0));
    assert!(text("total:number", 0).contains("let total: number"), "{}", text("total:number", 0));
    assert!(text("j=3", 0).contains("let j: int"), "{}", text("j=3", 0));
    // Uses resolve to the later declarators.
    assert!(text("label.length", 0).contains("let label: string"), "{}", text("label.length", 0));
    assert!(text("j--", 0).contains("let j: int"), "{}", text("j--", 0));
    let definition = ask(source, "textDocument/definition", "label.length", 0);
    assert_eq!(definition["range"]["start"]["character"], at(source, "label=", 0).character, "{definition}");
    let partial = "function main(){let count=1,label=\"x\";label.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "toUpperCase"), "{completion}");
}

#[test]
fn destructuring_assignment_targets_and_shorthand_properties_resolve_to_their_names() {
    let source = "type Point={x:int;label:string;};\nfunction main(){let x=1,label=\"a\",other=2;[x,other]=[other,x];const point:Point={x,label};({x,label}=point);console.log(x,other,label.length);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let text = |needle: &str, n: usize| ask(source, "textDocument/hover", needle, n)["contents"]["value"].as_str().unwrap_or("").to_owned();
    // Targets of an array pattern assignment.
    assert!(text("x,other]=", 0).contains("let x: int"), "{}", text("x,other]=", 0));
    assert!(text("other]=", 0).contains("let other: int"), "{}", text("other]=", 0));
    // Shorthand properties read the variables of the same names.
    assert!(text("x,label};(", 0).contains("let x: int"), "{}", text("x,label};(", 0));
    assert!(text("label};(", 0).contains("let label: string"), "{}", text("label};(", 0));
    // Targets of a record pattern assignment.
    assert!(text("label}=point", 0).contains("let label: string"), "{}", text("label}=point", 0));
    let definition = ask(source, "textDocument/definition", "label}=point", 0);
    assert_eq!(definition["range"]["start"]["character"], at(source, "label=\"a\"", 0).character, "{definition}");
    let references = ask(source, "textDocument/references", "other=2", 0);
    assert_eq!(references.as_array().unwrap().len(), 4, "{references}");
}

#[test]
fn single_statement_bodies_keep_names_types_and_loop_scopes() {
    let source = "function main(){let total=0;const words=[\"a\",\"bb\"];for(let i=0;i<2;i++)total+=i;for(const word of words)total+=word.length;if(total>1)total-=1;else total+=1;while(total>9)total--;for(let i=0;i<1;i++)console.log(i,total);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let text = |needle: &str, n: usize| ask(source, "textDocument/hover", needle, n)["contents"]["value"].as_str().unwrap_or("").to_owned();
    assert!(text("i=0", 0).contains("let i: int"), "{}", text("i=0", 0));
    assert!(text("i;for", 0).contains("let i: int"), "{}", text("i;for", 0));
    assert!(text("word of", 0).contains("const word: string"), "{}", text("word of", 0));
    assert!(text("word.length", 0).contains("const word: string"), "{}", text("word.length", 0));
    assert!(text("total-=1", 0).contains("let total: int"), "{}", text("total-=1", 0));
    // Each loop variable resolves to its own loop header.
    let first = ask(source, "textDocument/definition", "i;for", 0);
    assert_eq!(first["range"]["start"]["character"], at(source, "i=0", 0).character, "{first}");
    let second = ask(source, "textDocument/definition", "i,total", 0);
    assert_eq!(second["range"]["start"]["character"], at(source, "i=0", 1).character, "{second}");
    let partial = "function main(){const words=[\"a\"];for(const word of words)word.;}";
    let completion = ask(partial, "textDocument/completion", ";}", 0);
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "toUpperCase"), "{completion}");
}

#[test]
fn switch_clauses_keep_names_types_and_narrowed_members() {
    let source = "function label(value:int|string,count:int):string{switch(value){case \"go\":return value.toUpperCase();case 1:return String(count+value);default:switch(count){case 0:return \"zero\";default:return \"many\";}}}\nfunction main(){console.log(label(\"go\",1));}";
    assert!(tsrust_compiler::check(source).is_ok());
    let text = |needle: &str, n: usize| ask(source, "textDocument/hover", needle, n)["contents"]["value"].as_str().unwrap_or("").replace(" | ", "|");
    assert!(text("value){", 0).contains("value: int|string"), "{}", text("value){", 0));
    assert!(text("count+value", 0).contains("count: int"), "{}", text("count+value", 0));
    assert!(text("count){", 0).contains("count: int"), "{}", text("count){", 0));
    assert!(!ask(source, "textDocument/hover", "toUpperCase()", 0).is_null());
    let definition = ask(source, "textDocument/definition", "count+value", 0);
    assert_eq!(definition["range"]["start"]["character"], at(source, "count:int", 0).character, "{definition}");
}

#[test]
fn function_expressions_declare_parameters_and_callable_bindings() {
    let source = "function main(){const offset=3;const shift=function (value:int,label:string):int{return value+offset+label.length;};const words=[\"a\"];const upper=words.map(function (word){return word.toUpperCase();});console.log(shift(1,\"x\"),upper.length);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let text = |needle: &str, n: usize| ask(source, "textDocument/hover", needle, n)["contents"]["value"].as_str().unwrap_or("").to_owned();
    assert!(text("value:int", 0).contains("value: int"), "{}", text("value:int", 0));
    assert!(text("value+offset", 0).contains("value: int"), "{}", text("value+offset", 0));
    assert!(text("label.length", 0).contains("label: string"), "{}", text("label.length", 0));
    assert!(text("offset+label", 0).contains("const offset: int"), "{}", text("offset+label", 0));
    assert!(text("word.toUpperCase", 0).contains("string"), "{}", text("word.toUpperCase", 0));
    let shift = text("shift(1", 0);
    assert!(shift.contains("shift") && shift.contains("int"), "{shift}");
    let help = ask(source, "textDocument/signatureHelp", "1,\"x\")", 0);
    assert_eq!(help["signatures"][0]["parameters"].as_array().map(Vec::len), Some(2), "{help}");
    let definition = ask(source, "textDocument/definition", "label.length", 0);
    assert_eq!(definition["range"]["start"]["character"], at(source, "label:string", 0).character, "{definition}");
}

#[test]
fn global_number_names_have_hover_help_completion_and_types() {
    let source = "function main(){const count=parseInt(\"42px\",10);const ratio=parseFloat(\"1.5\");const bad=isNaN(count);const fine=isFinite(ratio);let limit=Infinity;const missing=NaN;const low=-Infinity;console.log(count,ratio,bad,fine,limit,missing,low);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let text = |needle: &str, n: usize| ask(source, "textDocument/hover", needle, n)["contents"]["value"].as_str().unwrap_or("").to_owned();
    assert!(text("parseInt(", 0).contains("parseInt(text: string, radix?: int): number"), "{}", text("parseInt(", 0));
    assert!(text("parseFloat(", 0).contains("parseFloat(text: string): number"));
    assert!(text("isNaN(", 0).contains("isNaN(value: number): boolean"));
    assert!(text("isFinite(", 0).contains("isFinite(value: number): boolean"));
    assert!(text("Infinity;", 0).contains("Infinity: number"), "{}", text("Infinity;", 0));
    assert!(text("NaN;", 0).contains("NaN: number"), "{}", text("NaN;", 0));
    for (needle, ty) in [("count,", "count: number"), ("ratio,", "ratio: number"), ("bad,", "bad: boolean"), ("fine,", "fine: boolean"), ("limit,", "limit: number"), ("missing,", "missing: number"), ("low)", "low: number")] {
        assert!(text(needle, 0).contains(ty), "{needle}: {}", text(needle, 0));
    }
    let help = ask(source, "textDocument/signatureHelp", "10)", 0);
    assert_eq!(help["signatures"][0]["parameters"].as_array().map(Vec::len), Some(2), "{help}");
    assert_eq!(help["activeParameter"], 1, "{help}");
    let typed = "function main(){const value=is";
    let mut pos = at(typed, "=is", 0);
    pos.character += 3;
    let items = SourceIndex::new(typed).request("textDocument/completion", &json!({"position":pos}), "file:///test.tsr").unwrap();
    for label in ["isNaN", "isFinite"] {
        let item = items.as_array().unwrap().iter().find(|item| item["label"] == label).unwrap_or_else(|| panic!("{label} {items}"));
        assert!(item["documentation"]["value"].as_str().unwrap().contains("```tsrust"), "{item}");
    }
    // A declaration with the same name is the user's, not the global.
    let shadowed = "function parseInt(text:string):int{return text.length;}function main(){console.log(parseInt(\"ab\"));}";
    let hover = ask(shadowed, "textDocument/hover", "parseInt(\"ab\"", 0)["contents"]["value"].as_str().unwrap_or("").to_owned();
    assert!(hover.contains("function parseInt(text:string):int") && !hover.contains("radix"), "{hover}");
}

#[test]
fn deferred_let_declarations_keep_types_definitions_and_members() {
    let source = "type Point={x:int;y:int;};function main(){let label:string;let point:Point,count:int;count=2;if(count>1){label=\"many\";}else{label=\"one\";}point={x:count,y:1};console.log(label.length,point.x,count);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let text = |needle: &str, n: usize| ask(source, "textDocument/hover", needle, n)["contents"]["value"].as_str().unwrap_or("").to_owned();
    assert!(text("label:string", 0).contains("label: string"), "{}", text("label:string", 0));
    assert!(text("label=\"many\"", 0).contains("label: string"), "{}", text("label=\"many\"", 0));
    assert!(text("label.length", 0).contains("label: string"), "{}", text("label.length", 0));
    assert!(text("point={", 0).contains("point: Point"), "{}", text("point={", 0));
    assert!(text("count=2", 0).contains("count: int"), "{}", text("count=2", 0));
    assert!(!ask(source, "textDocument/hover", "length,", 0).is_null());
    let definition = ask(source, "textDocument/definition", "point.x", 0);
    assert_eq!(definition["range"]["start"]["character"], at(source, "point:Point", 0).character, "{definition}");
    let typed = "type Point={x:int;y:int;};function main(){let point:Point;point={x:1,y:2};point.";
    let mut pos = at(typed, "point.", 0);
    pos.character += 6;
    let items = SourceIndex::new(typed).request("textDocument/completion", &json!({"position":pos}), "file:///test.tsr").unwrap();
    assert!(items.as_array().unwrap().iter().any(|item| item["label"] == "x"), "{items}");
}

#[test]
fn in_operator_and_for_in_have_boolean_and_string_types() {
 let completion=ask("function main(){i}","textDocument/completion","i}",0);
 assert!(completion.to_string().contains("\"label\":\"in\""),"{completion}");
 let source="type P={x:int;y:int;}; function main(){const point:P={x:1,y:2};const present=\"x\" in point;for(const key in point){console.log(key,present);}}";
 tsrust_compiler::check(source).unwrap();
 let key=ask(source,"textDocument/hover","key,present",0);assert!(key.to_string().contains("string"),"{key}");
 let present=ask(source,"textDocument/hover","present);",0);assert!(present.to_string().contains("boolean"),"{present}");
 let definition=ask(source,"textDocument/definition","key,present",0);assert_eq!(definition["range"]["start"],json!(at(source,"key in",0)),"{definition}");
}

#[test]
fn labels_resolve_separately_from_value_bindings() {
 let source="function main(){const outer=4;outer:for(let i=0;i<2;i++){while(true){break outer;}}console.log(outer);}";
 tsrust_compiler::check(source).unwrap();
 let definition=ask(source,"textDocument/definition","outer;",0);assert_eq!(definition["range"]["start"],json!(at(source,"outer:for",0)),"{definition}");
 let hover=ask(source,"textDocument/hover","outer;",0);assert!(hover.to_string().contains("label outer"),"{hover}");
 let value=ask(source,"textDocument/hover","outer);",0);assert!(value.to_string().contains("int"),"{value}");
 let labels=ask(source,"textDocument/references","outer;",0);assert_eq!(labels.as_array().unwrap().len(),2,"{labels}");
 let values=ask(source,"textDocument/references","outer);",0);assert_eq!(values.as_array().unwrap().len(),2,"{values}");
}

#[test]
fn callable_expression_results_and_iife_parameters_have_types() {
 let source="function make(base:int):Function<int,[value:int]>{return (value:int):int=>base+value;}function main(){const result=make(10)(2);const inline=((value)=>value+1)(3);console.log(result,inline);}";
 tsrust_compiler::check(source).unwrap();
 for needle in ["result,inline","inline);","value+1"] {
  let hover=ask(source,"textDocument/hover",needle,0);
  assert!(hover.to_string().contains("int"),"{needle}: {hover}");
 }
 let definition=ask(source,"textDocument/definition","value+1",0);
 assert_eq!(definition["range"]["start"],json!(at(source,"value)=>",0)),"{definition}");
}

#[test]
fn callable_expression_signature_help_uses_the_expression_contract() {
 let source="function make(base:int):Function<int,[value:int]>{return (value:int):int=>base+value;}function main(){const result=make(10)(2);const inline=((value:int,extra:int=1):int=>value+extra)(3,4);console.log(result,inline);}";
 tsrust_compiler::check(source).unwrap();
 let factory=ask(source,"textDocument/signatureHelp","2);",0);
 assert_eq!(factory["signatures"][0]["parameters"].as_array().map(Vec::len),Some(1),"{factory}");
 assert!(factory["signatures"][0]["parameters"][0]["label"].as_str().unwrap_or("").contains("value"),"{factory}");
 let iife=ask(source,"textDocument/signatureHelp","4);",0);
 assert_eq!(iife["signatures"][0]["parameters"].as_array().map(Vec::len),Some(2),"{iife}");
 assert_eq!(iife["activeParameter"],1,"{iife}");
}
