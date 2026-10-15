//! Separate compiler-owned runtime modules from program-specific ABI bridges.
//! Emit retains bundled modules; JSON also needs its dependency package.
//! Self-contained package export is tracked in the JSON plan. This is not a Rust parser:
//! only exact bundled module definitions produced by our emitter are extracted.
pub struct Units {
    pub application:String,
    pub runtime:Option<String>,
    pub dependencies:bool,
}

pub(crate) fn bytes_module(views:bool,utf8:bool)->String {
    let mut text=String::from("mod tsrust_native_bytes {\n");
    text.push_str(include_str!("../stdlib/bytes/runtime/mod.rs"));
    if views {text.push_str(include_str!("../stdlib/bytes/runtime/views.rs"));}
    if utf8 {text.push_str(include_str!("../stdlib/bytes/runtime/utf8.rs"));}
    text.push_str("}\n");text
}
pub(crate) fn files_module()->String {
    format!("mod tsrust_native_files {{ mod resource {{\n{}\n}} pub mod fs {{\n{}\n}} pub mod buffered_fs {{\n{}\n}} }}\n",include_str!("../stdlib/fs/runtime/resource.rs"),include_str!("../stdlib/fs/runtime/fs.rs"),include_str!("../stdlib/fs/runtime/buffered_fs.rs"))
}
pub(crate) fn path_module()->String {
    format!("mod tsrust_native_path {{\n{}{}\n}}\n",include_str!("../stdlib/path/runtime.rs"),include_str!("../stdlib/path/relative.rs"))
}
/// Compiler-owned JSON support; public language bindings are implemented separately.
pub fn json_module()->String {json_module_with_writer(false)}
pub fn json_module_with_writer(writer:bool)->String {
    let writer=if writer {format!("pub mod writer {{\n{}\n}}\n",include_str!("../stdlib/json/writer.rs").replace("crate::stdlib::fs::runtime","crate::tsrust_native_files"))}else{String::new()};
    format!("mod tsrust_native_json {{\npub use serde_json::Map;\n{}\npub mod lines {{\n{}\n}}\npub mod typed {{\n{}\n}}\npub mod raw {{\n{}\n}}\npub mod source {{\n{}\n}}\n{writer}}}\n", concat!(include_str!("../stdlib/json/runtime/parser.rs"), "\n", include_str!("../stdlib/json/runtime/operations.rs")),include_str!("../stdlib/json/lines.rs"),include_str!("../stdlib/json/typed.rs"),include_str!("../stdlib/json/raw.rs"),include_str!("../stdlib/json/source.rs"))
}

/// Keep application record/union types, borrow adapters and generic ownership code
/// in the application crate. Extract only selected self-contained support modules.
pub fn separate(source:&str)->Units {
    let mut application=source.to_owned();let mut runtime=String::new();
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

    if source.contains("use rayon as tsr_rayon;") {application=application.replace("use rayon as tsr_rayon;","use tsrust_runtime::rayon as tsr_rayon;");runtime.push_str("pub use rayon;\n");}
    let json=[true,false].into_iter().map(json_module_with_writer).find(|module|source.contains(module));
    let dependencies=json.as_ref().is_some_and(|module|source.contains(module)) || source.contains(crate::stdlib::futures::RUNTIME) || source.contains(crate::stdlib::threads::RUNTIME) || source.contains("mod tsr_rust");
    let mut extract=|name:&str,module:String| {
        if application.contains(&module) {
            application=application.replace(&module,&format!("use tsrust_runtime::{name};\n"));
            runtime.push_str("pub ");runtime.push_str(&module);
        }
    };
    extract("tsrust_native_http",crate::stdlib::http::RUNTIME.to_owned());
    extract("tsrust_native_futures", crate::stdlib::futures::RUNTIME.to_owned());
    extract("tsrust_native_threads", crate::stdlib::threads::RUNTIME.to_owned());
    // Match complete variants; no feature is added merely because another is used.
    for (views,utf8) in [(true,true),(true,false),(false,true),(false,false)] {extract("tsrust_native_bytes",bytes_module(views,utf8));}
    if let Some(module)=json {extract("tsrust_native_json",module);}
    extract("tsrust_native_files",files_module());extract("tsrust_native_path",path_module());
    if application.contains(crate::runtime::STRINGS) {
        // Other application helpers can still call the small number tie helper.
        application=application.replace(crate::runtime::STRINGS,&format!("use tsrust_runtime::text::*;\n{}",crate::runtime::NUMBER_TIE));
        runtime.push_str("pub mod text {\n");runtime.push_str(crate::runtime::STRINGS);runtime.push_str("\n}\n");
    }
    let console=include_str!("../runtime/console.rs");
    let time=crate::stdlib::time::RUNTIME;
    if application.contains(time) {
        application=application.replace(time,"use tsrust_runtime::time::tsrust_std_time_monotonic_now;\n");
        runtime.push_str("pub mod time {\n");runtime.push_str(time);runtime.push_str("\n}\n");
    }
    if application.contains(console) {
        application=application.replace(console,"use tsrust_runtime::console::*;\n");
        runtime.push_str("pub mod console {\n");
        runtime.push_str(include_str!("../runtime/number_tie.rs"));
        runtime.push_str(console);runtime.push_str("\n}\n");
    }
    Units{application,dependencies,runtime:(!runtime.is_empty()).then(||format!("#![allow(dead_code, unused_variables, unused_mut)]\n{runtime}"))}
}
