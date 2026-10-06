mod support;
use support::{execute, reject};

#[test]
fn string_raw_interleaves_segments_converts_values_and_keeps_sources() {
    execute(r#"
type Template = { raw: string[]; };
function main() {
    const template: Template = { raw: ["a", "b", "c"] };
    console.log(String.raw(template, 7, true));
    console.log(String.raw(template), String.raw(template, "X"), String.raw(template, "X", "Y", "ignored"));
    const empty: string[] = [];
    console.log(String.raw({ raw: empty }, 9).length, String.raw({ raw: ["only"] }, 9));
    console.log(String.raw({ raw: ["é", "😀", "\\n"] }, null, undefined));
    console.log(String.raw({ raw: ["", "", "", ""] }, -0.0, Number.POSITIVE_INFINITY, Number.NaN));
    console.log(String.raw({ raw: ["x", "y"] }, [1,2]));
    const inserts = ["X", "Y"];
    console.log(String.raw(template, ...inserts), inserts.join(","), template.raw.join("|"));
}
"#, "a7btruec\nabc aXbc aXbYc\n0 only\nénull😀undefined\\n\n0InfinityNaN\nx1,2y\naXbYc X,Y a|b|c\n", None);
}

#[test]
fn string_raw_evaluates_source_and_even_unused_substitutions_once_in_order() {
    execute(r#"
type Template = { raw: string[]; };
function source(log: int[]): Template { log.push(1); return {raw:["a","b"]}; }
function value(log: int[], n: int): int { log.push(n); return n; }
function main() {
 const log: int[]=[];
 console.log(String.raw(source(log), value(log,2), value(log,3)), log.join(","));
 const empty: Template={raw:[]};
 console.log(String.raw(empty,value(log,4)).length,log.join(","));
}
"#, "a2b 1,2,3\n0 1,2,3,4\n", None);
}

#[test]
fn string_raw_rejects_missing_or_invalid_template() {
    reject("function main(){String.raw();}", "arity");
    for source in [
      "function main(){String.raw(7);}",
      "function main(){String.raw({raw:[1,2]});}",
      "function main(){String.raw({raw:\"abc\"});}",
    ] { reject(source,"type"); }
}

#[test]
fn string_raw_accepts_inline_template_in_assignment_context() {
 execute(r#"function main(){const result=String.raw({raw:["a","b"]},7);console.log(result);}"#, "a7b\n", None);
}

#[test]
fn string_raw_checks_spread_types_and_source_loans() {
 reject("type Template={raw:string[];};function main(){const template:Template={raw:[\"a\",\"b\"]};const values=[1,2];String.raw(template,...values);}", "type");
 reject("type Other={value:int;};function main(){const other:Other={value:1};String.raw(other);}", "type");
 reject("type Template={raw:string[];};function grow(template:Template):int{template.raw.push(\"c\");return 1;}function main(){const template:Template={raw:[\"a\",\"b\"]};String.raw(template,grow(template));}", "conflicting-access");
 reject("function action():int{return 1;}function main(){String.raw({raw:[\"a\",\"b\"]},action);}", "unsupported");
}
