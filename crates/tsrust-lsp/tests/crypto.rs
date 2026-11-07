mod support;
use support::session::Session;
#[test]
fn hasher_alias_help_and_password_signatures_keep_exact_contracts(){
 let source="import {createHasher,Hasher as State,hashPassword,verifyPasswordBounded} from 'std:crypto';function inspect(state:Mutable<State>):void{state.}function main(){}";let mut session=Session::start(&[]);session.open("main.tsr",source);session.settle();let names=session.completion("main.tsr",source,source.find("state.}").unwrap()+6).unwrap();for name in ["update","finalize"]{assert!(names.iter().any(|value|value==name),"{names:?}");}
 for (name,payload) in [("hashPassword","string"),("verifyPassword","boolean"),("verifyPasswordBounded","boolean")] {
  let source=format!("import {{{name} as operation}} from 'std:crypto';function main(){{}}");let mut session=Session::start(&[]);session.open("main.tsr",&source);session.settle();
  let hover=session.hover("main.tsr",&source,source.find("operation").unwrap()+1).unwrap();for word in ["ThreadPool",&format!("Task<Result<{payload}"),"SpawnError","ThreadError<undefined>"]{assert!(hover.contains(word),"{hover}");}
  assert!(!hover.contains("match (operation)"),"A function or Task is not itself a Result: {hover}");
 }

}
#[test]
fn all_new_crypto_record_fields_have_behavior_examples_and_ownership(){let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();for name in ["PasswordHashOptions","PasswordVerifyOptions","CryptoError"]{let declaration=tsrust_compiler::native_types::lookup("std:crypto",name).unwrap();let entry=catalog["std:crypto"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();for field in declaration.fields{let docs=entry["fields"][field.name]["documentation"].as_str().unwrap();for marker in ["Ownership:","```tsrust","Output:"]{assert!(docs.contains(marker),"{name}.{}: {marker}",field.name);}assert!(!entry["fields"][field.name]["examples"].as_array().unwrap().is_empty());}}}
