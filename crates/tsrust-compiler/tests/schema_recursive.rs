//! Public recursive-schema acceptance; implementation starts from these witnesses.
mod support;

#[test]
fn inferred_recursive_schema_validates_nested_json_and_infers_access() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
const tree=s.recursive(self=>s.object({value:s.int(),children:s.array(self)}));
function main(){
  {const outcome0=tree.safeParseJson('{"value":1,"children":[{"value":2,"children":[]}]}');if(outcome0.success){const value=outcome0.data;strictEqual(value.value,1);strictEqual(value.children[0].value,2);}else{const error=outcome0.error;throw 'Valid recursive data rejected';}}
  {const outcome1=tree.safeParseJson('{"value":1,"children":[{"value":"bad","children":[]}]}');if(outcome1.success){const value=outcome1.data;throw 'Invalid nested value accepted';}else{const error=outcome1.error;
      if(error.kind==='validation'){
        strictEqual(error.error.issues[0].path[0].field,'children');
        strictEqual(error.error.issues[0].path[1].index,0);
        strictEqual(error.error.issues[0].path[2].field,'value');
      }else{throw 'Unexpected JSON error';}
    }}
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn explicit_recursive_schema_supports_nullable_links_and_multiple_self_uses() {
    let source=r#"
import {s as builders} from 'std:schema';
import {strictEqual} from 'std:testing';
type Tree={right:Tree|null;value:int;left:Tree|null;};
const tree=builders.recursive<Tree>(self=>builders.object({value:builders.int(),left:self.nullable(),right:self.nullable()}));
function main(){
  {const outcome2=tree.safeParseJson('{"value":7,"left":null,"right":null}');if(outcome2.success){const value=outcome2.data;
      const named:Tree=value;
      strictEqual(named.value,7);strictEqual(named.left,null);strictEqual(named.right,null);
      strictEqual(Object.keys(named).join(','),'value,left,right');
    }else{const error=outcome2.error;throw 'Valid explicit recursive data rejected';}}
  {const outcome3=tree.safeParseJson('{"value":7,"left":{"value":9,"left":null,"right":null},"right":null}');if(outcome3.success){const value=outcome3.data;
      const named:Tree=value;
      if(named.left!==null){strictEqual(named.left.value,9);}else{throw 'Lost recursive child';}
    }else{const error=outcome3.error;throw 'Explicit recursive child rejected';}}
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn recursive_owned_builder_runs_once_and_validation_reuses_the_plan() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
type OwnedTree={value:int;children:OwnedTree[];};
let builds:int=0;
const tree=s.recursive(self=>{
  builds+=1;
  return s.object({value:s.int(),children:s.array(self)});
});
function main(){
  strictEqual(builds,1);
  const first:OwnedTree={value:3,children:[{value:4,children:[]}]};
  {const outcome4=tree.safeParse(first);if(outcome4.success){const value=outcome4.data;strictEqual(value.children[0].value,4);}else{const error=outcome4.error;throw 'Owned recursive input rejected';}}
  {const outcome5=tree.safeParseJson('{"value":5,"children":[]}');if(outcome5.success){const value=outcome5.data;strictEqual(value.value,5);}else{const error=outcome5.error;throw 'Reusable recursive plan rejected';}}
  strictEqual(builds,1);
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn inferred_recursive_array_and_union_roots_validate_without_named_types() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
const arrays=s.recursive(self=>s.array(self));
const nested=s.recursive(self=>s.union([s.int(),s.array(self)]));
function main(){
  {const outcome6=arrays.safeParseJson('[[],[[]]]');if(outcome6.success){const value=outcome6.data;
      strictEqual(value.length,2);
      strictEqual(value[1][0].length,0);
      const copied=clone(value);
      strictEqual(copied[1][0].length,0);
      strictEqual(value.length,2);
    }else{const error=outcome6.error;throw 'Recursive array root rejected';}}
  {const outcome7=nested.safeParseJson('[1,[2,[]]]');if(outcome7.success){const value=outcome7.data;}else{const error=outcome7.error;throw 'Recursive union root rejected';}}
  {const outcome8=nested.safeParseJson('[1,["bad"]]');if(outcome8.success){const value=outcome8.data;throw 'Invalid recursive union accepted';}else{const error=outcome8.error;}}
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn recursive_placeholders_reject_unproductive_definitions_and_escape() {
    for (definition,code) in [
        ("const schema=s.recursive(self=>self);","schema-recursion"),
        ("const schema=s.recursive(self=>s.union([self,s.int()]));","schema-recursion"),
        ("const schema=s.recursive(self=>()=>self);","schema-recursion"),
        ("const schema=s.recursive(self=>s.set(self));","set-key"),
        ("const schema=s.recursive(self=>s.map(self,s.int()));","map-key"),
        ("type Tree={children:Map<string,Tree>;value:string;};const schema=s.recursive<Tree>(self=>s.object({children:s.record(self),value:s.int()}));","schema-type"),
        ("type Tree={value:int;children:Tree[];};const schema=s.recursive<Tree>(self=>s.object({value:s.string(),children:s.array(self)}));","schema-type"),
    ] {
        let source=format!("import {{s}} from 'std:schema';{definition} function main(){{}}");
        let directory=support::TempDirectory::new();
        let file=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(file),30);
        let diagnostics=String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(),"Invalid recursive schema accepted: {definition}");
        assert!(diagnostics.contains(code),"Expected {code} for {definition}: {diagnostics}");
    }
}

#[test]
fn recursive_owned_transforms_preserve_distinct_input_and_output_shapes() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
type InputTree={value:string;children:InputTree[];};
const tree=s.recursive(self=>s.object({
  value:s.string().transform(value=>value.length),
  children:s.array(self)
}));
function main(){
  const input:InputTree={value:'root',children:[{value:'kid',children:[]}]};
  {const outcome9=tree.safeParse(input);if(outcome9.success){const value=outcome9.data;strictEqual(value.value,4);strictEqual(value.children[0].value,3);}else{const error=outcome9.error;throw 'Recursive transform rejected owned input';}}
  {const outcome10=tree.safeParseJson('{"value":"root","children":[{"value":"kid","children":[]}]}');if(outcome10.success){const value=outcome10.data;strictEqual(value.value,4);strictEqual(value.children[0].value,3);}else{const error=outcome10.error;throw 'Recursive transform rejected JSON input';}}
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn recursive_owned_input_rejects_mismatch_after_recursive_edge() {
    let source=r#"
import {s} from 'std:schema';
type Wrong={children:Wrong[];value:string;};
const tree=s.recursive(self=>s.object({children:s.array(self),value:s.int()}));
function main(){
  const input:Wrong={children:[],value:'wrong'};
  tree.safeParse(input);
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("check").arg(file).arg("--json"),10);
    assert_eq!(output.status.code(),Some(1));
    let diagnostics=String::from_utf8(output.stdout).unwrap();
    assert!(diagnostics.contains("\"code\":\"type\""),"{diagnostics}");
}

#[test]
fn recursive_record_collections_validate_nested_values_and_paths() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
const tree=s.recursive(self=>s.object({value:s.int(),children:s.record(self)}));
function main(){
  {const outcome11=tree.safeParseJson('{"value":1,"children":{"leaf":{"value":2,"children":{}}}}');if(outcome11.success){const value=outcome11.data;strictEqual(value.value,1);strictEqual(value.children.size,1);}else{const error=outcome11.error;throw 'Recursive record collection rejected';}}
  {const outcome12=tree.safeParseJson('{"value":1,"children":{"leaf":{"value":"bad","children":{}}}}');if(outcome12.success){const value=outcome12.data;throw 'Invalid recursive record collection accepted';}else{const error=outcome12.error;
      if(error.kind==='validation'){
        strictEqual(error.error.issues[0].path[0].field,'children');
        strictEqual(error.error.issues[0].path[1].field,'leaf');
        strictEqual(error.error.issues[0].path[2].field,'value');
      }else{throw 'Unexpected JSON error';}
    }}
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn recursive_collections_accept_owned_input_and_named_output() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
type Tree={value:int;children:Map<string,Tree>;};
type Leaf={count:int;};
type Branch={children:Map<string,Leaf>;};
const bag=s.recursive<Branch>(self=>s.object({children:s.record(s.object({count:s.int()}))}));
const schema=s.recursive<Tree>(self=>s.object({children:s.record(self),value:s.int()}));
function main(){
  {const outcome13=bag.safeParseJson('{"children":{"leaf":{"count":4}}}');if(outcome13.success){const value=outcome13.data;const named:Branch=value;strictEqual(named.children.size,1);}else{const error=outcome13.error;throw 'Nested named Map output rejected';}}
  const leaf:Tree={value:2,children:new Map<string,Tree>()};
  let children=new Map<string,Tree>();
  children.set('leaf',leaf);
  const input:Tree={value:1,children:children};
  {const outcome14=schema.safeParse(input);if(outcome14.success){const value=outcome14.data;const named:Tree=value;strictEqual(named.value,1);strictEqual(named.children.size,1);}else{const error=outcome14.error;throw 'Owned recursive collection rejected';}}
  {const outcome15=schema.safeParseJson('{"value":3,"children":{}}');if(outcome15.success){const value=outcome15.data;const named:Tree=value;strictEqual(named.value,3);strictEqual(named.children.size,0);}else{const error=outcome15.error;throw 'Named recursive collection rejected';}}
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn recursive_schema_composes_with_nested_definition_and_multi_issue_refinement() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
function main(){
  const schema=s.recursive(self=>s.object({
    value:s.int(),children:s.array(self),
    metadata:s.recursive(entry=>s.object({label:s.string(),children:s.array(entry)}))
  })).superRefine((value,ctx)=>{
    if(value.value<0){ctx.addIssue({path:['value'],message:'Negative root'});}
  });
  {const outcome16=schema.safeParseJson('{"value":2,"children":[],"metadata":{"label":"root","children":[{"label":"child","children":[]}]}}');if(outcome16.success){const value=outcome16.data;strictEqual(value.metadata.children[0].label,'child');}else{const error=outcome16.error;throw 'Nested definition rejected';}}
  {const outcome17=schema.safeParseJson('{"value":-1,"children":[],"metadata":{"label":"root","children":[]}}');if(outcome17.success){const value=outcome17.data;throw 'Refinement bypassed';}else{const error=outcome17.error;if(error.kind==='validation'){strictEqual(error.error.issues[0].message,'Negative root');strictEqual(error.error.issues[0].path[0].field,'value');}else{throw 'Wrong error';}}}
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}
