mod support;
use serde_json::{json,Value};
use tsrust_lsp::{intelligence::SourceIndex, workspace::Workspace, project_diagnostics::source_id, project_intelligence::ProjectIndex};

fn request(source:&str,needle:&str,method:&str)->Value {
    let offset=source.find(needle).unwrap()+needle.len()-usize::from(method=="textDocument/hover");
    SourceIndex::new(source).request(method,&json!({"position":{"line":0,"character":offset}}),"file:///threads.tsr").unwrap()
}

#[test]
fn imports_hover_and_parameter_help_describe_ownership_and_failures() {
    let source="import {spawn as start,join,detach,ThreadError} from 'std:threads';function main(){start(():int=>42);}";
    let hover=request(source,"main(){start","textDocument/hover").to_string();
    assert!(hover.contains("Thread<T, E>")&&hover.contains("consumes")&&hover.contains("spawn"),"{hover}");
    let help=request(source,"main(){start(","textDocument/signatureHelp").to_string();
    assert!(help.contains("Owned<Function<T>>"),"{help}");
    let error=request(source,"ThreadError","textDocument/hover").to_string();
    assert!(error.contains("error?: E")&&error.contains("panic"),"{error}");
}

#[test]
fn checked_worker_and_completion_types_reach_hover() {
    let source="import {spawn,join} from 'std:threads';function main(){match(spawn(():int=>42)){Ok(worker)=>{match(join(worker)){Ok(answer)=>{console.log(answer);},Err(error)=>{console.log(error.code);}}},Err(error)=>{console.log(error.code);}}}";
    let uri="file:///threads.tsr";let id=source_id(uri).unwrap();
    let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.into());
    let checked=workspace.snapshot().check(&id).unwrap();
    let index=ProjectIndex::checked(&checked);
    for (needle,expected) in [("join(worker","Thread<int, undefined>"),("console.log(answer","int")] {
        let offset=source.find(needle).unwrap()+needle.len()-1;
        let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
        assert!(hover.to_string().contains(expected),"{hover}");
    }
}

struct Directory(std::path::PathBuf);
impl Drop for Directory {
    fn drop(&mut self) { std::fs::remove_dir_all(&self.0).unwrap(); }
}

#[test]
fn thread_catalog_examples_check_and_execute() {
    let path = std::env::temp_dir().join(format!("tsrust-thread-catalog-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir(&path).unwrap();
    let directory = Directory(path);
    let catalog: serde_json::Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entries = catalog["std:threads"].as_array().unwrap();
    assert_eq!(entries.len(), 14);
    let mut seen=std::collections::HashSet::new();
    for entry in entries.iter().chain(catalog["workerPool"].as_array().unwrap()).chain(catalog["Pool"].as_array().unwrap()).chain(catalog["std:result"].as_array().unwrap()).chain(catalog["globals"].as_array().unwrap().iter().filter(|entry| entry["name"] == "unwrap")) {
        for example in entry["examples"].as_array().unwrap() {
            if !seen.insert(example["source"].as_str().unwrap()) { continue; }
            let source = tsrust_compiler::format(example["source"].as_str().unwrap()).unwrap();
            assert_eq!(tsrust_compiler::format(&source).unwrap(), source);
            let id = tsrust_compiler::project::SourceId(directory.0.join("example.tsr").to_str().unwrap().into());
            let mut workspace = Workspace::default();
            workspace.open(id.clone(), 1, source.clone());
            let checked = workspace.snapshot().check(&id).unwrap();
            let _ = checked.emit_rust();
            let input = directory.0.join("example.tsr");
            std::fs::write(&input, source).unwrap();
            let compiler = std::env::var_os("TSRUST").map(std::path::PathBuf::from).unwrap_or_else(|| std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("tsrust"));
            for level in ["0", "3"] {
                let binary = directory.0.join("example");
                let mut command = std::process::Command::new(&compiler);
                command.arg("build").arg(&input).arg("-o").arg(&binary);
                if level == "3" { command.arg("--release"); }
                let compiled = command.output().unwrap();
                assert!(compiled.status.success(), "{}: {}", entry["name"], String::from_utf8_lossy(&compiled.stderr));
                let output = std::process::Command::new(&binary).current_dir(&directory.0).output().unwrap();
                assert!(output.status.success(), "{}: {}", entry["name"], String::from_utf8_lossy(&output.stderr));
                assert_eq!(String::from_utf8(output.stdout).unwrap(), example["output"].as_str().unwrap(), "{} O{level}", entry["name"]);
            }
        }
    }
}

#[test]
fn completion_keeps_inferred_worker_error_fields_during_editing() {
    use support::session::Session;
    let source="import {spawn,join} from 'std:threads';function main(){match(spawn(():int=>42)){Ok(worker)=>{match(join(worker)){Ok(answer)=>{},Err(error)=>{console.log(error.code);}}},Err(error)=>{console.log(error.code);}}}";
    let mut session=Session::start(&[("main.tsr",source)]);session.open("main.tsr",source);
    let uri=session.uri("main.tsr");
    session.request("textDocument/hover",json!({"textDocument":{"uri":uri},"position":{"line":0,"character":source.find("join(worker").unwrap()+8}}));
    let changed=source.replacen("console.log(error.code);","error.\nconsole.log(error.code);",1);
    let id=source_id(&uri).unwrap();
    let mut workspace=Workspace::default();workspace.open(id.clone(),2,changed.clone());
    let graph=tsrust_compiler::project::load_project_headers(&workspace.snapshot(),&id.0).unwrap();
    let index=ProjectIndex::new(&graph);
    let offset=changed.find("error.\n").unwrap()+6;
    let direct=index.request("textDocument/completion",&json!({"position":{"line":0,"character":offset}}),&uri).unwrap().unwrap();
    assert!(!direct.as_array().unwrap().is_empty(), "direct: {direct}");
    session.change("main.tsr",&changed);
    let offset=changed.find("error.\n").unwrap()+6;
    let items=session.request("textDocument/completion",json!({"textDocument":{"uri":uri},"position":{"line":0,"character":offset}}));
    for field in ["code","error"] {assert!(items.as_array().unwrap().iter().any(|item|item["label"]==field),"{items}");}
}

#[test]
fn declared_thread_error_fields_survive_unrelated_syntax_errors() {
    let source="import {ThreadError} from 'std:threads';function probe(error:ReadOnly<ThreadError<string>>){error.\n}function main(){}function broken(){const=;}";
    let index=SourceIndex::new(source);
    let offset=source.find("{error.").unwrap()+7;
    let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":offset}}),"file:///threads.tsr").unwrap();
    let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset-2}}),"file:///threads.tsr").unwrap();
    assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="error"),"{items}; {hover}");
}

