mod support;
use tsrust_compiler::check;

#[test]
fn owned_arrays_widen_elements_once_and_keep_exact_values() {
    support::execute(r#"
function wider(values:(int|string)[]):(int|string|boolean)[]{return values;}
function optional(values:string[]|undefined):(string|boolean)[]|undefined{return values;}
function absent():string[]|undefined{return undefined;}
function present():string[]|undefined{return ['optional'];}
function main(){
 const strings=['first','second'];const mixed:(string|boolean)[]=strings;
 console.log(mixed[0],mixed[1]);
 const numbers=[9007199254740993];const tagged:(int|string)[]=numbers;
 const expanded=wider(tagged);console.log(expanded[0]);
 const nullable=optional(present());if(nullable!==undefined){console.log(nullable[0]);}
 console.log(optional(absent())===undefined);
}
"#, "first second\n9007199254740993\noptional\ntrue\n", None);
}

#[test]
fn widening_rejects_implicit_borrow_mutable_variance_loss_and_reuse() {
    for source in [
        "function see(values:ReadOnly<(string|boolean)[]>){console.log(values.length);}function main(){const values=['a'];see(values);}",
        "function change(values:Mutable<(string|boolean)[]>){values.push(true);}function main(){let values=['a'];change(values);}",
        "function main(){const values=['a'];const mixed:(string|boolean)[]=values;console.log(values.length);}",
        "function main(){const values=[1];const mixed:(number|string)[]=values;}",
        "function main(){const values:(string|boolean)[]=['a',true];const narrow:string[]=values;}",
    ] {
        assert!(check(source).is_err(), "{source}");
    }
}
