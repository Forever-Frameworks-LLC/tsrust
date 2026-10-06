mod support;
use tsrust_compiler::{Mode, check};

fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis.emit_rust();
    assert!(!rust.contains("unsafe"));
    let rust = rust.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("string-construction.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("string-construction-{level}"));
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
        let output = support::run(&mut std::process::Command::new(executable), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn construction_matches_unicode_empty_pattern_template_and_split_limit_contracts() {
    native(
        r#"
    function main(){
        console.log("ab".repeat(0));console.log("é😀\u0000".repeat(2));console.log("".repeat(9223372036854775807).length);
        console.log("banana".replace("ana","X"));console.log("banana".replaceAll("ana","X"));console.log("aaaa".replace("aa","X"));console.log("aaaa".replaceAll("aa","X"));
        console.log("abc".replace("b","$$|$&|$`|$'"));console.log("aba".replaceAll("a","[$`][$&][$']"));
        console.log("a".replace("a","$$$&-$x-$$&-$"));console.log("x".replace("x","$1/$12/$<name>/$0"));
        console.log("é".replaceAll("z","$&$`$'$$"));console.log("😀é".replace("","X"));console.log("😀é".replaceAll("","X"));console.log("".replaceAll("","$&$$"));
        console.log("a\u0000a".replaceAll("a","é"));console.log("e\u0301".replaceAll("é","z"));
        const parts="a,,b,".split(",");console.log(parts.length);console.log(parts[0]);console.log(parts[1]);console.log(parts[2]);console.log(parts[3]);
        const limited="a,,b,".split(",",2);console.log(limited.length);console.log(limited[1]);
        const whole="a,b".split();console.log(whole.length);console.log(whole[0]);console.log("".split().length);
        console.log("".split(",").length);console.log("".split("").length);
        const scalars="😀é\u0000".split("");console.log(scalars.length);console.log(scalars[0]);console.log(scalars[1]);console.log(scalars[2]);console.log("e\u0301".split("").length);
        console.log("a,b,c".split(",",0).length);console.log("a,b,c".split(",",-1).length);
        console.log("a,b,c".split(",",4294967296).length);console.log("a,b,c".split(",",4294967297).length);
        console.log("a,b,c".split(",",-4294967296).length);console.log("a,b,c".split(",",-9223372036854775808).length);console.log("a,b,c".split(",",9223372036854775807).length);
        console.log("😀é".split("",1)[0]);console.log("  a,a  ".trim().replaceAll("a","é").toUpperCase());
    }"#,
        "\né😀\0é😀\0\n0\nbXna\nbXna\nXaa\nXX\na$|b|a|cc\n[][a][ba]b[ab][a][]\n$a-$x-$&-$\n$1/$12/$<name>/$0\né\nX😀é\nX😀XéX\n$\né\0é\né\n4\na\n\nb\n\n2\n\n1\na,b\n1\n1\n0\n3\n😀\né\n\0\n2\n0\n3\n0\n1\n0\n0\n3\n😀\nÉ,É\n",
        "",
        "",
    );
}

#[test]
fn operands_run_once_and_owned_results_integrate_with_captures_records_and_arrays() {
    let analysis = native(
        r#"
    type Box={value:string;};
    function receiver():string{console.log("receiver");return "a,b,a";}
    function pattern():string{console.log("pattern");return "a";}
    function replacement():string{console.log("replacement");return "X";}
    function count():int{console.log("count");return 2;}
    function separator():string{console.log("separator");return ",";}
    function limit():int{console.log("limit");return 2;}
    function copy(value:string,needle:string,text:string):string{return value.replaceAll(needle,text);}
    function pieces(value:string):string[]{return value.split(",");}
    function main(){
        console.log(receiver().replaceAll(pattern(),replacement()));console.log(receiver().replace(pattern(),replacement()));
        console.log(receiver().repeat(count()).length);const selected=receiver().split(separator(),limit());console.log(selected.length);console.log(selected[1]);
        const input=" a,b ";const read=():string=>{return input.trim().replaceAll(" ","").repeat(2);};console.log(read());console.log(read());console.log(input);
        console.log(copy(input,"a","x"));const box:Box={value:"left,right"};const word=box.value.split(",")[1];box.value="changed";console.log(word);
        const values=["a,b","x,y"];console.log(values[0].split(",")[1]);
        const rows:string[][]=[pieces("p,q")];const mapped=rows.map((row:string[]):string[]=>{return row.map((part:string):string=>{return part.repeat(1);});});
        mapped[0][0]="changed";console.log(rows[0][0]);console.log(mapped[0][0]);
        const replaced=values.map((value:string):string=>{return value.replaceAll(",",":");});console.log(replaced[1]);console.log(values[1]);
    }"#,
        "receiver\npattern\nreplacement\nX,b,X\nreceiver\npattern\nreplacement\nX,b,a\nreceiver\ncount\n10\nreceiver\nseparator\nlimit\n2\nb\na,ba,b\na,ba,b\n a,b \n x,b \nright\nb\np\nchanged\nx:y\nx,y\n",
        "",
        "",
    );
    for name in ["copy.value", "copy.needle", "copy.text", "pieces.value"] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Read));
    }
}

