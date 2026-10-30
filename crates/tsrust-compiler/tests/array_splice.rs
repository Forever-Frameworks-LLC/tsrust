mod support;
use tsrust_compiler::{Mode, check};

fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("input.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let evidence = std::env::var_os("TSR_GATE90_EVIDENCE_DIR").map(|root| {
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
    let path = directory.0.join("array-splice.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-splice-{level}"));
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
fn splice_defaults_clamps_dense_families_ranks_and_bits_are_exact() {
    native(
        r#"
        type Row={id:int;text:string;};
        function range(a:int[],start:int,count:int):int[]{return a.splice(start,count,-7,-8);}
        function bits(a:number[],start:int,count:int):number[]{return a.splice(start,count,-0.0,0.0/0.0);}
        function main(){
            const a=[1,2,3,4];console.log(a.splice().join());console.log(a.join());console.log(a.splice(-2).join());console.log(a.join());console.log(a.splice(-9,-3,7,8,9).join());console.log(a.join());console.log(a.splice(9223372036854775807,9223372036854775807,6).join());console.log(a.join());console.log(a.splice(-9223372036854775808,9223372036854775807).join());console.log(a.length);
            const flags=[true,false,true];console.log(flags.splice(1,1,false,true).join("|"));console.log(flags.join("|"));
            const words=["é\0".repeat(1),"😀".repeat(1),"tail".repeat(1)];const text="new".repeat(1);const removed=words.splice(1,1,text);console.log(removed.join("|"));console.log(words.join("|"));removed.push("saved");console.log(removed.join("|"));
            const nested:int[][]=[[1,2],[],[3]];console.log(nested.splice(1,1,[4,5],[6]).join("|"));console.log(nested.join("|"));
            const rows:Row[]=[{id:1,text:"a".repeat(1)},{id:2,text:"b".repeat(1)}];const saved=rows.splice(0,1,{id:3,text:"c".repeat(1)});console.log(saved[0].text);console.log(rows[0].text);console.log(rows[1].text);
            const nils=[null,null];console.log(nils.splice(0,1,null,null).length);console.log(nils.join("|"));const absent=[undefined,undefined];console.log(absent.splice(-1).length);console.log(absent.length);
            const emptyStrings:string[]=[].splice();console.log(emptyStrings.length);console.log([].splice(0,0,"s").length);const emptyNested:int[][]=[].splice(0,0,[1]);console.log(emptyNested.length);const row:Row={id:1,text:"x"};console.log([].splice(0,0,row).length);console.log([1,2].splice(0,1).join());
        }
    "#,
        "\n1,2,3,4\n3,4\n1,2\n\n7,8,9,1,2\n\n7,8,9,1,2,6\n7,8,9,1,2,6\n0\nfalse\ntrue|false|true|true\n😀\né\0|new|tail\n😀|saved\n\n1,2|4,5|6|3\na\nc\nb\n1\n||\n1\n1\n0\n0\n0\n0\n1\n",
        "",
        r#"
        for len in [0usize,1,4,5,17]{for start in [i64::MIN,-20,-3,-1,0,1,3,20,i64::MAX]{for count in [i64::MIN,-3,0,1,3,20,i64::MAX]{
            let original=(0..len).map(|i|i as i64+10).collect::<Vec<_>>();let position=if start<0{(len as i128+start as i128).clamp(0,len as i128) as usize}else{(start as i128).min(len as i128) as usize};let deleted=(count as i128).clamp(0,(len-position) as i128) as usize;
            let mut a=NumericArray::from([]);for n in &original{a.push(*n);}let removed=function0(&mut a,start,count);assert_eq!(removed.as_slice(),&original[position..position+deleted]);let want=original[..position].iter().copied().chain([-7,-8]).chain(original[position+deleted..].iter().copied()).collect::<Vec<_>>();assert_eq!(a.as_slice(),want);
            let raw=[0x8000000000000000,0x7ff8000000000042,0,0x7ff0000000000000,0xfff0000000000000];let mut floats=(0..len).map(|i|f64::from_bits(raw[i%5])).collect::<Vec<_>>();let original=floats.iter().map(|v|v.to_bits()).collect::<Vec<_>>();let removed=function1(&mut floats,start,count);assert_eq!(removed.iter().map(|v|v.to_bits()).collect::<Vec<_>>(),original[position..position+deleted]);let want=original[..position].iter().copied().chain([0x8000000000000000,(0.0f64/0.0).to_bits()]).chain(original[position+deleted..].iter().copied()).collect::<Vec<_>>();assert_eq!(floats.iter().map(|v|v.to_bits()).collect::<Vec<_>>(),want);
        }}}
    "#,
    );
}

