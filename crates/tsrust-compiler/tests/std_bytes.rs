mod support;

#[test]
fn native_byte_type_imports_and_result_payloads_execute() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/values.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_callable_and_generic_aggregate_storage_execute() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/storage.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn exported_result_aliases_keep_builtin_identity_and_native_payloads() {
    let directory = support::TempDirectory::new();
    let library = support::format_round_trip("import {Bytes,ByteError,fromArray} from 'std:bytes';export type Outcome=Result<Bytes,ByteError>;export function build():Outcome{return fromArray([1,2]);}export type Plain=Result<int,string>;");
    std::fs::write(directory.0.join("types.tsr"), library).unwrap();
    let main = support::format_round_trip("import {Outcome as Batch,Plain,build} from './types.tsr';import {length} from 'std:bytes';import {strictEqual} from 'std:testing';function inspect(result:ReadOnly<Batch>):int{match(result){Ok(bytes)=>{return length(bytes);},Err(error)=>{return -1;}}}function main(){const result:Batch=build();strictEqual(inspect(result),2);const plain:Plain=Ok(4);match(plain){Ok(value)=>{strictEqual(value,4);},Err(error)=>{throw error;}}}");
    let path = directory.source(&main);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_access_and_mutation_validate_before_writing() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/access.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_allocation_and_resize_preserve_defaults_and_atomic_errors() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/growth.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_append_is_atomic_and_range_copies_are_independent() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/copy-append.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_clone_storage_capture_and_evaluation_contracts_execute() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/ownership.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_utf8_round_trips_and_strict_error_offsets_execute() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/utf8.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn byte_views_read_and_write_through_public_imports() {
    for fixture in [
        include_str!("fixtures/native-byte-views/read.tsr"),
        include_str!("fixtures/native-byte-views/values.tsr"),
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(fixture));
        support::success(&support::run(support::cli().arg("run").arg(path), 120));
    }
}

#[test]
fn byte_view_ownership_rules_are_source_diagnostics() {
    let imports = "import { Bytes, ByteView, ByteViewMut, alloc, view, viewMut, length, set, resize } from \"std:bytes\";";
    let inside = |body: &str| format!("function main() {{ match (alloc(2, 1)) {{ Ok(bytes) => {{ {body} }}, Err(error) => {{}}, }} }}");
    let with_view = |factory: &str, body: &str| inside(&format!("match ({factory}(bytes, 0, 1)) {{ Ok(part) => {{ {body} }}, Err(error) => {{}}, }}"));
    for (label, source, code, text) in [
        ("returned view of a local owner", "function leak(): ByteView { match (alloc(2, 1)) { Ok(bytes) => { match (view(bytes, 0, 1)) { Ok(part) => { return part; }, Err(error) => { throw error.message; }, } }, Err(error) => { throw error.message; }, } } function main() { console.log(length(leak())); }".to_owned(), "native-view-escape", "local owner"),
        ("owner write while a view is live", with_view("view", "match (set(bytes, 0, 9)) { Ok(_) => {}, Err(error) => {}, } console.log(length(part));"), "native-view-conflict", "bytes"),
        ("owner resize while a view is live", with_view("view", "match (resize(bytes, 9)) { Ok(_) => {}, Err(error) => {}, } console.log(length(part));"), "native-view-conflict", "bytes"),
        ("owner move while a view is live", with_view("view", "const moved = bytes; console.log(length(part));"), "native-view-conflict", "bytes"),
        ("owner read while an exclusive view is live", with_view("viewMut", "console.log(length(bytes)); console.log(length(part));"), "native-view-conflict", "bytes"),
        ("write through a readonly view", with_view("view", "match (set(part, 0, 9)) { Ok(_) => {}, Err(error) => {}, }"), "type", "Expected Bytes, found ByteView"),
        ("exclusive view of a readonly view", with_view("view", "match (viewMut(part, 0, 1)) { Ok(inner) => {}, Err(error) => {}, }"), "type", "Expected Bytes, found ByteView"),
        ("clone of a view", with_view("view", "const other = clone(part); console.log(length(other));"), "clone-type", "copy(view, start, end)"),
        ("constructed view", "function main() { const made: ByteView = { }; }".to_owned(), "native-opaque", "constructors"),
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&format!("{imports}\n{source}"));
        let output = support::run(support::cli().arg("check").arg(path), 60);
        assert!(!output.status.success(), "{label}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(&format!("error[{code}]")) && stderr.contains(text), "{label}: {stderr}");
    }
}
