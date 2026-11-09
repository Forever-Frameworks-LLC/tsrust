mod support;
use tsrust_compiler::project::{FileSourceProvider, ProjectAnalysis, ProjectDiagnostic, check_graph, load_project};

fn checked(directory: &support::TempDirectory, source: &str) -> Result<ProjectAnalysis, ProjectDiagnostic> {
    let path = directory.source(&support::format_round_trip(source));
    check_graph(load_project(&FileSourceProvider, path.to_str().unwrap())?)
}

fn native(directory: &support::TempDirectory, source: &str, command: &str, expected: Option<&[u8]>) {
    let path = directory.source(&support::format_round_trip(source));
    for release in [false, true] {
        let mut invocation = support::cli();
        invocation.arg(command).arg(&path);
        if release { invocation.arg("--release"); }
        let output = support::run(&mut invocation, 180);
        support::success(&output);
        if let Some(expected) = expected { assert_eq!(output.stdout, expected, "release={release}"); }
    }
}

#[test]
fn url_owners_properties_queries_and_components_execute() {
    let directory=support::TempDirectory::new();
    native(&directory, include_str!("fixtures/url/url.test.tsr"), "test", None);
}

#[test]
fn url_example_and_all_catalog_examples_execute_together() {
    let directory=support::TempDirectory::new();
    native(&directory, include_str!("../../../examples/url/urls.tsr"), "run", Some(b"https://example.com/search?tag=one&tag=two\n7\ntag=one&tag=two&page=1\n?tag=one&tag=two&page=1\n/new\n%C3%A9%20%2B\n\xc3\xa9 +\n"));
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut imports=std::collections::BTreeSet::new();let mut functions=String::new();let mut calls=String::new();let mut expected=String::new();let mut count=0;
    for family in ["std:url","Url","SearchParams"] {
        for entry in catalog[family].as_array().unwrap() {for example in entry["examples"].as_array().unwrap() {
            let (prefix,body)=example["source"].as_str().unwrap().split_once("function main()").unwrap();
            let start=prefix.find('{').unwrap();let end=prefix.find('}').unwrap();
            for name in prefix[start+1..end].split(','){imports.insert(name.trim().to_owned());}
            functions.push_str(&format!("function urlExample{count}(){body}\n"));calls.push_str(&format!("urlExample{count}();"));expected.push_str(example["output"].as_str().unwrap());count+=1;
        }}
    }
    assert_eq!(count,34);
    let source=format!("import {{{}}} from 'std:url';{functions}function main(){{{calls}}}",imports.into_iter().collect::<Vec<_>>().join(","));
    native(&directory, &source, "run", Some(expected.as_bytes()));
}

#[test]
fn computed_properties_preserve_whole_owner_and_selected_container_loans() {
    let directory=support::TempDirectory::new();
    let invalid=[
        ("function main(){const value=unwrap(parseUrl('https://example.com/a'));const text=value.pathname;value.hostname='other';console.log(text);}", "native-view-conflict", "value.hostname", Some("value.pathname")),
        ("function main(){const values=[unwrap(parseUrl('https://example.com/a'))];values[0].pathname=invalidate(values);}function invalidate(values:Mutable<Url[]>):string{values.pop();return '/bad';}", "conflicting-access", "invalidate(values)", Some("values[0]")),
        ("function edit(value:ReadOnly<Url>):void{value.pathname='/bad';}function main(){}", "parameter-access", "value: ReadOnly<Url>", None),
        ("function main(){const value=unwrap(parseUrl('https://example.com/'));value.origin='other';}", "readonly", "value.origin", None),
        ("function main(){const value=unwrap(parseUrl('https://example.com/'));value.pathname+='more';}", "url-property", "value.pathname", None),
        ("function main(){const value:Url={};}", "native-opaque", "{}", None),
        ("function main(){const params=parseQuery('x=one');const text=params.get('x');params.set('x','two');console.log(text);}", "native-view-conflict", "params.set", Some("params.get")),
    ];
    for (body,code,offending,creation) in invalid {
        let source=support::format_round_trip(&format!("import {{Url,parseUrl,parseQuery}} from 'std:url';{body}"));
        let error=checked(&directory,&source).err().unwrap_or_else(||panic!("accepted {source}"));
        assert_eq!(error.code,code,"{source}: {}",error.message);
        let expected_source=std::fs::canonicalize(directory.0.join("input.tsr")).unwrap();
        assert_eq!(error.location.source.0,expected_source.to_str().unwrap());
        let start=source.rfind(offending).unwrap();
        assert!(error.location.span.start<start+offending.len() && start<error.location.span.end,"{source}: {error:?}");
        if let Some(creation)=creation {
            let start=source.find(creation).unwrap();
            assert!(error.related.iter().any(|(location,_)|location.source==error.location.source && location.span.start<start+creation.len() && start<location.span.end),"{source}: {error:?}");
        }
    }
    for body in [
        "function main(){const value=unwrap(parseUrl('https://example.com/a'));const text=String(value.pathname);value.hostname='other';console.log(text);}",
        "function read(value:ReadOnly<Url>):string{return String(value.pathname);}function main(){const value=unwrap(parseUrl('https://example.com/a'));console.log(read(value));}",
        "function main(){const params=parseQuery('x=one');{const text=params.get('x');console.log(text);}params.set('x','two');}",
    ] {checked(&directory,&format!("import {{Url,parseUrl,parseQuery}} from 'std:url';{body}")).unwrap();}
}

