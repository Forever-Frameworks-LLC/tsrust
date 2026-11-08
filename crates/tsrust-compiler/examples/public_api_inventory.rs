//! Reproducible audit input; discovery is not semantic acceptance.
use serde_json::{Value, json};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap, BTreeSet};
use tsrust_compiler::{native_bindings::BINDINGS, native_types};

fn main() {
    let catalog: BTreeMap<String, Vec<Value>> = serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let retained_catalog=catalog.clone();
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
    let types=include_str!("../src/native/signatures.rs").split("pub enum NativeTypeId {").nth(1).unwrap().split('}').next().unwrap();
    let mut names=rows.values().filter_map(|row|row["name"].as_str()).map(str::to_owned).collect::<BTreeSet<_>>();
    names.extend(rows.values().filter_map(|row|row["group"].as_str()).map(str::to_owned));
    for name in types.split(',').map(str::trim).filter(|name|!name.is_empty()) {
        assert!(name.chars().all(|c|c.is_ascii_alphanumeric()),"review native identity syntax");
        names.insert(name.to_owned());
        let mut declarations=BTreeMap::new();
        for module in &modules {for candidate in &names {
            if let Some(declaration)=native_types::lookup(module,candidate) && format!("{:?}",declaration.id)==name {
                declarations.insert((declaration.module,declaration.name),declaration);
            }
        }}
        if name=="TestRegistry" {
            assert!(declarations.is_empty(),"review newly importable test registry");
            rows.insert("std:testing-runtime::TestRegistry".into(),json!({"identity":"std:testing-runtime::TestRegistry","group":"std:testing-runtime","name":name,"private":true,"review":"internal non-importable registry; declaration metadata is not exposed by lookup"}));
            continue;
        }
        assert_eq!(declarations.len(),1,"missing or ambiguous declaration: {name}");
        let declaration=*declarations.values().next().unwrap();
        let key=format!("{}::{}",declaration.module,declaration.name);
        let row=rows.entry(key.clone()).or_insert_with(||json!({"identity":key,"group":declaration.module,"name":declaration.name,"review":"pending"}));
        row["declaration"]=json!({"representation":format!("{:?}",declaration.representation),"independentClone":declaration.independent_clone,"threadTransfer":declaration.thread_transfer,"typeParameters":declaration.type_parameters,"fields":declaration.fields.iter().map(|field|json!({"name":field.name,"type":format!("{:?}",field.signature),"optional":field.optional})).collect::<Vec<_>>()});
    }
    for (module,file,source) in [
        ("std:testing","crates/tsrust-compiler/src/stdlib/testing/api.rs",include_str!("../src/stdlib/testing/api.rs")),
        ("std:log","crates/tsrust-compiler/src/stdlib/logging/source.tsr",include_str!("../src/stdlib/logging/source.tsr")),
        ("std:iterator","crates/tsrust-compiler/src/stdlib/iterator/source.tsr",include_str!("../src/stdlib/iterator/source.tsr")),
    ] {
        for (line_number,line) in source.lines().enumerate() {
            let Some(export)=line.trim_start().strip_prefix("export ") else {continue};
            let mut words=export.split_whitespace();
            let kind=words.next().unwrap();let kind=if kind=="async" {words.next().unwrap()}else{kind};
            assert!(matches!(kind,"function"|"type"|"interface"|"const"|"class"),"Review source export: {line}");
            let name=words.next().unwrap().split(['<','(',':','=','{']).next().unwrap();
            let key=format!("{module}::{name}");
            let row=rows.entry(key.clone()).or_insert_with(||json!({"identity":key,"group":module,"name":name,"review":"pending"}));
            row["bundledSource"]=json!({"file":file,"line":line_number+1,"declaration":line.trim()});
        }
    }
    let routes:Value=serde_json::from_str(include_str!("../../../docs/plans/documentation-reference-coverage.json")).unwrap();
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut family_selectors=BTreeMap::new();
    for (bundle,targets) in [
        ("B01",vec!["native_filesystem_utilities","native_full_files","native_file_cleanup"]),
        ("B02",vec!["std_bytes","byte_operations","byte_codecs"]),
        ("B03",vec!["path_env_os","environment"]),
        ("B04",vec!["url_values","url_features","url_export"]),
        ("B05",vec!["async_io","async_io_runtime"]),
        ("B06",vec!["task_ergonomics","task_runtime"]),
        ("B07",vec!["process","process_runtime"]),
        ("B08",vec!["network_addresses","network_transport"]),
        ("B09",vec!["compression","compression_runtime","compression_streams_runtime"]),
        ("B10",vec!["http_client_uploads","http_client_forms","http_middleware"]),
        ("B11",vec!["realtime"]),
        ("B12",vec!["regex_text"]),
        ("B13",vec!["explicit_locale"]),
        ("B14",vec!["collections","stdlib_completion","numeric_boundaries","temporal_values"]),
        ("B15",vec!["owned_iterators"]),
        ("B16",vec!["crypto_primitives"]),
        ("B17",vec!["structured_logging"]),
        ("B18",vec!["testing_everyday","testing_async"]),
        ("B19",vec!["rust_bridge","rust_adapter_generation"]),
    ] {
        let mut selectors=Vec::new();
        for target in targets {
            let file=format!("crates/tsrust-compiler/tests/{target}.rs");
            let source=std::fs::read_to_string(root.join(&file)).expect("Existing witness target");
            for item in syn::parse_file(&source).expect("Rust witness source").items {
                if let syn::Item::Fn(function)=item && function.attrs.iter().any(|attribute|attribute.path().is_ident("test")) {
                    selectors.push(json!({"file":file,"package":"tsrust-compiler","target":target,"case":function.sig.ident.to_string()}));
                }
            }
        }
        family_selectors.insert(bundle,json!({"status":"existing family selectors; per-export coverage and execution require review","selectors":selectors}));
    }
    let mut retained_examples=BTreeMap::new();
    for (group,expected) in [("std:fs","1a5d6a26902ca4f353f33253b07ca023cff83c49926e2e02ad854493823ed4df"),("std:url","0ce43b51e149471f9dfcf1bf3babcdda2bc5f48438798296f1d00294a1e6a6ac"),("Url","4a1736088bcdaad95887266eecdd985ff100d66e8d509ce2d6605fd536b6075e"),("SearchParams","96b3ab23ff9deface3e2359d862a8a663ab5ddeedc2ef915d63ad8eb6643fa1a")] {
        let examples=retained_catalog[group].iter().map(|entry|(entry["name"].as_str().unwrap(),entry.get("examples").cloned().unwrap_or(json!([])))).collect::<BTreeMap<_,_>>();
        let actual=Sha256::digest(serde_json::to_vec(&examples).unwrap()).iter().map(|byte|format!("{byte:02x}")).collect::<String>();
        retained_examples.insert(group,actual==expected);
    }
    let mut receiver_modules=BTreeMap::new();
    for row in rows.values() {if row.get("declaration").is_some(){receiver_modules.insert(row["name"].as_str().unwrap().to_owned(),row["group"].as_str().unwrap().to_owned());}}
    for row in rows.values_mut() {
        let group=row["group"].as_str().unwrap().to_owned();let name=row["name"].as_str().unwrap().to_owned();
        row["compilerExport"]=match tsrust_compiler::native_exports::lookup(&group,&name) {
            Ok(export)=>json!({"lookup":"recognized","metadata":format!("{export:?}"),"scope":"Native lookup only; source modules and receiver dispatch have separate identities"}),
            Err(_)=>json!({"lookup":"not a direct native export","status":if row.get("bundledSource").is_some(){"bundled source declaration"}else{"missing per-export compiler-dispatch review"}}),
        };
        if let Some(binding)=row.get_mut("binding") {binding["private"]=json!(matches!(group.as_str(),"std:testing-runtime"|"std:log-runtime"|"std:iterator-runtime") || tsrust_compiler::native_exports::lookup(&group,&name).is_err());}
        row["reference"]=routes["receivers"].get(&group).map(|route|json!({"route":route["route"],"source":"docs/plans/documentation-reference-coverage.json"})).unwrap_or(json!({"status":"missing route review"}));
        let examples=row.get("catalog").and_then(|catalog|catalog.get("examples")).and_then(Value::as_array).map(|examples|(0..examples.len()).map(|index|json!({"file":"crates/tsrust-lsp/src/builtins.json","receiver":group,"name":name,"example":index,"execution":"unreviewed; catalog presence is not a pass"})).collect::<Vec<_>>()).unwrap_or_default();
        let module=receiver_modules.get(&group).map(String::as_str).unwrap_or(&group);
        let bundle=match module {
            "std:fs"=>Some("B01"),"std:bytes"=>Some("B02"),"std:path"|"std:env"|"std:os"|"std:signals"=>Some("B03"),
            "std:url"=>Some("B04"),"std:io"=>Some("B05"),"std:futures"|"std:threads"=>Some("B06"),"std:process"=>Some("B07"),
            "std:net"|"std:dns"=>Some("B08"),"std:compression"=>Some("B09"),"std:http"=>Some("B10/B11"),"std:regex"=>Some("B12"),
            "std:locale"=>Some("B13"),"std:collections"|"std:numeric"|"std:date"=>Some("B14"),"std:iterator"=>Some("B15"),
            "std:crypto"=>Some("B16"),"std:log"|"std:log-runtime"=>Some("B17"),"std:testing"|"std:testing-runtime"=>Some("B18"),_=>None,
        };
        row["bundle"]=json!(bundle);
        row["familyWitnesses"]=json!(match bundle {Some("B10/B11")=>vec!["B10","B11"],Some(bundle)=>vec![bundle],None=>vec![]});
        row["witnesses"]=json!({"catalogExamples":examples,"positive":"missing per-export execution reconciliation","negative":"missing per-export review","alias":"missing per-export review","platform":"unverified per-export native-host coverage"});
        if retained_examples.get(group.as_str())==Some(&true) && row["witnesses"]["catalogExamples"].as_array().is_some_and(|examples|!examples.is_empty()) {
            let (package,target,case,file)=if group=="std:fs" {
                ("tsrust-lsp","native_file_catalog","every_file_catalog_example_checks_and_executes","crates/tsrust-lsp/tests/native_file_catalog.rs")
            } else {
                ("tsrust-compiler","url_values","url_example_and_all_catalog_examples_execute_together","crates/tsrust-compiler/tests/url_values.rs")
            };
            row["witnesses"]["positive"]=json!({"file":file,"package":package,"target":target,"case":case,"status":"retained pass","source":"965de83f","coverage":"exact catalog-example iteration; current per-group example digest matches reviewed input"});
        }
    }
    println!("{}",serde_json::to_string_pretty(&json!({"schemaVersion":1,"status":"discovery only; per-symbol semantic review pending","rows":rows.into_values().collect::<Vec<_>>(),"familyWitnessSelectors":family_selectors})).unwrap());
}
