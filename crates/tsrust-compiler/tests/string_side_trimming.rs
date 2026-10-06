mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, body: &str, globals: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("side-trim.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("side-trim.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("side-trim-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(&path)
                .arg("-o")
                .arg(&executable),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(&executable), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

const METHODS: &str = r#"
function start(value:string):string{return value.trimStart();}
function end(value:string):string{return value.trimEnd();}
function left(value:string):string{return value.trimLeft();}
function right(value:string):string{return value.trimRight();}
"#;

#[test]
fn sides_and_aliases_preserve_exact_native_whitespace_unicode_nul_and_interior_bytes() {
    let source = METHODS.to_owned()
        + r#"
        function main(){
            console.log(" x ".trimStart()==="x ");console.log(" x ".trimLeft()==="x ");console.log(" x ".trimEnd()===" x");console.log(" x ".trimRight()===" x");
            console.log("\u0085A\u0085".trimStart().length);console.log("\uFEFF A \uFEFF".trimEnd().length);
            console.log(" \t\u2003".trimLeft().length);console.log(" \t\u2003".trimRight().length);console.log("😀".trimLeft().length);console.log(" a\u0000b ".trimRight().length);
            const words=[" a "," b "];console.log(words[0].trimStart().trimEnd());
        }
    "#;
    let analysis = native(
        &source,
        "true\ntrue\ntrue\ntrue\n3\n9\n0\n0\n4\n4\na\n",
        r#"
        let cases=[
            ("","",""),(" x ","x "," x"),(" \t\r\n","",""),
            ("\u{85}A\u{85}","A\u{85}","\u{85}A"),("\u{feff} A \u{feff}","\u{feff} A \u{feff}","\u{feff} A \u{feff}"),
            ("\u{2003}é \u{2003}","é \u{2003}","\u{2003}é"),("\u{3000}😀\u{2028}","😀\u{2028}","\u{3000}😀"),
            (" \u{301}e ","\u{301}e "," \u{301}e"),(" a\0b ","a\0b "," a\0b"),("x \t y","x \t y","x \t y"),
            ("\t\n\u{b}\u{c}\r \u{85}\u{a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}","","")
        ];
        for(raw,start,end)in cases{for owned in [false,true]{
            let value=if owned{String::Owned(raw.to_owned())}else{String::Borrowed(raw)};let pointer=value.as_ptr();
            let results=[function0(&value),function1(&value),function2(&value),function3(&value)];
            for(index,result)in results.iter().enumerate(){let expected=if index%2==0{start}else{end};assert_eq!(result.as_ref(),expected);
                if owned{assert!(matches!(result,String::Owned(_)));if !result.is_empty(){assert_ne!(result.as_ptr(),pointer);}}
                else{assert!(matches!(result,String::Borrowed(_)));let offset=if index%2==0{raw.len()-expected.len()}else{0};assert_eq!(result.as_ptr(),raw.as_ptr().wrapping_add(offset));}
            }assert_eq!(value.as_ref(),raw);
        }}
    "#,
        "",
    );
    for name in ["start.value", "end.value", "left.value", "right.value"] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Read));
    }
    assert!(analysis.emit_rust().contains("LiteralStringArray"));
}

