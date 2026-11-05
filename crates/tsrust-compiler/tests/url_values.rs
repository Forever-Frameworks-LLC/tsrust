mod support;

#[test]
fn url_owners_properties_queries_and_components_execute() {
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip(include_str!("fixtures/url/url.test.tsr"));
    support::success(&support::run(support::cli().arg("test").arg(directory.source(&source)),180));
}

#[test]
fn url_example_and_all_catalog_examples_execute_together() {
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip(include_str!("../../../examples/url/urls.tsr"));
    let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);
    assert_eq!(output.stdout,b"https://example.com/search?tag=one&tag=two\n7\ntag=one&tag=two&page=1\n?tag=one&tag=two&page=1\n/new\n%C3%A9%20%2B\n\xc3\xa9 +\n");
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
    let output=support::run(support::cli().arg("run").arg(directory.source(&support::format_round_trip(&source))),180);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());
}

#[test]
fn computed_properties_preserve_whole_owner_and_selected_container_loans() {
    let invalid=[
        "function main(){const value=unwrap(parseUrl('https://example.com/a'));const text=value.pathname;value.hostname='other';console.log(text);}",
        "function main(){const values=[unwrap(parseUrl('https://example.com/a'))];values[0].pathname=invalidate(values);}function invalidate(values:Mutable<Url[]>):string{values.pop();return '/bad';}",
        "function edit(value:ReadOnly<Url>):void{value.pathname='/bad';}function main(){}",
        "function main(){const value=unwrap(parseUrl('https://example.com/'));value.origin='other';}",
        "function main(){const value=unwrap(parseUrl('https://example.com/'));value.pathname+='more';}",
        "function main(){const value:Url={};}",
        "function main(){const params=parseQuery('x=one');const text=params.get('x');params.set('x','two');console.log(text);}",
    ];
    for body in invalid {
        let source=format!("import {{Url,parseUrl,parseQuery}} from 'std:url';{body}");
        assert!(tsrust_compiler::check(&source).is_err(),"accepted {source}");
    }
}

#[test]
fn getter_functions_emit_without_throwing_result_wrappers() {
    let source="import {Url,parseUrl} from 'std:url';function length(value:ReadOnly<Url>):int{const text=value.hostname;return text===undefined?0:text.length;}function main(){const value=unwrap(parseUrl('https://example.com/a'));console.log(length(value));}";
    let emitted=tsrust_compiler::check(source).unwrap().emit_rust();
    let parsed=syn::parse_file(&emitted).unwrap();
    let function=parsed.items.iter().find_map(|item|if let syn::Item::Fn(function)=item{(function.sig.ident=="function0").then_some(function)}else{None}).expect("first source function");
    assert!(matches!(&function.sig.output,syn::ReturnType::Type(_,ty) if matches!(ty.as_ref(),syn::Type::Path(path) if path.path.is_ident("i64"))),"{}",emitted);
}
