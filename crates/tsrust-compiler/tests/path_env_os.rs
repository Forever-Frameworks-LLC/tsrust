mod support;
#[path="support/path_env_os_suite_reports.rs"] mod path_env_os_suite_reports;
use tsrust_compiler::project::{FileSourceProvider,check_graph,load_project};

fn emitted(source:&str)->String {
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap().emit_rust()
}
#[test]
fn path_env_os_example_and_controlled_observers_execute_at_o0_and_o3(){
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip(include_str!("../../../examples/path-env-os/main.tsr"));
    let path=directory.source(&source);
    for release in [false,true] {
        let binary=directory.0.join(format!("path-env-os{}",std::env::consts::EXE_SUFFIX));
        let mut command=support::cli();command.arg("build").arg(&path).arg("-o").arg(&binary);if release{command.arg("--release");}
        support::success(&support::run(&mut command,180));
        let output=support::run(std::process::Command::new(&binary).env("TSRUST_B03_TEXT","héllo 🦀").env("TSRUST_B03_EMPTY",""),30);
        support::success(&output);assert_eq!(output.stdout,b"Path, environment and OS checks passed\n");
    }
}
#[cfg(unix)]
#[test]
fn native_environment_observers_match_the_controlled_child_context_at_o0_and_o3(){
    let directory=support::TempDirectory::new();
    let home=directory.0.join("home-é");let temporary=directory.0.join("temporary-🦀");let cwd=directory.0.join("cwd-é-🦀");
    for path in [&home,&temporary,&cwd] {std::fs::create_dir(path).unwrap();}
    let home=std::fs::canonicalize(home).unwrap();let temporary=std::fs::canonicalize(temporary).unwrap();let cwd=std::fs::canonicalize(cwd).unwrap();
    let source=support::format_round_trip(r#"
import {vars as environmentEntries,get as lookup,currentDir as workingDirectory,currentExe as executablePath,tempDir as temporaryDirectory,homeDir as userHome} from 'std:env';
import {strictEqual} from 'std:testing';
function main(){
  const entries=environmentEntries;const get=lookup;const currentDir=workingDirectory;const currentExe=executablePath;const tempDir=temporaryDirectory;const homeDir=userHome;
  const original=unwrap(entries());const rows=original;let found=0;
  for(let index=0;index<rows.length;index++){if(rows[index].name==='TSRUST_B03_TEXT'){
    found++;strictEqual(rows[index].value,'héllo 🦀','snapshot keeps the controlled Unicode value');
    rows[index].value='independent snapshot';
    strictEqual(rows[index].value,'independent snapshot');
    strictEqual(unwrap(get('TSRUST_B03_TEXT'))==='héllo 🦀',true,'snapshot mutation leaves the process environment unchanged');
  }}
  strictEqual(found,1,'snapshot includes the controlled name exactly once');
  strictEqual(unwrap(get('TSRUST_B03_EMPTY'))==='',true,'empty is present');
  strictEqual(unwrap(get('TSRUST_B03_MISSING'))===undefined,true,'missing remains absent');
  strictEqual(unwrap(currentDir())===unwrap(get('TSRUST_B03_EXPECT_CWD')),true,'currentDir uses the child working directory');
  strictEqual(unwrap(currentExe())===unwrap(get('TSRUST_B03_EXPECT_EXE')),true,'currentExe identifies the executed binary');
  strictEqual(unwrap(tempDir())===unwrap(get('TSRUST_B03_EXPECT_TEMP')),true,'tempDir observes TMPDIR');
  strictEqual(unwrap(homeDir())===unwrap(get('TSRUST_B03_EXPECT_HOME')),true,'homeDir observes HOME');
}
"#);
    let path=directory.source(&source);
    for release in [false,true] {
        let binary=directory.0.join(if release{"controlled-release"}else{"controlled-debug"});
        let mut build=support::cli();build.arg("build").arg(&path).arg("-o").arg(&binary);if release{build.arg("--release");}
        support::success(&support::run(&mut build,180));
        let binary=std::fs::canonicalize(binary).unwrap();
        let mut child=std::process::Command::new(&binary);
        child.current_dir(&cwd).env_clear().env("HOME",&home).env("TMPDIR",&temporary)
            .env("TSRUST_B03_TEXT","héllo 🦀").env("TSRUST_B03_EMPTY","")
            .env("TSRUST_B03_EXPECT_CWD",&cwd).env("TSRUST_B03_EXPECT_EXE",&binary)
            .env("TSRUST_B03_EXPECT_TEMP",&temporary).env("TSRUST_B03_EXPECT_HOME",&home);
        let output=support::run(&mut child,30);support::success(&output);assert!(output.stdout.is_empty(),"release={release}");
    }
}
#[test]
fn path_env_os_catalog_examples_execute_without_disclosing_os_values(){
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let selected=[("std:path",&["PathParts","PathFormatOptions","PathError","parse","format","resolve","separator","delimiter"][..]),("std:env",&["EnvVar","vars","currentDir","currentExe","tempDir","homeDir"][..]),("std:os",&["OsError","availableParallelism","hostname"][..])];
    let mut examples=std::collections::BTreeMap::new();let mut exports=0;
    for (module,names) in selected {for row in catalog[module].as_array().unwrap().iter().filter(|row|names.contains(&row["name"].as_str().unwrap())){
        exports+=1;assert!(row["documentation"].as_str().unwrap().contains("Ownership:"));
        for example in row["examples"].as_array().unwrap(){examples.insert(example["source"].as_str().unwrap().to_owned(),example["output"].as_str().unwrap().to_owned());}
    }}
    assert_eq!(exports,17);
    assert_eq!(examples.len(),15);
    assert_eq!(path_env_os_suite_reports::CASES.len(),examples.len());
    for (source,expected) in examples {path_env_os_suite_reports::assert_registered(&source,&expected);}
    path_env_os_suite_reports::execute();
}
#[test]
fn unused_imports_and_std_only_observers_do_not_select_hostname(){
    for source in [
        "import {hostname,OsError} from 'std:os';function main(){}",
        "import {parse,PathParts} from 'std:path';import {vars,EnvVar} from 'std:env';function main(){}",
    ]{let source=emitted(source);assert!(!tsrust_compiler::native_runtime::requires_dependencies(&source));assert!(!tsrust_compiler::native_runtime::requires_hostname(&source));assert!(!source.contains("mod tsrust_native_hostname"));}
    for source in [
        "import {parse,format} from 'std:path';function main(){console.log(format({base:parse('file.txt').base}));}",
        "import {resolve} from 'std:path';function main(){console.log(unwrap(resolve()));}",
        "import {vars,currentDir,currentExe,tempDir,homeDir} from 'std:env';function main(){console.log(unwrap(vars()).length,unwrap(currentDir()).length,unwrap(currentExe()).length,unwrap(tempDir()).length,unwrap(homeDir())===undefined);}",
        "import {availableParallelism} from 'std:os';function main(){console.log(unwrap(availableParallelism()));}",
    ]{let source=emitted(source);assert!(!tsrust_compiler::native_runtime::requires_dependencies(&source));assert!(!tsrust_compiler::native_runtime::requires_hostname(&source));assert!(!source.contains("tsrust_native_ui"));}
    let source=emitted("import {hostname} from 'std:os';function main(){console.log(unwrap(hostname()).length);}");
    assert!(tsrust_compiler::native_runtime::requires_hostname(&source));assert!(tsrust_compiler::native_runtime::requires_dependencies(&source));
    let units=tsrust_compiler::native_runtime::separate(&source);let runtime=units.runtime.unwrap();assert!(runtime.contains("pub mod tsrust_native_hostname"));assert!(units.application.contains("use tsrust_runtime::tsrust_native_hostname"));assert!(!runtime.contains("tsrust_native_ui"));
}

#[test]
fn path_env_os_aliases_preserve_argument_and_snapshot_access_contracts(){
    for source in [
        "import {parse as inspect} from 'std:path';function main(){const saved=inspect;saved(1);}",
        "import {format as render} from 'std:path';function main(){const saved=render;saved({ext:1});}",
        "import {resolve as absolute} from 'std:path';function main(){const saved=absolute;saved(true);}",
        "import {vars as entries} from 'std:env';function main(){const saved=entries;saved(1);}",
        "import {currentDir as cwd} from 'std:env';function main(){const saved=cwd;saved('extra');}",
        "import {hostname as host} from 'std:os';function main(){const saved=host;saved(1);}",
        "import {availableParallelism as capacity} from 'std:os';function main(){const saved=capacity;saved(1);}",
        "import {PathParts} from 'std:path';function edit(parts:ReadOnly<PathParts>){parts.name='changed';}function main(){}",
        "import {EnvVar} from 'std:env';function edit(row:ReadOnly<EnvVar>){row.value='changed';}function main(){}",
    ] {
        let directory=support::TempDirectory::new();
        let path=directory.source(&support::format_round_trip(source));
        let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();
        assert!(check_graph(graph).is_err(),"accepted invalid alias/access contract: {source}");
    }
}
