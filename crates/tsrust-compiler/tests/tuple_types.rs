mod support;
use support::{execute, reject};

#[test]
fn tuple_types_preserve_positions_and_array_identity() {
    execute(
        r#"function pair(value:ReadOnly<string>):[int,string]{return [3,value.slice()];}
function main(){
 const item:[int,string]=pair("cat");
 console.log(item[0],item[1],item.length,Array.isArray(item));
 const rows:[int,string][]=[[0,"first"],[1,"second"]];
 console.log(rows[0][0],rows[1][1],rows.length);
 console.log(item,rows);
}"#,
        "3 cat 2 true\n0 second 2\n[ 3, 'cat' ] [ [ 0, 'first' ], [ 1, 'second' ] ]\n",
        None,
    );
}

#[test]
fn tuple_types_check_length_positions_and_bounds() {
    reject("function main(){const item:[int,string]=[1];}", "type");
    reject("function main(){const item:[int,string]=[1,2];}", "type");
    reject("function main(){const item:[int,string]=[1,\"x\",3];}", "type");
    reject("function main(){const item:[int,string]=[1,\"x\"];console.log(item[2]);}", "bounds");
}

#[test]
fn tuple_types_mutate_copy_clone_and_transfer_owned_positions() {
    execute(r#"function replace(item:Mutable<[int,string]>){item[0]=9;item[1]="new";}
function make():[int,string]{return [4,"temporary".repeat(2)];}
function main(){let item:[int,string]=[1,"old".repeat(2)];replace(item);
console.log(item[0],item[1],make()[0],make().length);
const copy=clone(item);const text=item[1];console.log(text,copy[1]);
item[1]="restored";console.log(item[0],item[1]);
const empty:[]=[];console.log(empty.length,Array.isArray(empty),empty);
const nested:[[int,string],boolean]=[[2,"nest"],true];console.log(nested[0][1],nested[1]);
}"#, "9 new 4 2\nnew new\n9 restored\n0 true []\nnest true\n",None);
    reject("function main(){const item:[int,string]=[1,\"x\".repeat(2)];const text=item[1];console.log(item);}", "partial-transfer");
}

#[test]
fn tuple_types_are_structural_through_generics_and_record_fields() {
    execute(r#"type Box<T>={value:T;};
function identity<T>(value:Owned<T>):T{return value;}
function first<T>(value:ReadOnly<[T,string]>):T{return value[0];}
function same(value:ReadOnly<[(int),string]>):int{return value[0];}
function main(){const item:[int,string]=[5,"five"];console.log(first(item),same(item));
const box:Box<[int,string]>={value:identity<[int,string]>([6,"six"])};
console.log(box.value[0],box.value[1]);
}"#, "5 5\n6 six\n",None);
}

#[test]
fn tuple_types_use_array_text_and_recursive_locale_display() {
 execute(r#"function main(){const item:[int,string,boolean,null,undefined]=[1234,"x",true,null,undefined];console.log(String(item));const rows:[int,string][]=[[1000,"a"],[2000,"b"]];console.log(rows.join(";"),rows.toLocaleString(),String(rows));}"#,
 "1234,x,true,,\n1000,a;2000,b 1,000,a,2,000,b 1000,a,2000,b\n",None);
}

#[test]
fn tuple_types_handle_nullable_positions_and_whole_tuple_views() {
 execute(r#"function choose(present:boolean):[int,string]|undefined{return present?[7,"seven"]:undefined;}
function main(){const item:[int,string|null,boolean|undefined]=[1,null,undefined];console.log(String(item));
const rows:[int,string|null][]=[[2,null],[3,"x"]];console.log(rows.join(";"));
const value=choose(true);console.log(Array.isArray(value),String(value));const missing=choose(false);console.log(Array.isArray(missing),String(missing));}"#,
 "1,,\n2,;3,x\ntrue 7,seven\nfalse undefined\n",None);
}

#[test]
fn tuple_types_reject_flattening_without_a_homogeneous_result_type() {
 reject("function main(){const rows:[int,string][]=[[1,\"x\"]];rows.flat();}","unsupported");
 reject("function main(){const values=[1];values.flatMap((value:int):[int,string]=>{return [value,\"x\"];});}","unsupported");
 execute("function main(){const rows:[int,string][]=[[1,\"x\"]];console.log(rows.flat(0));}","[ [ 1, 'x' ] ]\n",None);
}

#[test]
fn tuple_types_keep_formatter_idempotent_and_bound_annotation_depth() {
 let source="function pair(value:ReadOnly<[int,string]>):[int,string] { return [value[0],value[1].slice()]; } function main(){const rows:[int,string][]=[[1,\"x\"]];console.log(pair(rows[0]));}";
 let formatted=tsrust_compiler::format(source).unwrap();
 assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
 execute(&formatted,"[ 1, 'x' ]\n",None);
 let annotation=format!("{}int{}","[".repeat(2000),"]".repeat(2000));
 assert!(tsrust_compiler::type_syntax::parse(&annotation).is_ok());
}

#[test]
fn tuple_types_preserve_nominal_members_across_project_imports() {
 use tsrust_compiler::project::{SourceId,SourceProvider,load_project,check_graph,ProjectType};
 struct Files;
 impl SourceProvider for Files {
  fn resolve(&self,_:Option<&SourceId>,path:&str)->Result<SourceId,String>{Ok(SourceId(path.trim_start_matches("./").into()))}
  fn read(&self,id:&SourceId)->Result<String,String>{Ok(match id.0.as_str(){
   "main.tsr"=>"import {pair, User} from './dep.tsr';function main(){const item:[User,string]=pair();console.log(item[0].name,item[1]);}",
   "dep.tsr"=>"export type User={name:string;};export function pair():[User,string]{return [{name:\"Ada\"},\"tuple\"];}",
   _=>return Err("missing source".into()),
  }.into())}
 }
 let project=check_graph(load_project(&Files,"main.tsr").unwrap()).unwrap();
 let facts=project.checked_type_facts();
 assert!(facts.method_results.iter().any(|fact|matches!(&fact.ty,ProjectType::Tuple(positions) if matches!(&positions[0],ProjectType::Named{name,declaration:Some(origin),..} if name=="User" && origin.source.0=="dep.tsr"))));
 let directory=support::TempDirectory::new();let input=directory.0.join("project.rs");let binary=directory.0.join("project");std::fs::write(&input,project.emit_rust()).unwrap();
 support::success(&support::run(support::rustc().args(["--edition=2024","-Awarnings"]).arg(&input).arg("-o").arg(&binary),60));
 let output=support::run(&mut std::process::Command::new(binary),10);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),"Ada tuple\n");
}

#[test]
fn tuple_types_contextualize_present_literals_in_nullable_results() {
 execute(r#"function direct():[int,string]|undefined{return [8,"eight"];}
function main(){const value:[int,string]|null=[9,"nine"];console.log(String(value),String(direct()));}"#,"9,nine 8,eight\n",None);
}

#[test]
fn tuple_types_accept_bounded_deep_products_on_normal_test_stack() {
 let depth=100;
 let annotation=format!("{}int{}","[".repeat(depth),"]".repeat(depth));
 let value=format!("{}1{}","[".repeat(depth),"]".repeat(depth));
 let source=format!("function main(){{const value:{annotation}={value};console.log(value{});}}","[0]".repeat(depth));
 execute(&source,"1\n",None);
}
