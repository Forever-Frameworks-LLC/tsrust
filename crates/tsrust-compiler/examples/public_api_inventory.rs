//! Reproducible audit input; discovery is not semantic acceptance.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use tsrust_compiler::{native_bindings::BINDINGS, native_types};

fn main() {
    let catalog: BTreeMap<String, Vec<Value>> = serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut rows = BTreeMap::new();
    let mut modules = BTreeSet::new();
    for (group, entries) in catalog {
        if group.starts_with("std:") { modules.insert(group.clone()); }
        for entry in entries {
            let name = entry["name"].as_str().expect("catalog name");
            let key = format!("{group}::{name}");
            assert!(!rows.contains_key(&key), "duplicate catalog identity: {key}");
            rows.insert(key.clone(), json!({"identity":key,"group":group,"name":name,"catalog":entry,"review":"pending"}));
        }
    }
    // Some global/primitive hovers live outside builtins.json. Keep these in
    // discovery so a catalog-only audit cannot silently omit their spellings.
    let editor=include_str!("../../tsrust-lsp/src/intelligence.rs");
    let fallback=editor.split("let (signature, docs, parameters): (&str, &str, Vec<&str>) = match self.text(token) {").nth(1).expect("review changed editor fallback").split("_ => return None,").next().unwrap();
    for line in fallback.lines().map(str::trim).filter(|line|line.ends_with("=> (")) {
        let name=line.strip_prefix('"').unwrap().split('"').next().unwrap();
        let key=format!("globals::{name}");
        let row=rows.entry(key.clone()).or_insert_with(||json!({"identity":key,"group":"globals","name":name,"review":"pending"}));
        row["editorFallback"]=json!({"file":"crates/tsrust-lsp/src/intelligence.rs","anchor":"let (signature, docs, parameters)"});
    }
    for binding in BINDINGS {
        modules.insert(binding.module.to_owned());
        let key = format!("{}::{}", binding.module, binding.name);
        let row = rows.entry(key.clone()).or_insert_with(||json!({"identity":key,"group":binding.module,"name":binding.name,"review":"pending"}));
        assert!(row.get("binding").is_none(),"duplicate native binding");
        row["binding"]=json!({
            "parameters":binding.parameters.iter().enumerate().map(|(index,p)|json!({"name":p.name,"type":format!("{:?}",p.ty.signature()),"access":format!("{:?}",p.access),"optional":p.optional,"rest":p.rest,"alternatives":format!("{:?}",binding.argument_alternatives(index))})).collect::<Vec<_>>(),
            "result":format!("{:?}",binding.result.signature()),"returnedBorrow":format!("{:?}",binding.returned_borrow),"backend":format!("{:?}",binding.backend),"features":binding.features,
            "private":binding.module=="std:testing-runtime"
        });
    }
    // Discover declaration identities independently of catalog coverage. Reject
    // unfamiliar enum syntax rather than silently ignoring a new native type.
    let types=include_str!("../src/native_signatures.rs").split("pub enum NativeTypeId {").nth(1).unwrap().split('}').next().unwrap();
    for line in types.lines().map(str::trim).filter(|line|!line.is_empty()) {
        let name=line.strip_suffix(',').expect("review changed NativeTypeId syntax");
        assert!(name.chars().all(|c|c.is_ascii_alphanumeric()),"review native identity syntax");
        let declarations=modules.iter().filter_map(|module|native_types::lookup(module,name)).collect::<Vec<_>>();
        assert_eq!(declarations.len(),1,"missing or ambiguous declaration: {name}");
        let declaration=declarations[0];
        let key=format!("{}::{}",declaration.module,declaration.name);
        let row=rows.entry(key.clone()).or_insert_with(||json!({"identity":key,"group":declaration.module,"name":declaration.name,"review":"pending"}));
        row["declaration"]=json!({"representation":format!("{:?}",declaration.representation),"independentClone":declaration.independent_clone,"threadTransfer":declaration.thread_transfer,"typeParameters":declaration.type_parameters,"fields":declaration.fields.iter().map(|field|json!({"name":field.name,"type":format!("{:?}",field.signature),"optional":field.optional})).collect::<Vec<_>>()});
    }
    let source=include_str!("../src/stdlib/testing/api.rs");
    for (line_number,line) in source.lines().enumerate() {
        let Some(export)=line.trim_start().strip_prefix("export ") else {continue};
        let name=export.split_whitespace().nth(1).unwrap().split(['<','(',':','=','{']).next().unwrap();
        let key=format!("std:testing::{name}");
        let row=rows.entry(key.clone()).or_insert_with(||json!({"identity":key,"group":"std:testing","name":name,"review":"pending"}));
        row["bundledSource"]=json!({"file":"crates/tsrust-compiler/src/stdlib/testing/api.rs","line":line_number+1,"declaration":line.trim()});
    }
    println!("{}",serde_json::to_string_pretty(&json!({"schemaVersion":1,"status":"discovery only; per-symbol semantic review pending","rows":rows.into_values().collect::<Vec<_>>()})).unwrap());
}
