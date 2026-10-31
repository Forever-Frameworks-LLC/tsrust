mod support;
use tsrust_compiler::{Mode, check};

fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("input.tsr", source)));
    let rust = analysis
        .emit_rust()
        .replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("higher-order-values.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("higher-order-values-{level}"));
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
                .arg(&binary),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(binary), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

fn reject(source: &str, code: &str) {
    let error = check(source).err().expect("source must be rejected");
    assert_eq!(error.code, code, "{source}\n{}", error.message);
}

// Batched future O0/O3 fixtures. This source proposal has not been executed.
#[test]
fn data_ranks_void_and_flat_borrowed_forwarding_remain_reusable() {
    let cases = [
        ("int", "7", "return value;", 7),
        ("number", "1.25", "if(value===1.25){return 7;}return 0;", 7),
        ("boolean", "true", "if(value){return 7;}return 0;", 7),
        ("string", r#""é😀a\u0000b""#, "return value.length;", 9),
        ("Row", "{n:7}", "return value.n;", 7),
        ("int[]", "[1,2,3,4,5]", "return value.length;", 5),
        (
            "number[]",
            "[1.25,-0.0,Number.NaN]",
            "return value.length;",
            3,
        ),
        ("boolean[]", "[true,false]", "return value.length;", 2),
        ("string[]", r#"["é","a\u0000b"]"#, "return value.length;", 2),
        ("Row[]", "[{n:1},{n:2}]", "return value.length;", 2),
        ("int[][]", "[[1],[],[2,3]]", "return value.length;", 3),
        (
            "number[][]",
            "[[1.25],[],[-0.0]]",
            "return value.length;",
            3,
        ),
        (
            "boolean[][]",
            "[[true],[],[false]]",
            "return value.length;",
            3,
        ),
        (
            "string[][]",
            r#"[["é"],[],["😀"]]"#,
            "return value.length;",
            3,
        ),
        ("Row[][]", "[[{n:1}],[],[{n:2}]]", "return value.length;", 3),
        (
            "Node",
            "{n:5,children:[{n:6,children:[]},{n:7,children:[]}]}",
            "return value.n+value.children.length;",
            7,
        ),
        ("Node[]", "[{n:1,children:[]}]", "return value.length;", 1),
    ];
    // One combined native program preserves all17 family cases and their
    // seven outputs each. Each family needs at most six concrete named/method
    // specialization entries (measure, apply, copyValue, copying, and two pass
    // origins); flat inline/arrowCopy callbacks retain ordinary generic bodies.
    let mut source = String::from("type Row={n:int;};type Node={n:int;children:Node[];};\n");
    let mut expected_output = String::new();
    let mut main_calls = String::new();
    for (case, (leaf, input, body, expected)) in cases.into_iter().enumerate() {
        let argument = if matches!(leaf, "int" | "number" | "boolean") {
            leaf.to_owned()
        } else {
            format!("ReadOnly<{leaf}>")
        };
        let body = body.replace("value", &format!("value{case}"));
        let copied_value = if matches!(leaf, "int" | "number" | "boolean") {
            format!("value{case}")
        } else {
            format!("clone(value{case})")
        };
        source += &format!(
            r#"
            function measure{case}(value{case}:{leaf}):int{{{body}}}
            function apply{case}(action{case}:Mutable<Function<int, [value{case}:{argument}]>>,value{case}:{leaf}):int{{return action{case}(value{case});}}
            function copyValue{case}(value{case}:{leaf}):{leaf}{{return {copied_value};}}
            function copying{case}(action{case}:Mutable<Function<{leaf}, [value{case}:{argument}]>>,value{case}:{leaf}):{leaf}{{return action{case}(value{case});}}
            function pass{case}(consumer{case}:Mutable<Function<int, [action{case}:Mutable<Function<int, [value{case}:{argument}]>>,value{case}:{argument}]>>,
                action{case}:Mutable<Function<int, [value{case}:{argument}]>>,value{case}:{leaf}):int{{return consumer{case}(action{case},value{case});}}
            function main{case}(){{const data{case}:{leaf}={input};const consumer{case}=apply{case};const action{case}=measure{case};
                console.log(consumer{case}(action{case},data{case}));console.log(consumer{case}(action{case},data{case}));
                console.log(pass{case}(consumer{case},action{case},data{case}));
                const inline{case}=(inlineAction{case}:Mutable<Function<int, [value{case}:{argument}]>>,value{case}:{leaf}):int=>{{return inlineAction{case}(value{case});}};
                console.log(inline{case}(action{case},data{case}));console.log(pass{case}(inline{case},action{case},data{case}));
                const copier{case}=copying{case};const copied{case}=copier{case}(copyValue{case},data{case});console.log(measure{case}(copied{case}));
                const arrowCopy{case}=(f{case}:Mutable<Function<{leaf}, [value{case}:{argument}]>>,value{case}:{leaf}):{leaf}=>{{return f{case}(value{case});}};
                const another{case}=arrowCopy{case}(copyValue{case},data{case});console.log(measure{case}(another{case}));
            }}
        "#
        );
        expected_output += &format!(
            "{expected}\n{expected}\n{expected}\n{expected}\n{expected}\n{expected}\n{expected}\n"
        );
        main_calls += &format!("main{case}();");
    }
    source += &format!("function main(){{{main_calls}}}");
    native(&source, &expected_output, "", "");
    native(
        r#"
        function duplicate(values:int[][][]):int[][][]{return clone(values);}
        function copying(f:Mutable<Function<int[][][], [values:ReadOnly<int[][][]>]>>,values:int[][][]):int[][][]{return f(values);}
        function show(values:int[][][]){console.log(values.length);for(const plane of values){console.log(plane.length);for(const row of plane){console.log(row.length);for(const value of row){console.log(value);}}}}
        function main(){const original:int[][][]=[[[1,2],[]],[],[[3],[4,5]]];
            const declared=copying;const first=declared(duplicate,original);first[0][0][0]=99;
            const inline=(f:Mutable<Function<int[][][], [values:ReadOnly<int[][][]>]>>,values:int[][][]):int[][][]=>{return f(values);};
            const second=inline(duplicate,original);second[2][1][1]=77;
            show(original);show(first);show(second);show(original);}
    "#,
        concat!(
            "3\n2\n2\n1\n2\n0\n0\n2\n1\n3\n2\n4\n5\n",
            "3\n2\n2\n99\n2\n0\n0\n2\n1\n3\n2\n4\n5\n",
            "3\n2\n2\n1\n2\n0\n0\n2\n1\n3\n2\n4\n77\n",
            "3\n2\n2\n1\n2\n0\n0\n2\n1\n3\n2\n4\n5\n"
        ),
        "",
        "",
    );
    native(
        r#"
        type Node={n:int;children:Node[];};
        function duplicate(values:Node[][]):Node[][]{return clone(values);}
        function copying(f:Mutable<Function<Node[][], [values:ReadOnly<Node[][]>]>>,values:Node[][]):Node[][]{return f(values);}
        function node(value:Node){console.log(value.n);console.log(value.children.length);for(const child of value.children){node(child);}}
        function show(values:Node[][]){console.log(values.length);for(const row of values){console.log(row.length);for(const value of row){node(value);}}}
        function main(){const original:Node[][]=[[{n:1,children:[{n:2,children:[]}]}],[],[{n:3,children:[]}]];
            const declared=copying;const first=declared(duplicate,original);first[0][0].children[0].n=99;
            const inline=(f:Mutable<Function<Node[][], [values:ReadOnly<Node[][]>]>>,values:Node[][]):Node[][]=>{return f(values);};
            const second=inline(duplicate,original);second[2][0].n=77;
            show(original);show(first);show(second);show(original);}
    "#,
        concat!(
            "3\n1\n1\n1\n2\n0\n0\n1\n3\n0\n",
            "3\n1\n1\n1\n99\n0\n0\n1\n3\n0\n",
            "3\n1\n1\n1\n2\n0\n0\n1\n77\n0\n",
            "3\n1\n1\n1\n2\n0\n0\n1\n3\n0\n"
        ),
        "",
        "",
    );
    native(
        r#"
        function visit(value:int){console.log(value);}
        function apply(action:Mutable<Function<void, [value:int]>>,value:int){action(value);}
        function pass(consumer:Mutable<Function<void, [action:Mutable<Function<void, [value:int]>>,value:int]>>,action:Mutable<Function<void, [value:int]>>,value:int){consumer(action,value);}
        function main(){const consumer=apply;const action=visit;consumer(action,1);pass(consumer,action,2);
            const inline=(f:Mutable<Function<void, [value:int]>>,value:int):void=>{f(value);};inline(action,3);pass(inline,action,4);}
    "#,
        "1\n2\n3\n4\n",
        "",
        "",
    );
    native(
        r#"
        function add(a:int,v:int):int{return a+v;}
        function total(f:Mutable<Function<int, [a:int,v:int]>>,values:int[]):int{return values.reduce(f,0);}
        function twice(f:Mutable<Function<int, [v:int]>>,values:int[]):int[]{return values.map(f);}
        function next(v:int):int{return v+1;}
        function main(){const fold=total;const map=twice;const values=[1,2,3];console.log(fold(add,values));
            const output=map(next,values);console.log(output[2]);console.log([].map(next).length);
            console.log([].reduce(add,0));}
    "#,
        "6\n4\n0\n0\n",
        "",
        "",
    );
}

#[test]
fn recursive_signatures_passing_returning_consumers_and_distinct_witnesses() {
    native(
        r#"
        function add(x:int):int{return x+1;}
        function double(x:int):int{return x*2;}
        function apply(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}
        function pass(consumer:Mutable<Function<int, [f:Mutable<Function<int, [x:int]>>,x:int]>>,f:Mutable<Function<int, [x:int]>>,x:int):int{return consumer(f,x);}
        function factory():(f:Mutable<Function<int, [x:int]>>,x:int)=>int{return apply;}
        function factoryFactory():()=>(f:Mutable<Function<int, [x:int]>>,x:int)=>int{return factory;}
        function keep(f:Function<int, [x:int]>):(x:int)=>int{return f;}
        function keeperFactory():(f:Function<int, [x:int]>)=>(x:int)=>int{return keep;}
        function keepPass(consumer:Mutable<Function<(x:int)=>int, [f:Function<int, [x:int]>]>>,f:Function<int, [x:int]>):(x:int)=>int{return consumer(f);}
        function main(){
            const consumer=apply;const a=add;const b=double;
            console.log(consumer(a,3));console.log(consumer(b,3));console.log(pass(consumer,a,4));console.log(pass(consumer,b,4));
            const make=factoryFactory;const maker=make();const selected=maker();console.log(selected(a,5));console.log(selected(b,5));
            const returned=keeperFactory();const first=returned(add);const second=returned(double);console.log(first(6));console.log(second(6));
            const via=keepPass(returned,add);console.log(via(7));
            const inline=(f:Mutable<Function<int, [x:int]>>,x:int):int=>{return f(x);};console.log(pass(inline,a,8));console.log(pass(inline,b,8));
            const arrowKeep=(f:Function<int, [x:int]>):(x:int)=>int=>{return f;};
            const arrowFirst=arrowKeep(add);const arrowSecond=arrowKeep(double);console.log(arrowFirst(9));console.log(arrowSecond(9));
            const borrowedArrow=(f:Function<int, [x:int]>):(x:int)=>int=>{return f;};
            const relayFirst=keepPass(borrowedArrow,add);const relaySecond=keepPass(borrowedArrow,double);console.log(relayFirst(10));console.log(relaySecond(10));
        }
    "#,
        "4\n6\n5\n8\n6\n10\n7\n12\n8\n9\n16\n10\n18\n11\n20\n",
        "",
        "",
    );
    native(
        r#"
        function pair(left:Mutable<Function<int, [x:int]>>,right:Mutable<Function<int, [x:int]>>,x:int):int{return left(x)*10+right(x);}
        function add(x:int):int{return x+1;}function double(x:int):int{return x*2;}
        function main(){const consumer=pair;const a=add;const b=double;console.log(consumer(a,b,2));console.log(consumer(b,a,2));
            const arrow=(left:Mutable<Function<int, [x:int]>>,right:Mutable<Function<int, [x:int]>>,x:int):int=>{return left(x)*10+right(x);};
            console.log(arrow(a,b,3));console.log(arrow(b,a,3));}
    "#,
        "34\n43\n46\n64\n",
        "",
        "",
    );
}

#[test]
fn all_callback_kinds_and_argument_dependent_owned_factory_methods() {
    native(
        r#"
        function add(x:int):int{return x+1;}
        function borrowed(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}
        function once(f:Once<Function<int, [x:int]>>,x:int):int{return f(x);}
        function owned(f:Function<int, [x:int]>,x:int):int{return f(x);}
        function ownedOnce(f:Once<Function<int, [x:int]>>,x:int):int{return f(x);}
        function invokeConsumer(c:Once<Function<int, [f:Once<Function<int, [x:int]>>,x:int]>>,f:Once<Function<int, [x:int]>>,x:int):int{return c(f,x);}
        function make(prefix:string):(f:Function<int, [x:int]>)=>(x:int)=>int{
            return(f:Function<int, [x:int]>):(x:int)=>int=>{const copied=clone(prefix);return(x:int):int=>{return f(x)+copied.length;};};
        }
        function chain(f:Function<int, [x:int]>):()=>(x:int)=>int{return():(x:int)=>int=>{return f;};}
        function fixed():(x:int)=>int{return add;}
        function carryFactory(f:Function<(x:int)=>int>):()=>(x:int)=>int{return():(x:int)=>int=>{return f();};}
        function main(){const reuse=borrowed;const a=add;console.log(reuse(a,1));console.log(reuse(a,2));
            const one=once;console.log(one(add,3));const many=owned;console.log(many(add,4));console.log(many(add,5));
            const last=ownedOnce;console.log(last(add,6));console.log(invokeConsumer(ownedOnce,add,7));
            const ownedArrow=(f:Function<int, [x:int]>,x:int):int=>{return f(x);};console.log(many(add,8));console.log(ownedArrow(add,9));
            const onceArrow=(f:Once<Function<int, [x:int]>>,x:int):int=>{return f(x);};console.log(invokeConsumer(onceArrow,add,10));
            const makeValue=make;const consumer=makeValue("é");const first=consumer(add);const second=consumer((x:int):int=>{return x*2;});
            console.log(first(11));console.log(second(11));console.log(first(12));
            const outer=chain;const factory=outer(add);const returned=factory();console.log(returned(13));
            const carry=carryFactory;const factoryResult=carry(fixed);const callbackResult=factoryResult();console.log(callbackResult(14));
        }
    "#,
        "2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n14\n24\n15\n14\n15\n",
        "",
        "",
    );
    // One stable materialized alias accepts distinct owned environment origins.
    // Their capture storage stays separate after both callback results escape.
    native(
        r#"
        function plus(n:int):(x:int)=>int{return(x:int):int=>{return n+x;};}
        function times(n:int):(x:int)=>int{return(x:int):int=>{return n*x;};}
        function main(){const keep=(f:Function<int, [x:int]>):(x:int)=>int=>{return f;};
            const left=plus(7);const right=times(3);const first=keep(left);const second=keep(right);
            console.log(first(2));console.log(second(2));console.log(first(4));console.log(second(4));}
    "#,
        "9\n6\n11\n12\n",
        "",
        "",
    );
    // Inferred and explicitly typed arrows returning callbacks/factories are
    // the same approved scope as declared consumers, including once ownership.
    native(
        r#"
        function add(x:int):int{return x+1;}
        function use(c:Function<(x:int)=>int, [f:Function<int, [x:int]>]>,f:Function<int, [x:int]>):(x:int)=>int{return c(f);}
        function useOnce(c:Once<Function<()=>int, [f:Once<Function<int, [x:int]>>]>>,f:Once<Function<int, [x:int]>>):()=>int{return c(f);}
        function main(){const returned=use((f:Function<int, [x:int]>):(x:int)=>int=>{return(x:int):int=>{return f(x)+2;};},add);
            console.log(returned(1));console.log(returned(2));
            const last=useOnce((f:Once<Function<int, [x:int]>>):()=>int=>{return():int=>{return f(3);};},add);console.log(last());}
    "#,
        "4\n5\n4\n",
        "",
        "",
    );
}

#[test]
fn aliases_captures_exact_contracts_source_order_and_loan_rejections() {
    native(&support::format_round_trip("function apply(f:Mutable<Function<int,[x:int]>>):int{const c=():int=>f(1);return c();}function main(){console.log(apply((x:int):int=>x+4));}"), "5\n", "", "");
    let analysis = native(
        r#"
        function add(x:int):int{return x+1;}
        function apply(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}
        function markAction(n:int):(x:int)=>int{console.log(n);return add;}
        function mark(n:int):int{console.log(n);return n;}
        function captured():(f:Mutable<Function<int, [x:int]>>,x:int)=>int{const bound=apply;return(f:Mutable<Function<int, [x:int]>>,x:int):int=>{return bound(f,x);};}
        function main(){const consumer=apply;console.log(consumer(markAction(1),mark(2)));
            const moved=consumer;console.log(moved(add,3));let replace=apply;replace=apply;console.log(replace(add,4));
            const global=(f:Mutable<Function<int, [x:int]>>,x:int):int=>{return apply(f,x);};console.log(global(add,5));
            const bound=captured();console.log(bound(add,6));console.log(bound(add,7));}
    "#,
        "1\n2\n3\n4\n5\n6\n7\n8\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts()["apply.f"], Mode::Mutate);
    for (source, code) in [
        (
            "function apply(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}function main(){const c=apply;const moved=c;c((x:int):int=>{return x;},1);}",
            "closure-consumed",
        ),
        (
            "function a(f:Mutable<Function<int, [x:int]>>):int{return f(1);}function b(f:Mutable<Function<int, [x:int]>>):int{return f(2);}function main(){let c=a;c=b;}",
            "callable-origin",
        ),
        (
            "function bad(f:Mutable<Function<int, [x:int]>>):(x:int)=>int{return f;}function main(){}",
            "callback-escape",
        ),
        (
            "function keep(f:Function<int, [x:int]>):(x:int)=>int{return f;}function bad(c:Mutable<Function<(x:int)=>int, [f:Function<int, [x:int]>]>>):(f:Function<int, [x:int]>)=>(x:int)=>int{return c;}function main(){bad(keep);}",
            "callback-escape",
        ),
        (
            "function one(f:Once<Function<int, [x:int]>>):int{return f(1)+f(2);}function main(){}",
            "callback-consumed",
        ),
        (
            "function own(f:Function<int, [x:int]>):int{return f(1);}function bad(f:Mutable<Function<int, [x:int]>>):int{return own(f);}function main(){}",
            "callback-contract",
        ),
        (
            "function consumer(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}function take(c:Function<int, [f:Mutable<Function<int, [x:int]>>,x:int]>):int{return 1;}function main(){const c=consumer;const a=(x:int):int=>{return x;};c(a,take(c));}",
            "conflicting-access",
        ),
        (
            "function consumer(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}function main(){const c=consumer;c((x:string):int=>{return x.length;},1);}",
            "type",
        ),
        (
            "function consumer(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}function use(c:Mutable<Function<int, [f:Mutable<Function<boolean, [x:int]>>,x:int]>>):int{return 0;}function main(){use(consumer);}",
            "type",
        ),
        (
            "function own(f:Function<int, [x:int]>):int{return f(1);}function use(f:ReadOnly<Function<int, [x:int]>>):int{return own(f);}function main(){}",
            "callback-contract",
        ),
        (
            "function use(c:Mutable<Function<int, [f:Mutable<Function<int, [x:int]>>]>>):int{return 0;}function main(){use((f:Function<int, [x:int]>):int=>{return f(1);});}",
            "callback-access",
        ),
        (
            "function use(c:Mutable<Function<string, [text:ReadOnly<string>]>>):string{return c(\"x\");}function main(){use((text:string):string=>{return text;});}",
            "callback-access",
        ),
    ] {
        reject(source, code);
    }
    for (source, code) in [
        // The first invocation has already concretized the higher-order input.
        (
            "function add(x:int):int{return x+1;}function apply(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}function reusable(c:Mutable<Function<int, [f:Mutable<Function<int, [x:int]>>,x:int]>>):int{return c(add,2);}function onceConsumer(c:Once<Function<int, [f:Mutable<Function<int, [x:int]>>,x:int]>>):int{c(add,1);return reusable(c);}function main(){onceConsumer(apply);}",
            "callback-contract",
        ),
        (
            "function add(x:int):int{return x+1;}function apply(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}function owned(c:Function<int, [f:Mutable<Function<int, [x:int]>>,x:int]>):int{return c(add,2);}function borrowed(c:Mutable<Function<int, [f:Mutable<Function<int, [x:int]>>,x:int]>>):int{c(add,1);return owned(c);}function main(){borrowed(apply);}",
            "callback-contract",
        ),
        (
            "function apply(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}function accepts(c:Mutable<Function<int, [f:Mutable<Function<int, [x:int,y:int]>>,x:int]>>):int{return 0;}function main(){accepts(apply);}",
            "arity",
        ),
        (
            "function apply(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}function accepts(c:Mutable<Function<int, [f:Mutable<Function<boolean, [x:int]>>,x:int]>>):int{return 0;}function main(){accepts(apply);}",
            "type",
        ),
        (
            "function use(f:Mutable<Function<int, [text:ReadOnly<string>]>>,text:string):int{return f(text);}function accepts(c:Mutable<Function<int, [f:Mutable<Function<int, [text:Owned<string>]>>,text:ReadOnly<string>]>>):int{return 0;}function main(){accepts(use);}",
            "callback-access",
        ),
    ] {
        reject(source, code);
    }
    reject(
        "function choose(flag:boolean):(f:Mutable<Function<int, [x:int]>>,x:int)=>int{if(flag){return a;}return b;}function a(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x);}function b(f:Mutable<Function<int, [x:int]>>,x:int):int{return f(x)+1;}function main(){}",
        "unsupported",
    );
}

#[test]
fn recursion_annotation_depth_and_typed_native_graph_paths_preserve_limits() {
    native(
        r#"
        function next(x:int):int{return x+1;}
        function recurse(f:Mutable<Function<int, [x:int]>>,n:int):int{if(n===0){return f(0);}return recurse(f,n-1)+1;}
        function mutual(f:Mutable<Function<int, [x:int]>>,n:int):int{if(n===0){return f(1);}return other(f,n-1);}
        function other(f:Mutable<Function<int, [x:int]>>,n:int):int{return mutual(f,n);}
        function higher(c:Mutable<Function<int, [f:Mutable<Function<int, [x:int]>>,n:int]>>,f:Mutable<Function<int, [x:int]>>,n:int):int{if(n===0){return c(f,2);}return higher(c,f,n-1);}
        function factory(n:int):(f:Mutable<Function<int, [x:int]>>,n:int)=>int{if(n===0){return recurse;}return factory(n-1);}
        function main(){const value=recurse;console.log(value(next,3));console.log(value(next,4));console.log(mutual(next,3));
            const c=factory(2);console.log(higher(c,next,2));
            const arrow=(f:Mutable<Function<int, [x:int]>>,n:int):int=>{return recurse(f,n);};console.log(higher(arrow,next,2));}
    "#,
        "4\n5\n2\n3\n3\n",
        "",
        "",
    );
    for depth in [100, 200] {
        let expression = format!("{}1{}", "-(".repeat(depth), ")".repeat(depth));
        let source = format!(
            "function apply(f:Mutable<Function<int, [x:int]>>):int{{return f({expression});}}function next(x:int):int{{return x;}}function main(){{const c=apply;console.log(c(next));}}"
        );
        if depth == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            support::accept_deep(&source);
        }
        let mut parameter = "int".to_string();
        let mut result = "int".to_string();
        for _ in 0..depth {
            parameter = format!("(p:{parameter})=>int");
            result = format!("()=>{result}");
        }
        let param_source = format!("function unused(f:{parameter}){{}}function main(){{}}");
        let result_source =
            format!("function unused():{result}{{return unused();}}function main(){{}}");
        if depth == 100 {
            // The unused nested contract remains a non-emitted template.
            check(&param_source).unwrap().emit_rust();
            // Real returning function values validate the entire nested result
            // tree without conflating parsing with a recursive-origin rejection.
            let mut definitions = "function result0():int{return 1;}".to_string();
            let mut annotation = "int".to_string();
            for n in 1..=depth {
                annotation = format!("()=>{annotation}");
                definitions += &format!(
                    "function result{n}():{annotation}{{return result{};}}",
                    n - 1
                );
            }
            definitions += "function main(){}";
            check(&definitions).unwrap().emit_rust();
            reject(&result_source, "unsupported");
        } else {
            // Deep annotations have no nesting limit; the self-returning result
            // is still rejected as a recursive origin.
            support::accept_deep(&param_source);
            assert_eq!(support::check_deep(&result_source).unwrap_err().code, "unsupported");
        }
    }
    // Like rustc, distinct closed arrow origins instantiate without a count limit.
    for count in [8, 129] {
        let mut body = String::new();
        for n in 0..count {
            body += &format!("console.log(c((x:int):int=>{{return x+{n};}},0));");
        }
        let source = format!(
            "function apply(f:Mutable<Function<int, [x:int]>>,n:int):int{{if(n===0){{return f(0);}}return apply(f,n-1);}}function main(){{const c=apply;{body}}}"
        );
        check(&source).unwrap().emit_rust();
    }
    // Named owned callback specializations and inline materialized methods
    // instantiate per origin without a count limit.
    for count in [128, 129] {
        let mut body = String::new();
        for n in 0..count {
            let target = if n % 2 == 0 { "named" } else { "inline" };
            body += &format!("console.log({target}((x:int):int=>{{return x+{n};}}));");
        }
        let source = format!(
            "function named(f:Function<int, [x:int]>):int{{return f(0);}}function main(){{const inline=(f:Function<int, [x:int]>):int=>{{return f(0);}};{body}}}"
        );
        check(&source).unwrap().emit_rust();
    }
    // A generic flat argument is reborrowed into one owned-dependent method.
    // The method has one source specialization, but receives distinct closed
    // environment types through real typed caller edges of bridge().
    for count in [128, 129] {
        let mut body = String::new();
        for n in 0..count {
            body += &format!("console.log(bridge((x:int):int=>{{return x+{n};}}));");
        }
        let source = format!(
            "function zero():int{{return 0;}}function bridge(f:Mutable<Function<int, [x:int]>>):int{{const invoke=(action:Mutable<Function<int, [x:int]>>,maker:Function<int>):int=>{{return action(maker());}};return invoke(f,zero);}}function main(){{{body}}}"
        );
        check(&source).unwrap().emit_rust();
    }
    // Repeated visits of the SAME flat witness are deduplicated by typed target
    // and vector, rather than charged as new origins or native instantiations.
    let repeated = "console.log(bridge(same));".repeat(160);
    let source = format!(
        "function zero():int{{return 0;}}function bridge(f:Mutable<Function<int, [x:int]>>):int{{const invoke=(action:Mutable<Function<int, [x:int]>>,maker:Function<int>):int=>{{return action(maker());}};return invoke(f,zero);}}function main(){{const same=(x:int):int=>{{return x+1;}};{repeated}}}"
    );
    native(&source, &"1\n".repeat(160), "", "");
}

#[test]
fn owned_results_ieee_bits_cleanup_and_standard_capture_layout() {
    let source = r#"
        function copy(text:string):string{return clone(text);}
        function take(text:string):string{return text;}
        function fail(text:string,values:int[]):string{values.remove(-1);return text;}
        function drive(f:Mutable<Function<string, [text:ReadOnly<string>]>>,text:string):string{return f(text);}
        function transfer(f:Once<Function<string, [text:Owned<string>]>>,text:string):string{return f(text);}
        function numbers(values:number[]):number[]{return values;}
        function transferNumbers(f:Once<Function<number[], [values:Owned<number[]>]>>,values:number[]):number[]{return f(values);}
        function panicText(text:string):string{const values=[1];values.remove(-1);return text;}
        function makeOwned(prefix:string):(f:Once<Function<string, [text:Owned<string>]>>,text:Owned<string>)=>string{
            return(f:Once<Function<string, [text:Owned<string>]>>,text:string):string=>{
                if(prefix.length===0){return text;}return f(text);
            };
        }
        function trigger(text:string){const consumer=makeOwned("captured".repeat(64));consumer(panicText,text);}
        function bitTransfer(values:number[]):number[]{const consumer=transferNumbers;return consumer(numbers,values);}
        function textTransfer(text:string):string{const consumer=transfer;return consumer(take,text);}
        function main(){const c=drive;const failure=fail;console.log(c(copy,"é😀a\u0000b").length);
            const t=transfer;console.log(t(take,"owned").length);const b=transferNumbers;
            const output=b(numbers,[-0.0,Number.NaN,Number.POSITIVE_INFINITY]);console.log(output.length);}
    "#;
    let checked = check(source).unwrap();
    let id = |name: &str| {
        checked
            .closures()
            .into_iter()
            .find(|c| c.function == name && c.depth == 0)
            .unwrap()
            .id
    };
    let copy = id("copy");
    let take = id("take");
    let fail = id("fail");
    let drive = id("drive");
    let environment = checked
        .closures()
        .into_iter()
        .find(|c| c.function == "makeOwned" && c.depth == 1 && !c.template)
        .unwrap();
    assert_eq!(environment.captures.len(), 1);
    assert_eq!(environment.captures[0].name, "prefix");
    assert_eq!(environment.captures[0].storage, Mode::Consume);
    let owner = environment.id;
    // All descriptor IDs below come from the exact source compiled natively.
    let source = source.to_string();
    let asserts = format!(
        r#"
        fn send_sync<T:Send+Sync>(){{}}send_sync::<Closure{drive}>();
        assert_eq!(std::mem::size_of::<Closure{drive}>(),0);
        send_sync::<Closure{owner}>();
        let warm=function8(String::Owned("warm thread".repeat(64)));
        std::thread::spawn(move||drop(warm)).join().unwrap();
        let start=LIVE.load(Ordering::SeqCst);
        {{
            let input:String=String::Owned("é😀\0".repeat(64));let pointer=input.as_ptr();let mut action=Closure{copy}{{}};
            let output=function3(&mut action,&input);assert_eq!(output.as_ref(),input.as_ref());assert_ne!(output.as_ptr(),pointer);
            drop(output);drop(input);
            let input:String=String::Owned("moved".repeat(64));let pointer=input.as_ptr();
            let output=function11(input);assert_eq!(output.as_ptr(),pointer);drop(output);
            let mut values=NumericArray::from([1i64,2,3]);let old=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{{}}));
            let panic=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{{
                Closure{fail}{{}}.call(String::Owned("failure".repeat(64)),&mut values)
            }}));assert!(panic.is_err());drop(panic);drop(values);
            let panic=std::panic::catch_unwind(||function9(String::Owned("owned unwind".repeat(64))));
            std::panic::set_hook(old);assert!(panic.is_err());drop(panic);
            let input=vec![-0.0f64,0.0,f64::INFINITY,f64::NEG_INFINITY,f64::from_bits(0x7ff8_0000_0000_1234)];
            let expected=input.iter().map(|v|v.to_bits()).collect::<Vec<_>>();let pointer=input.as_ptr();let output=function10(input);
            assert_eq!(output.as_ptr(),pointer);assert_eq!(output.iter().map(|v|v.to_bits()).collect::<Vec<_>>(),expected);drop(output);drop(expected);
            let owned:String=String::Owned("captured".repeat(64));let pointer=owned.as_ptr();
            {{let environment=Closure{take}{{}};let moved=environment.call(owned);assert_eq!(moved.as_ptr(),pointer);drop(moved);}}
        }}assert_eq!(LIVE.load(Ordering::SeqCst),start);
    "#
    );
    native(&source, "9\n5\n3\n", ALLOCATOR, &asserts);
    let ordinary=native("function use(f:Mutable<Function<number, [v:number]>>,values:number[]):number{return f(values[0]);}function next(v:number):number{return v+1.0;}function main(){const c=use;const values:number[]=[1.0,2.0,3.0];console.log(c(next,values));}","2\n","","").emit_rust();
    // Generic callback invocation receives a copied scalar from this array;
    // the existing conservative proof retains its ordinary Vec ABI.
    assert!(ordinary.contains("value1: &Vec<f64>"));
    assert!(!ordinary.contains("[f64; 3]"));
    let fixed=native("function first(values:number[]):number{return values[0];}function next(v:number):number{return v+1.0;}function main(){const action=next;const values:number[]=[1.0,2.0,3.0];console.log(first(values));console.log(action(3.0));}","1\n4\n","","").emit_rust();
    // A direct readonly call still specializes even beside a declared value.
    assert!(fixed.contains("[f64; 3]"));
}
const ALLOCATOR: &str = r#"
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);
struct Observer;
unsafe impl GlobalAlloc for Observer{
    unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);unsafe{System.dealloc(p,l)}}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let q=unsafe{System.realloc(p,l,n)};if !q.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);}q}
}
#[global_allocator]static GLOBAL:Observer=Observer;
"#;

