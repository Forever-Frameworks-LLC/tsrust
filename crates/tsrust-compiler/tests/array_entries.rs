mod support;
use support::{execute,reject};

#[test]
fn array_entries_returns_ordered_independent_index_value_pairs() {
 execute(r#"type Row={text:string;};
function main(){
 const values=["a".repeat(2),"b"];
 const pairs:[int,string][]=values.entries();
 console.log(pairs.length,pairs[0][0],pairs[0][1],pairs[1][0],pairs[1][1]);
 pairs[0][1]="changed";console.log(values.join(","),pairs[0][1]);
 const rows:Row[]=[{text:"old".repeat(2)}];const recordPairs=rows.entries();recordPairs[0][1].text="new";
 console.log(rows[0].text,recordPairs[0][1].text);
 const nested=[[1,2],[3]];const nestedPairs=nested.entries();nestedPairs[0][1].push(4);console.log(nested[0].join(),nestedPairs[0][1].join());
 const empty:int[]=[];console.log(empty.entries().length);
 const entries=[4,5].entries();let sum=0;for(const pair of entries){sum+=pair[0]+pair[1];}console.log(sum,entries);
}"#, "2 0 aa 1 b\naa,b changed\noldold new\n1,2 1,2,4\n0\n10 [ [ 0, 4 ], [ 1, 5 ] ]\n",None);
}

#[test]
fn array_entries_rejects_arguments_non_arrays_and_wrong_pair_types() {
 reject("function main(){[1].entries(1);}","arity");
 reject("function main(){\"x\".entries();}","unsupported");
 reject("function main(){const pairs:[string,int][]=[1].entries();}","type");
}

#[test]
fn array_entries_evaluates_receiver_once_and_supports_all_data_leaves() {
 execute(r#"function source(log:int[]):string[]{log.push(1);return ["x".repeat(2),"y"];}
function main(){const log:int[]=[];const pairs=source(log).entries();console.log(log.join(),pairs[0][1],pairs[1][0]);
console.log([1.5,Number.NaN].entries(),[true,false].entries(),[null,null].entries(),[undefined,undefined].entries());
const tuples:[int,string][]=[[2,"two"]];const entries=tuples.entries();entries[0][1][1]="changed";console.log(tuples[0][1],entries[0][1][1]);
const selected=true?["last"]:["first"];console.log(selected.entries()[0][1]);
}"#,"1 xx 1\n[ [ 0, 1.5 ], [ 1, NaN ] ] [ [ 0, true ], [ 1, false ] ] [ [ 0, null ], [ 1, null ] ] [ [ 0, undefined ], [ 1, undefined ] ]\ntwo changed\nlast\n",None);
}

#[test]
fn array_entries_composes_with_array_from_spread_map_and_generics() {
 execute(r#"function identity<T>(value:Owned<T>):T{return value;}
function main(){const pairs=identity([4,5].entries());const copied=Array.from(pairs);
console.log(copied[1][0],copied[1][1]);const extra:[int,int][]= [...clone(pairs),[2,6]];console.log(extra.length,extra[2][1]);
const sums=pairs.map((pair:ReadOnly<[int,int]>,index:int):int=>{return pair[0]+pair[1]+index;});console.log(sums.join());
console.log([8].entries().map((pair:ReadOnly<[int,int]>):int=>{return pair[1];}).join());
}"#, "1 5\n3 6\n4,7\n8\n",None);
}

#[test]
fn array_entries_cannot_copy_callable_owners() {
 reject("function main(){const actions:(()=>int)[]=[():int=>{return 7;}];actions.entries();}","clone-type");
 reject("type Row={action:Once<Function<int>>;};function main(){const rows:Row[]=[{action:():int=>{return 7;}}];rows.entries();}","clone-type");
}

#[test]
fn array_entries_contextualizes_empty_sources_and_temporary_results() {
 execute(r#"function main(){const empty:[int,string][]=[].entries();console.log(empty.length);console.log(["fresh".repeat(2)].entries()[0][1]);const words=["a","b"];const literal=words.entries();console.log(literal[1][1]);}"#,"0\nfreshfresh\nb\n",None);
}

#[test]
fn array_entries_preserves_generic_results_snapshot_and_fixed_record_storage() {
 execute(r#"type Row={values:int[];};
function indexed<T>(values:ReadOnly<T[]>):[int,T][]{return values.entries();}
function main(){const source=[2,3];const pairs=indexed(source);source[0]=9;source.push(4);console.log(pairs.length,pairs[0][1],source.join());
const words=["a","b"];console.log(indexed(words)[1][1]);
const rows:Row[]=[{values:[5,6]},{values:[7]}];const entries=rows.entries();entries[0][1].values[0]=8;console.log(rows[0].values[0],entries[0][1].values[0]);}"#, "2 2 9,3,4\nb\n5 8\n",None);
}

#[test]
fn array_entries_late_tuple_selection_keeps_once_only_owned_generic_receivers() {
 execute(r#"function identity<T>(value:Owned<T>):T{return value;}
function make(log:int[]):[int,string]{log.push(1);return [2,"owned".repeat(2)];}
function main(){const log:int[]=[];const text=identity(make(log))[1];console.log(text,log.join());const maybe:[int,string]|null=[3,"present"];console.log(maybe![0],maybe![1]);}"#,"ownedowned 1\n3 present\n",None);
 reject("function main(){const pairs=[\"owned\".repeat(2)].entries();const text=pairs[0][1];}","borrowed-element");
}

#[test]
fn array_entries_materializes_pairs_consistently_in_recursive_functions() {
 execute(r#"function total(values:ReadOnly<int[]>,depth:int):int{const pairs=values.entries();if(depth===0){return pairs.length;}return pairs[0][1]+total(values,depth-1);}function main(){const values=[2,3];console.log(total(values,3),values.join());}"#,"8 2,3\n",None);
}
