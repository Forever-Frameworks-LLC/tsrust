//! Public multiple-issue validation contracts; run after complete integration.
mod support;

#[test]
fn super_refine_keeps_typed_output_and_nested_issue_paths() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
function main(){
const person=s.object({password:s.string(),confirm:s.string(),age:s.int()}).superRefine((value,ctx)=>{
  if(value.password!==value.confirm){ctx.addIssue({path:['confirm'],message:'Passwords differ'});}
  if(value.age<18){ctx.addIssue({path:['age'],message:'Must be an adult'});}
});
const people=s.array(person);
  {const outcome0=people.safeParseJson('[{"password":"a","confirm":"b","age":12}]');if(outcome0.success){const value=outcome0.data;throw 'Invalid person accepted';}else{const error=outcome0.error;
      if(error.kind==='validation'){
        strictEqual(error.error.issues.length,2);
        strictEqual(error.error.issues[0].message,'Passwords differ');
        strictEqual(error.error.issues[0].path[0].index,0);
        strictEqual(error.error.issues[0].path[1].field,'confirm');
        strictEqual(error.error.issues[1].message,'Must be an adult');
        strictEqual(error.error.issues[1].path[1].field,'age');
      }else{throw 'Unexpected JSON error';}
    }}
  {const outcome1=people.safeParseJson('[{"password":"a","confirm":"a","age":20}]');if(outcome1.success){const value=outcome1.data;strictEqual(value[0].age,20);}else{const error=outcome1.error;throw 'Valid person rejected';}}
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn super_refine_runs_all_additions_but_bounds_retained_issues() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
const schema=s.int().superRefine((value,ctx)=>{
  let i=0;
  while(i<100){ctx.addIssue({message:'Rejected'});i++;}
});
function main(){
  {const outcome2=schema.safeParse(1);if(outcome2.success){const value=outcome2.data;throw 'Rejected value accepted';}else{const error=outcome2.error;strictEqual(error.issues.length,64);strictEqual(error.truncated,true);}}
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn super_refine_rejects_unknown_paths_and_input_mutation() {
    for (callback,code) in [
        ("(value,ctx)=>{ctx.addIssue({path:['missing'],message:'Bad path'});}","schema-path"),
        ("(value,ctx)=>{value.age=0;}","schema-refinement"),
    ] {
        let source=format!("import {{s}} from 'std:schema';const schema=s.object({{age:s.int()}}).superRefine({callback});function main(){{}}");
        let directory=support::TempDirectory::new();
        let file=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(file),30);
        assert!(!output.status.success(),"Invalid callback accepted: {callback}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(code),"{}",String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn super_refine_restores_union_input_and_propagates_exceptions() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
function main(){
  const schema=s.union([
    s.array(s.int()).superRefine((value,ctx)=>{ctx.addIssue({message:'Try next branch'});}),
    s.array(s.int())
  ]);
  {const outcome3=schema.safeParse([1,2]);if(outcome3.success){const value=outcome3.data;strictEqual(value.length,2);}else{const error=outcome3.error;throw 'Fallback rejected';}}
  const throwing=s.int().superRefine((value,ctx)=>{throw 'Callback failed';});
  try {throwing.safeParse(1);throw 'Missing exception';}
  catch(error){if(error!=='Callback failed'){throw 'Wrong exception';}}
  try {throwing.safeParseJson('1');throw 'Missing JSON exception';}
  catch(error){if(error!=='Callback failed'){throw 'Wrong JSON exception';}}
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn super_refine_context_cannot_be_cloned_or_retained_by_owned_callback() {
    for body in [
        "const copied=clone(ctx);",
        "const nested=s.int().refine(other=>{ctx.addIssue({message:'Escaped'});return true;});",
    ] {
        let source=format!("import {{s}} from 'std:schema';function main(){{const schema=s.int().superRefine((value,ctx)=>{{{body}}});}}");
        let directory=support::TempDirectory::new();
        let file=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(file),30);
        assert!(!output.status.success(),"Validation context escaped: {body}");
        let error=String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("clone") || error.contains("callback-escape") || error.contains("callback-access") || error.contains("schema-context"),"{error}");
    }
}
