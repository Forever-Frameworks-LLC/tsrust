//! Content builds write one self-contained typed module; Markdown never executes code.
use std::{collections::{BTreeMap,BTreeSet},env,fs,path::{Path,PathBuf},io::Write};
use sha2::{Digest,Sha256};
use serde_json::{Value,json};
use tsrust_compiler::{documentation as docs,project::{self,FileSourceProvider,SourceId,SourceProvider}};
fn hash(bytes:&[u8])->String{Sha256::digest(bytes).iter().map(|byte|format!("{byte:02x}")).collect()}
fn file(root:&Path,relative:&Path)->Result<PathBuf,String>{
    tsrust_compiler::ui_project::safe_relative(relative)?;
    let mut path=root.to_owned();
    for part in relative.components(){path.push(part);match fs::symlink_metadata(&path){Ok(value) if value.file_type().is_symlink()=>return Err(format!("Documentation paths cannot traverse symlinks: {}",path.display())),Ok(_)=>{},Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},Err(error)=>return Err(error.to_string())}}
    Ok(path)
}
fn pin(inputs:&mut BTreeMap<String,String>,name:String,bytes:&[u8])->Result<(),String>{
 let next=hash(bytes);if inputs.get(&name).is_some_and(|old|*old!=next){return Err(format!("{name} changed during documentation build; retry"));}inputs.insert(name,next);Ok(())
}
fn read(root:&Path,relative:&Path,inputs:&mut BTreeMap<String,String>)->Result<String,String>{
    let path=file(root,relative)?;let text=fs::read_to_string(&path).map_err(|error|format!("{}: {error}",path.display()))?;
    pin(inputs,relative.to_string_lossy().replace('\\',"/"),text.as_bytes())?;Ok(text)
}
fn markdown_files(root:&Path,relative:&Path,output:&mut Vec<PathBuf>)->Result<(),String>{
    let path=file(root,relative)?;let mut children=fs::read_dir(&path).map_err(|error|format!("{}: {error}",path.display()))?.collect::<Result<Vec<_>,_>>().map_err(|error|error.to_string())?;children.sort_by_key(|entry|entry.file_name());
    for child in children{let relative=relative.join(child.file_name());let path=file(root,&relative)?;if path.is_dir(){markdown_files(root,&relative,output)?;}else if path.extension().is_some_and(|value|value=="md"){output.push(relative);}}
    Ok(())
}
fn route(path:&Path)->Result<String,String>{
    let mut parts=path.with_extension("").components().map(|part|part.as_os_str().to_str().map(str::to_owned).ok_or("Documentation path must be UTF-8")).collect::<Result<Vec<_>,_>>()?;
    if parts.last().is_some_and(|part|part=="index"){parts.pop();}
    if parts.iter().any(|part|!docs::identifier(part)){return Err(format!("Documentation routes need lowercase hyphenated path segments: {}",path.display()));}
    Ok(format!("/{}",parts.join("/")))
}
struct Inputs {entry:SourceId,source:String}
impl SourceProvider for Inputs{
    fn resolve(&self,importer:Option<&SourceId>,path:&str)->Result<SourceId,String>{
        if importer.is_none()&&path==self.entry.0{return Ok(self.entry.clone());}
        if importer==Some(&self.entry)&&(path.starts_with("./")||path.starts_with("../")){
            // The virtual generated module is checked before its output directory exists.
            let mut normalized=PathBuf::new();
            for part in Path::new(&self.entry.0).parent().unwrap().join(path).components(){match part{std::path::Component::CurDir=>{},std::path::Component::ParentDir=>{normalized.pop();},other=>normalized.push(other.as_os_str())}}
            return FileSourceProvider.resolve(None,normalized.to_str().ok_or("Documentation imports must be UTF-8")?);
        }
        FileSourceProvider.resolve(importer,path)
    }
    fn read(&self,id:&SourceId)->Result<String,String>{if *id==self.entry{Ok(self.source.clone())}else{FileSourceProvider.read(id)}}
}
fn graph(root:&Path,config:&docs::Config)->Result<project::ProjectAnalysis,String>{
    let entry=SourceId(root.join("src/.tsrust-docs-inputs.tsr").to_string_lossy().into_owned());
    let mut source=String::new();let mut sources=config.api_sources.clone();sources.extend(config.examples.iter().map(|example|example.source.clone()));sources.sort();sources.dedup();
    for (index,path) in sources.iter().enumerate(){
        let target=if path=="std:ui"{path.clone()}else{file(root,Path::new(path))?.to_string_lossy().into_owned()};
        if path=="std:ui" {
            // Resolve the bundled facade in the actual project's configuration context.
            let provider=Inputs{entry:entry.clone(),source:"import {Node} from 'std:ui';function main(){}".into()};
            let graph=project::load_project_headers(&provider,&entry.0).map_err(|error|error.message)?;
            let exports=&graph.modules.get(&SourceId("std:ui".into())).ok_or("Missing std:ui facade")?.exports;
            let imports=exports.keys().enumerate().map(|(item,name)|format!("{name} as D{index}_{item}")).collect::<Vec<_>>().join(",");
            source.push_str(&format!("import {{{imports}}} from 'std:ui';\n"));continue;
        }
        let headers=project::load_project_headers(&FileSourceProvider,&target).map_err(|error|format!("{}: {}",error.location.source.0,error.message))?;
        let exports=&headers.modules[&headers.entry].exports;
        let imports=exports.keys().filter(|name|name.as_str()!="default").enumerate().map(|(item,name)|format!("{name} as D{index}_{item}")).collect::<Vec<_>>().join(",");
        if imports.is_empty(){return Err(format!("Documentation API/example has no named exports: {path}"));}
        source.push_str(&format!("import {{{imports}}} from {};\n",tsrust_compiler::json_string(&format!("../{path}"))));
    }
    source.push_str("function main() {}\n");let provider=Inputs{entry:entry.clone(),source};
    project::check_graph(project::load_project(&provider,&entry.0).map_err(|error|format!("{}: {}",error.location.source.0,error.message))?).map_err(|error|format!("{}: {}",error.location.source.0,error.message))
}
fn library(root:&Path,config:&docs::Config,inputs:&mut BTreeMap<String,String>,pages:&mut Vec<docs::Page>)->Result<Value,String>{
    let Some(path)=&config.inventory else{return Ok(json!({"components":[],"recipes":[],"blocks":[]}));};
    let source=read(root,Path::new(path),inputs)?;let inventory:Value=serde_json::from_str(&source).map_err(|error|error.to_string())?;
    let mut output=serde_json::Map::new();
    for (group,expected) in [("components",64),("recipes",3),("blocks",30)]{
        let entries=inventory[group].as_array().ok_or("Invalid library inventory group")?;
        if entries.len()!=expected{return Err(format!("Pinned inventory {group} changed: expected {expected}, got {}",entries.len()));}
        let mut names=BTreeSet::new();let mut public=Vec::new();
        for entry in entries{
            let name=entry["name"].as_str().ok_or("Inventory entry has no name")?;
            if !docs::identifier(name)||!names.insert(name){return Err("Invalid or duplicate component inventory name".into());}
            let status=entry["status"].as_str().ok_or("Inventory entry has no status")?;
            let description=entry["description"].as_str().unwrap_or("This entry is part of the pinned library implementation inventory.");
            public.push(json!({"name":name,"status":status,"description":description,"dependencies":entry.get("registryDependencies").cloned().unwrap_or(json!([]))}));
            let route=format!("/ui/{group}/{name}");
            if !pages.iter().any(|page|page.route==route){
                let text=format!("# {}\n\n{}\n\nImplementation status: {}. This is an inventory destination, not a working preview.\n",name.split('-').map(|word|{let mut chars=word.chars();chars.next().unwrap().to_uppercase().collect::<String>()+chars.as_str()}).collect::<Vec<_>>().join(" "),description,status);
                pages.push(docs::Page{metadata:docs::Metadata{id:format!("{group}-{name}"),group:group.into(),order:1000,status:if status=="partial"{docs::Status::Partial}else{docs::Status::Pending},targets:vec![docs::Target::Web,docs::Target::Native],examples:vec![],api:vec![],component:(group=="components").then(||name.to_string())},route,source:format!("inventory:{path}#{group}/{name}"),sha256:hash(text.as_bytes()),document:docs::parse(&text,0)?,markdown:text});
            }
        }
        output.insert(group.into(),Value::Array(public));
    }
    Ok(Value::Object(output))
}
fn links(root:&Path,content:&Path,pages:&mut [docs::Page],inputs:&mut BTreeMap<String,String>)->Result<(),String>{
    let destinations=pages.iter().map(|page|(page.route.clone(),page.document.headings.iter().map(|heading|heading.anchor.clone()).collect::<BTreeSet<_>>())).collect::<BTreeMap<_,_>>();
    let sources=pages.iter().filter(|page|Path::new(&page.source).extension().is_some_and(|value|value=="md")).map(|page|(PathBuf::from(&page.source),page.route.clone())).collect::<BTreeMap<_,_>>();
    for page in pages{
        for node in &mut page.document.nodes{
            if !matches!(node.kind,docs::Kind::Link|docs::Kind::Image){continue;}
            let value=node.destination.clone();let lower=value.to_ascii_lowercase();
            if lower.starts_with("https://")||lower.starts_with("http://")||lower.starts_with("mailto:"){continue;}
            let (path,fragment)=value.split_once('#').map_or((value.as_str(),None),|(path,fragment)|(path,Some(fragment)));
            if path.contains(['%', '?']){return Err(format!("Internal documentation paths use literal directory routes without percent encoding or query: {value}"));}
            let target=if path.is_empty(){page.route.clone()}else if path.starts_with('/') {path.trim_end_matches('/').to_string()}else{
                let base=Path::new(&page.source).parent().unwrap_or(content);let mut normalized=Vec::new();
                for part in base.join(path).components(){match part{std::path::Component::Normal(value)=>normalized.push(value.to_owned()),std::path::Component::ParentDir=>{if normalized.pop().is_none(){return Err("Documentation link escapes the project".into());}},std::path::Component::CurDir=>{},_=>return Err("Invalid relative documentation link".into())}}
                let relative=normalized.iter().collect::<PathBuf>();
                if node.kind==docs::Kind::Image{
                    let path=file(root,&relative)?;let bytes=fs::read(&path).map_err(|error|format!("Missing documentation image {}: {error}",path.display()))?;inputs.insert(relative.to_string_lossy().replace('\\',"/"),hash(&bytes));
                    node.destination=format!("asset:{}",relative.to_string_lossy().replace('\\',"/"));continue;
                }
                sources.get(&relative).cloned().ok_or_else(||format!("Unresolved documentation link in {}: {value}",page.source))?
            };
            let target=if target.is_empty(){"/".to_string()}else{target};
            let anchors=destinations.get(&target).ok_or_else(||format!("Unknown documentation route in {}: {value}",page.source))?;
            if fragment.is_some_and(|fragment|!anchors.contains(fragment)){return Err(format!("Unknown documentation heading in {}: {value}",page.source));}
            node.destination=match fragment{Some(fragment)=>format!("{target}#{fragment}"),None=>target};
        }
    }
    Ok(())
}
fn publish(root:&Path,name:&str,bytes:&[u8])->Result<(),String>{
    let path=file(root,&Path::new("target/docs-content").join(name))?;
    if path.exists()&&fs::read(&path).map_err(|error|error.to_string())?==bytes{return Ok(());}
    let staged=path.with_extension(format!("{}.new",path.extension().unwrap_or_default().to_string_lossy()));
    let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&staged).map_err(|error|format!("Cannot stage {}: {error}",staged.display()))?;
    let result=(||{file.write_all(bytes).map_err(|error|error.to_string())?;file.sync_all().map_err(|error|error.to_string())?;drop(file);fs::rename(&staged,&path).map_err(|error|error.to_string())})();
    if result.is_err(){let _=fs::remove_file(&staged);}result
}
pub(super) fn run(mut arguments:impl Iterator<Item=String>)->Result<u8,String>{
    if arguments.next().as_deref()!=Some("content"){return Err("Usage: tsrust docs content [--cwd project]".into());}
    let mut directory=env::current_dir().map_err(|error|error.to_string())?;
    while let Some(argument)=arguments.next(){match argument.as_str(){"--cwd"=>directory=PathBuf::from(arguments.next().ok_or("--cwd needs a directory")?),_=>return Err(format!("Unknown docs content option: {argument}"))}}
    let root=directory.canonicalize().map_err(|error|error.to_string())?;
    super::project_config::load(&root)?;
    let mut inputs=BTreeMap::new();let text=read(&root,Path::new("tsrust.config.json"),&mut inputs)?;let value:Value=serde_json::from_str(&text).map_err(|error|error.to_string())?;
    let config:docs::Config=serde_json::from_value(value.get("documentation").cloned().ok_or("Add documentation configuration to tsrust.config.json")?).map_err(|error|error.to_string())?;config.validate()?;
    let content=Path::new(&config.content);let mut files=Vec::new();markdown_files(&root,content,&mut files)?;
    let mut pages=Vec::new();
    for path in files{
        let source=read(&root,&path,&mut inputs)?;let (metadata,offset)=docs::metadata(&source).map_err(|error|format!("{}: {error}",path.display()))?;
        let document=docs::parse(&source,offset).map_err(|error|format!("{}: {error}",path.display()))?;
        if document.headings.iter().filter(|heading|heading.level==1).count()!=1{return Err(format!("{} needs exactly one page heading",path.display()));}
        pages.push(docs::Page{metadata,route:route(path.strip_prefix(content).unwrap())?,source:path.to_string_lossy().replace('\\',"/"),sha256:hash(source.as_bytes()),document,markdown:source});
    }
    let library=library(&root,&config,&mut inputs,&mut pages)?;
    let mut ids=BTreeSet::new();let mut routes=BTreeSet::new();
    for page in &pages{if !ids.insert(&page.metadata.id)||!routes.insert(&page.route){return Err(format!("Duplicate documentation id/route: {} {}",page.metadata.id,page.route));}}
    links(&root,content,&mut pages,&mut inputs)?;
    let checked=graph(&root,&config)?;
    let mut examples=Vec::new();
    for example in &config.examples{
        let path=file(&root,Path::new(&example.source))?.canonicalize().map_err(|error|error.to_string())?;
        let id=SourceId(path.to_string_lossy().into_owned());let module=checked.graph.modules.get(&id).ok_or("Example is outside the checked graph")?;
        if !module.exports.contains_key(&example.export){return Err(format!("{} does not export {}",example.source,example.export));}
        let code=read(&root,Path::new(&example.source),&mut inputs)?;
        if code!=module.source.text{return Err(format!("{} changed during documentation checking; retry",example.source));}
        examples.push(docs::Example{kind:example.kind.clone(),id:example.id.clone(),source:example.source.clone(),export:example.export.clone(),sha256:hash(code.as_bytes()),tokens:docs::highlight(&code,"tsrx")?,code});
    }
    for page in &pages{for id in &page.metadata.examples{if !examples.iter().any(|example|example.id==*id){return Err(format!("Unknown example {id} in {}",page.source));}}}
    let api_sources=config.api_sources.iter().map(|path|if path=="std:ui"{Ok(SourceId(path.clone()))}else{let path=file(&root,Path::new(path))?.canonicalize().map_err(|error|error.to_string())?;Ok(SourceId(path.to_string_lossy().into_owned()))}).collect::<Result<Vec<_>,String>>()?;
    let mut api=docs::api(&checked,&api_sources)?;
    for page in &pages{for reference in &page.metadata.api{let definition=docs::builtin(reference)?;if !api.iter().any(|entry|entry.name==definition.name&&entry.source==definition.source){api.push(definition);}}}
    for definition in &mut api{definition.source=definition.source.strip_prefix(&format!("{}/",root.display())).unwrap_or(&definition.source).to_string();for field in &mut definition.fields{field.source=field.source.strip_prefix(&format!("{}/",root.display())).unwrap_or(&field.source).to_string();}}
    for (id,module) in &checked.graph.modules{if id.0.starts_with("std:"){pin(&mut inputs,id.0.clone(),module.source.text.as_bytes())?;}else if let Ok(relative)=Path::new(&id.0).strip_prefix(&root){if id.0!=checked.graph.entry.0{pin(&mut inputs,relative.to_string_lossy().replace('\\',"/"),module.source.text.as_bytes())?;}}}
    pages.sort_by(|a,b|(&a.metadata.group,a.metadata.order,&a.route).cmp(&(&b.metadata.group,b.metadata.order,&b.route)));
    let catalog=docs::Catalog{schema_version:1,pages,examples,inputs,api,library};
    let json=serde_json::to_string_pretty(&catalog).map_err(|error|error.to_string())?;
    let module=super::docs_model::module(&json,&catalog.examples)?;
    for (path,digest) in &catalog.inputs{if !path.starts_with("std:") {let bytes=fs::read(file(&root,Path::new(path))?).map_err(|error|error.to_string())?;if hash(&bytes)!=*digest{return Err(format!("{path} changed during documentation build; retry"));}}}
    let generated=Inputs{entry:SourceId(root.join("target/docs-content/catalog.tsr").to_string_lossy().into_owned()),source:module.clone()};
    project::check_graph(project::load_project(&generated,&generated.entry.0).map_err(|error|format!("Generated documentation module: {}",error.message))?).map_err(|error|format!("Generated documentation module: {}",error.message))?;
    let destination=file(&root,Path::new("target/docs-content"))?;
    if destination.exists(){
        let old=fs::read_to_string(file(&root,Path::new("target/docs-content/inputs.json"))?).map_err(|_|"Refusing to overwrite an unowned docs-content directory")?;
        let old:Value=serde_json::from_str(&old).map_err(|_|"Invalid docs-content ownership manifest")?;
        if old["generator"]!="tsrust-docs-content-v1"{return Err("Refusing to overwrite an unowned docs-content directory".into());}
    }
    fs::create_dir_all(&destination).map_err(|error|error.to_string())?;
    let manifest=json!({"generator":"tsrust-docs-content-v1","inputs":catalog.inputs,"catalogSha256":hash(json.as_bytes()),"moduleSha256":hash(module.as_bytes())});
    publish(&root,"catalog.json",json.as_bytes())?;publish(&root,"inputs.json",serde_json::to_string_pretty(&manifest).unwrap().as_bytes())?;publish(&root,"catalog.tsr",module.as_bytes())?;
    println!("Built {} documentation pages, {} checked examples and {} API declarations (offline).",catalog.pages.len(),catalog.examples.len(),catalog.api.len());Ok(0)
}
