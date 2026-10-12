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
