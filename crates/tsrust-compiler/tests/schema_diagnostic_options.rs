//! Public diagnostic options: run with the completed compiler/editor section.
mod support;

#[test]
fn schema_messages_and_nested_refinement_paths_survive_reuse_and_json_decoding() {
    let source = r#"
import { s } from 'std:schema';
import { strictEqual } from 'std:testing';
function main() {
  const integer = s.int({ message: 'Expected a whole count' });
  {const outcome0=integer.safeParse(JSON.parse('"wrong"'));if(outcome0.success){const value=outcome0.data; throw 'Invalid integer accepted'; }else{const error=outcome0.error; strictEqual(error.issues[0].message, 'Expected a whole count'); }}
  {const outcome1=integer.safeParseJson('1.5');if(outcome1.success){const value=outcome1.data; throw 'Fraction accepted'; }else{const failure=outcome1.error;
      if (failure.kind === 'validation') {
        strictEqual(failure.error.issues[0].message, 'Expected a whole count');
      } else { throw 'Wrong error kind'; }
    }}
  const account = s.object({ password: s.string(), confirm: s.string() })
    .refine(value => value.password === value.confirm, {
      path: ['confirm'], message: 'Passwords must match'
    });
  const form = s.object({ account });
  {const outcome2=form.safeParseJson('{"account":{"password":"secret","confirm":"different"}}');if(outcome2.success){const value=outcome2.data; throw 'Mismatch accepted'; }else{const failure=outcome2.error;
      if (failure.kind === 'validation') {
        strictEqual(failure.error.issues[0].path[0].field, 'account');
        strictEqual(failure.error.issues[0].path[1].field, 'confirm');
        strictEqual(failure.error.issues[0].message, 'Passwords must match');
        strictEqual(failure.error.issues[0].code, 'refinement-failed');
      } else { throw 'Wrong error kind'; }
    }}
  const object = s.object({ count: s.int() }, { message: 'Expected a count object' });
  {const outcome3=object.safeParseJson('null');if(outcome3.success){const value=outcome3.data; throw 'Null object accepted'; }else{const failure=outcome3.error;
      if (failure.kind === 'validation') {
        strictEqual(failure.error.issues[0].message, 'Expected a count object');
      } else { throw 'Wrong error kind'; }
    }}
}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn refinement_paths_reject_unknown_fields_and_wrong_segment_kinds() {
    for path in ["['missing']", "['items', '0']", "['items', 0, 'missing']", "['items', -1]"] {
        let source = format!("import {{s}} from 'std:schema'; function main() {{ const schema = s.object({{items:s.array(s.object({{name:s.string()}}))}}).refine(value => true, {{path:{path}}}); }}");
        let directory = support::TempDirectory::new();
        let file = directory.source(&source);
        let output = support::run(support::cli().arg("check").arg(file), 30);
        assert!(!output.status.success(), "Invalid path accepted: {path}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("schema-path"), "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn constructor_messages_preserve_constraint_chains_and_child_precedence() {
    let source = r#"
import { s } from 'std:schema';
import { strictEqual } from 'std:testing';
function main() {
  const count = s.int({ message: 'Invalid count' }).min(2).max(4);
  {const outcome4=count.safeParse(1);if(outcome4.success){const value=outcome4.data; throw 'Lower bound ignored'; }else{const error=outcome4.error; strictEqual(error.issues[0].message, 'Invalid count'); }}
  {const outcome5=count.safeParse(3);if(outcome5.success){const value=outcome5.data; strictEqual(value, 3); }else{const error=outcome5.error; throw 'Valid count rejected'; }}
  const form = s.object({ count: s.int({ message: 'Invalid child' }) }, {
    message: 'Invalid form'
  });
  {const outcome6=form.safeParseJson('{"count":"wrong"}');if(outcome6.success){const value=outcome6.data; throw 'Invalid child accepted'; }else{const failure=outcome6.error;
      if (failure.kind === 'validation') {
        strictEqual(failure.error.issues[0].message, 'Invalid child');
        strictEqual(failure.error.issues[0].path[0].field, 'count');
      } else { throw 'Wrong failure kind'; }
    }}
}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn constructor_options_reject_unknown_fields_and_nonstring_messages() {
    for expression in [
        "s.int({message:7})",
        "s.string({mesage:'typo'})",
        "s.object({name:s.string()},{message:false})",
        "s.array(s.int(),{unknown:'option'})",
    ] {
        let directory = support::TempDirectory::new();
        let file = directory.source(&format!(
            "import {{s}} from 'std:schema';function main(){{const schema={expression};}}"
        ));
        let output = support::run(support::cli().arg("check").arg(file), 30);
        assert!(!output.status.success(), "Invalid options accepted: {expression}");
    }
}

#[test]
fn object_messages_survive_strip_and_retain_without_changing_output() {
    let source = r#"
import { s } from 'std:schema';
import { strictEqual } from 'std:testing';
function main() {
  const stripped = s.object({count:s.int()}, {message:'Invalid object'}).strip();
  {const outcome7=stripped.safeParseJson('{"count":3,"extra":4}');if(outcome7.success){const value=outcome7.data; strictEqual(value.count, 3); }else{const error=outcome7.error; throw 'Strip failed'; }}
  const retained = s.object({count:s.int({message:'Invalid count'})}, {message:'Invalid object'}).retain();
  {const outcome8=retained.safeParseJson('{"count":3,"extra":4}');if(outcome8.success){const value=outcome8.data; strictEqual(value.value.count, 3); strictEqual(value.extra.size, 1); }else{const error=outcome8.error; throw 'Retain failed'; }}
  {const outcome9=retained.safeParseJson('null');if(outcome9.success){const value=outcome9.data; throw 'Null accepted'; }else{const error=outcome9.error;
      if (error.kind === 'validation') {
        strictEqual(error.error.issues[0].message, 'Invalid object');
      } else { throw 'Wrong error kind'; }
    }}
}
"#;
    let directory = support::TempDirectory::new();
    let file = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file), 60));
}

#[test]
fn message_constraints_preserve_evaluation_order_and_construction_errors() {
    let source = r#"
import { s } from 'std:schema';
import { strictEqual } from 'std:testing';
let order = 0;
function message(): string { order = order * 10 + 1; return 'Invalid count'; }
function limit(): int { order = order * 10 + 2; return 3; }
function main() {
  const schema = s.int({message:message()}).min(limit());
  strictEqual(order, 12);
  let caught = false;
  try { s.number({message:'Invalid number'}).min(2.5).max(1.5); }
  catch (error) {
    caught = true;
    strictEqual(error.code, 'contradictory-bounds');
    strictEqual(error.operation, 'max');
    strictEqual(error.limit, 1.5);
  }
  strictEqual(caught, true);
}
"#;
    let directory = support::TempDirectory::new();
    let file = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file), 60));
}

