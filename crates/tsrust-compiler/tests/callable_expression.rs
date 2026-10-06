mod support;
#[test]
fn factory_results_iifes_and_conditional_callees_match_node(){
 let source=r#"import { strictEqual } from "std:testing";
 function make(base:int):Function<int,[value:int]>{return (value:int):int=>base+value;}
 function mark(trace:int[],value:int):int{trace.push(value);return value;}
 function factory(trace:int[]):Function<int,[value:int]>{trace.push(1);return (value:int):int=>value+10;}
 function chain(a:int):Function<Function<int,[c:int]>,[b:int]>{return (b:int):Function<int,[c:int]> => (c:int):int=>a+b+c;}
 function main(){
  strictEqual(chain(1)(2)(3),6);
  const list=[1];(()=>{list.push(2);})();strictEqual(list.length,2);
  strictEqual(make(10)(2),12);
  strictEqual(((value:int):int=>value+1)(3),4);
  strictEqual((function(value:int):int{return value*2;})(5),10);
  const trace:int[]=[];strictEqual(factory(trace)(mark(trace,2)),12);strictEqual(trace.join(","),"1,2");
  const first=make(1);const second=make(2);
  strictEqual((true?first:second)(7),8);
 }
 "#;
 let directory=support::TempDirectory::new();let path=directory.source(source);
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn expression_callees_keep_generic_defaults_rest_and_spread(){
 let source=r#"import { strictEqual } from "std:testing";
 type Point={x:int;};
 type Box={apply:Function<int,[value:int]>;};
 function make<T>(value:T):Function<T,[next:T]>{return (next:T):T=>next;}
 function main(){
  strictEqual(make(1)(3),3);
  strictEqual(((value:Point):int=>value.x)({x:1}),1);
  strictEqual(((value)=>value)(4),4);
  strictEqual(((value)=>value)(...[undefined]),undefined);
  strictEqual(((value:int=7):int=>value)(),7);
  strictEqual(((...values:int[]):int=>values.length)(1,2,3),3);
  const args=[2,3];strictEqual(((a:int,b:int):int=>a+b)(...args),5);
  const actions:(()=>int)[]=[():int=>7,():int=>9];strictEqual(actions[0](),7);
  strictEqual(((a,b)=>a+b)(...args),5);
  const box:Box={apply:(value:int):int=>value+1};
  strictEqual(box.apply(4),5);
 }
 "#;
 let directory=support::TempDirectory::new();let path=directory.source(source);
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn expression_calls_keep_ownership_and_argument_contracts(){
 for(source,code) in [
  ("function main(){const a=[1];const b=[\"x\"];((x)=>x)(...a,...b);}","parameter-inference"),
  ("function main(){const values=[1];const change=(()=>()=>{values.push(2);})();console.log(values.length);change();}","transferred"),
  ("function main(){((value)=>value.x)({x:1});}","parameter-inference"),
  ("function main(){((x:int):int=>x)(\"wrong\");}","type"),
  ("function main(){((x:int):int=>x)();}","arity"),
  ("function make():int{return 1;}function main(){make()(2);}","type"),
  ("function main(){const text=\"a\";(()=>text)();console.log(text);}","transferred"),
 ]{support::reject(source,code);}
}
