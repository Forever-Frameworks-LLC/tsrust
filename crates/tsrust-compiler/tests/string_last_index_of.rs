mod support;
use tsrust_compiler::{Mode, check};

fn native(
    source: &str,
    expected: &str,
    body: &str,
    globals: &str,
    failure: Option<&str>,
) -> tsrust_compiler::Analysis {
    let analysis = check(source)
        .unwrap_or_else(|error| panic!("{}", error.render("string-last-index.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("string-last-index.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("string-last-index-{level}"));
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
        assert_eq!(output.stdout, expected.as_bytes());
        if let Some(message) = failure {
            assert!(!output.status.success());
            assert!(String::from_utf8_lossy(&output.stderr).contains(message));
        } else {
            support::success(&output);
            assert!(output.stderr.is_empty());
        }
    }
    analysis
}

#[test]
fn forward_boundary_oracle_preserves_overlaps_empty_needles_extremes_and_complete_matches() {
    let analysis = native(
        r#"
        function bounded(text:string,needle:string,position:int):int{return text.lastIndexOf(needle,position);}
        function defaultPosition(text:string,needle:string):int{return text.lastIndexOf(needle);}
        function main(){
            console.log("ababa".lastIndexOf("aba"));console.log("ababa".lastIndexOf("aba",1));console.log("ababa".lastIndexOf("aba",2));console.log("aaaa".lastIndexOf("aa",1));
            const text="Aé😀\u0000Aé";console.log(text.lastIndexOf("é"));console.log(text.lastIndexOf("é",1));console.log(text.lastIndexOf(""));console.log(text.lastIndexOf("",-1));
            console.log("".lastIndexOf(""));console.log("".lastIndexOf("x"));console.log("é".lastIndexOf("e\u0301"));console.log("e\u0301".lastIndexOf("\u0301",1));
            console.log("aé".lastIndexOf("a",1));console.log("aba😀".lastIndexOf("aba",3));
            console.log("ababa".lastIndexOf("aba",-9223372036854775808));console.log("ababa".lastIndexOf("aba",9223372036854775807));
        }
    "#,
        "2\n0\n2\n1\n9\n1\n11\n0\n0\n-1\n-1\n1\n0\n0\n0\n2\n",
        r#"
        // Independent ascending scan retains every eligible match, including overlaps.
        fn expected(text:&str,needle:&str,position:Option<i64>)->i64{
            let bound=position.map_or(text.len() as i128,|value|(value as i128).clamp(0,text.len() as i128));let mut found=-1;
            for index in 0..=text.len(){if index as i128<=bound&&text.is_char_boundary(index)&&text[index..].starts_with(needle){found=index as i64;}}found
        }
        for text in ["","a","ababa","aaaaa","Aé😀\0Aé","e\u{301}","é","aé","aba😀","😀a😀a","ééé"]{
            let mut positions=vec![i64::MIN,-3,-1,i64::MAX,text.len() as i64,text.len() as i64+1];positions.extend(text.char_indices().map(|(byte,_)|byte as i64));
            for owned in [false,true]{let receiver=if owned{String::Owned(text.to_owned())}else{String::Borrowed(text)};
                for needle in ["","a","aba","aa","é","😀","\0","Aé","é😀","e\u{301}","\u{301}","missing longer pattern"]{
                    let needle=String::Owned(needle.to_owned());assert_eq!(function1(&receiver,&needle),expected(text,needle.as_ref(),None));
                    for &position in &positions{assert_eq!(function0(&receiver,&needle,position),expected(text,needle.as_ref(),Some(position)),"text={text:?},needle={needle:?},position={position}");}
                    assert_eq!(receiver.as_ref(),text);
                }
            }
        }
    "#,
        "",
        None,
    );
    for name in [
        "bounded.text",
        "bounded.needle",
        "defaultPosition.text",
        "defaultPosition.needle",
    ] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Read));
    }
}

#[test]
fn selected_boundaries_fail_after_operands_and_selection_failures_precede_later_arguments() {
    native(
        r#"function bounded(text:string,needle:string,position:int):int{return text.lastIndexOf(needle,position);}function main(){}"#,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        for text in ["é","😀","aé😀e\u{301}"]{for needle in ["","absent","é"]{
            let receiver=String::Owned(text.to_owned());let needle=String::Borrowed(needle);
            for position in 0..=text.len(){if !text.is_char_boundary(position){assert!(std::panic::catch_unwind(||function0(&receiver,&needle,position as i64)).is_err());}}
        }}std::panic::set_hook(hook);
    "#,
        "",
        None,
    );
    native(
        r#"function receiver():string{console.log(1);return "😀";}function needle():string{console.log(2);return "";}function position():int{console.log(3);return 1;}function main(){console.log(receiver().lastIndexOf(needle(),position()));}"#,
        "1\n2\n3\n",
        "",
        "",
        Some("string search position is not on a UTF-8 boundary"),
    );
    native(
        r#"function receiver():string[]{console.log(1);return ["text"];}function mark(tag:int,value:int):int{console.log(tag);return value;}function needle():string{console.log(3);return "";}function main(){console.log(receiver()[mark(2,9)].lastIndexOf(needle(),mark(4,0)));}"#,
        "1\n2\n",
        "",
        "",
        Some("array index out of bounds at source bytes"),
    );
}

