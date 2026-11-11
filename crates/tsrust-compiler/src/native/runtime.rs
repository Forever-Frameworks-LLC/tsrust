//! Separate compiler-owned runtime modules from program-specific ABI bridges.
//! Emit retains bundled modules; JSON also needs its dependency package.
//! Self-contained package export is tracked in the JSON plan. This is not a Rust parser:
//! only exact bundled module definitions produced by our emitter are extracted.
pub struct Units {
    pub application:String,
    pub runtime:Option<String>,
    pub dependencies:bool,
}

pub fn requires_hostname(source:&str)->bool {source.contains(crate::stdlib::os::HOSTNAME_RUNTIME)}

pub fn requires_byte_codecs(source:&str)->bool {source.contains("fn codec_base64_engine(")}
pub fn requires_byte_search(source:&str)->bool {source.contains(include_str!("../stdlib/bytes/runtime/search.rs"))}
pub fn requires_temporary_files(source:&str)->bool {source.contains("mod tsrust_native_tempfiles {")}

pub(crate) fn bytes_module(views:bool,utf8:bool,codecs:bool,base64:bool,operations:bool,binary:bool,search:bool)->String {
    let mut text=String::from("mod tsrust_native_bytes {\n");
    text.push_str(include_str!("../stdlib/bytes/runtime/mod.rs"));
    if views {text.push_str(include_str!("../stdlib/bytes/runtime/views.rs"));}
    if utf8 {text.push_str(include_str!("../stdlib/bytes/runtime/utf8.rs"));}
    if codecs || base64 {text.push_str(include_str!("../stdlib/bytes/runtime/codecs.rs"));}
    if base64 {text.push_str(include_str!("../stdlib/bytes/runtime/base64.rs"));}
    if operations {text.push_str(include_str!("../stdlib/bytes/runtime/operations.rs"));}
    if binary {text.push_str(include_str!("../stdlib/bytes/runtime/binary.rs"));}
    if search {text.push_str(include_str!("../stdlib/bytes/runtime/search.rs"));}
    text.push_str("}\n");text
}
pub(crate) fn tempfiles_module()->String {
    let source=include_str!("../stdlib/fs/runtime/tempfiles.rs").split("\n#[cfg(test)]").next().unwrap()
        .replace("crate::stdlib::fs::runtime::fs","crate::tsrust_native_files::fs");
    format!("mod tsrust_native_tempfiles {{\n{source}\n}}\n")
}

pub(crate) fn files_module()->String {
    let runtime=|source:&'static str|source.split_once("\n#[cfg(test)]").expect("file backend tests follow runtime").0;
    format!("mod tsrust_native_files {{ mod resource {{\n{}\n}} pub mod fs {{\n{}\n}} pub mod buffered_fs {{\n{}\n}} }}\n",runtime(include_str!("../stdlib/fs/runtime/resource.rs")),runtime(include_str!("../stdlib/fs/runtime/fs.rs")),runtime(include_str!("../stdlib/fs/runtime/buffered_fs.rs")))
}
pub(crate) fn path_module()->String {
    format!("mod tsrust_native_path {{\n{}{}\n}}\n",include_str!("../stdlib/path/runtime.rs"),include_str!("../stdlib/path/relative.rs"))
}
/// Compiler-owned JSON support; public language bindings are implemented separately.
pub fn json_module()->String {json_module_with_writer(false)}
pub fn json_module_with_writer(writer:bool)->String {
    // JSON Lines tests inspect raw descriptors using FFI; they are not runtime code.
    let lines=include_str!("../stdlib/json/lines.rs").split_once("\n#[cfg(test)]").expect("JSON Lines tests follow runtime").0;
    let writer=if writer {format!("pub mod writer {{\n{}\n}}\n",include_str!("../stdlib/json/writer.rs").replace("crate::stdlib::fs::runtime","crate::tsrust_native_files"))}else{String::new()};
    format!("mod tsrust_native_json {{\npub use serde_json::Map;\n{}\npub mod lines {{\n{}\n}}\npub mod typed {{\n{}\n}}\npub mod raw {{\n{}\n}}\npub mod source {{\n{}\n}}\n{writer}}}\n", concat!(include_str!("../stdlib/json/runtime/parser.rs"), "\n", include_str!("../stdlib/json/runtime/operations.rs")),lines,include_str!("../stdlib/json/typed.rs"),include_str!("../stdlib/json/raw.rs"),include_str!("../stdlib/json/source.rs"))
}