#[test]
fn receiver_effects_chains_fields_indices_callbacks_and_captures_keep_owners_reusable() {
    let analysis = native(
        r#"
        type Box={text:string;other:string;};
        function make():string{console.log("receiver");return " A ".toLowerCase();}
        function invoke(f:(value:ReadOnly<string>)=>string,value:string):string{return f(value);}
        function factory(value:string):()=>string{return():string=>{return value.trimStart().trimEnd();};}
        function pick():int{console.log("index");return 0;}
        function main(){
            console.log(make().trimStart().trimEnd());console.log(make().trimLeft().length);console.log(make().trimRight().length);
            const text=" A ".toLowerCase();const first=text.trimStart();const second=text.trimEnd();console.log(first.length);console.log(second.length);console.log(text.length);
            const box:Box={text:" x ".toUpperCase(),other:"saved"};console.log(box.text.trimLeft().trimRight());const moved=box.other;console.log(box.text.trimEnd().length);box.other=moved;console.log(box.other);
            const words=[" y ".toUpperCase()," z "];const result=words[0].trimLeft();words[0]="changed";console.log(result.length);console.log(words[1].trimRight().trimLeft());
            const repeat=():string=>{return text.trimLeft().trimRight();};console.log(repeat());console.log(repeat());
            const saved=factory(" B ".toLowerCase());console.log(saved());console.log(saved());
            console.log(invoke((input:string):string=>{return input.trimRight();},text).length);console.log(words[pick()].trimRight());
            console.log(" A ".trimStart().toLowerCase().length);console.log(" a ".trimEnd().toUpperCase().length);
        }
    "#,
        "receiver\na\nreceiver\n2\nreceiver\n2\n2\n2\n3\nX\n2\nsaved\n2\nz\na\na\nb\nb\n2\nindex\nchanged\n2\n2\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("invoke.value"), Some(&Mode::Read));
    assert_eq!(analysis.closures()[1].invocation, Mode::Read);
}

#[test]
fn four_method_diagnostics_preserve_zero_arity_types_transfers_and_capture_loans() {
    for method in ["trimStart", "trimEnd", "trimLeft", "trimRight"] {
        for (invocation, code) in [
            (format!("\"x\".{method}(1)"), "arity"),
            (format!("(1).{method}()"), "type"),
            (format!("true.{method}()"), "type"),
            (format!("[\"x\"].{method}()"), "type"),
        ] {
            let source = format!("function main(){{{invocation};}}");
            assert_eq!(
                check(&source)
                    .err()
                    .unwrap_or_else(|| panic!("Expected {code}: {source}"))
                    .code,
                code
            );
        }
        for (source, code) in [
            (
                format!("function main(){{const result:int=\"x\".{method}();}}"),
                "type",
            ),
            (
                format!("function main(){{const text=\" x \";const moved=text;text.{method}();}}"),
                "transferred",
            ),
            (
                format!(
                    "type Box={{text:string;}};function main(){{const box:Box={{text:\" x \"}};const moved=box.text;box.text.{method}();}}"
                ),
                "partial-transfer",
            ),
            (
                format!(
                    "function take(value:string):string{{return value;}}function main(){{const text=\" x \".toLowerCase();const action=():string=>{{return text.{method}();}};const moved=take(text);console.log(action());}}"
                ),
                "capture-conflict",
            ),
        ] {
            assert_eq!(
                check(&source)
                    .err()
                    .unwrap_or_else(|| panic!("Expected {code}: {source}"))
                    .code,
                code,
                "{source}"
            );
        }
    }
    native(
        "function take(value:string):string{return value;}function main(){const value=\" x \".toUpperCase();const result=value.trimStart();const moved=take(value);console.log(result.trimEnd());console.log(moved.trimLeft().trimRight());}",
        "X\nX\n",
        "",
        "",
    );
}

#[test]
fn borrowed_results_allocate_nothing_and_owned_results_dispose_normally_and_on_unwind() {
    let source = METHODS.to_owned() + "function main(){}";
    native(
        &source,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for raw in [""," \t\u{85}"," \u{85}é😀\0 e\u{301} \u{3000}","unchanged"]{
            let borrowed=String::Borrowed(raw);let count=ALLOCATIONS.load(Ordering::SeqCst);let results=[function0(&borrowed),function1(&borrowed),function2(&borrowed),function3(&borrowed)];assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert!(results.iter().all(|value|matches!(value,String::Borrowed(_))));drop(results);
            let owned=String::Owned(raw.to_owned());let input=LIVE.load(Ordering::SeqCst);let count=ALLOCATIONS.load(Ordering::SeqCst);
            let results=[function0(&owned),function1(&owned),function2(&owned),function3(&owned)];assert!(results.iter().all(|value|matches!(value,String::Owned(_))));assert_eq!(ALLOCATIONS.load(Ordering::SeqCst)-count,results.iter().filter(|value|!value.is_empty()).count());
            assert_eq!(LIVE.load(Ordering::SeqCst)-input,results.iter().map(|value|value.len() as isize).sum::<isize>());for result in &results{if !result.is_empty(){assert_ne!(result.as_ptr(),owned.as_ptr());}}drop(results);assert_eq!(LIVE.load(Ordering::SeqCst),input);
            assert!(std::panic::catch_unwind(||{let owner=owned.clone();let results=[function0(&owner),function1(&owner),function2(&owner),function3(&owner)];std::hint::black_box(&results);panic!("dispose source owner and four results");}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),input);
            drop(owned);assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
        }std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
    );
}

// Test-only forwarding allocator; generated implementation stays safe Rust.
const ALLOCATOR: &str = r#"
use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCATIONS:AtomicUsize=AtomicUsize::new(0);struct Observer;
unsafe impl GlobalAlloc for Observer{
unsafe fn alloc(&self,layout:Layout)->*mut u8{let p=unsafe{System.alloc(layout)};if !p.is_null(){LIVE.fetch_add(layout.size() as isize,Ordering::SeqCst);ALLOCATIONS.fetch_add(1,Ordering::SeqCst);}p}
unsafe fn dealloc(&self,p:*mut u8,layout:Layout){LIVE.fetch_sub(layout.size() as isize,Ordering::SeqCst);unsafe{System.dealloc(p,layout)}}
unsafe fn realloc(&self,p:*mut u8,layout:Layout,size:usize)->*mut u8{let q=unsafe{System.realloc(p,layout,size)};if !q.is_null(){LIVE.fetch_add(size as isize-layout.size() as isize,Ordering::SeqCst);ALLOCATIONS.fetch_add(1,Ordering::SeqCst);}q}
}
#[global_allocator]static GLOBAL:Observer=Observer;
"#;
