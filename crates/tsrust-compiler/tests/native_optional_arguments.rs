mod support;

#[test]
fn optional_native_arguments_preserve_defaults_borrows_order_and_function_values() {
    let source=r#"
import {basename} from 'std:path';
import {parse,stringify,parseJsonc,JsonLimits,fromArray,createJsonl,writeJsonl,closeJsonlWriter} from 'std:json';
import {alloc,resize,at,length} from 'std:bytes';
import {readFile} from 'std:fs';
import {strictEqual} from 'std:testing';
function suffix(path:string,ending?:string):string{return basename(path,ending);}
function path(log:int[]):string{log.push(1);return '/tmp/report.txt';}
function absent(log:int[]):undefined{log.push(2);return undefined;}
function optional(action:Mutable<Function<string,[path:ReadOnly<string>,suffix?:ReadOnly<string>]>>):string{return action('/tmp/report.txt',undefined);}
function required(action:Mutable<Function<string,[path:ReadOnly<string>,suffix:ReadOnly<string>]>>):string{return action('/tmp/report.txt','.txt');}
type Paths={name:Function<string,[path:ReadOnly<string>,suffix?:ReadOnly<string>]>;};
function main(){
 const source='/tmp/report.txt';const ending='.txt';
 strictEqual(basename(source,undefined),'report.txt');strictEqual(basename(source,ending),'report');
 strictEqual(suffix(source),'report.txt');strictEqual(suffix(source,ending),'report');
 strictEqual(source,'/tmp/report.txt');strictEqual(ending,'.txt');
 const trace:int[]=[];strictEqual(basename(path(trace),absent(trace)),'report.txt');strictEqual(trace.join(','),'1,2');
 const native=basename;strictEqual(native(source,undefined),'report.txt');strictEqual(optional(native),'report.txt');strictEqual(required(native),'report');
 strictEqual((true?basename:basename)(source,undefined),'report.txt');
 const paths:Paths={name:basename};strictEqual(paths.name(source),'report.txt');strictEqual(paths.name(source,undefined),'report.txt');strictEqual(paths.name(source,ending),'report');
 const endings:undefined[]=[undefined];strictEqual(basename(source,...endings),'report.txt');
 const limits:JsonLimits={maxBytes:32,maxDepth:4,maxEntries:4,maxNodes:4};
 const value=unwrap(parse('null',undefined));strictEqual(unwrap(stringify(value,undefined)),'null');
 strictEqual(unwrap(stringify(unwrap(parse('null',limits)))),'null');strictEqual(limits.maxBytes,32);
 strictEqual(unwrap(stringify(unwrap(parseJsonc('/* ok */ null',undefined,undefined)))),'null');
 strictEqual(unwrap(stringify(unwrap(fromArray([unwrap(parse('null'))],undefined)))),'[null]');
 const bytes=unwrap(alloc(2,undefined));strictEqual(unwrap(at(bytes,0)),0);unwrap(resize(bytes,3,undefined));strictEqual(length(bytes),3);
 const filled=unwrap(alloc(2,7));strictEqual(unwrap(at(filled,1)),7);
 const writer=unwrap(createJsonl('output.jsonl',undefined,32));unwrap(writeJsonl(writer,value));unwrap(closeJsonlWriter(writer));
 strictEqual(length(unwrap(readFile('output.jsonl',undefined))),5);
}
"#;
    let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(&path).current_dir(&dir.0),120);support::success(&output);
    assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
    assert_eq!(std::fs::read(dir.0.join("output.jsonl")).unwrap(),b"null\n");
}

#[test]
fn optional_native_arguments_keep_type_errors_and_ownership_checks() {
    let dir=support::TempDirectory::new();
    for (body,code) in [
        ("basename(undefined);","type"),
        ("basename('file',42);","type"),
        ("const action:Function<string,[path:ReadOnly<string>,suffix?:ReadOnly<string|int>]> = basename;action('file',42);","type"),
        ("const action:Function<string,[path:ReadOnly<string|int>]> = basename;action(42);","type"),
        ("const action:Function<string,[path:ReadOnly<string>,suffix?:ReadOnly<string|null>]> = basename;action('file',null);","type"),
        ("const path='file';basename(path,undefined);const moved=path;console.log(path);","transferred"),
        ("const bytes=unwrap(alloc(2));const moved=bytes;resize(bytes,3,undefined);","transferred"),
    ] {
        let source=format!("import {{basename}} from 'std:path';import {{alloc,resize}} from 'std:bytes';function main(){{{body}}}");
        let path=dir.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success(),"accepted {source}");assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")),"{}",String::from_utf8_lossy(&output.stderr));
    }
}
