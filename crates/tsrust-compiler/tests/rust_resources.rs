use tsrust_compiler::project::{SourceProvider, SourceId, FileSourceProvider, load_project, check_graph};

struct Source<'a> { entry: SourceId, text: &'a str }
impl SourceProvider for Source<'_> {
    fn resolve(&self, importer: Option<&SourceId>, path: &str) -> Result<SourceId,String> {
        FileSourceProvider.resolve(importer,path)
    }
    fn read(&self, id: &SourceId) -> Result<String,String> {
        if id==&self.entry {Ok(self.text.into())} else {FileSourceProvider.read(id)}
    }
}
fn check(text:&str)->Result<tsrust_compiler::project::ProjectAnalysis,tsrust_compiler::project::ProjectDiagnostic> {
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rust-resources/checks.tsr").canonicalize().unwrap();
    let entry=SourceId(path.to_str().unwrap().into());
    check_graph(load_project(&Source{entry:entry.clone(),text},&entry.0)?)
}

#[test]
fn opaque_rust_resources_keep_nominal_identity_and_ownership_guards() {
    let imports="import {spawn} from 'std:threads';import {Session,create,read,append,transfer} from './owner.rs';import {Session as Other,create as number,read as readNumber} from './other.rs';";
    check(&format!("{imports}function main(){{const session=create('a');append(session,'b');console.log(read(session));const moved=transfer(session);console.log(read(moved));console.log(readNumber(number(7)));}}")).unwrap();
    for body in [
        "const session:Session={};",
        "const session=create('a');const duplicate=clone(session);",
        "const session=create('a');console.log(session.text);",
        "const session=create('a');const moved=transfer(session);console.log(read(session));",
        "console.log(readNumber(create('a')));",
        "const wrong=create('a') as Other;console.log(readNumber(wrong));",
        "const session=create('a');const later=()=>read(session);append(session,'b');console.log(later());",
        "const session=create('a');const worker=unwrap(spawn(()=>read(session)));worker.join();",
        "console.log(JSON.stringify(create('a')));",
        "console.log(Object.keys(create('a')));",
        "const erased=create('a') as {};console.log(JSON.stringify(erased));",
    ] {
        assert!(check(&format!("{imports}function main(){{{body}}}")).is_err(),"accepted {body}");
    }
}
