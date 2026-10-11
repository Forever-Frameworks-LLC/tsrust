mod support;
use tsrust_compiler::check;
fn native(source: &str, stdout: &str, stderr: &str) {
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("input.tsr", source)));
    let dir = support::TempDirectory::new();
    let input = dir.0.join("console.rs");
    let code = analysis.emit_rust();
    assert!(!code.contains("unsafe"));
    std::fs::write(&input, code).unwrap();
    for level in ["0", "3"] {
        let binary = dir.0.join(format!("console-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-Awarnings",
                    "-C",
                    &format!("opt-level={level}"),
                ])
                .arg(&input)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(binary), 15);
        support::success(&output);
        assert_eq!(output.stdout, stdout.as_bytes());
        assert_eq!(output.stderr, stderr.as_bytes());
    }
}
fn node_native(source: &str) {
    let version = std::process::Command::new("node")
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        "v24.11.1",
        "console differential fixtures require the pinned Node reference"
    );
    let typescript = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/runtime/node_modules/typescript");
    let script = "const ts=require(process.argv[1]); const js=ts.transpileModule(process.argv[2],{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText; eval(js+'\\nmain();');";
    let reference = support::run(
        std::process::Command::new("node")
            .arg("-e")
            .arg(script)
            .arg(typescript)
            .arg(source),
        15,
    );
    support::success(&reference);
    native(
        source,
        std::str::from_utf8(&reference.stdout).unwrap(),
        std::str::from_utf8(&reference.stderr).unwrap(),
    );
}
#[test]
fn variadic_methods_streams_and_once_only_order() {
    node_native(
        r#"function main(){let n=0;console.log();console.info("count", n++, n++, true, 1.5);console.debug("debug", n);console.warn("warning", n++);console.error("error",n);console.log("%s", "literal");}"#,
    );
}
#[test]
fn structured_borrowed_values_tables_and_nested_records() {
    node_native(
        r#"type User={name:string;age:int;};type Group={users:User[];};function main(){const user:User={name:"Pat",age:7};const users:User[]=[{name:"Sam",age:9}];const group:Group={users:[{name:"Ada",age:2}]};console.log(user, users, group);console.log([1,2], [[3],[4]], [true,false], [1.5,2.0], ["a","é"]);console.table(user);console.table([1,2]);console.table(users);console.table("hello");console.log(user.name, users.length);}"#,
    );
}
#[test]
fn earlier_aggregate_loans_reject_later_mutation_and_consumption() {
    for source in [
        r#"function grow(v:int[]):int{v.push(9);return 1;}function main(){let v=[1];console.log(v,grow(v));}"#,
        r#"function take(v:int[]):int{const moved=v;return moved.length;}function main(){const v=[1];console.log(v,take(v));}"#,
        r#"function main(){console.log(():int=>{return 1;});}"#,
        r#"function main(){console.table(1,2);}"#,
        r#"function main(){console.dir(1,2);}"#,
        r#"function main(){console.dir(1,{colors:7});}"#,
        r#"function main(){console.dir(1,{depth:true});}"#,
    ] {
        assert!(check(source).is_err(), "{source}");
    }
}
#[test]
fn recursive_records_have_bounded_inspection() {
    node_native(
        r#"type Node={value:int;children:Node[];};function main(){const node:Node={value:1,children:[{value:2,children:[{value:3,children:[]}]}]};console.log(node);}"#,
    );
}

#[test]
fn table_columns_empty_inputs_and_control_characters() {
    node_native(
        r#"type User={name:string;age:int;};function main(){const users:User[]=[{name:"A\tB",age:4}];console.table(users,["age"]);console.table(users,[]);const empty:int[]=[];console.table(empty);console.table(["a\nb"]);}"#,
    );
}

#[test]
fn temporaries_and_post_call_mutation_keep_native_lifetimes() {
    node_native(
        r#"type Box={values:int[];};function make():int[]{return [1,2];}function main(){let values=[3];console.log(make(),true?values:[9],"é".repeat(2));values.push(4);console.log(values);const box:Box={values:[5]};console.log(box);console.table(make());}"#,
    );
}

#[test]
fn inspection_temporaries_are_disposed_after_output() {
    let source = r#"type User={name:string;values:int[];};function main(){const users:User[]=[{name:"é".repeat(3),values:[1,2,3]}];console.log(users);console.table(users);console.error(users);}"#;
    let mut rust = check(source)
        .unwrap()
        .emit_rust()
        .replace("fn main()", "fn console_source_main()");
    rust.push_str(r#"
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicIsize,Ordering};
static CONSOLE_LIVE:AtomicIsize=AtomicIsize::new(0);
struct ConsoleAllocations;
unsafe impl GlobalAlloc for ConsoleAllocations {
    unsafe fn alloc(&self,layout:Layout)->*mut u8 {
        let pointer=unsafe{System.alloc(layout)};
        if !pointer.is_null(){CONSOLE_LIVE.fetch_add(1,Ordering::SeqCst);}
        pointer
    }
    unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){unsafe{System.dealloc(pointer,layout)};CONSOLE_LIVE.fetch_sub(1,Ordering::SeqCst);}
    unsafe fn realloc(&self,pointer:*mut u8,layout:Layout,size:usize)->*mut u8{unsafe{System.realloc(pointer,layout,size)}}
}
#[global_allocator]static CONSOLE_ALLOCATOR:ConsoleAllocations=ConsoleAllocations;
fn main(){
    // Warm persistent stdio buffers before measuring operation-owned storage.
    console_source_main();
    let baseline=CONSOLE_LIVE.load(Ordering::SeqCst);
    for _ in 0..3 {console_source_main();assert_eq!(CONSOLE_LIVE.load(Ordering::SeqCst),baseline);}
}
"#);
    let dir = support::TempDirectory::new();
    let path = dir.0.join("console-disposal.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = dir.0.join(format!("console-disposal-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-Awarnings",
                    "-C",
                    &format!("opt-level={level}"),
                ])
                .arg(&path)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        support::success(&support::run(&mut std::process::Command::new(binary), 15));
    }
}

