mod support;
#[test]
fn nested_array_and_object_rest_targets_copy_then_bind(){
 let source=r#"import {strictEqual} from "std:testing";
 function describe([,...[a,b="tail"]]:string[]):string{return a+":"+b;}
 function main(){const words=["a","b","c"];{const [first,...[second,...others]]=words;strictEqual(first,"a");strictEqual(second,"b");strictEqual(others.join(","),"c");others.push("d");strictEqual(words.length,3);}words.push("d");strictEqual(words.length,4);const [...{length}]=words;strictEqual(length,4);strictEqual(describe(["a","b"]),"b:tail");const tuple:[int,string,int]=[1,"two",3];const [head,...[text,amount]]=tuple;strictEqual(head,1);strictEqual(text,"two");strictEqual(amount,3);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn nested_rest_keeps_last_position_default_and_depth_rules(){
 support::reject("function main(){const values=[1];const [...[a],b]=values;}","syntax");
 support::reject("function main(){const values=[1];const [...[a]=[2]]=values;}","syntax");
 let mut pattern="[a]".to_owned();for _ in 0..33{pattern=format!("[...{pattern}]");}
 let source=format!("function main(){{const values=[1];const {pattern}=values;}}");support::accept_deep(&source);
}
#[test]
fn nested_rest_targets_work_in_arrows_and_for_of(){
 let source=r#"import {strictEqual} from "std:testing";function main(){const upper=([,...[text="missing"]]:string[]):string=>text.toUpperCase();strictEqual(upper(["a","b"]),"B");strictEqual(upper(["a"]),"MISSING");const rows=[["a","b"],["c","d"]];const result:string[]=[];for(const [,...[word]] of rows){result.push(word.slice());}strictEqual(result.join(","),"b,d");}"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
