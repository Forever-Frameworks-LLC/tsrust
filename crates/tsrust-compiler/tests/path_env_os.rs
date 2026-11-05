mod support;
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
    for (source,expected) in examples {let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());}
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