pub(crate) fn futures_module(testing:bool)->String {
    let source=crate::stdlib::futures::RUNTIME;
    if testing {source.replace("fn capture_testing_context<F:Future>(future:F)->impl Future<Output=F::Output>{future}","fn capture_testing_context<F:Future>(future:F)->impl Future<Output=F::Output>{super::tsr_console_test_future(future)}")}else{source.to_owned()}
}
pub(crate) fn threads_module(asynchronous:bool,testing:bool)->String {
    let source=if asynchronous{crate::stdlib::threads::ASYNC_RUNTIME}else{crate::stdlib::threads::RUNTIME};
    if testing {source.replace("fn capture_testing_context<F:FnOnce()->R,R>(work:F)->impl FnOnce()->R{work}","fn capture_testing_context<F:FnOnce()->R,R>(work:F)->impl FnOnce()->R{super::tsr_console_test_work(work)}")}else{source.to_owned()}
}
pub(crate) fn console_module(asynchronous_testing:bool)->String {
    let mut source=include_str!("../runtime/console.rs").to_owned();
    if asynchronous_testing {
        source=source.replace("fn tsr_console_test_context()->TsrConsoleTestContext{", "fn tsr_console_test_context()->TsrConsoleTestContext{if let Ok(context)=TSR_CONSOLE_ASYNC_TEST.try_with(Clone::clone){return context;}");
        source.push_str(include_str!("../runtime/console_async.rs"));
    }
    source
}

/// Whether the selected emitted support needs a compiler-owned Cargo dependency graph.
/// This predicate preserves exact bundled module and Rust-binding detection without extraction.
pub fn requires_dependencies(source:&str)->bool {
    if source.contains(crate::stdlib::logging::RUNTIME){return true;}
    let json=[true,false].into_iter().map(json_module_with_writer).find(|module|source.contains(module));
    let tasks=[true,false].into_iter().any(|testing|source.contains(&futures_module(testing)) || [true,false].into_iter().any(|asynchronous|source.contains(&threads_module(asynchronous,testing))));
    source.contains(crate::stdlib::process::RUNTIME) || source.contains(crate::stdlib::compression::RUNTIME) || source.contains(crate::stdlib::net::TRANSPORT_RUNTIME) || source.contains(crate::stdlib::net::DNS_RUNTIME) || source.contains(&crate::stdlib::io::runtime()) || requires_hostname(source) || source.contains(crate::stdlib::locale::RUNTIME) || source.contains(crate::stdlib::regex::RUNTIME) || source.contains(crate::stdlib::url::RUNTIME) || requires_byte_codecs(source) || requires_byte_search(source) || source.contains(&files_module()) || source.contains(&crate::stdlib::ui::runtime(true)) || source.contains(&crate::stdlib::ui::runtime(false))
        || source.contains(crate::stdlib::signals::RUNTIME) || source.contains(crate::stdlib::crypto::RUNTIME) || source.contains(crate::stdlib::crypto::PASSWORD_RUNTIME) || source.contains(crate::stdlib::uuid::RUNTIME) || source.contains(crate::stdlib::date::RUNTIME) || source.contains(crate::stdlib::decimal::RUNTIME) || source.contains(crate::stdlib::sql::RUNTIME) || source.contains(crate::stdlib::sqlite::RUNTIME) || json.as_ref().is_some_and(|module|source.contains(module)) || tasks || source.contains(crate::stdlib::futures::CANCELLATION_RUNTIME) || source.contains("use rayon as tsr_rayon;") || source.contains(crate::project::rust::dependencies::MARKER)
}