#[test]
fn splice_argument_effects_indices_zero_reads_and_late_traps_are_ordered() {
    let analysis = native(
        r#"
        type Box={items:int[];note:string;};
        function start(a:int[]):int{console.log("start");a.push(9);return -2;}
        function count(a:int[]):int{console.log("count");a.push(10);return 1;}
        function value(a:int[]):int{console.log("value");a.push(13);return 3;}
        function receiver(trace:int[]):int[]{trace.push(1);console.log("receiver");return [4,5];}
        function arg(trace:int[],n:int):int{trace.push(n);console.log(n);return n;}
        function index(trace:int[]):int{trace.push(1);console.log("index");return 0;}
        function readonly(a:int[]):int{return a.splice().length;}
        function fixed():int{const a=[1,2,3];const b=[9,10];return a.splice().length+a.length+b[0];}
        function fail(a:int[]):int{a.push(99);const empty:int[]=[];return empty.remove(0);}
        function late(a:int[]):int[]{return a.splice(0,1,value(a),fail(a));}
        function bounds(a:int[][],trace:int[]):int[]{return a[-1].splice(arg(trace,1));}
        function main(){
            const a=[1,2];console.log(a.splice(start(a),count(a),value(a)).join());console.log(a.join());
            const trace:int[]=[];console.log(receiver(trace).splice(arg(trace,1),arg(trace,1),arg(trace,7)).join());console.log(trace.join());
            const rows=[[8,9]];console.log(rows[index(trace)].splice(arg(trace,1),arg(trace,1),arg(trace,6)).join());console.log(rows[0].join());console.log(trace.join());
            const owner=["a".repeat(1),"b".repeat(1)];console.log(owner.splice(0,0,owner.remove(1)).length);console.log(owner.join());
            const nullable:Box|null={items:[1,2],note:"kept"};if(nullable!==null){console.log(nullable.items.splice(0,1).join());console.log(nullable.items.length);console.log(nullable.note);}
            const nested=[[1,2]];{const view=nested.at(0);if(view!==undefined){console.log(view.splice().length);console.log(view.length);}}
            console.log(readonly(a));console.log(fixed());const lengths=[a.splice(0,0).length,true?a.splice().length:1];console.log(lengths.join());
        }
    "#,
        "start\ncount\nvalue\n10\n1,2,9,3,13\nreceiver\n1\n1\n7\n5\n1,1,1,7\nindex\n1\n1\n6\n9\n8,6\n1,1,1,7,1,1,1,6\n0\nb,a\n1\n1\nkept\n0\n2\n0\n12\n0,0\nvalue\n",
        "",
        r#"
        std::panic::set_hook(Box::new(|_|{}));let mut a=NumericArray::from([1,2]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function9(&mut a))).is_err());assert_eq!(a.as_slice(),&[1,2,13,99]);let mut rows=Array::from([NumericArray::from([8])]);let mut trace=NumericArray::from([]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function10(&mut rows,&mut trace))).is_err());assert!(trace.as_slice().is_empty());assert_eq!(rows.get(0).unwrap().as_slice(),&[8]);
    "#,
    );
    assert_eq!(analysis.contracts()["readonly.a"], Mode::Read);
    let fixed=check("function fixed():int{const a=[1,2,3];return a.splice().length+a[0];}function main(){console.log(fixed());}").unwrap().emit_rust();
    assert!(fixed.contains(": [i64; 3] = ["));
}

#[test]
fn splice_owners_move_without_clones_and_normal_unwind_disposal_is_complete() {
    native(
        r#"
        type Node={id:int;text:string;children:Node[];};
        function strings(a:string[],v:string):string[]{return a.splice(0,1,v);}
        function nodes(a:Node[],v:Node):Node[]{return a.splice(0,1,v);}
        function fail():string{const a:int[]=[];a.remove(0);return "never";}
        function late(a:string[],v:string):string[]{return a.splice(0,1,v,fail());}
        function main(){const a:int[]=[];a.splice(0,0,1);}
    "#,
        "",
        r#"        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Counts;
        unsafe impl GlobalAlloc for Counts{
            unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}
            unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);}
            unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let p=unsafe{System.realloc(p,l,n)};if !p.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}
        }
        #[global_allocator]static COUNTS:Counts=Counts;
