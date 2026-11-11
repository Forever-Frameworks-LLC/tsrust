use tsrust_compiler::project::{SourceId, SourceProvider, check_graph, load_project};

struct Provider(String);
impl SourceProvider for Provider {
    fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> { Ok(SourceId(path.into())) }
    fn read(&self, id: &SourceId) -> Result<String, String> {
        match id.0.as_str() {
            "main.tsr" => Ok(self.0.clone()),
            "shared.tsr" => Ok("import {StringView} from 'std:url';export type Text=StringView;".into()),
            "alias.tsr" => Ok("import {SeekOrigin as NativeOrigin} from 'std:fs'; export type Origin = NativeOrigin;".into()),
            _ => Err(format!("Missing {}", id.0)),
        }
    }
}
fn checked(source: &str) -> Result<tsrust_compiler::project::ProjectAnalysis, tsrust_compiler::project::ProjectDiagnostic> {
    check_graph(load_project(&Provider(tsrust_compiler::format(source).unwrap()), "main.tsr")?)
}
#[test]
fn native_literal_aliases_expand_without_selecting_runtime_dependencies() {
    use tsrust_compiler::native_exports::{Export, lookup};
    let Export::Alias(alias) = lookup("std:fs", "SeekOrigin").unwrap() else { panic!("transparent alias") };
    assert_eq!(alias.signature.render(|_| None).unwrap(), "(\"start\" | \"current\" | \"end\")");
    assert!(tsrust_compiler::native_bindings::selected_features([("std:fs", "SeekOrigin")]).unwrap().is_empty());
    for import in ["import {SeekOrigin as Origin} from 'std:fs';", "import {Origin} from 'alias.tsr';"] {
        let source = format!("{import} function take(value:Origin):Origin{{return value;}} function observe(value:ReadOnly<Origin>):int{{if(value.startsWith('c')&&value.includes('r')){{return value.length;}}return value.toUpperCase().length;}}function main(){{const origin:Origin='current';console.log(observe(origin));console.log(take(origin));}}");
        let analysis = checked(&source).unwrap();
        let rust = analysis.emit_rust();
        assert!(!rust.contains("mod tsrust_native_files"), "{rust}");
        assert!(!tsrust_compiler::native_runtime::separate(&rust).dependencies);
        for body in ["const origin:Origin='invalid';", "const raw:string='start';const origin:Origin=raw;", "console.log(Origin);"] {
            assert!(checked(&format!("{import} function main(){{{body}}}")).is_err(), "{body}");
        }
    }
}

#[test]
fn shared_native_type_imports_keep_the_written_module_for_every_binding_order() {
    for names in ["StringView as Text,Url,UrlError,SearchParams,parseUrl,formatUrl", "Url,UrlError,SearchParams,parseUrl,formatUrl,StringView as Text"] {
        let source=format!("import {{{names}}} from 'std:url';function length(value:ReadOnly<Text>):int{{return value.length;}}function main(){{const address=unwrap(parseUrl('https://example.com/'));console.log(length(formatUrl(address)));}}");
        let checked=checked(&source).unwrap();
        let rust=checked.emit_rust();
        assert!(rust.contains("mod tsrust_native_url"));
        assert!(!rust.contains("mod tsrust_native_json"),"a shared view type must not select the JSON runtime");
    }
    checked("import {Text} from 'shared.tsr';import {parseUrl,formatUrl} from 'std:url';function length(value:ReadOnly<Text>):int{return value.length;}function main(){const address=unwrap(parseUrl('https://example.com/'));console.log(length(formatUrl(address)));}").unwrap();
}
