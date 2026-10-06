mod support;
#[test]
fn array_rest_explicitly_copies_without_consuming_source(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/destructuring-rest/arrays.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn rest_must_be_last_and_cannot_have_a_default(){
 support::reject("function main(){const [...rest,last]=[1,2];}","syntax");
 support::reject("function main(){const [...rest=[]]=[1,2];}","syntax");
}
#[test]
fn rest_copies_nested_owners_from_readonly_sources(){
 let source=r#"import {strictEqual} from "std:testing";
 type Row={name:string;values:int[];};
 function copy([...rest]:ReadOnly<Row[]>):Row[]{return rest;}
 function main(){const rows:Row[]=[{name:"a",values:[1,2]}];const [...rest]=rows;rest[0].values.push(3);strictEqual(rows[0].values.length,2);strictEqual(rest[0].values.length,3);const again=copy(rows);again[0].values.push(4);strictEqual(rows[0].values.length,2);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn rest_in_arrow_parameters_copies_primitive_union_alternatives(){
 let source=r#"import {strictEqual} from "std:testing";
 function main(){const values:(int|string)[]=[1,"a",2];const get=([,...rest]:ReadOnly<(int|string)[]>):(int|string)[]=>rest;const copied=get(values);strictEqual(copied.join(","),"a,2");copied.push("b");strictEqual(values.length,3);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn tuple_rest_retains_position_types_and_copies_owners(){
 let source=r#"import {strictEqual} from "std:testing";
 function tail([,...rest]:ReadOnly<[int,string,int[]]>):[string,int[]]{return rest;}
 function main(){const tuple:[int,string,int[]]=[1,"a",[2]];const rest=tail(tuple);rest[1].push(3);strictEqual(tuple[2].length,1);strictEqual(rest[0],"a");strictEqual(rest[1].length,2);const [first,...again]=tuple;strictEqual(first,1);strictEqual(again[0],"a");const [,,,...empty]=tuple;strictEqual(empty.length,0);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn tuple_rest_reads_only_remaining_fields_after_an_earlier_move(){
 let source=r#"import {strictEqual} from "std:testing";function main(){const value:[string,string]=["first","last"];const [first,...rest]=value;strictEqual(first,"first");strictEqual(rest[0],"last");strictEqual(value[1],"last");}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn empty_tuple_rest_does_not_read_transferred_fields(){
 let source=r#"import {strictEqual} from "std:testing";function main(){const value:[string]=["a"];const [first,...rest]=value;strictEqual(first,"a");strictEqual(rest.length,0);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn tuple_rest_rejects_reading_a_transferred_remaining_field(){
 support::reject(r#"function main(){const value:[int,string]=[1,"a"];const moved=value[1];const [first,...rest]=value;}"#,"partial-transfer");
}
#[test]
fn record_rest_copies_remaining_fields_in_constructor_order(){
 let source=r#"import {strictEqual} from "std:testing";
 type Row={name:string;count:int;values:int[];};
 type Remaining={count:int;values:int[];};
 function copy({name,...rest}:ReadOnly<Row>):Remaining{return rest;}
 function main(){const row:Row={values:[1],name:"a",count:2};const {name:label,...rest}=row;strictEqual(label,"a");strictEqual(Object.keys(rest).join(","),"values,count");rest.values.push(3);strictEqual(row.values.length,1);strictEqual(rest.count,2);const again=copy({count:4,values:[5],name:"b"});strictEqual(Object.keys(again).join(","),"count,values");strictEqual(again.count,4);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn record_rest_excludes_defaulted_missing_names(){
 let source=r#"import {strictEqual} from "std:testing";type Row={value:int;};function main(){const row:Row={value:2};const {missing=1,...rest}=row;strictEqual(missing,1);strictEqual(rest.value,2);const {value,...empty}=row;strictEqual(Object.keys(empty).length,0);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn tuple_rest_copies_scalar_positions(){
 let source=r#"import {strictEqual} from "std:testing";function main(){const value:[int,int,boolean]=[1,2,true];const [first,...rest]=value;strictEqual(rest[0],2);strictEqual(rest[1],true);strictEqual(value[1],2);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn record_rest_works_in_arrows_nested_patterns_and_loops(){
 let source=r#"import {strictEqual} from "std:testing";type Row={name:string;count:int;};type Box={row:Row;};
 function main(){const row:Row={count:2,name:"a"};const arrow=({name,...rest}:ReadOnly<Row>):int=>rest.count;strictEqual(arrow(row),2);const box:Box={row:row};const {row:{name,...rest}}=box;strictEqual(name,"a");strictEqual(rest.count,2);const rows:Row[]=[{name:"b",count:3}];const counts:int[]=[];for(const {name:loopName,...loopRest} of rows){counts.push(loopRest.count);}strictEqual(counts.join(","),"3");}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn record_rest_rejects_invalid_placement_and_transferred_remaining_fields(){
 support::reject("type Row={x:int;};function main(){const row:Row={x:1};const {...rest,x}=row;}","syntax");
 support::reject("type Row={x:int;};function main(){const row:Row={x:1};const {...rest={}}=row;}","syntax");
 support::reject(r#"type Row={name:string;count:int;};function main(){const row:Row={name:"a",count:1};const moved=row.name;const {count,...rest}=row;}"#,"partial-transfer");
}
#[test]
fn object_spread_copies_sources_merges_and_preserves_first_key_order(){
 let source=r#"import {strictEqual} from "std:testing";type Row={name:string;count:int;};type Extra={count:int;active:boolean;};
 function main(){const row:Row={count:2,name:"a"};const copied={...row};strictEqual(copied.name,"a");strictEqual(row.count,2);const extra:Extra={active:true,count:7};const merged={before:1,...row,count:3,...extra,after:4};strictEqual(merged.count,7);strictEqual(merged.active,true);strictEqual(Object.keys(merged).join(","),"before,count,name,active,after");strictEqual(Object.keys(row).join(","),"count,name");}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn object_spread_evaluates_sources_and_fields_once_left_to_right(){
 let source=r#"import {strictEqual} from "std:testing";type Row={value:int;};
 function make(trace:int[],value:int):Row{trace.push(value);return {value:value};}
 function next(trace:int[],value:int):int{trace.push(value);return value;}
 function main(){const trace:int[]=[];const result={...make(trace,1),value:next(trace,2),...make(trace,3),last:next(trace,4)};strictEqual(trace.join(","),"1,2,3,4");strictEqual(result.value,3);strictEqual(result.last,4);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn object_spread_copies_readonly_nested_owners_and_respects_annotations(){
 let source=r#"import {strictEqual} from "std:testing";type Row={name:string;values:int[];};
 function copy(value:ReadOnly<Row>):Row{return {...value};}
 function main(){const row:Row={values:[1],name:"a"};const copied=copy(row);copied.values.push(2);strictEqual(row.values.length,1);strictEqual(copied.values.length,2);const reversed:Row={name:"b",...row};strictEqual(reversed.name,"a");strictEqual(Object.keys(reversed).join(","),"name,values");const changed={...row,name:7};strictEqual(changed.name,7);strictEqual(row.name,"a");}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn object_spread_handles_empty_records_and_inline_sources(){
 let source=r#"import {strictEqual} from "std:testing";type Empty={};function main(){const empty:Empty={};const copy={...empty};strictEqual(Object.keys(copy).length,0);const result={...{x:1,y:"a"},x:2};strictEqual(result.x,2);strictEqual(result.y,"a");}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn object_spread_rejects_nonrecords_partial_sources_and_wrong_annotations(){
 support::reject("function main(){const result={...[1,2]};}","type");
 support::reject(r#"type Row={name:string;};function main(){const row:Row={name:"a"};const moved=row.name;const copy={...row};}"#,"partial-transfer");
 support::reject("type Row={x:int;};function main(){const source:Row={x:1};const result:Row={...source,extra:2};}","type");
 support::reject(r#"type Row={x:int;};function main(){const source:Row={x:1};const result:Row={...source,x:"a"};}"#,"type");
}
#[test]
fn annotated_object_spread_checks_the_final_overridden_type(){
 let source=r#"import {strictEqual} from "std:testing";type Row={x:int;};function main(){const value:Row={x:"ignored",...{x:2}};strictEqual(value.x,2);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn object_spread_widens_source_fields_into_annotated_unions(){
 let source=r#"import {strictEqual} from "std:testing";type Source={x:int;};type Result={x:int|string;};function main(){const source:Source={x:2};const result:Result={...source};strictEqual(String(result.x),"2");}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn object_spread_checks_only_final_union_field_alternatives(){
 let source=r#"import {strictEqual} from "std:testing";type Source={x:int;};type Result={x:int|undefined;};function main(){const source:Source={x:2};const result:Result={x:"ignored",...source};strictEqual(String(result.x),"2");}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn object_spread_finishes_copy_before_later_source_mutation(){
 let source=r#"import {strictEqual} from "std:testing";type Row={x:int;};function change(row:Mutable<Row>):int{row.x=9;return 9;}function main(){const row:Row={x:1};const result={...row,after:change(row)};strictEqual(result.x,1);strictEqual(result.after,9);strictEqual(row.x,9);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn object_spread_copies_nested_spread_results_once(){
 let source=r#"import {strictEqual} from "std:testing";type Row={x:int;};function main(){const row:Row={x:1};const result={...{...row},y:2};strictEqual(result.x,1);strictEqual(result.y,2);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
