mod support;

#[test]
fn buffered_files_transfer_ownership_and_flush_at_close() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(r#"
import { create, open, bufferWriter, bufferReader, writeAll, read, close, flush, BufferedWriter, BufferedReader } from "std:fs";
import { fromUtf8, toUtf8, view } from "std:bytes";
import { strictEqual } from "std:testing";
function writeText(writer: Mutable<BufferedWriter>): void {
  match (fromUtf8("hello")) {
    Ok(bytes) => { match (view(bytes, 0, 5)) { Ok(window) => { match (writeAll(writer, window)) { Ok(_) => {}, Err(e) => { throw e.message; } } }, Err(e) => { throw e.message; } } },
    Err(e) => { throw e.message; }
  }
}
function readText(reader: Mutable<BufferedReader>): void {
  match (read(reader, 5)) {
    Ok(bytes) => { match (toUtf8(bytes)) { Ok(text) => { strictEqual(text, "hello"); }, Err(e) => { throw e.message; } } },
    Err(e) => { throw e.message; }
  }
}
function main() {
  match (create("output.bin")) {
    Ok(file) => { match (bufferWriter(file)) {
      Ok(writer) => { writeText(writer); match (flush(writer)) { Ok(_) => {}, Err(e) => { throw e.message; } } writeText(writer); match (close(writer)) { Ok(_) => {}, Err(e) => { throw e.message; } } },
      Err(e) => { throw e.message; }
    } }, Err(e) => { throw e.message; }
  }
  match (open("output.bin")) {
    Ok(file) => { match (bufferReader(file, 5)) {
      Ok(reader) => { readText(reader); readText(reader); match (close(reader)) { Ok(_) => {}, Err(e) => { throw e.message; } } },
      Err(e) => { throw e.message; }
    } }, Err(e) => { throw e.message; }
  }
}
"#);
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path).current_dir(&directory.0), 60));
    assert_eq!(std::fs::read(directory.0.join("output.bin")).unwrap(), b"hellohello");
}

#[test]
fn buffered_files_reject_owner_reuse_clone_and_wrong_access() {
    let cases = [
        ("function test(file:File){bufferReader(file);close(file);}", "transferred"),
        ("function test(file:File){bufferWriter(file,0);close(file);}", "transferred"),
        ("function test(writer:BufferedWriter){clone(writer);}", "clone"),
        ("function test(reader:BufferedReader){clone(reader);}", "clone"),
        ("function test(writer:ReadOnly<BufferedWriter>){flush(writer);}", "ReadOnly"),
        ("function test(writer:Mutable<BufferedWriter>){read(writer,1);}", "Expected"),
        ("function test(reader:Mutable<BufferedReader>,bytes:ReadOnly<Bytes>){writeAll(reader,bytes);}", "Expected"),
        ("function test(writer:BufferedWriter){close(writer);flush(writer);}", "transferred"),
    ];
    let directory=support::TempDirectory::new();
    for (body,expected) in cases {
        let source=support::format_round_trip(&format!("import {{File,BufferedReader,BufferedWriter,bufferReader,bufferWriter,close,flush,read,writeAll}} from 'std:fs';import {{Bytes}} from 'std:bytes';{body}"));
        let output=support::run(support::cli().arg("check").arg(directory.source(&source)),20);
        assert!(!output.status.success(),"{body}");
        let error=String::from_utf8_lossy(&output.stderr);
        assert!(error.to_lowercase().contains(&expected.to_lowercase()),"{body}: {error}");
    }
}
