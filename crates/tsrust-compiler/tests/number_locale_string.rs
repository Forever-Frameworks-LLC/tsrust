mod support;
use support::{execute, reject};

#[test]
fn number_locale_string_matches_node_grouping_rounding_and_special_values() {
 execute(include_str!("fixtures/number-locale-string/values.tsr"),include_str!("fixtures/number-locale-string/values.stdout"),None);
}

#[test]
fn integer_locale_string_keeps_full_range_and_evaluates_receiver_once() {
 execute(r#"function next(log:int[]):number{log.push(1);return 1234.5675;}
function main(){const log:int[]=[];console.log(next(log).toLocaleString(),log.length);
const min:int=-9223372036854775808;const max:int=9223372036854775807;
console.log(min.toLocaleString(),max.toLocaleString());const small:int=-1234;console.log(small.toLocaleString(),small);}"#,
 "1,234.568 1\n-9,223,372,036,854,775,808 9,223,372,036,854,775,807\n-1,234 -1234\n", None);
}

#[test]
fn number_locale_string_rejects_locale_options_and_invalid_receiver() {
 for source in ["function main(){const n=1.5;n.toLocaleString(\"en-US\");}","function main(){const n=1;n.toLocaleString(\"en-US\",7);}"] {reject(source,"unsupported");}
 reject("function main(){const flag=true;flag.toLocaleString();}","type");
}