#[test]
fn logging_schema_results_initializes_observable_record_metadata() {
    let directory = support::TempDirectory::new();
    std::fs::write(directory.0.join("greeting.tsr"), "export function greeting():string{return 'Hello schema';}").unwrap();
    let source = r#"
import { greeting } from './greeting.tsr';
import { s } from 'std:schema';
const schema = s.string();
function main() {
  const parsed = schema.safeParse(greeting());
  console.log(parsed);
  console.log(schema.safeParse(JSON.parse('7')));
  console.log(schema.safeParseJson('"Hello JSON"'));
  console.log(schema.safeParseJson('7'));
  console.log(schema.safeParseJson('broken'));
}
"#;
    let file = directory.0.join("main.tsr");
    std::fs::write(&file, support::format_round_trip(source)).unwrap();
    let output = support::run(support::cli().arg("run").arg(file), 60);
    support::success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in ["Hello schema", "Hello JSON", "issues:", "validation", "invalid-json"] {
        assert!(stdout.contains(expected), "Missing {expected}: {stdout}");
    }
}

#[test]
fn logging_typed_object_schema_results_keeps_field_metadata() {
    let source = r#"
import { s } from 'std:schema';
function main() {
  const schema = s.object({ name: s.string(), age: s.int() });
  console.log(schema.safeParse({ name: 'Pat', age: 7 }));
  console.log(schema.safeParseJson('{"name":"Sam","age":8}'));
  console.log(schema.safeParse(JSON.parse('{"name":"Lee","age":9}')));
  const empty = s.object({});
  console.log(empty.safeParse({}));
  console.log(empty.safeParseJson('{}'));
}
"#;
    let directory = support::TempDirectory::new();
    let file = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(file), 60);
    support::success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Pat") && stdout.contains("Sam") && stdout.contains("Lee"), "{stdout}");
}

