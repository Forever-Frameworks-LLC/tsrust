mod support;
#[test]fn sqlite_readable_examples_execute_and_format(){
 let dir=support::TempDirectory::new();for(source,expected)in[(include_str!("../../../examples/sqlite/people.tsr"),"1: Ada\n2: Grace\n"),(include_str!("../../../examples/sqlite/rollback.tsr"),"Transfer failed\n0\n")]{let source=support::format_round_trip(source);let output=support::run(support::cli().arg("run").arg(dir.source(&source)).env("TSRUST_CACHE_DIR",dir.0.join("cache")),300);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());}
}
#[test]fn sqlite_ownership_rejects_live_borrows_and_reuse(){
 for body in [
 "const stmt=unwrap(db.prepare('SELECT 1'));unwrap(db.close());stmt.query();",
 "const stmt=unwrap(db.prepare('SELECT 1'));stmt.query();unwrap(db.close());",
 "const tx=unwrap(db.begin());db.prepare('SELECT 1');tx.rollback();",
 "const tx=unwrap(db.begin());const stmt=unwrap(tx.prepare('SELECT 1'));tx.commit();stmt.query();",
 "unwrap(db.close());db.prepare('SELECT 1');",
 "const tx=unwrap(db.begin());tx.commit();tx.rollback();",
 "const other=clone(db);",
 "const stmt=unwrap(db.prepare('SELECT ?1'));stmt.query([true]);",
 ]{let source=format!("import {{open}} from 'std:sqlite';function main(){{const db=unwrap(open(':memory:'));{body}}}");let dir=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(dir.source(&source)),30);assert!(!output.status.success(),"accepted {source}");let error=String::from_utf8_lossy(&output.stderr);assert!(!error.contains("error[syntax]")&&!error.contains("error[import]"),"{error}");}
}
#[test]fn sqlite_rows_outlive_the_connection_and_bind_values(){
 let source="import {open} from 'std:sqlite';import {fromArray,length,at} from 'std:bytes';import {strictEqual} from 'std:testing';function main(){const connect=open;const db=unwrap(connect(':memory:',0));const stmt=unwrap(db.prepare('SELECT ?1 AS i, ?2 AS r, ?3 AS t, ?4 AS b, ?5 AS n'));const bytes=unwrap(fromArray([0,255]));const rows=unwrap(stmt.query([9223372036854775807,1.25,'hello',bytes,null]));unwrap(stmt.finalize());unwrap(db.close());strictEqual(unwrap(rows[0].int('i')),9223372036854775807);strictEqual(unwrap(rows[0].number('r')),1.25);strictEqual(unwrap(rows[0].text('t')),'hello');strictEqual(unwrap(rows[0].isNull('n')),true);const blob=unwrap(rows[0].blob('b'));strictEqual(length(blob),2);strictEqual(unwrap(at(blob,1)),255);}";
 let dir=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(dir.source(&support::format_round_trip(source))),300);support::success(&output);
}

#[test]fn unwrap_preserves_existing_byte_view_loans(){
 let source="import {fromArray,view,resize,length} from 'std:bytes';function main(){const bytes=unwrap(fromArray([1,2]));const borrowed=unwrap(view(bytes,0,1));resize(bytes,3);console.log(length(borrowed));}";
 let dir=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(dir.source(source)),30);assert!(!output.status.success());let message=String::from_utf8_lossy(&output.stderr);assert!(message.contains("borrow")||message.contains("loan"),"{message}");
}

#[test]fn sqlite_catalog_examples_execute(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let dir=support::TempDirectory::new();let mut seen=std::collections::BTreeSet::new();
 for(family,entries)in catalog.as_object().unwrap(){if family!="std:sqlite"&&!family.starts_with("Sqlite"){continue;}for entry in entries.as_array().unwrap(){for example in entry["examples"].as_array().unwrap(){let source=support::format_round_trip(example["source"].as_str().unwrap());if !seen.insert(source.clone()){continue;}let output=support::run(support::cli().arg("run").arg(dir.source(&source)).env("TSRUST_CACHE_DIR",dir.0.join("cache")),300);support::success(&output);assert_eq!(output.stdout,example["output"].as_str().unwrap().as_bytes());}}}
}