"#,
        r#"
        std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        for len in [0,1,4,5,17]{let mut a=Array::from([]);for _ in 0..len{a.push(String::Owned("oldé😀".into()));}let pointers=a.iter().map(|v|v.as_ptr()).collect::<Vec<_>>();let text=String::Owned("new\0".into());let pointer=text.as_ptr();let allocations=ALLOCS.load(Ordering::SeqCst);let removed=function0(&mut a,text);assert_eq!(removed.len(),usize::from(len>0));if len>0{assert_eq!(removed.get(0).unwrap().as_ptr(),pointers[0]);}assert_eq!(a.get(0).unwrap().as_ptr(),pointer);assert!(a.iter().skip(1).zip(pointers.iter().skip(1)).all(|(v,p)|v.as_ptr()==*p));if len<=4{assert_eq!(ALLOCS.load(Ordering::SeqCst),allocations);}drop(a);drop(removed);drop(pointers);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let leaf=Record0{field0:2,field1:String::Owned("leaf".into()),field2:RecordArray0::from([])};let old=Record0{field0:1,field1:String::Owned("old".into()),field2:RecordArray0::from([leaf])};let old_pointer=old.field1.as_ptr();let new=Record0{field0:3,field1:String::Owned("new".into()),field2:RecordArray0::from([])};let new_pointer=new.field1.as_ptr();let mut nodes=RecordArray0::from([old]);let removed=function1(&mut nodes,new);assert_eq!(removed[0].field1.as_ptr(),old_pointer);assert_eq!(removed[0].field2.len(),1);assert_eq!(nodes[0].field1.as_ptr(),new_pointer);drop(nodes);drop(removed);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let mut a=Array::from([String::Owned("kept".into())]);let pointer=a.get(0).unwrap().as_ptr();let state=LIVE.load(Ordering::SeqCst);let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(&mut a,String::Owned("discarded".into()))));assert!(result.is_err());drop(result);assert_eq!(a.len(),1);assert_eq!(a.get(0).unwrap().as_ptr(),pointer);assert_eq!(LIVE.load(Ordering::SeqCst),state);drop(a);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        if usize::BITS==64{let len=i64::MAX as usize+1;let mut too_many=vec![();len];let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||too_many.tsr_splice(0,None,[],"removed limit")));assert!(result.is_err());drop(result);assert_eq!(too_many.len(),len);drop(too_many);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);for len in [i64::MAX as usize,usize::MAX]{let mut a=vec![();len];let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||a.tsr_splice(0,Some(0),[()],"limit")));assert!(result.is_err());drop(result);assert_eq!(a.len(),len);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);drop(a);}}
        let mut a=Array::from([String::Owned("a".into()),String::Owned("b".into()),String::Owned("c".into()),String::Owned("d".into())]);let pointers=a.iter().map(|v|v.as_ptr()).collect::<Vec<_>>();let removed=a.tsr_splice(1,Some(1),[String::Owned("x".into()),String::Owned("y".into()),String::Owned("z".into())],"limit");assert_eq!(a.len(),6);assert_eq!(a.get(0).unwrap().as_ptr(),pointers[0]);assert_eq!(a.get(4).unwrap().as_ptr(),pointers[2]);assert_eq!(a.get(5).unwrap().as_ptr(),pointers[3]);assert_eq!(removed.get(0).unwrap().as_ptr(),pointers[1]);drop(a);drop(removed);drop(pointers);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    "#,
    );
}

#[test]
fn splice_strict_contracts_incoming_loans_views_and_normal_depth_are_preserved() {
    for (source, code) in [
        ("function main(){const a=[1];a.splice(true);}", "type"),
        (
            "function main(){const a=[1];a.splice(0,undefined);}",
            "type",
        ),
        ("function main(){const a=[1];a.splice(0,1,\"a\");}", "type"),
        (
            "function main(){const a:number[]=[1.0];let n:int=1;a.splice(0,1,n);}",
            "type",
        ),
        (
            "type A={n:int;};type B={n:int;};function main(){const a:A[]=[];const b:B={n:1};a.splice(0,0,b);}",
            "type",
        ),
        ("function main(){const a=\"s\";a.splice();}", "type"),
        (
            "function main(){const a=[\"a\"];a.splice(0,0,a[0]);}",
            "borrowed-element",
        ),
        (
            "function main(){const a=[\"a\"];const v=\"v\".repeat(1);a.splice(0,0,v);console.log(v);}",
            "transferred",
        ),
        (
            "function main(){const a=[[1]];const v=a.at(0);if(v!==undefined){v.splice(0);}}",
            "borrowed-view",
        ),
        (
            "function main(){const a=[\"a\"];const v=a.at(0);a.splice(0);console.log(String(v));}",
            "view-conflict",
        ),
        (
            "function read(a:int[],n:int):int{return a.length+n;}function main(){const a=[1];read(a,a.splice(0).length);}",
            "conflicting-access",
        ),
        (
            "function grow(a:int[][]):int{a.push([1]);return 1;}function main(){const a=[[1]];a[0].splice(grow(a));}",
            "conflicting-access",
        ),
        (
            "function take(a:int[]):int{a;return 0;}function main(){const a=[1];a.splice(take(a));}",
            "transferred",
        ),
    ] {
        let error = check(source).err().expect("invalid splice must reject");
        assert_eq!(error.code, code, "{}", error.render("input.tsr", source));
    }
    for count in [100, 200] {
        let source = format!(
            "function main(){{const a={};a.splice();}}",
            "[".repeat(count) + "1" + &"]".repeat(count)
        );
        if count == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            assert_eq!(check(&source).err().unwrap().code, "unsupported");
        }
        let source = format!(
            "function main(){{const a=[1];a[{}];}}",
            "a[".repeat(count) + "0" + &"]".repeat(count)
        );
        if count == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            support::accept_deep(&source);
        }
    }
    assert_eq!(
        check("function main(){const invalid=[];}")
            .err()
            .unwrap()
            .code,
        "array-type"
    );
}
