mod support;

const CONTRACT: &str = r#"
import {parse, stringify, JsonValue, JsonError} from 'std:json';
import {strictEqual} from 'std:testing';
function decode(text:string):JsonValue {
  return JSON.parse(text);
}
function main() {
  const value=decode('{"answer":42}');
  strictEqual(JSON.stringify(value),'{"answer":42}');
  match(JSON.tryStringify(value)) {
    Ok(text)=>{strictEqual(text,JSON.stringify(value));},
    Err(error)=>{throw error;}
  }
  match(parse('null')) {
    Ok(parsed)=>{match(stringify(parsed)) {
      Ok(text)=>{strictEqual(text,'null');}, Err(error)=>{throw error;}
    }}, Err(error)=>{throw error;}
  }
  let caught=0;
  let cleaned=0;
  const callback=():JsonValue=>decode('null secret');
  try {callback();} catch(error) {
    const typed:JsonError=error;
    strictEqual(typed.code,'trailing-data');
    strictEqual(typed.offset,5);
    match(JSON.tryParse('null secret')) {
      Ok(_)=>{throw 'accepted malformed JSON';},
      Err(explicit)=>{
        strictEqual(explicit.code,typed.code);
        strictEqual(explicit.message,typed.message);
        strictEqual(explicit.offset,typed.offset);
      }
    }
    caught+=1;
  } finally {cleaned+=1;}
  strictEqual(caught,1);
  strictEqual(cleaned,1);
  try {JSON.stringify(value,1);} catch(error) {
    strictEqual(error.code,'output-limit');
    caught+=1;
  }
  match(JSON.tryStringify(value,1)) {
    Ok(_)=>{throw 'accepted oversized output';},
    Err(error)=>{strictEqual(error.code,'output-limit');}
  }
  strictEqual(caught,2);
}
"#;

fn run(source: &str) {
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),90));
}

#[test]
fn imported_json_supports_the_namespace_error_contract() {
    // Establish that typed errors, callback propagation and finally already
    // support the contract, independently of global namespace resolution.
    let source=CONTRACT.replace("JSON.tryParse", "parse")
        .replace("JSON.tryStringify", "stringify")
        .replace("JSON.parse", "throwingParse")
        .replace("JSON.stringify", "throwingStringify");
    run(&format!("{source}\n{}",r#"
function throwingParse(text:string):JsonValue {
  match(parse(text)){Ok(value)=>{return value;},Err(error)=>{throw error;}}
}
function throwingStringify(value:JsonValue,maxBytes:int=16777216):string {
  match(stringify(value,maxBytes)){Ok(text)=>{return text;},Err(error)=>{throw error;}}
}
"#));
}

#[test]
fn json_namespace_preserves_typed_errors_and_imported_results() {
    run(CONTRACT);
}
