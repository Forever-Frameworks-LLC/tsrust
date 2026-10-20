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
 let button=data["api"].as_array().unwrap().iter().find(|entry|entry["name"]=="ButtonProps"&&entry["source"]=="src/components/ui/button.tsrx").unwrap();
 let inherited=button["fields"].as_array().unwrap().iter().find(|field|field["name"]=="id").unwrap();assert_eq!(inherited["source"],"std:ui");assert!(inherited["signature"].as_str().unwrap().contains("string"));
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

#[test]
fn execution_prepares_content_and_cache_tracks_compiler_sources_and_publications(){
 let temp=support::TempDirectory::new();let root=&temp.0;fs::create_dir_all(root.join("content")).unwrap();fs::create_dir_all(root.join("src")).unwrap();
 fs::write(root.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"content-app","build":{"entry":"src/main.tsr","output":"target/app"},"documentation":{"content":"content","examples":[],"apiSources":[],"inventory":null}}"#).unwrap();
 let header="<!-- tsrust-docs\n{\"id\":\"home\",\"group\":\"guides\",\"order\":0,\"status\":\"ready\",\"targets\":[\"web\",\"native\"]}\n-->\n";
 fs::write(root.join("content/index.md"),format!("{header}# Home\n")).unwrap();
 let entry=root.join("src/main.tsr");fs::write(&entry,support::format_round_trip("import {loadCatalog} from '../target/docs-content/catalog';function main(){const catalog=loadCatalog();console.log(catalog.pages[0].document.title);}")).unwrap();
 let check=support::run(support::cli().arg("check").arg(&entry),30);assert!(!check.status.success());assert!(!root.join("target").exists(),"check must stay read-only");
 let run=support::run(support::cli().arg("run").arg(&entry),180);support::success(&run);assert_eq!(run.stdout,b"Home\n");
 let module=root.join("target/docs-content/catalog.tsr");let accepted=fs::read(&module).unwrap();let before=fs::metadata(&module).unwrap().modified().unwrap();
 let content=||support::run(support::cli().args(["docs","content","--cwd"]).arg(root),60);
 support::success(&content());assert_eq!(before,fs::metadata(&module).unwrap().modified().unwrap());
 let manifest:serde_json::Value=serde_json::from_slice(&fs::read(root.join("target/docs-content/inputs.json")).unwrap()).unwrap();assert_eq!(manifest["compilerSha256"].as_str().unwrap().len(),64);
 let mut wrong_compiler=manifest.clone();wrong_compiler["compilerSha256"]=serde_json::json!("wrong compiler");fs::write(root.join("target/docs-content/inputs.json"),serde_json::to_vec(&wrong_compiler).unwrap()).unwrap();support::success(&content());
 let refreshed:serde_json::Value=serde_json::from_slice(&fs::read(root.join("target/docs-content/inputs.json")).unwrap()).unwrap();assert_eq!(manifest["compilerSha256"],refreshed["compilerSha256"]);
 fs::write(&module,"tampered").unwrap();support::success(&content());assert_eq!(fs::read(&module).unwrap(),accepted);
 let other=header.replace("\"home\"","\"other\"");fs::write(root.join("content/other.md"),format!("{other}# Other\n")).unwrap();support::success(&content());
 let catalog:serde_json::Value=serde_json::from_slice(&fs::read(root.join("target/docs-content/catalog.json")).unwrap()).unwrap();assert_eq!(catalog["pages"].as_array().unwrap().len(),2);
 fs::remove_file(root.join("content/other.md")).unwrap();fs::write(root.join("content/index.md"),format!("{header}# Updated\n")).unwrap();
 let binary=root.join("target/app");let build=support::run(support::cli().arg("build").arg(&entry).arg("-o").arg(&binary),180);support::success(&build);
 let output=support::run(std::process::Command::new(&binary).current_dir(root),30);support::success(&output);assert_eq!(output.stdout,b"Updated\n");
}

#[test]
fn page_sized_readable_loaders_and_generated_output_cache_remain_checked(){
 let temp=support::TempDirectory::new();let root=&temp.0;fs::create_dir_all(root.join("content")).unwrap();fs::create_dir_all(root.join("src")).unwrap();
 fs::write(root.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"lazy-content","build":{"entry":"src/main.tsr","output":"target/app"},"documentation":{"content":"content","examples":[],"apiSources":[],"inventory":null}}"#).unwrap();
 for (name,title) in [("index","Home"),("second","Second")]{let header=format!("<!-- tsrust-docs\n{{\"id\":\"{name}\",\"group\":\"guides\",\"order\":0,\"status\":\"ready\",\"targets\":[\"web\",\"native\"]}}\n-->\n# {title}\n\nExact café text and `code`.\n");fs::write(root.join(format!("content/{name}.md")),header).unwrap();}
 let entry=root.join("src/main.tsr");let source="import {loadNavigation,loadPage,loadCatalog,loadSearch,pageId} from '../target/docs-content/catalog';import {strictEqual} from 'std:testing';function main(){const index=loadNavigation();strictEqual(index.length,2);const selected=loadPage('/second');strictEqual(selected.pages.length,1);strictEqual(selected.pages[0].document.title,'Second');strictEqual(pageId('/second'),'second');const search=loadSearch();strictEqual(search.length,2);strictEqual(search[0].text.includes('café'),true);const full=loadCatalog();strictEqual(full.pages.length,2);console.log('Lazy data passed');}";
 fs::write(&entry,support::format_round_trip(source)).unwrap();let run=support::run(support::cli().arg("run").arg(&entry),180);support::success(&run);assert_eq!(run.stdout,b"Lazy data passed\n");
 let directory=root.join("target/docs-content");let module=fs::read_to_string(directory.join("catalog.tsr")).unwrap();assert!(module.len()<8000);assert!(!module.contains("JSON.parse"));let page=directory.join("pages/second.tsr");let accepted=fs::read(&page).unwrap();assert!(String::from_utf8_lossy(&accepted).contains("\"nodes\": ["));
 let before=fs::metadata(&page).unwrap().modified().unwrap();let build=||support::run(support::cli().args(["docs","content","--cwd"]).arg(root),60);support::success(&build());assert_eq!(before,fs::metadata(&page).unwrap().modified().unwrap());fs::write(&page,"tampered").unwrap();support::success(&build());assert_eq!(fs::read(&page).unwrap(),accepted);
 fs::remove_file(root.join("content/second.md")).unwrap();support::success(&build());assert!(!page.exists(),"Unchanged obsolete generated data must be removed");
}