#[test]
fn logging_retained_and_pipeline_results_preserves_union_retry_order() {
    let source = r#"
import { s } from 'std:schema';
import { strictEqual } from 'std:testing';
type Input = { a: int; b: int; };
function input(reverse: boolean): Input {
  if (reverse) { return { b: 2, a: 1 }; }
  return { a: 1, b: 2 };
}
function main() {
  const retained = s.object({ a: s.int() }).retain();
  {const outcome10=retained.safeParse({ a: 1 });if(outcome10.success){const value=outcome10.data; strictEqual(Object.keys(value).join(','), 'value,extra'); console.log(value.value); }else{const error=outcome10.error; throw 'Retained validation failed'; }}
  {const outcome11=retained.safeParse(JSON.parse('{"a":1,"extra":true}'));if(outcome11.success){const value=outcome11.data; strictEqual(Object.keys(value).join(','), 'value,extra'); console.log(value.value); }else{const error=outcome11.error; throw 'Retained validation failed'; }}
  {const outcome12=retained.safeParseJson('{"a":1,"extra":true}');if(outcome12.success){const value=outcome12.data; strictEqual(Object.keys(value).join(','), 'value,extra'); console.log(value.value); }else{const error=outcome12.error; throw 'Retained validation failed'; }}
  const pipeline = s.object({ a: s.int().refine(value => value > 0) }).retain();
  {const outcome13=pipeline.safeParse({ a: 1 });if(outcome13.success){const value=outcome13.data; strictEqual(Object.keys(value).join(','), 'value,extra'); console.log(value.value); }else{const error=outcome13.error; throw 'Retained validation failed'; }}
  {const outcome14=pipeline.safeParse(JSON.parse('{"a":1,"extra":true}'));if(outcome14.success){const value=outcome14.data; strictEqual(Object.keys(value).join(','), 'value,extra'); console.log(value.value); }else{const error=outcome14.error; throw 'Retained validation failed'; }}
  {const outcome15=pipeline.safeParseJson('{"a":1,"extra":true}');if(outcome15.success){const value=outcome15.data; strictEqual(Object.keys(value).join(','), 'value,extra'); console.log(value.value); }else{const error=outcome15.error; throw 'Retained validation failed'; }}
  const projected = s.union([
    s.object({ a: s.int(), b: s.int() }).refine(value => false),
    s.object({ a: s.int(), b: s.int() }).coerce((value: Input) => {
      strictEqual(Object.keys(value).join(','), 'b,a');
      return value;
    })
  ]);
  {const outcome16=projected.safeParse(input(true));if(outcome16.success){const value=outcome16.data; console.log(value); }else{const error=outcome16.error; throw 'Projection retry failed'; }}
  const choices = s.union([
    s.object({ a: s.int().refine(value => true), b: s.int().refine(value => false) }),
    s.object({ a: s.int(), b: s.int() }).coerce((value: Input) => {
      strictEqual(Object.keys(value).join(','), 'b,a');
      return value;
    })
  ]);
  {const outcome17=choices.safeParse(input(true));if(outcome17.success){const value=outcome17.data; console.log(value); strictEqual(value.a, 1); strictEqual(value.b, 2); }else{const error=outcome17.error; throw 'Union retry failed'; }}
}
"#;
    let directory = support::TempDirectory::new();
    let file = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file), 60));
}