#[test]
fn shared_search_contracts_keep_owner_loans_effect_order_captures_callbacks_and_array_overloads() {
    let analysis = native(
        r#"
        type Box={text:string;other:string;};
        function receiver(box:Box):string{console.log("receiver");return clone(box.text);}
        function needle():string{console.log("needle");return "aba";}
        function position(box:Box):int{console.log("position");box.text="changed";return 2;}
        function invoke(f:(text:ReadOnly<string>,needle:ReadOnly<string>)=>int,text:string,needle:string):int{return f(text,needle);}
        function factory(text:string):()=>int{return ():int=>{return text.lastIndexOf("a");};}
        function read(box:Box,needle:string):int{return box.text.lastIndexOf(needle);}
        function take(text:string):string{return text;}
        function main(){
            const box:Box={text:"ababa",other:"saved"};console.log(receiver(box).lastIndexOf(needle(),position(box)));console.log(box.text);
            const text="é😀é";const pattern="é";const reusable=():int=>{return text.lastIndexOf(pattern);};console.log(reusable());console.log(reusable());
            console.log(invoke((value:string,searchNeedle:string):int=>{return value.lastIndexOf(searchNeedle);},text,pattern));const retained=factory("ababa".toUpperCase());console.log(retained());
            const words=["é😀é","ababa"];console.log(words[0].lastIndexOf(pattern));for(const word of words){console.log(word.lastIndexOf("a"));}
            const other:Box={text:"ababa",other:"saved"};const moved=other.other;console.log(other.text.lastIndexOf("aba"));other.other=moved;console.log(other.other);
            const available="owned".toUpperCase();console.log(available.lastIndexOf("N"));console.log(take(available));
            const empty="";console.log(empty.lastIndexOf(needle(),position(box)));console.log([1,2,1].lastIndexOf(1));
        }
    "#,
        "receiver\nneedle\nposition\n2\nchanged\n6\n6\n6\n-1\n6\n-1\n4\n2\nsaved\n2\nOWNED\nneedle\nposition\n-1\n2\n",
        "",
        "",
        None,
    );
    assert_eq!(analysis.contracts().get("read.box"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("read.needle"), Some(&Mode::Read));
    for (source, code) in [
        ("function main(){\"x\".lastIndexOf();}", "arity"),
        ("function main(){\"x\".lastIndexOf(\"x\",0,1);}", "arity"),
        ("function main(){\"x\".lastIndexOf(1);}", "type"),
        ("function main(){\"x\".lastIndexOf(\"x\",true);}", "type"),
        ("function main(){\"x\".lastIndexOf(\"x\",0.0);}", "type"),
        (
            "function main(){const result:boolean=\"x\".lastIndexOf(\"x\");}",
            "type",
        ),
        (
            "function main(){const text=\"x\";const moved=text;text.lastIndexOf(\"x\");}",
            "transferred",
        ),
        (
            "type Box={text:string;};function main(){const box:Box={text:\"x\"};const moved=box.text;box.text.lastIndexOf(\"x\");}",
            "partial-transfer",
        ),
        (
            "type Box={text:string;};function change(box:Box):void{box.text=\"y\";}function main(){const box:Box={text:\"x\"};const read=():int=>{return box.text.lastIndexOf(\"x\");};change(box);console.log(read());}",
            "capture-conflict",
        ),
    ] {
        assert_eq!(
            check(source)
                .err()
                .unwrap_or_else(|| panic!("Expected {code}: {source}"))
                .code,
            code,
            "{source}"
        );
    }
    let declarations = "type Box={text:string;other:string;};function change(box:Box):string{box.text=\"changed\";return \"changed\";}function position(box:Box):int{box.other=\"changed\";return 0;}function take(text:string):string{return text;}";
    for expression in [
        "box.text.lastIndexOf(change(box))",
        "box.text.lastIndexOf(\"x\",position(box))",
        "\"receiver\".lastIndexOf(box.text,position(box))",
        "text.lastIndexOf(take(text))",
    ] {
        let source = format!(
            "{declarations}function main(){{const box:Box={{text:\"x\",other:\"saved\"}};const text=\"x\";console.log({expression});}}"
        );
        let error = check(&source).err().unwrap();
        assert_eq!(error.code, "conflicting-access");
        assert!(!error.related.is_empty());
    }
}

#[test]
fn native_search_allocates_nothing_and_owned_inputs_dispose_normally_and_on_unwind() {
    native(
        r#"function query(text:string,needle:string,position:int):int{return text.lastIndexOf(needle,position);}function main(){}"#,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for count in [1,4,17,128]{
            let text=String::Owned("abaé😀\0e\u{301}".repeat(count));let needle=String::Owned("aba".to_owned());let input=LIVE.load(Ordering::SeqCst);let pointer=text.as_ptr();let needle_pointer=needle.as_ptr();let allocations=ALLOCATIONS.load(Ordering::SeqCst);
            assert_eq!(function0(&text,&needle,i64::MAX),((count-1)*13) as i64);assert_eq!(function0(&text,&needle,i64::MIN),0);assert_eq!(function0(&text,&needle,3),0);assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),allocations,"search cloned or allocated");assert_eq!(LIVE.load(Ordering::SeqCst),input);assert_eq!(text.as_ptr(),pointer);assert_eq!(needle.as_ptr(),needle_pointer);
            assert!(std::panic::catch_unwind(||{let text=text.clone();let needle=needle.clone();std::hint::black_box(function0(&text,&needle,i64::MAX));panic!("dispose search owners");}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),input);
            assert!(std::panic::catch_unwind(||{let text=String::Owned("😀".repeat(count));let needle=String::Owned("".to_owned());function0(&text,&needle,1);}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),input);
            drop(text);drop(needle);assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
        }std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
        None,
    );
}

// Test-only forwarding allocator; emitted implementation uses only safe Rust.
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