/// Keep application record/union types, borrow adapters and generic ownership code
/// in the application crate. Extract only selected self-contained support modules.
pub fn separate(source:&str)->Units {
    let mut application=source.to_owned();let mut runtime=if source.contains(crate::stdlib::http::MIDDLEWARE_MARKER){crate::stdlib::http::MIDDLEWARE_MARKER.to_owned()}else{String::new()};
    if source.contains(crate::stdlib::http::WEBSOCKET_MARKER){runtime.push_str(crate::stdlib::http::WEBSOCKET_MARKER);}
    // Unused Kit wrappers are pruned, so the component name appears only when used.
    // Editors, diffs and Markdown code blocks use the selected Kit grammars.
    if ["CodeEditor","Markdown","Diff"].iter().any(|name|source.contains(&format!("String::Borrowed(\"{name}\")"))){runtime.push_str(crate::stdlib::ui::HIGHLIGHT_MARKER);}
    if source.contains("String::Borrowed(\"SpeechButton\")"){runtime.push_str(crate::stdlib::ui::SPEECH_MARKER);}
    if ["Avatar","Image"].iter().any(|name|source.contains(&format!("String::Borrowed(\"{name}\")"))){runtime.push_str(crate::stdlib::ui::HTTP_MARKER);}
    // GPUI Kit embeds only its default icons; a program naming any other catalog
    // icon registers the complete bundled set instead.
    let all_icons=include_str!("../stdlib/ui/catalog-icons.txt").lines().any(|name|source.contains(&format!("String::Borrowed(\"{name}\")")));
    let mut binding_crates=std::collections::BTreeSet::new();
    for line in source.lines().filter(|line|line.starts_with(crate::project::rust::dependencies::MARKER)) {
        if let Ok(package)=serde_json::from_str::<crate::project::rust::dependencies::Package>(line.trim_start_matches(crate::project::rust::dependencies::MARKER)) {
            runtime.push_str(line);runtime.push('\n');
            for name in &package.crates {
                application=application.replace(&format!("use {name};"),&format!("use tsrust_runtime::{name};"));
                if binding_crates.insert(name.clone()){runtime.push_str(&format!("pub use {name};\n"));}
            }
        }
    }

    if source.contains("use tracing as tsr_tracing;"){application=application.replace("use tracing as tsr_tracing;","use tsrust_runtime::tracing as tsr_tracing;");runtime.push_str("pub use tracing;\n");}
    if source.contains("use rayon as tsr_rayon;") {application=application.replace("use rayon as tsr_rayon;","use tsrust_runtime::rayon as tsr_rayon;");runtime.push_str("pub use rayon;\n");}
    let json=[true,false].into_iter().map(json_module_with_writer).find(|module|source.contains(module));
    let dependencies=requires_dependencies(source);
    let mut extract=|name:&str,module:String| {
        if application.contains(&module) {
            application=application.replace(&module,&format!("use tsrust_runtime::{name};\n"));
            runtime.push_str("pub ");runtime.push_str(&module);
        }
    };
    for native in [true, false] {
        extract("tsrust_native_ui", crate::stdlib::ui::runtime(native));
    }
    extract("tsrust_native_date",crate::stdlib::date::RUNTIME.to_owned(),
    );
    extract("tsrust_native_signals",crate::stdlib::signals::RUNTIME.to_owned(),
    );
    extract("tsrust_native_crypto",crate::stdlib::crypto::RUNTIME.to_owned(),
    );
    extract("tsrust_native_crypto_password",crate::stdlib::crypto::PASSWORD_RUNTIME.to_owned());
    extract("tsrust_native_uuid",crate::stdlib::uuid::RUNTIME.to_owned(),
    );
    extract("tsrust_native_decimal",crate::stdlib::decimal::RUNTIME.to_owned(),
    );
    extract("tsrust_native_hostname",crate::stdlib::os::HOSTNAME_RUNTIME.to_owned());
    extract("tsrust_native_locale",crate::stdlib::locale::RUNTIME.to_owned());
    extract("tsrust_native_net",crate::stdlib::net::RUNTIME.to_owned());
    extract("tsrust_native_dns",crate::stdlib::net::DNS_RUNTIME.to_owned());
    extract("tsrust_native_net_transport",crate::stdlib::net::TRANSPORT_RUNTIME.to_owned());
    extract("tsrust_native_net_io",crate::stdlib::net::IO_RUNTIME.to_owned());
    extract("tsrust_native_regex",crate::stdlib::regex::RUNTIME.to_owned());
    extract("tsrust_native_url",crate::stdlib::url::RUNTIME.to_owned());
    extract("tsrust_native_process",crate::stdlib::process::RUNTIME.to_owned());
    extract("tsrust_native_compression_streams",crate::stdlib::compression::STREAM_RUNTIME.to_owned());
    extract("tsrust_native_compression",crate::stdlib::compression::RUNTIME.to_owned());
    extract("tsrust_native_io",crate::stdlib::io::runtime());
    extract("tsrust_native_http_io",crate::stdlib::io::HTTP_RUNTIME.to_owned());
    extract("tsrust_native_sql",crate::stdlib::sql::RUNTIME.to_owned());
    extract("tsrust_native_sqlite",crate::stdlib::sqlite::RUNTIME.to_owned());
    extract("tsrust_native_http",crate::stdlib::http::runtime());
    for testing in [true,false] {
        extract("tsrust_native_futures",futures_module(testing));
        for asynchronous in [true,false] {extract("tsrust_native_threads",threads_module(asynchronous,testing));}
    }
    extract("tsrust_native_cancellation",crate::stdlib::futures::CANCELLATION_RUNTIME.to_owned());
    // Match complete variants; no feature is added merely because another is used.
    for views in [true,false] {for utf8 in [true,false] {for (codecs,base64) in [(false,false),(true,false),(true,true)] {for operations in [false,true] {for binary in [false,true] {for search in [false,true] {extract("tsrust_native_bytes",bytes_module(views,utf8,codecs,base64,operations,binary,search));}}}}}}
    if let Some(module)=json {extract("tsrust_native_json",module);}
    extract("tsrust_native_logging",crate::stdlib::logging::RUNTIME.to_owned());
    extract("tsrust_native_files",files_module());extract("tsrust_native_tempfiles",tempfiles_module());extract("tsrust_native_path",path_module());
    if application.contains(crate::runtime::STRINGS) {
        // Other application helpers can still call the small number tie helper.
        application=application.replace(crate::runtime::STRINGS,&format!("use tsrust_runtime::text::*;\n{}",crate::runtime::NUMBER_TIE));
        runtime.push_str("pub mod text {\n");runtime.push_str(crate::runtime::STRINGS);runtime.push_str("\n}\n");
    }
    let time=crate::stdlib::time::RUNTIME;
    if application.contains(time) {
        application=application.replace(time,"use tsrust_runtime::time::tsrust_std_time_monotonic_now;\n");
        runtime.push_str("pub mod time {\n");runtime.push_str(time);runtime.push_str("\n}\n");
    }
    for asynchronous_testing in [true,false] {
        let console=console_module(asynchronous_testing);
        if application.contains(&console) {
            application=application.replace(&console,"use tsrust_runtime::console::*;\n");
            runtime.push_str("pub mod console {\n");
            runtime.push_str(include_str!("../runtime/number_tie.rs"));
            runtime.push_str(&console);runtime.push_str("\n}\n");
            // Raw programs expose these at their root; shared runtimes alias the
            // same collector functions rather than introducing another state.
            runtime.push_str("pub use console::tsr_console_test_work;\n");
            if asynchronous_testing {runtime.push_str("pub use console::tsr_console_test_future;\n");}
        }
    }
    let runtime=if all_icons {runtime.replace("application().with_assets(assets::Assets)","application().with_assets(assets::AllAssets)")} else {runtime};
    Units{application,dependencies,runtime:(!runtime.is_empty()).then(||format!("#![allow(dead_code, unused_variables, unused_mut)]\n{runtime}"))}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_predicate_agrees_with_extraction_for_plain_files_url_and_bindings() {
        let package=crate::project::rust::dependencies::Package {manifest:String::new(),lock:String::new(),crates:vec!["probe_crate".into()]};
        let binding=format!("{}{}\nmod tsr_rust {{}}",crate::project::rust::dependencies::MARKER,serde_json::to_string(&package).unwrap());
        for(source,expected)in[("fn main(){}".to_owned(),false),(files_module(),true),(crate::stdlib::url::RUNTIME.to_owned(),true),(binding,true),("mod tsr_rust0 {pub fn value()->i64{1}}".to_owned(),false),("use rayon as tsr_rayon;\nmod tsr_rust0 {use super::tsr_rayon as rayon;}".to_owned(),true)]{
            assert_eq!(requires_dependencies(&source),expected);assert_eq!(separate(&source).dependencies,expected);
        }
    }

    #[test]
    fn selected_byte_support_extracts_exactly_and_only_base64_or_search_needs_dependencies() {
        for views in [false,true] {for utf8 in [false,true] {for (codecs,base64) in [(false,false),(true,false),(true,true)] {for operations in [false,true] {for binary in [false,true] {for search in [false,true] {
            let source=bytes_module(views,utf8,codecs,base64,operations,binary,search);
            let units=separate(&source);
            assert_eq!(units.dependencies,base64||search);
            assert_eq!(requires_dependencies(&source),units.dependencies);
            assert_eq!(units.application,"use tsrust_runtime::tsrust_native_bytes;\n");
            let runtime=units.runtime.unwrap();
            assert_eq!(requires_byte_codecs(&runtime),base64);
            assert_eq!(runtime.contains("pub fn to_hex("),codecs);
            assert_eq!(runtime.contains("pub fn from_utf8("),utf8);
            assert_eq!(requires_byte_search(&runtime),search);
            assert_eq!(runtime.contains("pub fn concat("),operations);
            assert_eq!(runtime.contains("pub fn read_int("),binary);
        }}}}}}
    }
}

