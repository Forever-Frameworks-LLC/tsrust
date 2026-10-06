mod support;

// `key in value` tests for an own key, like `Object.hasOwn(value, key)`.
// `for (const key in value)` visits the same keys as `Object.keys(value)`.
// The independent Node run of this source passes.
#[test]
fn in_operator_and_for_in_use_own_keys() {
    let source = r#"import { strictEqual } from "std:testing";
type Point = { x: int; y: int; };
type Named = { name: string; point: Point; };
function count(point: Point): int {
    let total = 0;
    for (const key in point) { if (key !== "y") total += 1; }
    return total;
}
function nextKey(trace: int[]): string { trace.push(1); return "x"; }
function nextPoint(trace: int[]): Point { trace.push(2); return { x: 1, y: 2 }; }
function main() {
    const trace: int[] = [];
    strictEqual(nextKey(trace) in nextPoint(trace), true);
    strictEqual(trace.join(","), "1,2");
    const point: Point = { x: 1, y: 2 };
    strictEqual("x" in point, true);
    strictEqual("z" in point, false);
    const wanted = "y";
    strictEqual(wanted in point, true);
    strictEqual(!("q" in point), true);
    strictEqual("x" in point && "y" in point, true);
    strictEqual("x" in point === true, true);
    const label = "z" in point ? "has" : "lacks";
    strictEqual(label, "lacks");
    const keys: string[] = [];
    for (const key in point) { keys.push(clone(key)); }
    strictEqual(keys.join(","), "x,y");
    strictEqual(count(point), 1);
    const named: Named = { name: "a", point: { x: 3, y: 4 } };
    let fields = "";
    for (let key in named) { fields += key.toUpperCase(); }
    strictEqual(fields, "NAMEPOINT");
    strictEqual("x" in named.point, true);
    const values = [5, 6, 7];
    let indices = "";
    for (const index in values) indices += index;
    strictEqual(indices, "012");
    strictEqual("1" in values, true);
    strictEqual("3" in values, false);
    strictEqual(2 in values, true);
    strictEqual(1.5 in values, false);
    const pair:[int,string]=[3,"a"];
    strictEqual(1 in pair,true);
    strictEqual("length" in pair,true);
    let tupleKeys="";for(const key in pair){tupleKeys+=key;}strictEqual(tupleKeys,"01");
    let snapshot="";for(const key in values){snapshot+=key;if(key==="0")values.push(8);}strictEqual(snapshot,"012");
    for (const key in 5) { strictEqual(key, "unreachable"); }
    let seen = 0;
    for (const key in point) { if (key === "x") continue; seen += 1; break; }
    strictEqual(seen, 1);
    strictEqual(point.x + point.y, 3);
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn in_operator_and_for_in_keep_static_contracts() {
    for (source, code) in [
        (r#"type K={name:string;};type P={x:int;};function change(key:K):P{key.name="y";return {x:1};}function main(){const key:K={name:"x"};const yes=key.name in change(key);}"#, "conflicting-access"),
        (r#"function main(){const yes="0" in "abc";}"#, "type"),
        (r#"type P={x:int;};function main(){const p:P={x:1};for(let key in p){key="y";}}"#, "borrowed-element"),

        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; const a = true in p; }", "type"),
        ("function main() { const a = \"x\" in 5; }", "type"),
        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; const a: int = \"x\" in p; }", "type"),
        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; for (const [a] in p) { } }", "unsupported"),
        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; for (key in p) { } }", "unsupported"),
        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; for (const key in p) { key = \"a\"; } }", "borrowed-element"),
        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; for (const key in p) { const n: int = key; } }", "type"),
    ] {
        support::reject(source, code);
    }
}

#[test]
fn in_syntax_is_independent_of_an_object_binding() {
 let source=r#"import { strictEqual } from "std:testing";
 type P={x:int;};function main(){const Object=7;const p:P={x:1};strictEqual("x" in p,true);let keys="";for(const key in p){keys+=key;}strictEqual(keys,"x");strictEqual(Object,7);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}

#[test]
fn in_chains_preserve_expression_depth_limit() {
 let source=format!("type P={{x:int;}};function main(){{const p:P={{x:1}};const a=\"x\"{};}}", " in p".repeat(200));
 support::reject(&source,"unsupported");
}