#[test]
fn once_higher_order_inputs_do_not_promote_borrowed_environments_to_owners() {
    reject(
        r#"
        function retain(action: Once<Function<int>>): Once<Function<int>> { return action; }
        function callScoped(consumer: Once<Function<Once<Function<int>>, [action: Once<Function<int>>]>>): int {
            const values = [7];
            const result = consumer((): int => values[0]);
            return result();
        }
        function main() { console.log(callScoped(retain)); }
        "#,
        "callback-escape",
    );
}

#[test]
fn suspended_closure_methods_preserve_named_and_expression_invocation_results() {
    let mut source = String::new();
    for (name, conditional) in [("namedChain", false), ("expressionChain", true)] {
        source.push_str(&format!("function {name}():int{{const step0=(f:Function<int,[x:int]>):int=>f(0);"));
        for index in 1..8 {
            let previous = format!("step{}", index - 1);
            let callee = if conditional { format!("(true?{previous}:{previous})") } else { previous };
            source.push_str(&format!("const step{index}=(f:Function<int,[x:int]>):int=>{callee}(f);"));
        }
        source.push_str("return step7((x:int):int=>x+7);}");
    }
    source.push_str("function main(){console.log(namedChain(),expressionChain());}");
    let source = support::format_round_trip(&source);
    native(&source, "7 7\n", "", "");
}
