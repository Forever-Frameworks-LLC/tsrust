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