#[test]
fn getter_functions_emit_without_throwing_result_wrappers() {
    let source="import {Url,parseUrl} from 'std:url';function length(value:ReadOnly<Url>):int{const text=value.hostname;return text===undefined?0:text.length;}function main(){const value=unwrap(parseUrl('https://example.com/a'));console.log(length(value));}";
    let directory=support::TempDirectory::new();
    let emitted=checked(&directory,source).unwrap().emit_rust();
    let parsed=syn::parse_file(&emitted).unwrap();
    let function=parsed.items.iter().find_map(|item|if let syn::Item::Fn(function)=item{(function.sig.ident=="function0").then_some(function)}else{None}).expect("first source function");
    assert!(matches!(&function.sig.output,syn::ReturnType::Type(_,ty) if matches!(ty.as_ref(),syn::Type::Path(path) if path.path.is_ident("i64"))),"{}",emitted);
}

#[test]
fn optional_url_setters_keep_absent_and_stored_union_types_at_o0_and_o3() {
    let directory=support::TempDirectory::new();
    let source=r#"
import {parseUrl} from 'std:url';
import {strictEqual} from 'std:testing';
function maybeText(present:boolean,text:string):string|undefined{if(present){return text;}return undefined;}
function maybePort(present:boolean,port:int):int|undefined{if(present){return port;}return undefined;}
function select(count:Mutable<int[]>):int{count[0]+=1;return 0;}
function nextPort(count:Mutable<int[]>):int{strictEqual(count[0],1);count[0]+=10;return 4321;}
function main(){
  const address=unwrap(parseUrl('https://user:secret@example.com:8080/start?q=1#top'));
  address.password=undefined;address.port=undefined;address.search=undefined;address.hash=undefined;
  strictEqual(address.password===undefined,true);strictEqual(address.port===undefined,true);
  strictEqual(address.search===undefined,true);strictEqual(address.hash===undefined,true);
  const password=maybeText(true,'restored');address.password=password;strictEqual(String(address.password),'restored');
  const port=maybePort(true,0);address.port=port;strictEqual(address.port===0,true);
  const search=maybeText(true,'?x=two');address.search=search;strictEqual(String(address.search),'?x=two');
  const hash=maybeText(true,'#next');address.hash=hash;strictEqual(String(address.hash),'#next');
  const noPassword=maybeText(false,'unused');address.password=noPassword;
  const noPort=maybePort(false,1);address.port=noPort;
  const noSearch=maybeText(false,'unused');address.search=noSearch;
  const noHash=maybeText(false,'unused');address.hash=noHash;
  strictEqual(address.password===undefined,true);strictEqual(address.port===undefined,true);
  strictEqual(address.search===undefined,true);strictEqual(address.hash===undefined,true);
  const assigned=(address.port=1234);strictEqual(assigned===1234,true);strictEqual(address.port===1234,true);
  const cleared=(address.port=undefined);strictEqual(cleared===undefined,true);strictEqual(address.port===undefined,true);
  const normalized=(address.port=443);strictEqual(normalized===443,true);strictEqual(address.port===undefined,true);
  const addresses=[unwrap(parseUrl('https://example.com/'))];const count=[0];
  const selected=(addresses[select(count)].port=nextPort(count));
  strictEqual(selected===4321,true);strictEqual(addresses[0].port===4321,true);strictEqual(count[0],11);
  const host=unwrap(parseUrl('custom://example.com/path'));host.hostname=undefined;
  strictEqual(host.hostname===undefined,true);
  const hostname=maybeText(true,'other.example');host.hostname=hostname;strictEqual(String(host.hostname),'other.example');
  const noHostname=maybeText(false,'unused');host.hostname=noHostname;strictEqual(host.hostname===undefined,true);
}
"#;
    native(&directory,source,"run",Some(b""));
}

#[test]
fn public_file_urls_round_trip_absolute_unicode_paths_at_o0_and_o3() {
    let directory=support::TempDirectory::new();
    // Path conversion is lexical: the child is intentionally never created.
    let host=directory.0.join("é 🦀 #%.txt").to_str().unwrap().to_owned();
    let mut cases=vec![(host,None)];
    #[cfg(unix)] cases.push(("/tmp/é name#?".to_owned(),Some("file:///tmp/%C3%A9%20name%23%3F")));
    #[cfg(windows)] {
        cases.push((r"C:\Temp\é name#%.txt".to_owned(),Some("file:///C:/Temp/%C3%A9%20name%23%25.txt")));
        cases.push((r"\\server\share\é name#%.txt".to_owned(),Some("file://server/share/%C3%A9%20name%23%25.txt")));
    }
    let mut body=String::new();
    for (index,(path,expected)) in cases.iter().enumerate() {
        let path=serde_json::to_string(path).unwrap();
        body.push_str(&format!("const url{index}=unwrap(pathToFileUrl({path}));strictEqual(unwrap(fileUrlToPath(url{index})),{path});"));
        if let Some(expected)=expected {body.push_str(&format!("strictEqual(String(formatUrl(url{index})),{});",serde_json::to_string(expected).unwrap()));}
    }
    let source=format!("import {{pathToFileUrl,fileUrlToPath,formatUrl}} from 'std:url';import {{strictEqual}} from 'std:testing';function main(){{{body}}}");
    native(&directory,&source,"run",Some(b""));
}
