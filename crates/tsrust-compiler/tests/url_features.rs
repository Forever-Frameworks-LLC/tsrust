mod support;
use std::collections::BTreeSet;

#[test]
fn url_features_follow_emitted_uses_instead_of_imports() {
    let directory = support::TempDirectory::new();
    for (name,source,used) in [
        ("unused function", "import {parseUrl} from 'std:url';function main(){console.log('plain');}", false),
        ("unused type", "import {Url} from 'std:url';function main(){console.log('plain');}", false),
        ("unused type alias", "import {Url as Address} from 'std:url';type Unused=Address;function main(){console.log('plain');}", false),
        ("shadowed function import", "import {parseUrl} from 'std:url';function main(){const parseUrl=3;console.log(parseUrl);}", false),
        ("property name is not an import use", "import {parseUrl} from 'std:url';type Holder={parseUrl:int;};function main(){const value:Holder={parseUrl:3};console.log(value.parseUrl);}", false),
        ("plain error record", "import {UrlError} from 'std:url';function main(){const value:UrlError={code:'input',operation:'test',message:'bad'};console.log(value.code);}", false),
        ("plain query entry record", "import {QueryEntry} from 'std:url';function main(){const value:QueryEntry={name:'a',value:'b'};console.log(value.name);}", false),
        ("void method", "import {parseQuery,formatQuery} from 'std:url';function main(){const params=parseQuery('b=2&a=1');params.sort();console.log(formatQuery(params));}", true),
        ("parsed owner", "import {parseUrl} from 'std:url';function main(){const value=unwrap(parseUrl('https://example.com/a'));console.log(value.pathname);}", true),
        ("query owner", "import {parseQuery,formatQuery} from 'std:url';function main(){const params=parseQuery('x=one');console.log(formatQuery(params));}", true),
        ("component encoder without owner", "import {encodeURIComponent} from 'std:url';function main(){console.log(encodeURIComponent('a b'));}", true),
        ("component decoder without owner", "import {decodeURIComponent} from 'std:url';function main(){console.log(unwrap(decodeURIComponent('a%20b')));}", true),
        ("source-only owner signature", "import {Url} from 'std:url';export function independent(value:ReadOnly<Url>):Url{return clone(value);}function main(){}", true),
    ] {
        let source=support::format_round_trip(source);
        let path = directory.source(&source);
        let graph = tsrust_compiler::project::load_project(&tsrust_compiler::project::FileSourceProvider, path.to_str().unwrap()).unwrap();
        let checked=tsrust_compiler::project::check_graph(graph).unwrap_or_else(|error|panic!("{name}: {error:?}"));
        let (rust,features)=checked.emit_rust_with_native_features();
        assert_eq!(features,if used{vec!["url"]}else{vec![]},"{name}");
        assert_eq!(rust.contains("mod tsrust_native_url"),used,"{name}");
        for legacy in ["tsrust_native_ui","vello","winit","wgpu"] {assert!(!rust.contains(legacy),"{name}: introduced {legacy}");}
        let units=tsrust_compiler::native_runtime::separate(&rust);
        assert_eq!(units.dependencies,used,"{name}");
        let runtime=units.runtime.as_deref().unwrap_or("");
        if used {
            assert!(runtime.contains("pub mod tsrust_native_url"),"{name}");
            assert!(runtime.contains("url::")&&runtime.contains("percent_encoding::"),"{name}");
            assert!(!units.application.contains("mod tsrust_native_url"),"{name}");
            assert!(units.application.contains("use tsrust_runtime::tsrust_native_url;"),"{name}");
        }else{assert!(!rust.contains("tsrust_native_url")&&!runtime.contains("tsrust_native_url"),"{name}");}
        if name=="source-only owner signature" {
            assert!(rust.contains("fn function0("),"source-only URL function was not emitted");
            assert!(rust.split_whitespace().collect::<String>().contains("=tsrust_native_url::Url;"),"emitted URL signature lost its native alias");
        }
    }
}

#[test]
fn url_dependency_features_are_precise_and_do_not_activate_legacy_ui() {
    for (name,manifest) in [
        ("backend",include_str!("../backend-dependencies/Cargo.toml")),
        ("native",include_str!("../native-dependencies/Cargo.toml")),
    ] {
        let document=manifest.parse::<toml_edit::DocumentMut>().unwrap();
        let features=document["features"]["url"].as_array().expect("URL feature").iter().map(|value|value.as_str().unwrap()).collect::<BTreeSet<_>>();
        assert_eq!(features,BTreeSet::from(["dep:url","dep:percent-encoding"]),"{name}");
        for (dependency,version) in [("url","=2.5.8"),("percent-encoding","=2.3.2")] {
            let selected=document["dependencies"][dependency].as_inline_table().unwrap();
            assert_eq!(selected["version"].as_str(),Some(version),"{name}/{dependency}");
            assert_eq!(selected["optional"].as_bool(),Some(true),"{name}/{dependency}");
        }
        assert!(document["features"]["default"].as_array().unwrap().is_empty(),"{name}");
        if name=="backend" {
            for legacy in ["winit","vello","taffy","parley","accesskit","accesskit_winit","arboard"] {
                assert!(document["dependencies"].get(legacy).is_none(),"backend includes legacy {legacy}");
            }
        }
    }
}
