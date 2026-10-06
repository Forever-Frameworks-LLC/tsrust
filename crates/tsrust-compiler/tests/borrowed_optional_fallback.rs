mod support;
use support::execute;
#[test]
fn borrowed_optional_fallbacks_select_without_cloning(){
 execute(include_str!("fixtures/borrowed-optional-fallback/values.tsr"),include_str!("fixtures/borrowed-optional-fallback/values.stdout"),None);
}

#[test]
fn borrowed_optional_fallbacks_keep_loans_and_require_explicit_clone(){
 use support::reject;
 reject("type Row={label:string;};function take(value:string):string{return value;}function main(){const rows:(Row|null)[]=[{label:\"x\"}];const label=rows[0]?.label??\"none\";take(label);}","borrowed-place");
 reject("type Row={label:string;};function main(){const rows:(Row|null)[]=[{label:\"x\"}];const label=rows[0]?.label??\"none\";rows.push(null);console.log(label);}","view-conflict");
 reject("type Row={label:string;};function main(){const rows:(Row|null)[]=[null];const label=rows[0]?.label??\"new\".repeat(1);console.log(label);}","borrowed-view");
 execute("type Row={label:string;};function main(){const rows:(Row|null)[]=[{label:\"x\"}];const label=clone(rows[0]?.label??\"none\");rows.push(null);console.log(label,rows.length);}","x 2\n",None);
}
