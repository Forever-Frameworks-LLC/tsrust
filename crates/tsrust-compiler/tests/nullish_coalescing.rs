mod support;
use tsrust_compiler::check;

fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("input.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let assertions = if assertions.contains("__NUMBER_FUNCTION__") {
        let functions = emitted
            .lines()
            .filter(|line| {
                line.starts_with("fn function") && line.contains("(value0: f64) -> Vec<f64>")
            })
            .collect::<Vec<_>>();
        assert_eq!(functions.len(), 1, "unique Number constructor witness");
        let name = functions[0]
            .split_once('(')
            .unwrap()
            .0
            .strip_prefix("fn ")
            .unwrap();
        assertions.replace("__NUMBER_FUNCTION__", name)
    } else {
        assertions.to_owned()
    };
    let rust = emitted.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let evidence = std::env::var_os("TSR_GATE94_EVIDENCE_DIR").map(|root| {
        std::path::PathBuf::from(root).join(format!(
            "case-{}",
            CASE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ))
    });
    if let Some(path) = &evidence {
        std::fs::create_dir_all(path).unwrap();
        std::fs::write(path.join("input.tsr"), source).unwrap();
        std::fs::write(path.join("expected.txt"), expected).unwrap();
        std::fs::write(path.join("generated.rs"), &rust).unwrap();
    }
    let directory = support::TempDirectory::new();
    let path = directory.0.join("nullish-coalescing.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("nullish-coalescing-{level}"));
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
        let output = support::run(&mut std::process::Command::new(executable), 15);
        if let Some(path) = &evidence {
            std::fs::write(path.join(format!("stdout-O{level}.txt")), &output.stdout).unwrap();
            std::fs::write(path.join(format!("stderr-O{level}.txt")), &output.stderr).unwrap();
        }
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn nullable_results_contexts_and_exact_scalar_bits_are_preserved() {
    let source = format!(
        "function f(flag:boolean):int{}{{const x=flag?Array.of<int{}>():null;return x??Array.of<int{}>();}}function main(){{console.log(f(true).length,f(false).length);}}",
        "[]".repeat(128),
        "[]".repeat(127),
        "[]".repeat(127)
    );
    native(&source, "0 0\n", "", "");
    native(
        r#"type Row={text:string;};type Holder={item:Row|null;};function refill(holder:Holder):Row{holder.item={text:"replaced"};return {text:"returned"};}function choose(holder:Holder):Row{return holder.item??refill(holder);}function length(x:string):int{return (x??"fallback").length;}function main(){console.log(choose({item:null}).text,choose({item:{text:"present"}}).text);const text="alive".repeat(1);console.log(length(text),text);}"#,
        "returned present\n5 alive\n",
        "",
        "",
    );
    native(
        "function negative(x:number|null):number{return x??-5;}function certain():number{return -5??0;}function main(){console.log(negative(null),negative(2),certain());}",
        "-5 2 -5\n",
        "",
        "",
    );
    for expression in [
        "null??((value:int):int=>{return value+1;})",
        "(null)??((value:int):int=>{return value+1;})",
    ] {
        let source = format!("function main(){{const f={expression};console.log(f(1));}}");
        let facts = check(&source).unwrap().checked_type_facts();
        let start = source.find(expression).unwrap();
        let end = start + expression.len();
        assert!(
            facts
                .method_results
                .iter()
                .any(|fact| fact.span.start == start && fact.span.end == end),
            "{facts:?}"
        );
    }
    native(
        r#"function n(x:number|null,flag:boolean):number{return x??(flag?1:2);}function a(x:number[]|null,flag:boolean):number[]{return x??(flag?[1]:[2]);}function residual(x:number|null,flag:boolean):number|null{return x??(flag?1:null);}function main(){console.log(n(null,true),n(null,false),n(3,false));console.log(a(null,true).join(),a(null,false).join());console.log(String(residual(null,true)),String(residual(null,false)),String(residual(4,false)));}"#,
        "1 2 3\n1 2\n1 null 4\n",
        "",
        "",
    );
    native(
        r#"type Holder={item:string|null;};function takeNull(x:null):string{return "null";}function takeUndefined(x:undefined):string{return "undefined";}function takeBoth(x:null|undefined):string{return String(x);}function pick(x:string|null):string{return x??takeNull(x);}function missing(x:string|undefined):string{return x??takeUndefined(x);}function residual(x:string|null|undefined):string{if(x!==null){return x??takeUndefined(x);}return "null";}function field(x:Holder):string{return x.item??takeNull(x.item);}function either(x:string|null|undefined):string{return x??takeBoth(x);}function main(){console.log(pick(null),pick("present".repeat(1)),missing(undefined));console.log(residual(undefined),residual("live".repeat(1)),residual(null));console.log(field({item:null}),either(undefined),either(null));}"#,
        "null present undefined\nundefined live null\nnull undefined null\n",
        "",
        "",
    );
    for source in [
        "function take(x:null):string{return \"nil\";}function read(x:string):int{return x.length;}function main(){const x:string|null=\"present\";const selected=read(x??take(x));take(x);}",
        "type Holder={item:string|null;};function change(x:Holder):boolean{x.item=\"changed\";return true;}function take(x:null):string{return \"nil\";}function pick(x:Holder):string{return x.item??(change(x)?take(x.item):\"fallback\");}function main(){}",
    ] {
        let error = check(source).err().expect(source);
        assert_eq!(error.code, "type", "{}", error.message);
    }
    if std::env::var_os("TSR_GATE94_EDGES_ONLY").is_some() {
        return;
    }
    let analysis = native(
        r#"
      type Row={id:int;text:string;};type Box<T>={value:T;};type Node={name:string;children:Node[];};
      function number(x:number|null):number{return x??-0;}
      function contextual(x:number|null):number{return x??5;}
      function fallback(x:int|null|undefined,y:int|null):int|null{return x??y;}
      function text(x:string|null):string{return x??"fallback".repeat(1);}
      function array(x:int[]|undefined):int[]{return x??[7,8];}
      function row(x:Row|null):Row{return x??{id:2,text:"fallback"};}
      function boxed(x:Box<int>|null):Box<int>{return x??{value:9};}
      function node(x:Node|null):Node{return x??{name:"root",children:[]};}
      function generic<T>(x:T):T{return x??undefined;}
      function main(){
        console.log(contextual(null),contextual(2));
        console.log(String(fallback(null,4)),String(fallback(undefined,null)),String(fallback(0,9)));
        console.log(false??true,0??1,""??"fallback",Number.NaN??1);
        console.log(1/number(null),1/number(-0),number(Number.POSITIVE_INFINITY));
        console.log(text(null),text("".repeat(1)),array(undefined).join("|"),array([3]).join());
        console.log(row(null).text,boxed(null).value,node(null).children.length);
        const a:number[]|null=null;console.log((a??[1,-0]).length);
        const b:boolean[]|null=null;console.log((b??[true,false]).join());
        const c:string[]|null=null;console.log((c??["é","😀"]).join("|"));
        const d:null[]|null=null;console.log((d??[null]).length);
        const e:undefined[]|null=null;console.log((e??[undefined]).length);
        const f:int[][]|null=null;console.log((f??[[1],[]]).join("|"));
        const g:Row[]|null=null;console.log((g??[{id:8,text:"eight"}])[0].id);
        console.log(generic<int>(7),generic<string>("hello".repeat(1)));
      }
    "#,
        "5 2\n4 null 0\nfalse 0  NaN\n-Infinity -Infinity Infinity\nfallback  7|8 3\nfallback 9 0\n2\ntrue,false\né|😀\n1\n1\n1|\n8\n7 hello\n",
        "",
        "",
    );
    assert_eq!(analysis.checked_type_facts(), analysis.checked_type_facts());
    assert!(
        analysis
            .checked_type_facts()
            .method_results
            .iter()
            .any(|x| x.ty == "number")
    );
    native(
        "function exact(x:number|null):number{return x??-0;}function main(){}",
        "",
        "",
        r#"
      for bits in [0u64,0x8000000000000000,0x7ff8000000000042,0x7ff0000000000000,0xfff0000000000000] {assert_eq!(function0(Union0::V0(f64::from_bits(bits))).to_bits(),bits);}
      assert_eq!(function0(Union0::V1(TsrNull)).to_bits(),0x8000000000000000);
    "#,
    );
}

#[test]
fn nullish_evaluation_is_lazy_once_ordered_and_static_branches_preserve_contracts() {
    native(
        r#"
      type Trace={calls:int;};
      function nil(trace:int[]):null{trace.push(1);return null;}
      function left(trace:int[],x:int|null|undefined):int|null|undefined{trace.push(2);return x;}
      function right(trace:int[],x:int):int{trace.push(x);return x;}
      function consume(text:string):int{text;return 4;}
      function unreachable(n:int,text:string):int{return n??consume(text);}
      function count(trace:Trace):int{trace.calls+=1;return 9;}
      function one():int{return 7;}
      function make(text:string):()=>string{return ():string=>{return text;};}
      function call(action:Once<Function<int>>):int{return (action??0)();}
      function owned(action:Once<Function<int>>):int{return (null??action)();}
      function main(){
        const trace:int[]=[];console.log(left(trace,0)??right(trace,3),trace.join());
        console.log(left(trace,null)??right(trace,4),trace.join());
        console.log(nil(trace)??right(trace,5),trace.join());
        const t:Trace={calls:0};let x:int|null=1;console.log(x??count(t),t.calls);x=null;console.log(x??count(t),t.calls);
        const text="live".repeat(1);console.log(unreachable(1,text),text);
        let n=0;const a:int|null=null;const b:int|null=null;console.log(a??b??++n,n);
        console.log((null??true)||false,true&&(null??true));
        console.log((null??one)(),(one??9)(),call(one),owned(one));
        const f=make("captured".repeat(1));console.log((null??f)());
        const values=[1];console.log((0??values[9]),(false??(values[9]>0)));
      }
    "#,
        "0 2\n4 2,2,4\n5 2,2,4,1,5\n1 0\n9 1\n1 live\n1 1\ntrue true\n7 7 7 7\ncaptured\n0 false\n",
        "",
        "",
    );
    for source in [
        "function bad(action:Once<Function<int>>):int{const x=(action??0)();return action();}function main(){}",
        "function bad(action:Once<Function<int>>):int{const x=(null??action)();return action();}function main(){}",
        "function main(){const text=\"owned\".repeat(1);const f=():string=>{return text;};(f??0)();f();}",
    ] {
        assert!(check(source).is_err(), "{source}");
    }
}

#[test]
fn nullish_owners_views_and_unwind_disposal_use_standard_moves() {
    for (prefix, conflict) in [
        (
            "function use(x:string,n:int):int{return x.length+n;}function take(x:string):int{x;return 0;}function main(){const x:string|null=\"x\".repeat(1);const y=\"y\".repeat(1);use(x??y,0);}",
            "take(y)",
        ),
        (
            "function grow(values:string[]):int{values.push(\"z\");return 0;}function use(x:string,n:int):int{return x.length+n;}function main(){const values=[\"x\".repeat(1)];const fallback=\"fallback\".repeat(1);use(values.at(0)??fallback,0);}",
            "grow(values)",
        ),
    ] {
        check(prefix)
            .unwrap_or_else(|e| panic!("{}", e.render("prefix.tsr", prefix)))
            .emit_rust();
        let source = prefix.replace(",0);}", &format!(",{conflict});}}"));
        let error = check(&source).err().expect(&source);
        assert_eq!(error.code, "conflicting-access", "{}", error.message);
    }
    if std::env::var_os("TSR_GATE94_LOAN_ONLY").is_some() {
        return;
    }
    let analysis = native(
        r#"
      function choose(x:string|null):string{return x??"fallback".repeat(1);}
      function fail(x:string|null,values:int[]):string{return x??String(values[9]);}
      function read(x:string|null,y:string):int{return (x??y).length;}
      function inspect(x:string|null|undefined):int{return String(x).length;}
      function main(){
        const rows=["one".repeat(1),"two".repeat(1)];const selected=rows.at(0);const view=selected??rows.at(1);if(view!==undefined){console.log(view.length,rows.join("|"));}
        const copy=clone(rows.at(0))??"absent".repeat(1);console.log(copy,rows.join("|"));
        const text:string|null="live".repeat(1);console.log(read(text,"other"),String(text));
        const missing:string|null=null;console.log(read(missing,"é"));
        const values:int[]|null=[1,2];console.log((values??[3]).join(),String(values));
        console.log(choose(null),choose("present".repeat(1)));
      }
    "#,
        "3 one|two\none one|two\n4 live\n2\n1,2 1,2\nfallback present\n",
        r#"
      use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
      static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Counts;
      unsafe impl GlobalAlloc for Counts{unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);}unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let p=unsafe{System.realloc(p,l,n)};if !p.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}}
      #[global_allocator]static COUNTS:Counts=Counts;
    "#,
        r#"
      std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
      let text=String::Owned("owner".into());let pointer=text.as_ptr();ALLOCS.store(0,Ordering::SeqCst);let out=function0(Union0::V0(text));assert_eq!(out.as_ptr(),pointer);assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(out);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
      let values=NumericArray::from([1]);let failure=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function1(Union0::V1(TsrNull),&values)));assert!(failure.is_err());drop(failure);drop(values);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    "#,
    );
    assert_eq!(analysis.contracts()["read.x"], tsrust_compiler::Mode::Read);
    for source in [
        "function main(){const text:string|null=\"x\".repeat(1);const chosen=text??\"fallback\";console.log(text);}",
        "function main(){const values=[\"x\".repeat(1)];const chosen=values.at(0)??\"fresh\".repeat(1);}",
    ] {
        assert!(check(source).is_err(), "{source}");
    }
}

