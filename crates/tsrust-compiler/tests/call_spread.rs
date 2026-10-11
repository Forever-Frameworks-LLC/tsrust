mod support;
use tsrust_compiler::check;

fn recorded(
    directory: &std::path::Path,
    label: &str,
    command: &mut std::process::Command,
    timeout: u64,
) -> std::process::Output {
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
    let stamp = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    };
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|value| format!("{:?}", value.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(",");
    let start = stamp();
    let elapsed = Instant::now();
    let stdout = directory.join(format!("{label}.stdout"));
    let stderr = directory.join(format!("{label}.stderr"));
    command
        .stdout(std::fs::File::create(&stdout).unwrap())
        .stderr(std::fs::File::create(&stderr).unwrap());
    let mut child = command.spawn().unwrap();
    let pid = child.id();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if elapsed.elapsed() > Duration::from_secs(timeout) {
            child.kill().unwrap();
            panic!("native child timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    std::fs::write(directory.join(format!("{label}.json")),format!("{{\"argv\":[{argv}],\"pid\":{pid},\"startUtcEpochMillis\":{start},\"endUtcEpochMillis\":{},\"exitCode\":{}}}\n",stamp(),status.code().map_or("null".into(),|value|value.to_string()))).unwrap();
    std::process::Output {
        status,
        stdout: std::fs::read(stdout).unwrap(),
        stderr: std::fs::read(stderr).unwrap(),
    }
}

fn native(source: &str, expected: &str) {
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let persistent = std::env::var_os("TSR_GATE100_NATIVE_DIR");
    let temporary = persistent.is_none().then(support::TempDirectory::new);
    let directory = if let Some(root) = persistent {
        let root = std::path::PathBuf::from(root);
        let case = CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if case == 0 {
            root
        } else {
            root.join(format!("case-{case}"))
        }
    } else {
        temporary.as_ref().unwrap().0.clone()
    };
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("input.tsr"), source).unwrap();
    std::fs::write(directory.join("expected.txt"), expected).unwrap();
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis.emit_rust();
    std::fs::write(directory.join("generated.rs"), &rust).unwrap();
    assert!(!rust.contains("unsafe"));
    if std::env::var_os("TSR_GATE100_EMIT_ONLY").is_some() {
        return;
    }
    let levels=std::env::var("TSR_GATE100_NATIVE_LEVELS").unwrap_or_else(|_|"0,3".into());
    for level in levels.split(',') {
        let path = directory.join(format!("native-O{level}"));
        let build = recorded(
            &directory,
            &format!("build-O{level}"),
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(directory.join("generated.rs"))
                .arg("-o")
                .arg(&path),
            60,
        );
        std::fs::write(
            directory.join(format!("build-O{level}-stdout.txt")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-O{level}-stderr.txt")),
            &build.stderr,
        )
        .unwrap();
        support::success(&build);
        let output = recorded(
            &directory,
            &format!("run-O{level}"),
            &mut std::process::Command::new(path),
            15,
        );
        std::fs::write(
            directory.join(format!("stdout-O{level}.txt")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("stderr-O{level}.txt")),
            &output.stderr,
        )
        .unwrap();
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn call_spread_fixed_and_rest_native() {
    native(r#"function add(a:int,b:int):int{return a+b;} function sum(...values:int[]):int {let out=0;for(const value of values){out+=value;}return out;} function main(){const values:int[]=[2,3]; console.log(add(...values));console.log(sum(1,...values,4));console.log(values.length);console.log(sum(...[]));}"#,"5\n10\n2\n0\n");
}

#[test]
fn call_spread_dense_data_and_constructor_native() {
    native(r#"
type Row={n:int;};
function copy<T>(...values:T[]):T[]{return values;}
function text(...values:string[]):int{let size=0;for(const value of values){size+=value.length;}return size;}
function rows(...values:Row[]):int{let out=0;for(const row of values){out+=row.n;}return out;}
function nested(...values:int[][]):int{let size=0;for(const value of values){size+=value.length;}return size;}
function booleans(...values:boolean[]):int{return values.length;}
function numbers(...values:number[]):number{let total=0.0;for(const value of values){total+=value;}return total;}
function nil(...values:null[]):int{return values.length;}
function missing(...values:undefined[]):int{return values.length;}
function pair(first:string,second:string):int{return first.length+second.length;}
function main(){
 const integers:int[]=[3,4];console.log(copy<int>(...integers).join(","));
 const words:string[]=["é","😀"];console.log(pair(...words));console.log(words.length);console.log(text(...words));
 const records:Row[]=[{n:4},{n:7}];console.log(rows(...records));
 const arrays:int[][]=[[1],[2,3]];console.log(nested(...arrays));
 const flags:boolean[]=[true,false];console.log(booleans(...flags));
 const values:number[]=[1.5,2.5];console.log(numbers(...values));
 const nulls:null[]=[null,null];const absent:undefined[]=[undefined];console.log(nil(...nulls),missing(...absent));
 console.log(Array.of<int>(0,...integers,5).join(","));console.log(Array.of<number>(...[1,2]).join(","));
}
"#, "3,4\n6\n2\n6\n11\n3\n2\n4\n2 1\n0,3,4,5\n1,2\n");
}

#[test]
fn call_spread_callable_and_defaults_native() {
    native(r#"
function sum(...values:int[]):int{let result=0;for(const value of values){result+=value;}return result;}
function read(action:ReadOnly<Function<int, [...values:int[]]>>,values:int[]):int{return action(...values);}
function mutate(action:Mutable<Function<int, [...values:int[]]>>,values:int[]):int{return action(...values);}
function once(action:Once<Function<int, [...values:int[]]>>,values:int[]):int{return action(...values);}
function defaults(first:int=7,second:int=9):int{return first*10+second;}
function main(){const values:int[]=[2,3];console.log(read(sum,values));let state=0;const changing=(...tail:int[]):int=>{state+=tail.length;return state;};console.log(mutate(changing,values));const final:Once<Function<int, [...tail:int[]]>>=(...tail:int[]):int=>{return tail.length;};console.log(once(final,values));const action:(ReadOnly<Function<int, [...tail:int[]]>>)|undefined=undefined;console.log(String(action?.(...values)));const empty:int[]=[];const one:int[]=[4];console.log(defaults(...empty),defaults(...one),defaults(...values));}
"#, "5\n2\n2\nundefined\n79 49 23\n");
}

#[test]
fn call_spread_nullable_selected_slots_native() {
    native(r#"
function selected(value:int=7):int{return value;}
function text(value:string|null):int{if(value===null){return 0;}return value.length;}
function consume(value:string|undefined="fallback"):string{return value;}
function main(){const empty:int[]=[];const one:int[]=[4];const absent:undefined[]=[undefined];const none:undefined[]=[];
 console.log(selected(...empty,...absent),selected(...one,...none));
 const words:string[]=["é"];const missing:null[]=[];const nil:null[]=[null];const blank:string[]=[];
 console.log(text(...words,...missing),text(...blank,...nil),words.length);
 const owned:string[]=["yes"];const unavailable:undefined[]=[];console.log(consume(...owned,...unavailable));}
"#, "7 4\n2 0 1\nyes\n");
}

#[test]
fn call_spread_runtime_count_and_snapshot_order_native() {
    native(r#"
function size(...values:undefined[]):int{return values.length;}
function invoke(action:ReadOnly<Function<int, [value?:undefined]>>,values:undefined[]):int{return action(...values);}
function mutate(values:int[]):int[]{values[0]=9;return [3];}
function join(...values:int[]):string{return values.join(",");}
function main(){const empty:undefined[]=[];const one:undefined[]=[undefined];const action:ReadOnly<Function<int, [value?:undefined]>>=size;
 console.log(invoke(action,empty),invoke(action,one));
 const values:int[]=[1,2];console.log(join(...values,...mutate(values)),values[0]);
 const called:(ReadOnly<Function<int, [value?:undefined]>>)|null=action;console.log(String(called?.(...one)));}
"#, "0 1\n1,2,3 9\n1\n");
}

fn rejected(source: &str, code: &str, message: &str, token: &str) {
    let error=check(source).err().expect("expected precise rejection");
    assert_eq!(error.code,code,"{}",error.render("input.tsr",source));
    assert!(error.message.contains(message),"{}",error.render("input.tsr",source));
    assert_eq!(&source[error.span.start..error.span.end],token);
}

#[test]
fn call_spread_arity_sources_and_loans_are_checked() {
    rejected("function add(a:int,b:int):int{return a+b;}function main(){add(...[1]);}","arity","statically supplied 1","add(...[1])");
    rejected("function add(a:int,b:int):int{return a+b;}function main(){add(...[1,2,3]);}","arity","statically supplied 3","add(...[1,2,3])");
    rejected("function take(...values:int[]):int{return values.length;}function main(){take(...\"x\");}","type","Expected int","\"x\"");
    rejected("type Item={n:int;};function take(...values:int[]):int{return values.length;}function main(){const item:Item={n:1};take(...item);}","spread-source","array, string, Set, Map, Deque or Heap","item");
    rejected("function pair(first:string,...values:string[]):int{return first.length+values.length;}function main(){const values:string[]=[\"x\"];pair(values[0],...values);}","conflicting-access","access","values");
}

#[test]
fn call_spread_normal_depth_and_original_limit() {
    for depth in [100,200] {
        let expression=format!("{}head(...[1]){}","head(".repeat(depth-1),")".repeat(depth-1));
        let source=format!("function head(value:int):int{{return value;}}function main(){{console.log({expression});}}");
        if depth==100 {let analysis=check(&source).unwrap_or_else(|error|panic!("{}",error.render("input.tsr",&source)));assert!(analysis.emit_rust().contains("expanded call arity"));}
        else {let error=check(&source).err().expect("200 levels reject");assert_eq!(error.code,"unsupported");assert!(error.message.contains("128"));}
    }
}

#[test]
fn call_spread_existing_nullable_scalar_and_mutable_slots_native() {
    native(r#"
function text(value:string|null,extra:int=0):int{if(value===null){return extra;}return value.length+extra;}
function update(value:int[]|null=null):int{if(value===null){return 0;}value.push(7);return value.length;}
function main(){const action:string|null="word";const empty:int[]=[];console.log(text(action,...empty),String(action));
 const missing:null[]=[null];const arrays:int[][]=[];console.log(update(...arrays,...missing));
 const values:int[][]=[[1]];const none:null[]=[];console.log(update(...values,...none),values[0].length);}
"#, "4 word\n0\n2 2\n");
}

#[test]
fn call_spread_static_empty_source_before_rest_native() {
    native(r#"
function total(first:int=5,...tail:int[]):int{let result=first;for(const value of tail){result+=value;}return result;}
function count(...values:int[]):int{return values.length;}
function main(){const none:int[]=[];console.log(total(...[]),total(...none),total(0,...[1,2],3),count(...[]),count(...none,...[4]));}
"#, "5 5 6 0 1\n");
}

#[test]
fn call_spread_narrowed_nullable_callee_native() {
    native(r#"
type Trace = {calls:int;defaults:int;};
function sum(...values:int[]):int{let total=0;for(const value of values){total+=value;}return total;}
function supply(trace:Trace):int[]{trace.calls=trace.calls+1;return [1,2];}
function make(text:string):Once<Function<string[], [trace:Mutable<Trace>,...values:int[]]>>{return (trace:Trace,...values:int[]):string[]=>{trace.calls=trace.calls+1;const result:string[]=[text];let at=0;while(at<values.length){result.push(String(values[at]));at=at+1;}return result;};}
function main(){const reader:(ReadOnly<Function<int, [...values:int[]]>>)|null=sum;if(reader!==null){console.log(reader(...[1,2]),reader(...[]));}
 const calls:Trace={calls:0,defaults:0};const supplied:Trace={calls:0,defaults:0};let i=0;
 while(i<2){let action:(Once<Function<string[], [trace:Mutable<Trace>,...values:int[]]>>)|undefined=undefined;if(i!==0){action=make("a");}let result:string[]=["missing"];if(action!==undefined){result=action(calls,...supply(supplied));}console.log(result.join(","),calls.calls,supplied.calls);i=i+1;}}
"#, "3 0\nmissing 0 0\na,1,2 1 1\n");
}