#[test]
fn scoped_workers_and_propagation_have_checked_hover_and_signatures() {
    let source="import {spawn,join} from 'std:threads';function main(){const first=unwrap(spawn(():int=>42));const value=unwrap(join(first));console.log(value);}";
    let uri="file:///scoped-workers.tsr";let id=source_id(uri).unwrap();
    let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.into());
    let checked=workspace.snapshot().check(&id).unwrap();let index=ProjectIndex::checked(&checked);
    for (needle,expected) in [("join(first","Thread<int, undefined>"),("console.log(value","int")] {
        let offset=source.find(needle).unwrap()+needle.len()-1;
        let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
        assert!(hover.to_string().contains(expected),"{hover}");
    }
    let help=request(source,"first=unwrap(","textDocument/signatureHelp").to_string();
    assert!(help.contains("Result<T, E>"),"{help}");
    let hover=request(source,"first=unwrap","textDocument/hover").to_string();
    assert!(hover.contains("throws")&&hover.contains("without cloning"),"{hover}");
}

#[test]
fn pool_and_cancellation_have_documentation_and_checked_types() {
    let source="import {pool,join,cancel,cancellationRequested} from 'std:threads';function main(){pool.configure({workers: 2, capacity: 8});const worker=unwrap(pool.task(():int=>1));cancel(worker);unwrap(join(worker));const stats=pool.stats();console.log(stats.outstanding);unwrap(pool.shutdown());}";
    let uri="file:///pool.tsr";let id=source_id(uri).unwrap();
    let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.into());
    let checked=workspace.snapshot().check(&id).unwrap();let index=ProjectIndex::checked(&checked);
    for name in ["configure","task","cancel","stats","shutdown"] {
        let offset=source.rfind(&format!("{name}(")).unwrap()+2;
        let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
        assert!(hover.to_string().contains(name),"{name}: {hover}");
    }
    let offset=source.find("stats.outstanding").unwrap()+8;
    let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
    assert!(hover.to_string().contains("int"),"{hover}");
}

#[test]
fn pool_statistics_completion_uses_checked_native_fields() {
    let source="import {pool} from 'std:threads';function main(){const stats=pool.stats();stats.outstanding;}";
    let uri="file:///pool-fields.tsr";let id=source_id(uri).unwrap();
    let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.into());
    let checked=workspace.snapshot().check(&id).unwrap();let index=ProjectIndex::checked(&checked);
    let offset=source.find("stats.outstanding").unwrap()+6;
    let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
    let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset-3}}),uri).unwrap();
    assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="outstanding"),"{items}; hover {hover:?}; facts {:?}",checked.checked_type_facts());
}

#[test]
fn scoped_pool_alias_methods_have_hover_parameters_and_result_fields() {
    let source="import {pool as workers,Pool as Jobs,join} from 'std:threads';function main(){const jobs=new Jobs({workers:2,capacity:8});const task=unwrap(jobs.task(():int=>42));console.log(unwrap(join(task)));const stats=jobs.stats();console.log(stats.outstanding);unwrap(jobs.shutdown());}";
    let uri="file:///scoped-pool.tsr";let id=source_id(uri).unwrap();
    let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.into());
    let checked=workspace.snapshot().check(&id).unwrap();let index=ProjectIndex::checked(&checked);
    for (needle,expected) in [("new Jobs","capacity"),("jobs.task","callback"),("jobs.stats","PoolStats"),("jobs.shutdown","shutdown")] {
        let offset=source.find(needle).unwrap()+needle.find('.').map_or(4, |dot| dot+2);
        let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
        assert!(hover.to_string().contains(expected),"{needle}: {hover}");
    }
    let offset=source.find("new Jobs(").unwrap()+"new Jobs(".len();
    let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
    assert!(help.to_string().contains("capacity"),"{help}");
    let offset=source.find("stats.outstanding").unwrap()+6;
    let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
    assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="outstanding"),"{items}");
}