#[test]
fn outputs_own_independent_buffers_and_negative_or_overflow_repetition_disposes_cleanly() {
    native(
        r#"
    function repeat(value:string,count:int):string{return value.repeat(count);}
    function replace(value:string,needle:string,text:string):string{return value.replace(needle,text);}
    function all(value:string,needle:string,text:string):string{return value.replaceAll(needle,text);}
    function pieces(value:string,separator:string,limit:int):string[]{return value.split(separator,limit);}
    function whole(value:string):string[]{return value.split();}
    function doomed():string{return "temporary".repeat(8).repeat(-1);}
    function bounds(value:string):string{return value.split(",")[99];}
    function main(){}
    "#,
        "",
        r#"
    use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,Ordering};
    static LIVE:AtomicIsize=AtomicIsize::new(0);struct Allocations;
    unsafe impl GlobalAlloc for Allocations{
        unsafe fn alloc(&self,layout:Layout)->*mut u8{let pointer=unsafe{System.alloc(layout)};if !pointer.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);}pointer}
        unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){unsafe{System.dealloc(pointer,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);}
    }
    #[global_allocator]static ALLOCATOR:Allocations=Allocations;
    "#,
        r#"
    let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
    let input=String::Owned("a,b,a".into());let live=LIVE.load(Ordering::SeqCst);let pointer=input.as_ptr();
    let result=function0(&input,1);assert!(matches!(result,String::Owned(_)));assert_eq!(result,input);assert_ne!(result.as_ptr(),pointer);drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let result=function1(&input,&String::Borrowed("z"),&String::Borrowed("$&"));assert!(matches!(result,String::Owned(_)));assert_eq!(result,input);assert_ne!(result.as_ptr(),pointer);drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let result=function2(&input,&String::Borrowed("a"),&String::Borrowed("[$&]"));assert_eq!(result.as_ref(),"[a],b,[a]");drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    assert!(std::panic::catch_unwind(||function5()).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);
    assert!(std::panic::catch_unwind(||function6(&input)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let empty=function3(&input,&String::Borrowed(","),0);assert_eq!(empty.len(),0);assert_eq!(LIVE.load(Ordering::SeqCst),live);drop(empty);
    let pieces=function3(&input,&String::Borrowed(","),-1);assert_eq!(pieces.len(),3);for piece in pieces.iter(){assert!(matches!(piece,String::Owned(_)));assert_ne!(piece.as_ptr(),pointer);}
    let whole=function4(&input);assert_eq!(whole.len(),1);assert_ne!(whole.get(0).unwrap().as_ptr(),pointer);drop(input);assert_eq!(pieces.get(0).unwrap().as_ref(),"a");assert_eq!(whole.get(0).unwrap().as_ref(),"a,b,a");drop(pieces);drop(whole);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    assert_eq!(function0(&String::Borrowed(""),i64::MAX).len(),0);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    for (text,count,expected) in [("x",-1,"nonnegative"),("x",i64::MIN,"nonnegative"),("ab",i64::MAX,"source bytes"),("abc",i64::MAX,"source bytes")]{
        let error=std::panic::catch_unwind(||function0(&String::Borrowed(text),count)).unwrap_err();
        let message=error.downcast_ref::<std::string::String>().map(|message|message.as_str()).or_else(||error.downcast_ref::<&str>().copied()).unwrap();assert!(message.contains(expected));drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    }
    std::panic::set_hook(hook);
    "#,
    );
    // The scalar-string program must not depend on any owning array support.
    let scalar = native(
        "function main(){console.log(\"x\".repeat(2).replaceAll(\"x\",\"y\"));}",
        "yy\n",
        "",
        "",
    )
    .emit_rust();
    assert!(!scalar.contains("enum Array<"));
}

#[test]
fn string_constructors_enforce_types_arity_and_complete_operand_loans() {
    for (source, code) in [
        ("function main(){\"x\".repeat();}", "arity"),
        ("function main(){\"x\".repeat(1,2);}", "arity"),
        ("function main(){\"x\".repeat(1.0);}", "type"),
        ("function main(){(1).repeat(2);}", "type"),
        ("function main(){\"x\".replace(\"x\");}", "arity"),
        (
            "function main(){\"x\".replaceAll(\"x\",\"y\",\"z\");}",
            "arity",
        ),
        ("function main(){\"x\".replace(1,\"y\");}", "type"),
        (
            "function main(){\"x\".replace(\"x\",():string=>{return \"y\";});}",
            "type",
        ),
        ("function main(){\"x\".split(\",\",1,2);}", "arity"),
        ("function main(){\"x\".split(1);}", "type"),
        ("function main(){\"x\".split(\",\",1.0);}", "type"),
        ("function main(){[1].split(\",\");}", "type"),
        (
            "type Box={value:string;};function change(box:Box):string{box.value=\"changed\";return \"x\";}function main(){const box:Box={value:\"x\"};box.value.replace(change(box),\"y\");}",
            "conflicting-access",
        ),
        (
            "type Box={value:string;};function count(box:Box):int{box.value=\"changed\";return 1;}function main(){const box:Box={value:\"x\"};box.value.repeat(count(box));}",
            "conflicting-access",
        ),
        (
            "type Box={value:string;};function limit(box:Box):int{box.value=\"changed\";return 1;}function main(){const box:Box={value:\",\"};\"a,b\".split(box.value,limit(box));}",
            "conflicting-access",
        ),
        (
            "function consume(value:string):string{return value;}function main(){const needle=\"x\";\"x\".replace(needle,consume(needle));}",
            "conflicting-access",
        ),
    ] {
        let error = check(source)
            .err()
            .unwrap_or_else(|| panic!("Expected {code}: {source}"));
        assert_eq!(error.code, code, "{source}: {}", error.message);
    }
}
