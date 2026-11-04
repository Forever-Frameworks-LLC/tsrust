mod support;

fn run(source:&str,direct:bool){
 let source=support::format_round_trip(source);let dir=support::TempDirectory::new();let path=dir.source(&source);
 let emitted=support::run(support::cli().arg("emit").arg(&path),60);support::success(&emitted);
 assert_eq!(String::from_utf8(emitted.stdout).unwrap().contains("direct_json_pipeline"),direct);
 support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn recursive_json_finalization_preserves_named_data_paths_and_first_failure(){
 run(r#"
import {s} from 'std:schema';import {strictEqual} from 'std:testing';
type Tree={label:string;value:int;children:Tree[];};
const tree=s.recursive<Tree>(self=>s.object({value:s.int().min(0),label:s.string(),children:s.array(self)}));
function main(){
 for(const text of ['{"value":9007199254740993,"label":"é","children":[{"value":2,"label":"leaf","children":[]}]}','{"value":0,"label":"empty","children":[]}']){
  const result=tree.safeParse(JSON.parse(text));if(result.success){
   const value:Tree=result.data;strictEqual(value.label,text.includes('leaf')?'é':'empty');
   strictEqual(Object.keys(value).join(','),'value,label,children');
   const copied=clone(value);strictEqual(JSON.stringify(copied),JSON.stringify(value));
   if(value.children.length>0){strictEqual(value.value,9007199254740993);strictEqual(value.children[0].value,2);}
  }else{throw 'valid tree rejected';}
 }
 for(const text of ['{"label":"x","children":[]}','{"value":-1,"label":"x","children":[]}','{"value":1,"label":"x","children":0}','{"value":1,"label":"x","children":[{"value":"bad","label":"x","children":[]}]}']){
  const result=tree.safeParse(JSON.parse(text));const reference=tree.safeParseJson(text);
  if(result.success){throw 'invalid tree accepted';}else{
   strictEqual(result.error.issues.length,1);strictEqual(result.error.truncated,false);
   if(reference.success){throw 'text validator disagrees';}else{if(reference.error.kind==='validation'){
    strictEqual(JSON.stringify(result.error),JSON.stringify(reference.error.error));
   }else{throw 'unexpected JSON error';}}
  }
 }
 const parsed=tree.parse(JSON.parse('{"value":1,"label":"parse","children":[]}'));strictEqual(parsed.label,'parse');
 const first=tree.safeParse(JSON.parse('{"value":"bad","label":false,"children":0}'));
 if(first.success){throw 'bad fields accepted';}else{strictEqual(first.error.issues.length,1);strictEqual(first.error.issues[0].path[0].field,'value');}
 const unknown=tree.safeParse(JSON.parse('{"value":1,"label":"x","children":[],"z":0,"a":0}'));
 if(unknown.success){throw 'unknown fields accepted';}else{strictEqual(unknown.error.issues.length,2);strictEqual(unknown.error.issues[0].path[0].field,'a');strictEqual(unknown.error.issues[1].path[0].field,'z');}
}
"#,true);
}

#[test]
fn recursive_json_finalization_handles_array_scalar_and_strip_roots(){
 run(r#"
import {s} from 'std:schema';import {strictEqual} from 'std:testing';
const arrays=s.recursive(self=>s.array(self));const scalar=s.recursive(self=>s.string());const empty=s.recursive(self=>s.object({}));
const nested=s.recursive(self=>s.object({value:s.int(),children:s.array(self),metadata:s.recursive(entry=>s.object({label:s.string(),children:s.array(entry)}))}));
const stripped=s.recursive(self=>s.object({value:s.int(),children:s.array(self)}).strip());
function main(){
 const zero=empty.safeParse(JSON.parse('{}'));if(zero.success){strictEqual(JSON.stringify(zero.data),'{}');}else{throw 'empty root rejected';}
 const both=nested.safeParse(JSON.parse('{"value":1,"children":[],"metadata":{"label":"outer","children":[{"label":"inner","children":[]}]}}'));if(both.success){strictEqual(both.data.metadata.children[0].label,'inner');}else{throw 'nested definition rejected';}
 const result=arrays.safeParse(JSON.parse('[[],[[]]]'));
 if(result.success){strictEqual(result.data[1][0].length,0);strictEqual(clone(result.data).length,2);}else{throw 'array root rejected';}
 const invalid=arrays.safeParse(JSON.parse('[[0]]'));if(invalid.success){throw 'bad array accepted';}else{strictEqual(invalid.error.issues[0].path.length,2);strictEqual(invalid.error.issues[0].path[0].index,0);strictEqual(invalid.error.issues[0].path[1].index,0);}
 const text=scalar.safeParse(JSON.parse('"hello"'));if(text.success){strictEqual(text.data,'hello');}else{throw 'scalar root rejected';}
 const value=stripped.safeParse(JSON.parse('{"value":1,"extra":true,"children":[{"value":2,"extra":true,"children":[]}]}'));
 if(value.success){strictEqual(JSON.stringify(value.data),'{"children":[{"children":[],"value":2}],"value":1}');}else{throw 'strip root rejected';}
}
"#,true);
}

#[test]
fn recursive_callbacks_and_choices_keep_reversible_trials(){
 run(r#"
import {s} from 'std:schema';import {strictEqual} from 'std:testing';
const transformed=s.recursive(self=>s.object({value:s.string().transform(value=>value.length),children:s.array(self)}));
const choices=s.recursive(self=>s.union([s.int(),s.array(self)]));
function main(){
 const result=transformed.safeParse(JSON.parse('{"value":"root","children":[{"value":"kid","children":[]}]}'));
 if(result.success){strictEqual(result.data.value,4);strictEqual(result.data.children[0].value,3);}else{throw 'callback plan rejected';}
 const selected=choices.safeParse(JSON.parse('[1,[2,[]]]'));if(selected.success){console.log(selected.data);}else{throw 'recursive choice rejected';}
 const rejected=choices.safeParse(JSON.parse('[1,["bad"]]'));if(rejected.success){throw 'bad choice accepted';}else{strictEqual(rejected.error.issues.length>0,true);}
}
"#,false);
}
