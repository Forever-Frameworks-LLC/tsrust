use serde_json::{Value, json};
use tsrust_lsp::{Snapshot, code_actions, diagnostics};

fn snapshot(source: &str, version: i32) -> Snapshot {
    Snapshot {
        uri: "file:///fix.tsr".into(),
        version,
        generation: 1,
        text: source.into(),
    }
}
fn request(source: &str) -> Value {
    json!({"range": diagnostics(source, "file:///fix.tsr").unwrap()[0].range, "context":{"diagnostics":[]}})
}
#[test]
fn compiler_edits_are_validated_and_versioned_with_utf16_and_comments() {
    for (source, replacement) in [
        (
            "function main(){const text=\"😀\" /* keep */\r\nconsole.log(text);}",
            ";",
        ),
        ("function main(){console.log(\"😀\";}", ")"),
        ("function main(){const values=[1,2;}", "]"),
        ("function main(){const text=\"😀\";", "}"),
    ] {
        let error = tsrust_compiler::check(source).err().unwrap();
        let fix = &error.fixes[0];
        assert_eq!(fix.replacement, replacement);
        let mut edited = source.to_owned();
        edited.replace_range(fix.span.start..fix.span.end, &fix.replacement);
        assert!(tsrust_compiler::check(&edited).is_ok(), "{edited}");
        let actions = code_actions(&snapshot(source, 7), &request(source)).unwrap();
        assert_eq!(actions.as_array().unwrap().len(), 1, "{source}");
        let change = &actions[0]["edit"]["documentChanges"][0];
        assert_eq!(change["textDocument"]["version"], 7);
        assert_eq!(change["edits"][0]["newText"], replacement);
        let prefix = &source[..fix.span.start];
        assert_eq!(
            change["edits"][0]["range"]["start"]["character"],
            prefix.rsplit('\n').next().unwrap().encode_utf16().count()
        );
    }
}
#[test]
fn stale_context_unrelated_ranges_and_unsafe_or_incomplete_fixes_are_ignored() {
    let source = "function main(){const x=1}";
    let mut params = request(source);
    params["context"]["only"] = json!(["refactor"]);
    assert_eq!(
        code_actions(&snapshot(source, 2), &params).unwrap(),
        json!([])
    );
    params = request(source);
    params["range"] = json!({"start":{"line":0,"character":0},"end":{"line":0,"character":1}});
    assert_eq!(
        code_actions(&snapshot(source, 2), &params).unwrap(),
        json!([])
    );
    // Old diagnostics cannot generate edits after source repair.
    assert_eq!(
        code_actions(
            &snapshot("function main(){const x=1;}", 3),
            &request(source)
        )
        .unwrap(),
        json!([])
    );
    for source in [
        "function main(){const x:int=true;}",
        "function main(){const x=1 const y:int=true;}",
        "function main(){const x=1",
    ] {
        assert_eq!(
            code_actions(&snapshot(source, 1), &request(source)).unwrap(),
            json!([]),
            "{source}"
        );
    }
}

#[test]
fn clone_suggestion_repairs_the_transfer_and_keeps_owned_values_independent() {
    let source = r#"type User={name:string;};function take(user:User):User{return user;}function main(){const user:User={name:"Pat"};const saved=take(user);saved.name="Sam";console.log(user.name);}"#;
    let error = tsrust_compiler::check(source).err().unwrap();
    assert_eq!(error.code, "transferred");
    let actions = code_actions(&snapshot(source, 9), &request(source)).unwrap();
    assert_eq!(
        actions[0]["title"],
        "Clone before transferring (copies data; may allocate)"
    );
    assert_eq!(
        actions[0]["edit"]["documentChanges"][0]["edits"][0]["newText"],
        "clone(user)"
    );
    let fix = &error.fixes[0];
    let mut edited = source.to_owned();
    edited.replace_range(fix.span.start..fix.span.end, &fix.replacement);
    assert!(tsrust_compiler::check(&edited).is_ok());
}

#[test]
fn successive_transfers_offer_one_complete_verified_clone_fix() {
    let source = r#"type User = { name: string; };
function take(user: User): User { return user; }
function main() {
    const user: User = { name: "Pat" };
    const user2 = user;
    const saved = take(user);
    console.log(saved.name);
    console.log(user.name);
}"#;
    let actions = code_actions(&snapshot(source, 12), &request(source)).unwrap();
    assert_eq!(
        actions[0]["title"],
        "Clone before 2 transfers (copies data; may allocate)"
    );
    let edits = actions[0]["edit"]["documentChanges"][0]["edits"]
        .as_array()
        .unwrap();
    assert_eq!(edits.len(), 2);
    // These two locations are before the diagnostic, not the failing later read.
    assert_eq!(edits[0]["range"]["start"]["line"], 4);
    assert_eq!(edits[1]["range"]["start"]["line"], 5);
    let edited = source
        .replace("const user2 = user;", "const user2 = clone(user);")
        .replace("take(user);", "take(clone(user));");
    assert!(tsrust_compiler::check(&edited).is_ok());
    let unsafe_source = source.replace(
        "console.log(user.name);",
        "const bad:int=true;console.log(user.name);",
    );
    assert_eq!(
        code_actions(&snapshot(&unsafe_source, 13), &request(&unsafe_source)).unwrap(),
        json!([])
    );
}

#[test]
fn typed_arrow_missing_opening_parenthesis_has_an_actionable_fix() {
    let source = "function main(){const users=[\"Alice\"];const result=users.map(user: string): string => {return user.toUpperCase();});console.log(result[0]);}";
    let reported = diagnostics(source, "file:///fix.tsr").unwrap();
    assert!(reported[0].message.contains("opening `(` is missing"));
    let actions = code_actions(&snapshot(source, 1), &request(source)).unwrap();
    assert_eq!(actions.as_array().unwrap().len(), 1);
    assert_eq!(actions[0]["title"], "Insert opening parenthesis for arrow parameter");
    assert_eq!(actions[0]["edit"]["documentChanges"][0]["edits"][0]["newText"], "(");
}