#[test]
fn constraint_messages_belong_to_the_constraint_that_failed() {
    let source = r#"
import { s } from 'std:schema';
import { strictEqual } from 'std:testing';
function main() {
  const schema = s.string({ message: 'Expected text' })
    .min(1, { message: 'Must be at least 1 character' })
    .max(3, { message: 'Must be at most 3 characters' });
  {const outcome18=schema.safeParse('');if(outcome18.success){const value=outcome18.data; throw 'Minimum ignored'; }else{const error=outcome18.error; strictEqual(error.issues[0].message, 'Must be at least 1 character'); }}
  {const outcome19=schema.safeParse('long');if(outcome19.success){const value=outcome19.data; throw 'Maximum ignored'; }else{const error=outcome19.error; strictEqual(error.issues[0].message, 'Must be at most 3 characters'); }}
  strictEqual(schema.parse('ok'), 'ok');
  const mixed = s.string().min(1, { message: 'Required' }).max(3);
  strictEqual(mixed.parse('ok'), 'ok');
  let constructionFailed = false;
  try { s.number().min(2.5, { message: 'Too small' }).max(1.5, { message: 'Too large' }); }
  catch (error) {
    constructionFailed = true;
    strictEqual(error.code, 'contradictory-bounds');
    strictEqual(error.operation, 'max');
    strictEqual(error.limit, 1.5);
  }
  strictEqual(constructionFailed, true);
  constructionFailed = false;
  try { s.int().min(5, { message: 'Minimum' }).refine(value => true).max(3, { message: 'Maximum' }); }
  catch (error) { constructionFailed = true; strictEqual(error.code, 'contradictory-bounds'); }
  strictEqual(constructionFailed, true);
  constructionFailed = false;
  try { s.int().min(5, { message: 'Minimum' }).default(() => 6).max(3, { message: 'Maximum' }); }
  catch (error) { constructionFailed = true; strictEqual(error.code, 'contradictory-bounds'); }
  strictEqual(constructionFailed, true);


  {const outcome20=schema.safeParseJson('42');if(outcome20.success){const value=outcome20.data; throw 'Type mismatch ignored'; }else{const failure=outcome20.error;
      if (failure.kind === 'validation') {
        strictEqual(failure.error.issues[0].message, 'Expected text');
      } else { throw 'Wrong error kind'; }
    }}
  try { schema.parse(''); }
  catch (error) { strictEqual(error.issues[0].message, 'Must be at least 1 character'); }
}
"#;
    let directory = support::TempDirectory::new();
    let file = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file), 60));
}

#[test]
fn constraint_message_catalog_examples_execute_without_other_pipeline_features() {
    let catalog: serde_json::Value = serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut source = String::from("import {s} from 'std:schema';\n");
    let mut calls = String::new();
    let mut expected = String::new();
    let mut count = 0;
    for key in ["Schema", "SchemaInt", "SchemaNumber"] {
        for entry in catalog[key].as_array().unwrap() {
            if !matches!(entry["name"].as_str(), Some("min" | "max" | "gt" | "lt")) { continue; }
            let example = entry["examples"].as_array().unwrap().last().unwrap();
            let code = example["source"].as_str().unwrap();
            assert!(code.contains("message:"));
            source.push_str(&code.replace("import {s} from 'std:schema';", "")
                .replace("function main()", &format!("function example{count}()")));
            calls.push_str(&format!("example{count}();"));
            expected.push_str(example["output"].as_str().unwrap());
            count += 1;
        }
    }
    assert_eq!(count, 10);
    source.push_str(&format!("function main(){{{calls}}}"));
    let directory = support::TempDirectory::new();
    let file = directory.source(&support::format_round_trip(&source));
    let output = support::run(support::cli().arg("run").arg(file), 60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}
