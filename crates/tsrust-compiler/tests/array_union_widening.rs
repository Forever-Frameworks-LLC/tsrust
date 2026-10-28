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

#[test]
fn spread_widens_selected_elements_without_widening_source_storage() {
    support::execute(r#"
function main(){
 const source=[9007199254740993];const mixed:(int|string)[]=[...source,"tail"];
 console.log(mixed[0],mixed[1],source[0]);
 const words=["first"];const owned:(string|boolean)[]=[...words,true];
 console.log(owned[0],owned[1]);
}
"#, "9007199254740993 tail 9007199254740993\nfirst true\n", None);
}

#[test]
fn union_of_owned_arrays_widens_each_selected_carrier() {
    support::execute(r#"
function widen(values:int[]|string[]):(int|string)[]{return values;}
function coalesce(values:(int|string)[]|undefined):(int|string)[]{return values??[];}
function main(){
 const numbers=widen([9007199254740993]);console.log(numbers[0]);
 const words=widen(["owned".repeat(1)]);console.log(words[0]);
 const absent=coalesce(undefined);console.log(absent.length);
 const present=coalesce([7,"text"]);console.log(present[0],present[1]);
}
"#, "9007199254740993\nowned\n0\n7 text\n", None);
}