#[cfg(test)]mod io_tests{
 #[test]fn io_runtime_extraction_keeps_test_only_includes_out_of_exported_sources(){
  let runtime=crate::stdlib::io::runtime();assert!(!runtime.contains("runtime_tests.rs"));let source=format!("{runtime}fn main(){{}}");let units=super::separate(&source);assert!(units.dependencies);assert!(units.application.contains("use tsrust_runtime::tsrust_native_io"));let runtime=units.runtime.unwrap();assert!(runtime.contains("pub mod tsrust_native_io"));assert!(!runtime.contains("include!"));
 }
}

#[cfg(test)]mod http_middleware_tests{
 use super::*;
 #[test]
 fn optional_http_policies_preserve_extraction_marker_without_test_feature_activation(){
  let ordinary=format!("{}{}",crate::stdlib::futures::RUNTIME,crate::stdlib::http::runtime());assert!(ordinary.contains("#[cfg(feature = \"http-middleware\")]"));assert!(!ordinary.contains("#[cfg(any(test, feature = \"http-middleware\"))]"));
  let units=separate(&ordinary);assert!(units.dependencies);assert!(!units.runtime.unwrap().contains(crate::stdlib::http::MIDDLEWARE_MARKER));
  let selected=format!("{ordinary}{}",crate::stdlib::http::MIDDLEWARE_MARKER);let units=separate(&selected);assert!(units.runtime.unwrap().strip_prefix("#![allow(dead_code, unused_variables, unused_mut)]\n").expect("runtime lint prelude").starts_with(crate::stdlib::http::MIDDLEWARE_MARKER));assert!(!units.application.contains("mod tsrust_native_http {"));
 }
}