#[test]
fn inline_objects_and_format_placeholders_match_node() {
    node_native(
        r#"function main(){console.table({name:"Pat",age:7});console.log({nested:{name:"Ada"},values:[1,2]});console.log([]);console.table([]);console.table({});console.table([1,2],["missing"]);console.log("%j",{text:"\u007f"});console.log("%s %d %i %f %j %o %O %%", "hello", "2.5bad", "2.5bad", "2.5bad", {a:1}, [1,2], {b:true});}"#,
    );
}

#[test]
fn numeric_format_coercion_matches_node() {
    node_native(
        r#"function main(){console.log("%i %i %i %f %d %d %s", "1e2", "0xff", "-0x10", "Infinity!", "0x10", "0b11", -0.0);console.log("%s",{a:{b:{c:1}}});console.log("%i %i %f %d %d",1e21,-0.0,-0.0,[[1]],"infinity");console.log(["\v","\u0001","a'b\\c"]);}"#,
    );
}

#[test]
fn console_state_assertion_and_inspection_methods_match_node() {
    node_native(
        r#"function main(){console.table();console.assert();console.assert(true,"hidden");console.assert(false,"bad %d",7);console.assert(0);console.assert("ok");console.count();console.count();console.countReset();console.count();console.count("x");console.count("x");console.countReset("x");console.count("x");console.group("outer %s","group");console.log("a\nb");console.groupCollapsed("inner");console.warn("warn");console.groupEnd();console.log("done");console.groupEnd();console.groupEnd();console.dir({a:{b:1}},{depth:0});console.dir([1,2],{showHidden:true});console.dir({name:"Pat",age:7},{colors:true});console.assert(false,"");console.assert(false,1,2);console.dir();console.dirxml({a:1});console.clear();console.profile();console.profileEnd();console.timeStamp();}"#,
    );
}

#[test]
fn timers_warnings_and_native_trace_have_explicit_contracts() {
    let source = r#"function main(){console.time("work");console.timeLog("work","%s",7);console.timeEnd("work");console.timeEnd("missing");console.countReset("missing");console.trace("here");}"#;
    let dir = support::TempDirectory::new();
    let input = dir.0.join("console-host.rs");
    std::fs::write(&input, check(source).unwrap().emit_rust()).unwrap();
    for level in ["0", "3"] {
        let binary = dir.0.join(format!("host-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-Awarnings",
                    "-C",
                    &format!("opt-level={level}"),
                ])
                .arg(&input)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(&binary), 15);
        support::success(&output);
        let stdout = String::from_utf8(output.stdout).unwrap();
        let lines = stdout.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        for (index, line) in lines.iter().enumerate() {
            let elapsed = line
                .strip_prefix("work: ")
                .unwrap()
                .strip_suffix(if index == 0 { "ms %s 7" } else { "ms" })
                .unwrap();
            assert!(elapsed.parse::<f64>().unwrap() >= 0.0);
        }
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.starts_with("No such label 'missing' for console.timeEnd()\nCount for 'missing' does not exist\nTrace: here\n"));
        assert!(stderr.contains("console_call") || stderr.contains("0:"));
    }
}

#[test]
fn nullable_console_values_preserve_payloads_absence_and_borrows() {
    native(r#"type Row={name:string|null;};function main(){
        const users=['Alice','Bob','Charlie'];
        const lower=users.map(user=>user.toLowerCase());
        const upper=users.map((user:string):string=>{return user.toUpperCase();});
        const last=users.pop();console.log(last);console.log(last);
        const empty:string[]=[];console.log(empty.pop());
        const n:int|null=7;const absent:int|null=null;console.log(n,absent);
        const row:Row={name:null};console.log(row);console.log(row.name);
        const named:Row={name:"Pat"};console.log(named);
        console.log(true?"present".repeat(1):null);
        const items:int[]|null=[1,2];console.log(items);console.log(items);
        console.log(lower.join('|'),upper.join('|'));
    }"#, "Charlie\nCharlie\nundefined\n7 null\n{ name: null }\nnull\n{ name: 'Pat' }\npresent\n[ 1, 2 ]\n[ 1, 2 ]\nalice|bob|charlie ALICE|BOB|CHARLIE\n", "");
}

#[test]
fn count_reset_requires_string_labels_without_resetting_numeric_labels() {
    let source=support::format_round_trip(r#"function main(){console.count(1);console.countReset(1);console.count(1);console.countReset("1");console.count(1);console.count();console.countReset(undefined);console.count();}"#);
    native(&source,"1: 1\n1: 2\n1: 1\ndefault: 1\ndefault: 1\n","Count for '1' does not exist\n");
}