#[test]
fn nullish_parser_type_borrow_and_normal_depth_diagnostics_are_precise() {
    for (source, code) in [
        ("function main(){true??false||true;}", "syntax"),
        ("function main(){true||false??true;}", "syntax"),
        ("function main(){true??false&&true;}", "syntax"),
        ("function main(){true&&false??true;}", "syntax"),
        ("function main(){let x=1;(true?x:x)??=2;}", "unsupported"),
        ("type Item={n:int};function main(){const x:Item={n:1};(x?.n)=2;}", "syntax"),
        (
            "function f(x:int|null):number{return x??5;}function main(){}",
            "type",
        ),
        (
            "function f(x:int|null):string{return x??\"x\";}function main(){}",
            "type",
        ),
        (
            "function f(x:int|null):int{return x??true;}function main(){}",
            "type",
        ),
        ("function main(){missing??0;}", "name"),
        ("function main(){0??missing;}", "name"),
    ] {
        let error = check(source).err().expect(source);
        assert_eq!(error.code, code, "{source}:{}", error.message);
        assert!(error.span.end > error.span.start);
    }
    for expression in [
        "(true??false)||true",
        "true??(false||true)",
        "(true||false)??true",
        "true||(false??true)",
        "true??(false??true)",
        "(true??false)??true",
    ] {
        check(&format!("function main(){{{expression};}}"))
            .unwrap()
            .emit_rust();
    }
    let accepted = format!(
        "function main(){{const a=[0];console.log({}0{});}}",
        "a[".repeat(100),
        "]".repeat(100)
    );
    check(&accepted).unwrap().emit_rust();
    let rejected = format!(
        "function main(){{const a=[0];console.log({}0{});}}",
        "a[".repeat(200),
        "]".repeat(200)
    );
    assert_eq!(check(&rejected).err().unwrap().code, "unsupported");
    let accepted_chain = format!("function main(){{console.log({}1);}}", "null??".repeat(100));
    check(&accepted_chain).unwrap().emit_rust();
    let chain = format!("function main(){{console.log({}1);}}", "null??".repeat(200));
    assert_eq!(check(&chain).err().unwrap().code, "unsupported");
    let source = format!(
        "function f(x:int{}|null):int{}{{return x??Array.of<int{}>();}}function main(){{}}",
        "[]".repeat(127),
        "[]".repeat(127),
        "[]".repeat(126)
    );
    native(&source, "", "", "");
    let source = format!(
        "function f(x:int{}):int{}{{return x??Array.of<int{}>();}}function main(){{}}",
        "[]".repeat(128),
        "[]".repeat(128),
        "[]".repeat(127)
    );
    native(&source, "", "", "");
}

#[test]
fn optional_call_results_take_a_fallback_under_an_expected_type() {
    support::execute("type H={run:ReadOnly<Function<int>>;};function pick(h:ReadOnly<H>|undefined):int{return 0;}function main():void{const h:H|undefined={run:():int=>3};const n:int=h?.run()??-1;const m=h?.run()??-1;console.log(n,m);}", "3 3\n", None);
}

#[test]
fn literal_fallback_preserves_existing_optional_union_members(){
 let source="type Direction='horizontal'|'vertical';function direction(value:Direction|undefined):Direction{const selected=value??'horizontal';return selected;}function flag(value:true|null):true{const selected=value??true;return selected;}function main(){console.log(direction(undefined),direction('vertical'),flag(null),flag(true));}";
 native(source,"horizontal vertical true true\n","","");
 for fallback in ["'diagonal'","String('horizontal')"] {
  let source=format!("type Direction='horizontal'|'vertical';function direction(value:Direction|undefined):Direction{{const selected=value??{fallback};return selected;}}function main(){{}}");assert!(check(&source).is_err(),"{fallback}");
 }
}
