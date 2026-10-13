//! Public recursive-schema acceptance; implementation starts from these witnesses.
mod support;

#[test]
fn inferred_recursive_schema_validates_nested_json_and_infers_access() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
const tree=s.recursive(self=>s.object({value:s.int(),children:s.array(self)}));
function main(){
  match(tree.safeParseJson('{"value":1,"children":[{"value":2,"children":[]}]}')){
    Ok(value)=>{strictEqual(value.value,1);strictEqual(value.children[0].value,2);},
    Err(error)=>{throw 'Valid recursive data rejected';}
  }
  match(tree.safeParseJson('{"value":1,"children":[{"value":"bad","children":[]}]}')){
    Ok(value)=>{throw 'Invalid nested value accepted';},
    Err(error)=>{
      if(error.kind==='validation'){
        strictEqual(error.error.issues[0].path[0].field,'children');
        strictEqual(error.error.issues[0].path[1].index,0);
        strictEqual(error.error.issues[0].path[2].field,'value');
      }else{throw 'Unexpected JSON error';}
    }
  }
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn explicit_recursive_schema_supports_nullable_links_and_multiple_self_uses() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
type Tree={value:int;left:Tree|null;right:Tree|null;};
const tree=s.recursive<Tree>(self=>s.object({value:s.int(),left:self.nullable(),right:self.nullable()}));
function main(){
  match(tree.safeParseJson('{"value":7,"left":null,"right":null}')){
    Ok(value)=>{strictEqual(value.value,7);strictEqual(value.left,null);strictEqual(value.right,null);},
    Err(error)=>{throw 'Valid explicit recursive data rejected';}
  }
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn recursive_builder_runs_once_and_owned_validation_reuses_the_plan() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
let builds:int=0;
const tree=s.recursive(self=>{
  builds+=1;
  return s.object({value:s.int(),children:s.array(self)});
});
function main(){
  strictEqual(builds,1);
  const first={value:3,children:[{value:4,children:[]}]};
  match(tree.safeParse(first)){
    Ok(value)=>{strictEqual(value.children[0].value,4);},
    Err(error)=>{throw 'Owned recursive input rejected';}
  }
  match(tree.safeParseJson('{"value":5,"children":[]}')){
    Ok(value)=>{strictEqual(value.value,5);},
    Err(error)=>{throw 'Reusable recursive plan rejected';}
  }
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
  match(arrays.safeParseJson('[[],[[]]]')){
    Ok(value)=>{strictEqual(value.length,2);strictEqual(value[1][0].length,0);},
    Err(error)=>{throw 'Recursive array root rejected';}
  }
  match(nested.safeParseJson('[1,[2,[]]]')){
    Ok(value)=>{},
    Err(error)=>{throw 'Recursive union root rejected';}
  }
  match(nested.safeParseJson('[1,["bad"]]')){
    Ok(value)=>{throw 'Invalid recursive union accepted';},
    Err(error)=>{}
  }
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
fn recursive_transforms_preserve_distinct_input_and_output_shapes() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
const tree=s.recursive(self=>s.object({
  value:s.string().transform(value=>value.length),
  children:s.array(self)
}));
function main(){
  const input={value:'root',children:[{value:'kid',children:[]}]};
  match(tree.safeParse(input)){
    Ok(value)=>{strictEqual(value.value,4);strictEqual(value.children[0].value,3);},
    Err(error)=>{throw 'Recursive transform rejected owned input';}
  }
  match(tree.safeParseJson('{"value":"root","children":[{"value":"kid","children":[]}]}')){
    Ok(value)=>{strictEqual(value.value,4);strictEqual(value.children[0].value,3);},
    Err(error)=>{throw 'Recursive transform rejected JSON input';}
  }
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}
