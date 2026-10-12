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
