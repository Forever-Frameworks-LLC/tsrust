mod support;

#[test]
fn file_reads_and_consuming_close_execute_through_public_imports() {
    let directory = support::TempDirectory::new();
    std::fs::write(directory.0.join("input.bin"), [0, 65, 255, 66]).unwrap();
    let source = support::format_round_trip(include_str!("fixtures/native-files/read-close.tsr"));
    let references = source.replace("function main() {", "function main() {const begin=open; const finish=close;")
        .replace("open(\"input.bin\")", "begin(\"input.bin\")").replace("close(file)", "finish(file)")
        .replace("match (read(file, count))", "const receive=read; match (receive(file, count))");
    for source in [source, support::format_round_trip(&references)] {
        let path = directory.source(&source);
        for release in [false, true] {
            let mut command = support::cli();
            command.arg("run").arg(&path).current_dir(&directory.0);
            if release { command.arg("--release"); }
            support::success(&support::run(&mut command, 60));
        }
    }
}

#[test]
fn file_close_and_capability_misuse_are_source_errors() {
    let directory = support::TempDirectory::new();
    for (body, code) in [
        ("close(file);close(file);", "transferred"),
        ("close(file);read(file,1);", "transferred"),
        ("match(close(file)){Ok(_)=>{},Err(error)=>{read(file,1);}}", "transferred"),
        ("console.log(file);", "native-capability"),
        ("const same=file===file;", "type"),
    ] {
        let source = format!("import {{open,close,read}} from 'std:fs';function main(){{match(open('input.bin')){{Ok(file)=>{{{body}}},Err(error)=>{{}}}}}}");
        let path = directory.source(&support::format_round_trip(&source));
        let output = support::run(support::cli().arg("check").arg(path), 60);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")), "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn file_type_identity_is_opaque_noncloneable_and_nonshareable() {
    use tsrust_compiler::{native_signatures::NativeTypeId, native_types::{declaration, Representation}};
    let file = declaration(NativeTypeId::File);
    assert_eq!(file.representation, Representation::Opaque);
    assert!(!file.independent_clone && !file.collection_key && !file.thread_transfer);
    assert!(file.fields.is_empty() && file.capabilities.is_empty());
    let directory = support::TempDirectory::new();
    for source in [
        "import {File} from 'std:fs';function main(){const file:File={};}",
        "import {open} from 'std:fs';function main(){match(open('input.bin')){Ok(file)=>{clone(file);},Err(error)=>{}}}",
    ] {
        let path = directory.source(&support::format_round_trip(source));
        let output = support::run(support::cli().arg("check").arg(path), 60);
        assert!(!output.status.success());
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(diagnostic.contains("error[") && !diagnostic.contains("native compilation"), "{diagnostic}");
    }
}
