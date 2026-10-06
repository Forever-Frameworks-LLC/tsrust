mod support;
#[test]
fn for_patterns_bind_once_and_keep_loop_control_and_scope(){
 let source=r#"import {strictEqual} from "std:testing";
 type Bounds={start:int;end:int;};
 function make(trace:int[]):Bounds{trace.push(1);return {start:0,end:5};}
 function main(){const trace:int[]=[];const seen:int[]=[];
 for(let {start:i,end}=make(trace);i<end;i++){if(i===1){continue;}if(i===4){break;}seen.push(i);}
 strictEqual(trace.length,1);strictEqual(seen.join(","),"0,2,3");
 for(let [i,end=3]=[0];i<end;i++){seen.push(i);}
 strictEqual(seen.join(","),"0,2,3,0,1,2");
 const words=["a","b"];for(const [first,...tail]=words;tail.length<2;tail.push("c")){strictEqual(first,"a");strictEqual(tail[0],"b");}words.push("c");strictEqual(words.length,3);
 }"#;
 let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn initializer_pattern_names_do_not_escape_the_loop(){
 support::reject("function main(){for(let [i,end]=[0,2];i<end;i++){}console.log(i);}","name");
}
