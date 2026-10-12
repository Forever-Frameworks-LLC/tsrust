mod support;
#[test]
fn jsonl_writer_creates_flushes_and_closes_owned_output() {
    let dir = support::TempDirectory::new();
    let output = dir.0.join("output.jsonl");
    std::fs::write(&output, "old contents").unwrap();
    let source = format!(
        r#"import {{createJsonl,flushJsonl,closeJsonlWriter}} from 'std:json';
 function main(){{match(createJsonl({path:?},64,8)){{Ok(writer)=>{{match(flushJsonl(writer)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}match(closeJsonlWriter(writer)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}}},Err(error)=>{{throw error.message;}}}}}}
 "#,
        path = output.to_str().unwrap()
    );
    let path = dir.source(&support::format_round_trip(&source));
    support::success(&support::run(support::cli().arg("run").arg(path), 90));
    assert_eq!(std::fs::read(output).unwrap(), b"");
}

#[test]
fn jsonl_writer_borrows_typed_records_and_stages_encoding_errors() {
    let dir = support::TempDirectory::new();
    let output = dir.0.join("typed.jsonl");
    let source = format!(
        r#"import {{createJsonl,writeJsonl,closeJsonlWriter}} from 'std:json';import {{strictEqual}} from 'std:testing';
 type Item={{name:string;count:int;}};
 function main(){{const item:Item={{name:'hello',count:42}};match(createJsonl({path:?})){{Ok(writer)=>{{
  match(writeJsonl(writer,item)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}
  strictEqual(item.name,'hello');
  match(writeJsonl(writer,undefined)){{Ok(_)=>{{throw 'undefined accepted';}},Err(error)=>{{strictEqual(error.code,'undefined-value');strictEqual(error.line,2);}}}}
  match(closeJsonlWriter(writer)){{Ok(_)=>{{throw 'terminal error lost';}},Err(error)=>{{strictEqual(error.code,'undefined-value');}}}}
 }},Err(error)=>{{throw error.message;}}}}}}
 "#,
        path = output.to_str().unwrap()
    );
    let path = dir.source(&support::format_round_trip(&source));
    support::success(&support::run(support::cli().arg("run").arg(path), 90));
    assert_eq!(
        std::fs::read_to_string(output).unwrap(),
        "{\"count\":42,\"name\":\"hello\"}\n"
    );
}

#[test]
fn jsonl_writer_invalid_options_do_not_truncate_and_ownership_is_checked() {
    let dir = support::TempDirectory::new();
    let output = dir.0.join("keep.jsonl");
    std::fs::write(&output, "preserve").unwrap();
    let source = format!(
        r#"import {{createJsonl}} from 'std:json';import {{strictEqual}} from 'std:testing';function main(){{
 match(createJsonl({path:?},-1)){{Ok(_)=>{{throw 'negative accepted';}},Err(error)=>{{strictEqual(error.code,'invalid-limits');}}}}
 match(createJsonl({path:?},32,0)){{Ok(_)=>{{throw 'zero capacity accepted';}},Err(error)=>{{strictEqual(error.code,'capacity-range');}}}}
 }}"#,
        path = output.to_str().unwrap()
    );
    let path = dir.source(&support::format_round_trip(&source));
    support::success(&support::run(support::cli().arg("run").arg(path), 90));
    assert_eq!(std::fs::read_to_string(output).unwrap(), "preserve");
    for body in [
        "clone(writer);",
        "closeJsonlWriter(writer);flushJsonl(writer);",
        "const fake:JsonlWriter={};",
    ] {
        let source = format!(
            "import {{JsonlWriter,closeJsonlWriter,flushJsonl}} from 'std:json';function invalid(writer:JsonlWriter){{{body}}}"
        );
        let path = dir.source(&support::format_round_trip(&source));
        let output = support::run(support::cli().arg("check").arg(path), 30);
        assert!(!output.status.success(), "{source}");
    }
}

#[test]
fn jsonl_writer_catalog_example_executes() {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let dir = support::TempDirectory::new();
    let mut examples = std::collections::HashSet::new();
    for name in [
        "JsonlWriter",
        "createJsonl",
        "writeJsonl",
        "flushJsonl",
        "closeJsonlWriter",
    ] {
        let entry = catalog["std:json"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["name"] == name)
            .unwrap();
        for example in entry["examples"].as_array().unwrap() {
            let source = example["source"].as_str().unwrap();
            if !examples.insert(source) {
                continue;
            }
            let path = dir.source(&support::format_round_trip(source));
            let output = support::run(support::cli().arg("run").arg(path).current_dir(&dir.0), 90);
            support::success(&output);
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                example["output"].as_str().unwrap()
            );
            assert_eq!(
                std::fs::read_to_string(dir.0.join("jsonl-writer-example.jsonl")).unwrap(),
                "{\"name\":\"hello\"}\n"
            );
        }
    }
}
