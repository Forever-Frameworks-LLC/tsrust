mod support;
use tsrust_compiler::project::{FileSourceProvider, check_graph, load_project};

fn checked(source: &str) -> Result<tsrust_compiler::project::ProjectAnalysis, tsrust_compiler::project::ProjectDiagnostic> {
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    check_graph(load_project(&FileSourceProvider,path.to_str().unwrap())?)
}

#[test]
fn generic_native_values_keep_aliases_optional_rest_and_returned_views() {
    let source=r#"
import {basename as name,join} from 'std:path';
import {fromUtf8,view,viewMut,set,length,toUtf8} from 'std:bytes';
import {strictEqual} from 'std:testing';
function identity<T>(value:T):T{return value;}
function main(){
  const base=identity(name);strictEqual(base('hello.txt'),'hello.txt');strictEqual(base('hello.txt','.txt'),'hello');
  const original=join;const combined=identity(original);strictEqual(combined('a','b','c'),join('a','b','c'));
  const bytes=unwrap(fromUtf8('hello'));
  {const observe=identity(view);const part=unwrap(observe(bytes,1,4));strictEqual(length(part),3);strictEqual(unwrap(toUtf8(part)),'ell');}
  {const exclusive=identity(viewMut);const part=unwrap(exclusive(bytes,0,1));unwrap(set(part,0,72));}
  strictEqual(unwrap(toUtf8(bytes)),'Hello');
}
"#;
    checked(source).unwrap();
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    for release in [false,true] {
        let binary=directory.0.join(format!("native-values{}",std::env::consts::EXE_SUFFIX));
        let mut command=support::cli();command.arg("build").arg(&path).arg("-o").arg(&binary);
        if release {command.arg("--release");}
        support::success(&support::run(&mut command,180));
        let output=support::run(&mut std::process::Command::new(&binary),30);support::success(&output);assert!(output.stdout.is_empty());
    }
}

#[test]
fn generic_native_values_preserve_readonly_consumed_and_returned_owner_checks() {
    let identity="function identity<T>(value:T):T{return value;}";
    for (source,code) in [
        ("import {File,seek} from 'std:fs';function action(file:ReadOnly<File>){const move=identity(seek);move(file,0,'start');}function main(){}","parameter-access"),
        ("import {File,position} from 'std:fs';function action(file:ReadOnly<File>){const query=identity(position);query(file);}function main(){}","parameter-access"),
        ("import {open,close,position} from 'std:fs';function main(){const file=unwrap(open('input'));const query=identity(position);unwrap(close(file));query(file);}","transferred"),
    ] {
        let error=checked(&format!("{identity}{source}")).err().expect("native access contract must survive generic identity");
        assert_eq!(error.code,code,"{}",error.message);
    }
    let source=format!("{identity}import {{fromUtf8,view,set,length}} from 'std:bytes';function main(){{const bytes=unwrap(fromUtf8('hello'));const observe=identity(view);const part=unwrap(observe(bytes,0,2));unwrap(set(bytes,0,65));console.log(length(part));}}");
    let error=checked(&source).err().expect("returned view must retain owner loan");
    assert_eq!(error.code,"native-view-conflict","{}",error.message);
    let direct=checked(&source.replace("identity(view)","view")).err().expect("direct view has the same owner conflict");
    assert_eq!(error.code,direct.code,"{}",error.message);
}
