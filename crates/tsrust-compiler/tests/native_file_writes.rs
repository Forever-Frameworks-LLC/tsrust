mod support;

#[test]
fn public_file_writes_borrow_owners_and_views_and_truncate() {
    let directory = support::TempDirectory::new();
    std::fs::write(directory.0.join("output.bin"), b"old trailing contents").unwrap();
    let source = support::format_round_trip(r#"
import { create, write, writeAll, close } from "std:fs";
import { fromUtf8, view, viewMut } from "std:bytes";
import { strictEqual } from "std:testing";
function main() {
  match (create("output.bin")) {
    Ok(file) => {
      match (fromUtf8("ABC")) {
        Ok(bytes) => {
          match (write(file, bytes)) {
            Ok(count) => { strictEqual(count, 3); },
            Err(error) => { throw error.message; }
          }
          match (view(bytes, 1, 3)) {
            Ok(window) => {
              match (writeAll(file, window)) {
                Ok(_) => {}, Err(error) => { throw error.message; }
              }
            }, Err(error) => { throw error.message; }
          }
          match (viewMut(bytes, 0, 1)) {
            Ok(window) => {
              match (writeAll(file, window)) {
                Ok(_) => {}, Err(error) => { throw error.message; }
              }
            }, Err(error) => { throw error.message; }
          }
        }, Err(error) => { throw error.message; }
      }
      match (close(file)) { Ok(_) => {}, Err(error) => { throw error.message; } }
    }, Err(error) => { throw error.message; }
  }
}
"#);
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path).current_dir(&directory.0), 60));
    assert_eq!(std::fs::read(directory.0.join("output.bin")).unwrap(), b"ABCBCA");
}

#[test]
fn file_write_bridge_needs_no_byte_operation_and_retains_read_only_handle_after_error() {
    let directory = support::TempDirectory::new();
    std::fs::write(directory.0.join("input.bin"), [0, 255, 65]).unwrap();
    let source = support::format_round_trip(r#"
import { open, read, create, write, writeAll } from "std:fs";
import { strictEqual } from "std:testing";
function main() {
  match (open("input.bin")) {
    Ok(input) => {
      match (read(input, 3)) {
        Ok(bytes) => {
          match (write(input, bytes)) {
            Ok(_) => { throw "read-only write succeeded"; },
            Err(error) => { strictEqual(error.operation, "write"); }
          }
          match (create("output.bin")) {
            Ok(output) => {
              match (writeAll(output, bytes)) {
                Ok(_) => {}, Err(error) => { throw error.message; }
              }
            }, Err(error) => { throw error.message; }
          }
          match (read(input, 0)) { Ok(_) => {}, Err(error) => { throw error.message; } }
        }, Err(error) => { throw error.message; }
      }
    }, Err(error) => { throw error.message; }
  }
}
"#);
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path).current_dir(&directory.0), 60));
    assert_eq!(std::fs::read(directory.0.join("output.bin")).unwrap(), [0, 255, 65]);
}

#[test]
fn file_writes_reject_consumed_handles_and_read_only_file_borrows() {
    for body in [
        "close(file); write(file, bytes);",
        "close(file); writeAll(file, bytes);",
    ] {
        let directory = support::TempDirectory::new();
        let source = support::format_round_trip(&format!("import {{create,close,write,writeAll}} from 'std:fs';import {{fromUtf8}} from 'std:bytes';function main(){{match(create('output.bin')){{Ok(file)=>{{match(fromUtf8('A')){{Ok(bytes)=>{{{body}}},Err(error)=>{{}}}}}},Err(error)=>{{}}}}}}"));
        let path=directory.source(&source);
        let output=support::run(support::cli().arg("check").arg(path),20);
        assert!(!output.status.success());
        let error=String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("error[transferred]"),"{error}");
    }
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip("import {File,write} from 'std:fs';import {Bytes} from 'std:bytes';function save(file:ReadOnly<File>,bytes:ReadOnly<Bytes>){write(file,bytes);}");
    let path=directory.source(&source);
    let output=support::run(support::cli().arg("check").arg(path),20);
    assert!(!output.status.success());
    let error=String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("read-only") || error.contains("ReadOnly"),"{error}");
}
