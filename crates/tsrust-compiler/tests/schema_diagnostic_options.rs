//! Public diagnostic options: run with the completed compiler/editor section.
mod support;

#[test]
fn schema_messages_and_nested_refinement_paths_survive_reuse_and_json_decoding() {
    let source = r#"
import { s } from 'std:schema';
import { strictEqual } from 'std:testing';
function main() {
  const integer = s.int({ message: 'Expected a whole count' });
  match (integer.safeParse(JSON.parse('"wrong"'))) {
    Ok(value) => { throw 'Invalid integer accepted'; },
    Err(error) => { strictEqual(error.issues[0].message, 'Expected a whole count'); }
  }
  match (integer.safeParseJson('1.5')) {
    Ok(value) => { throw 'Fraction accepted'; },
    Err(failure) => {
      if (failure.kind === 'validation') {
        strictEqual(failure.error.issues[0].message, 'Expected a whole count');
      } else { throw 'Wrong error kind'; }
    }
  }
  const account = s.object({ password: s.string(), confirm: s.string() })
    .refine(value => value.password === value.confirm, {
      path: ['confirm'], message: 'Passwords must match'
    });
  const form = s.object({ account });
  match (form.safeParseJson('{"account":{"password":"secret","confirm":"different"}}')) {
    Ok(value) => { throw 'Mismatch accepted'; },
    Err(failure) => {
      if (failure.kind === 'validation') {
        strictEqual(failure.error.issues[0].path[0].field, 'account');
        strictEqual(failure.error.issues[0].path[1].field, 'confirm');
        strictEqual(failure.error.issues[0].message, 'Passwords must match');
        strictEqual(failure.error.issues[0].code, 'refinement-failed');
      } else { throw 'Wrong error kind'; }
    }
  }
  const object = s.object({ count: s.int() }, { message: 'Expected a count object' });
  match (object.safeParseJson('null')) {
    Ok(value) => { throw 'Null object accepted'; },
    Err(failure) => {
      if (failure.kind === 'validation') {
        strictEqual(failure.error.issues[0].message, 'Expected a count object');
      } else { throw 'Wrong error kind'; }
    }
  }
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
  match (count.safeParse(1)) {
    Ok(value) => { throw 'Lower bound ignored'; },
    Err(error) => { strictEqual(error.issues[0].message, 'Invalid count'); }
  }
  match (count.safeParse(3)) {
    Ok(value) => { strictEqual(value, 3); },
    Err(error) => { throw 'Valid count rejected'; }
  }
  const form = s.object({ count: s.int({ message: 'Invalid child' }) }, {
    message: 'Invalid form'
  });
  match (form.safeParseJson('{"count":"wrong"}')) {
    Ok(value) => { throw 'Invalid child accepted'; },
    Err(failure) => {
      if (failure.kind === 'validation') {
        strictEqual(failure.error.issues[0].message, 'Invalid child');
        strictEqual(failure.error.issues[0].path[0].field, 'count');
      } else { throw 'Wrong failure kind'; }
    }
  }
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
  match (stripped.safeParseJson('{"count":3,"extra":4}')) {
    Ok(value) => { strictEqual(value.count, 3); },
    Err(error) => { throw 'Strip failed'; }
  }
  const retained = s.object({count:s.int({message:'Invalid count'})}, {message:'Invalid object'}).retain();
  match (retained.safeParseJson('{"count":3,"extra":4}')) {
    Ok(value) => { strictEqual(value.value.count, 3); strictEqual(value.extra.size, 1); },
    Err(error) => { throw 'Retain failed'; }
  }
  match (retained.safeParseJson('null')) {
    Ok(value) => { throw 'Null accepted'; },
    Err(error) => {
      if (error.kind === 'validation') {
        strictEqual(error.error.issues[0].message, 'Invalid object');
      } else { throw 'Wrong error kind'; }
    }
  }
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
