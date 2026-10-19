mod support;
use std::{fs,path::{Path,PathBuf}};
fn root()->PathBuf{Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs").canonicalize().unwrap()}
#[test]
fn authored_content_api_inventory_and_exact_examples_build_into_checked_native_data(){
 let root=root();let output=support::run(support::cli().args(["docs","content","--cwd"]).arg(&root),120);support::success(&output);
 let data:serde_json::Value=serde_json::from_slice(&fs::read(root.join("target/docs-content/catalog.json")).unwrap()).unwrap();
 let pages=data["pages"].as_array().unwrap();assert_eq!(pages.len(),103);
 for (group,count) in [("components",64),("recipes",3),("blocks",30)]{assert_eq!(data["library"][group].as_array().unwrap().len(),count);}
 let trim=pages.iter().find(|page|page["metadata"]["id"]=="strings").unwrap();assert!(trim["markdown"].as_str().unwrap().contains("No accepted isolated trim measurement"));
 let builtin=data["api"].as_array().unwrap().iter().find(|entry|entry["name"]=="string.trim").unwrap();assert_eq!(builtin["signature"],"trim(): string");assert!(builtin["documentation"].as_str().unwrap().contains("0085"));
 let props=data["api"].as_array().unwrap().iter().find(|entry|entry["name"]=="ViewProps").unwrap();let fields=props["fields"].as_array().unwrap();assert!(fields.iter().any(|field|field["name"]=="tabIndex"&&field["documentation"].as_str().unwrap().contains("sequential")));
 for page in pages{let markdown=page["markdown"].as_str().unwrap();for node in page["document"]["nodes"].as_array().unwrap(){let start=node["source"]["start"].as_u64().unwrap() as usize;let end=node["source"]["end"].as_u64().unwrap() as usize;assert!(start<=end&&end<=markdown.len()&&markdown.is_char_boundary(start)&&markdown.is_char_boundary(end));}}
 let example=&data["examples"][0];let exact=fs::read_to_string(root.join(example["source"].as_str().unwrap())).unwrap();assert_eq!(example["code"].as_str().unwrap(),exact);
 let module=root.join("target/docs-content/catalog.tsr");let generated=fs::read_to_string(&module).unwrap();assert_eq!(generated,support::format_round_trip(&generated));
 let inspection=root.join("target/docs-content/inspect.tsr");let source=format!("import {{loadCatalog,renderExample}} from './catalog';import {{strictEqual}} from 'std:testing';import {{document}} from 'std:ui';function main(){{const catalog=loadCatalog();strictEqual(catalog.pages.length,103);strictEqual(catalog.examples.length,2);strictEqual(catalog.examples[0].code,{});const preview=renderExample('counter');if(preview!==null){{strictEqual(document(preview,'Example').includes('Count: 0'),true);}}else{{throw 'Missing actual preview';}}strictEqual(renderExample('strings-trim')===null,true);console.log('Documentation content passed');}}",tsrust_compiler::json_string(&exact));
 fs::write(&inspection,support::format_round_trip(&source)).unwrap();let result=support::run(support::cli().arg("run").arg(&inspection),180);fs::remove_file(&inspection).unwrap();support::success(&result);assert_eq!(result.stdout,b"Documentation content passed\n");
 let result=support::run(support::cli().arg("run").arg(root.join("src/examples/strings-trim.tsr")),90);support::success(&result);assert_eq!(result.stdout,b"String trim documentation passed\n");
 let before=fs::metadata(&module).unwrap().modified().unwrap();let repeat=support::run(support::cli().args(["docs","content","--cwd"]).arg(&root),120);support::success(&repeat);assert_eq!(before,fs::metadata(&module).unwrap().modified().unwrap(),"Unchanged inputs must not cause dev rebuilds");
}
#[test]
fn invalid_content_never_replaces_accepted_output_and_checks_remain_read_only(){
 let temp=support::TempDirectory::new();let root=&temp.0;fs::create_dir_all(root.join("content")).unwrap();
 fs::write(root.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"content-test","build":{"entry":"src/main.tsr","output":"target/test"},"documentation":{"content":"content","examples":[],"apiSources":[],"inventory":null}}"#).unwrap();
 let header="<!-- tsrust-docs\n{\"id\":\"home\",\"group\":\"guides\",\"order\":0,\"status\":\"ready\",\"targets\":[\"web\",\"native\"]}\n-->\n# Home\n\n";
 fs::write(root.join("content/index.md"),format!("{header}Safe content.\n")).unwrap();let run=||support::run(support::cli().args(["docs","content","--cwd"]).arg(root),60);support::success(&run());let published=fs::read(root.join("target/docs-content/catalog.tsr")).unwrap();
 for bad in ["[unsafe](javascript:alert)","[missing](absent.md)","[missing](#absent)","[missing][undefined]","![no file](missing.png)"]{fs::write(root.join("content/index.md"),format!("{header}{bad}\n")).unwrap();let output=run();assert!(!output.status.success(),"{bad}");assert_eq!(published,fs::read(root.join("target/docs-content/catalog.tsr")).unwrap());}
 fs::write(root.join("content/index.md"),format!("{header}Safe.\n")).unwrap();fs::write(root.join("content/second.md"),format!("{header}Duplicate ID.\n")).unwrap();assert!(!run().status.success());fs::remove_file(root.join("content/second.md")).unwrap();
 #[cfg(unix)]{std::os::unix::fs::symlink(root.join("content/index.md"),root.join("content/link.md")).unwrap();assert!(!run().status.success());fs::remove_file(root.join("content/link.md")).unwrap();}
 assert!(!root.join("src").exists(),"Content checks must not create UI project source");
}
