mod support;
use support::{execute, reject};

// Expected values were checked against Node for the same expressions.
#[test]
fn object_is_compares_same_values_and_same_storage() {
    execute(
        r#"
type Row = { n: int; };
function left(log: int[]): int { log.push(1); return 4; }
function right(log: int[]): int { log.push(2); return 4; }
function same(first: Row, second: Row): boolean { return Object.is(first, second); }
function main() {
    const nan = Number.NaN;
    const negative: number = -0.0;
    console.log(Object.is(nan, nan), Object.is(0.0, negative), Object.is(negative, negative), Object.is(1, 1), Object.is(1, 2), Object.is(1, 1.0), Object.is(1.5, 1.5));
    console.log(Object.is("a", "a"), Object.is("a", "b"), Object.is(true, true), Object.is(true, false), Object.is(null, null), Object.is(undefined, undefined), Object.is(null, undefined));
    const a = [1];
    const b = [1];
    const r: Row = { n: 1 };
    const other: Row = { n: 1 };
    console.log(Object.is(a, a), Object.is(a, b), Object.is(r, r), Object.is(r, other), Object.is(1, "1"), Object.is(1, true), Object.is("é", "é"));
    const log: int[] = [];
    console.log(Object.is(left(log), right(log)), log.join(","), same(r, r), same(r, other), a.length, r.n);
    const text = "x".repeat(2);
    console.log(Object.is(text, "xx"), Object.is(text, text), Object.is([1], [1]), Object.is(2.5, 2));
}
"#,
        "true false true true false true true\ntrue false true false true true false\ntrue false true false false false false\ntrue 1,2 true false 1 1\ntrue true false false\n",
        None,
    );
}

#[test]
fn object_is_requires_two_supported_values() {
    for (source, code) in [
        ("function main() { console.log(Object.is(1)); }", "arity"),
        ("function main() { console.log(Object.is(1, 2, 3)); }", "arity"),
        ("function main() { const f = (): int => { return 1; }; console.log(Object.is(f, f)); }", "unsupported"),
        ("function main() { const value: boolean = Object.is(1, 1); const text: string = Object.is(1, 1); }", "type"),
    ] {
        reject(source, code);
    }
}

#[test]
fn object_is_compares_nullable_and_union_payloads_without_losing_same_value_semantics() {
    support::execute(r#"
function same(a:number|null|undefined,b:number|null|undefined):boolean{return Object.is(a,b);}
function data(a:string|null,b:string|null):boolean{return Object.is(a,b);}
function main(){
 console.log(same(Number.NaN,Number.NaN),same(0.0,-0.0),same(null,null),same(null,undefined));
 console.log(data("é","é"),data("é",null));
 const items:int[]|null=[1,2];const other:int[]|null=[1,2];
 console.log(Object.is(items,items),Object.is(items,other),Object.is(items,null));
}
"#,"true false true false\ntrue false\ntrue false false\n",None);
}