#[cfg(test)]
#[test]
fn password_runtime_extracts_with_the_async_pool_backend(){
 let source=format!("{}{}{}{}",crate::stdlib::futures::RUNTIME,crate::stdlib::threads::ASYNC_RUNTIME,crate::stdlib::crypto::RUNTIME,crate::stdlib::crypto::PASSWORD_RUNTIME);
 let units=separate(&source);assert!(units.dependencies);let runtime=units.runtime.unwrap();assert!(runtime.contains("pub mod tsrust_native_crypto_password {"));assert!(runtime.contains(crate::stdlib::threads::ASYNC_RUNTIME));assert!(!units.application.contains(crate::stdlib::crypto::PASSWORD_RUNTIME));assert!(units.application.contains("use tsrust_runtime::tsrust_native_crypto_password;"));
}

#[cfg(test)]
mod testing_worker_extraction_tests {
    use super::*;
    #[allow(dead_code)]
    mod console_context {
        include!("../runtime/number_tie.rs");
        include!("../runtime/console.rs");
        #[test]
        fn ended_scope_rejects_late_capture_without_reusing_or_changing_the_buffer() {
            let original=TSR_CONSOLE_TEST.with(|slot|slot.borrow().clone());
            let state=std::sync::Arc::new(std::sync::Mutex::new(TsrConsoleTest{name:"owned case".to_owned(),entries:Vec::new(),ended:false}));
            {
                let _scope=TsrConsoleTestScope::enter(Some(state.clone()));
                assert!(tsr_console_test_capture(false,"before end"));
                let entries={let mut state=state.lock().unwrap();state.ended=true;std::mem::take(&mut state.entries)};
                assert_eq!(entries.len(),1);assert_eq!(entries[0].2,"before end");
                assert!(!tsr_console_test_capture(false,"late detached output"));
                let state=state.lock().unwrap();assert!(state.ended);assert!(state.entries.is_empty());
            }
            let restored=TSR_CONSOLE_TEST.with(|slot|slot.borrow().clone());
            match (original,restored){(None,None)=>{},(Some(before),Some(after))=>assert!(std::sync::Arc::ptr_eq(&before,&after)),_=>panic!("worker scope changed the caller's test context")}
        }
    }
    #[test]
    fn selected_test_worker_variants_extract_one_shared_runtime_context() {
        for testing in [false,true] {for asynchronous in [false,true] {
            let source=format!("{}{}{}",futures_module(testing),threads_module(asynchronous,testing),console_module(testing));
            assert!(requires_dependencies(&source));let units=separate(&source);assert!(units.dependencies);
            let runtime=units.runtime.unwrap();
            for name in ["tsrust_native_futures","tsrust_native_threads","console"] {assert_eq!(runtime.matches(&format!("pub mod {name} {{")).count(),1);assert!(units.application.contains(&format!("use tsrust_runtime::{name}")));}
            assert!(!units.application.contains("tokio::task_local!"));assert!(!units.application.contains("fn capture_testing_context"));
            assert_eq!(runtime.contains("super::tsr_console_test_future"),testing);
            assert_eq!(runtime.contains("super::tsr_console_test_work"),testing);
            assert_eq!(runtime.matches("static TSR_CONSOLE_ASYNC_TEST:").count(),usize::from(testing));
            assert_eq!(runtime.matches("static SCOPE:").count(),1);
            assert!(runtime.contains("pub use console::tsr_console_test_work;"));
            assert_eq!(runtime.contains("pub use console::tsr_console_test_future;"),testing);
        }}
        let async_only=format!("{}{}",futures_module(true),console_module(true));
        assert!(requires_dependencies(&async_only));let units=separate(&async_only);assert!(units.dependencies);
        assert!(units.application.contains("use tsrust_runtime::tsrust_native_futures;"));assert!(units.application.contains("use tsrust_runtime::console::*;"));
        let runtime=units.runtime.unwrap();assert_eq!(runtime.matches("static TSR_CONSOLE_ASYNC_TEST:").count(),1);assert_eq!(runtime.matches("static SCOPE:").count(),1);assert!(!runtime.contains("pub mod tsrust_native_threads"));
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for (file,modules) in [("examples/testing/data-and-async.test.tsr",vec!["tsrust_native_futures","tsrust_native_threads"]),("crates/tsrust-compiler/tests/fixtures/async-io/runtime.test.tsr",vec!["tsrust_native_futures","tsrust_native_threads","tsrust_native_io","tsrust_native_http","tsrust_native_compression"])] {
            let checked=crate::project_cli::check_test_inputs(&[root.join(file)]).unwrap_or_else(|error|panic!("{}",error.render(file,&std::fs::read_to_string(root.join(file)).unwrap(),false)));let (source,features)=checked.emit_rust_with_native_features();assert!(features.contains(&"testing-runner"));assert!(source.contains(&futures_module(true)),"Task-derived async support must use the shared testing variant");assert!(requires_dependencies(&source));
            let units=separate(&source);assert!(units.dependencies);let runtime=units.runtime.unwrap();
            let application_items=syn::parse_file(&units.application).unwrap();let runtime_items=syn::parse_file(&runtime).unwrap();
            for name in modules {
                assert!(units.application.contains(&format!("use tsrust_runtime::{name};")));
                assert!(!application_items.items.iter().any(|item|matches!(item,syn::Item::Mod(module) if module.ident==name)),"{name} must live only in the shared runtime");
                assert_eq!(runtime_items.items.iter().filter(|item|matches!(item,syn::Item::Mod(module) if module.ident==name && matches!(module.vis,syn::Visibility::Public(_)))).count(),1,"exactly one public runtime module: {name}");
            }
            assert_eq!(runtime.matches("pub mod console {").count(),1);assert!(runtime.contains("TSR_CONSOLE_ASYNC_TEST.scope"));
            assert!(!units.application.contains("tokio::task_local!"));assert!(!units.application.contains("fn capture_testing_context"));
        }
    }
}
