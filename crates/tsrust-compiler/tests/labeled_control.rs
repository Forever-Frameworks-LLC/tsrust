mod support;
#[test]
fn labeled_loops_blocks_and_outer_updates_match_node() {
 let source=r#"import { strictEqual } from "std:testing";
 function take(value:string):string{return value;}
 function main(){
  empty:;
  let owned="a";moveLoop:for(let i=0;i<2;i++){for(;;){take(owned);owned="b";continue moveLoop;}}strictEqual(owned,"b");
  let trace="";let updates=0;
  outer:for(let i=0;i<4;i++,updates++){
   for(let j=0;j<3;j++){
    if(i===1)continue outer;
    if(i===3)break outer;
    trace+=String(i)+String(j);
   }
  }
  strictEqual(trace,"000102202122");strictEqual(updates,3);
  let count=0;again:while(count<3){count++;for(const x of [1,2]){if(x===1)continue again;count+=10;}}
  strictEqual(count,3);
  let post=0;postLoop:do{post++;for(;;){continue postLoop;}}while(post<2);strictEqual(post,2);
  let once=0;first:second:for(const value of [1,2,3]){once+=value;for(;;){break first;}}
  strictEqual(once,1);
  let delayed:int;ready:{if(true){delayed=4;break ready;}else{delayed=7;}}strictEqual(delayed,4);
  let block=0;done:{block=1;for(let i=0;i<1;i++){break done;}block=2;}strictEqual(block,1);
  chosen:if(true){block=3;break chosen;}else{block=4;}strictEqual(block,3);
  scan:for(const key in {x:1,y:2}){switch(key){case "x":break scan;default:break;}}
  strictEqual(block,3);
 }
 "#;
 let directory=support::TempDirectory::new();let path=directory.source(source);
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn labels_preserve_scope_and_outer_loop_ownership() {
 for (source,code) in [
  ("function main(){let value:int;again:do{continue again;}while(value>0);}","unassigned"),
  ("function main(){let value:int;again:for(let i=0;i<1;value++){continue again;}}","unassigned"),

  ("function main(){let value:int;done:{break done;}console.log(value);}","unassigned"),
  ("function main(){let value:int;done:{if(true){break done;}else{value=1;}}console.log(value);}","unassigned"),

  ("function main(){while(true){break missing;}}","loop-control"),
  ("function main(){done:{continue done;}}","loop-control"),
  ("function main(){same:while(true){same:while(true){break;}}}","duplicate"),
  ("function main(){outer:while(true){const f=()=>{break outer;};f();}}","loop-control"),
  ("function take(value:string):string{return value;}function main(){const value=\"a\";outer:while(true){while(true){take(value);continue outer;}}}","loop-transfer"),
  ("function take(value:string):string{return value;}function main(){const value=\"a\";outer:for(let i=0;i<2;i++){for(;;){take(value);continue outer;}}}","loop-transfer"),
  ("function main(){label:const value=1;}","syntax"),
  ("function main(){same:same:while(true){break same;}}","duplicate"),
  ("function take(value:string):string{return value;}function main(){const value=\"a\";outer:for(let i=0;i<2;i++){for(;;){take(value);break outer;}}console.log(value);}","transferred"),
 ]{let diagnostic=tsrust_compiler::check(source).err().unwrap_or_else(||panic!("Accepted rejection witness: {source}"));assert_eq!(diagnostic.code,code,"{source}: {diagnostic:?}");support::reject(source,code);}
}

#[test]
fn label_alias_limit_is_bounded() {
 let labels=(0..33).map(|n|format!("label{n}: ")).collect::<String>();
 support::reject(&format!("function main(){{{labels}while(true){{break label0;}}}}"),"unsupported");
 let source=labels.replace("label32: ","")+"while(true){break label31;}";
 tsrust_compiler::check(&format!("function main(){{{source}}}")).unwrap();
}
