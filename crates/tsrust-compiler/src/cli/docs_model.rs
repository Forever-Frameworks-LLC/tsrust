//! Small typed loaders and readable JSON text assets; payloads are not record-literal programs.
use std::collections::BTreeMap;
use tsrust_compiler::documentation::{self as docs,MODEL_SOURCE as MODEL};
use serde_json::json;
pub(super) struct Publication {pub files:BTreeMap<String,String>}
fn decoder(ty:&str)->String{format!("decode{}{}",ty.trim_end_matches("[]"),if ty.ends_with("[]"){"List"}else{""})}
fn data(files:&mut BTreeMap<String,String>,path:String,ty:&str,value:impl serde::Serialize)->Result<(),String>{
 let literal=serde_json::to_string_pretty(&value).map_err(|e|e.to_string())?;
 let payload=format!("{}.json.txt",path.trim_end_matches(".tsr"));let filename=payload.rsplit('/').next().unwrap();
 let source=format!("// Generated typed loader. Readable JSON text lives beside this module.\nimport {{{}}} from '../model';\nimport {{{decode}}} from '../decoder';\nimport content from './{filename}';\nexport function load():{ty}{{return {decode}(content);}}\n",ty.trim_end_matches("[]"),decode=decoder(ty));
 files.insert(payload,literal);
 files.insert(path,tsrust_compiler::format(&source).map_err(|e|e.message)?);Ok(())
}
fn belongs(page:&docs::Page,entry:&docs::ApiDefinition)->bool {
 if let Some(component)=&page.metadata.component {
  let name=component.split('-').map(|part|{let mut chars=part.chars();chars.next().map(|ch|ch.to_uppercase().to_string()+chars.as_str()).unwrap_or_default()}).collect::<String>();
  if entry.source.ends_with(&format!("/{component}.tsrx"))||entry.name==name||entry.name==format!("{name}Props"){return true;}
 }
 page.metadata.api.iter().any(|reference|entry.name==format!("{}.{}",reference.receiver,reference.name)||reference.receiver=="std:ui"&&entry.name==reference.name&&entry.kind=="interface")
}
pub(super) fn module(catalog:&docs::Catalog)->Result<Publication,String>{
 let mut files=BTreeMap::new();
 let model=format!("{MODEL}\n/** Lightweight route metadata; no Markdown, API trees or preview state. */\nexport interface DocNavigation {{id:string;group:string;route:string;title:string;}}\n/** Exact source-derived search text, loaded only by the documentation view. */\nexport interface DocSearchEntry {{route:string;text:string;}}\n");
 files.insert("model.tsr".into(),tsrust_compiler::format(&model).map_err(|e|e.message)?);
 let decoder_types=["DocPage","DocExample","DocApi","DocLibrary","DocNavigation[]","DocSearchEntry[]"];
 let mut decoders=format!("// One checked JSON decoder per DTO, shared by every payload.\nimport {{{}}} from './model';\n",decoder_types.iter().map(|ty|ty.trim_end_matches("[]")).collect::<Vec<_>>().join(","));
 for ty in decoder_types{decoders.push_str(&format!("export function {}(text:ReadOnly<string>):{ty}{{return JSON.parse<{ty}>(text);}}\n",decoder(ty)));}
 files.insert("decoder.tsr".into(),tsrust_compiler::format(&decoders).map_err(|e|e.message)?);
 let types=model.lines().filter_map(|line|line.strip_prefix("export interface ")).filter_map(|line|line.split_whitespace().next()).collect::<Vec<_>>();
 let imports=types.iter().map(|name|format!("{name} as Stored{name}")).collect::<Vec<_>>().join(",");
 let aliases=types.iter().map(|name|format!("export type {name}=Stored{name};\n")).collect::<String>();
 let mut source=format!("// Generated checked documentation loaders. Content lives in page-sized modules.\nimport {{{imports}}} from './model';\n{aliases}\n");
 data(&mut files,"data/library.tsr".into(),"DocLibrary",&catalog.library)?;
 source.push_str("import {load as LibraryData} from './data/library';\n");
 let mut registry=String::new();let mut page_data_cases=String::new();let mut all_examples=Vec::new();let mut page_loaders=String::new();let mut example_loaders=String::new();let mut search=Vec::new();let mut navigation=Vec::new();let mut page_ids=String::new();
 for (i,page) in catalog.pages.iter().enumerate(){data(&mut files,format!("pages/{}.tsr",page.metadata.id),"DocPage",page)?;source.push_str(&format!("import {{load as Page{i}}} from './pages/{}';\n",page.metadata.id));page_data_cases.push_str(&format!("if(index==={i}){{return Page{i}();}}\n"));navigation.push(json!({"id":page.metadata.id,"group":page.metadata.group,"route":page.route,"title":page.document.title}));
  page_ids.push_str(&format!("if(route==={}){{return {};}}\n",tsrust_compiler::json_string(&page.route),tsrust_compiler::json_string(&page.metadata.id)));
  let selected_api=catalog.api.iter().enumerate().filter(|(_,entry)|belongs(page,entry)).collect::<Vec<_>>();
  let selected_examples=catalog.examples.iter().enumerate().filter(|(_,example)|page.metadata.examples.contains(&example.id)).collect::<Vec<_>>();
  let mut text=format!("{} {}",page.document.title,page.markdown);
  for (_,entry) in &selected_api {text.push_str(&format!(" {} {} {}",entry.name,entry.signature,entry.documentation));for field in &entry.fields{text.push_str(&format!(" {} {} {}",field.name,field.signature,field.documentation));}}
  for (_,example) in &selected_examples{text.push(' ');text.push_str(&example.code);}
  search.push(json!({"route":page.route,"text":text}));
  let api=selected_api.iter().map(|(i,_)|format!("Api({i})")).collect::<Vec<_>>().join(",");let examples=selected_examples.iter().map(|(i,_)|format!("Example{i}()")).collect::<Vec<_>>().join(",");
  let mut context=format!("// Constructs only this page and its referenced examples/APIs.\nimport {{DocCatalog}} from '../model';import {{load as page}} from '../pages/{}';import {{load as Api}} from '../api';\n",page.metadata.id);
  for (index,example) in &selected_examples{context.push_str(&format!("import {{load as Example{index}}} from '../examples/{}';\n",example.id));}
  context.push_str(&format!("export function load():DocCatalog{{return {{schemaVersion:2,pages:[page()],examples:[{examples}],api:[{api}],library:{{components:[],recipes:[],blocks:[]}}}};}}\n"));
  files.insert(format!("contexts/{}.tsr",page.metadata.id),tsrust_compiler::format(&context).map_err(|e|e.message)?);
  source.push_str(&format!("import {{load as Context{i}}} from './contexts/{}';\n",page.metadata.id));
  page_loaders.push_str(&format!("if(route==={}){{return Context{i}();}}\n",tsrust_compiler::json_string(&page.route)));
 }
 let components=catalog.examples.iter().any(|e|e.kind==docs::ExampleKind::Component);if components{source.push_str("import {Node} from 'std:ui';\n");}
 for (i,example) in catalog.examples.iter().enumerate(){data(&mut files,format!("examples/{}.tsr",example.id),"DocExample",example)?;source.push_str(&format!("import {{load as Example{i}}} from './examples/{}';\n",example.id));all_examples.push(format!("Example{i}()"));example_loaders.push_str(&format!("if(id==={}){{return Example{i}();}}\n",tsrust_compiler::json_string(&example.id)));
  let preview=if example.kind==docs::ExampleKind::Component{source.push_str(&format!("import {{{} as Preview{i}}} from {};\n",example.export,tsrust_compiler::json_string(&format!("../../{}",example.source))));format!("Preview{i}()")}else{"null".into()};registry.push_str(&format!("if(id==={}){{return {preview};}}\n",tsrust_compiler::json_string(&example.id)));
 }
 // Keep construction branches in bounded functions: Wasm reserves their temporaries on entry.
 let mut api_source=String::from("// Shared API dispatcher; payloads remain readable UTF-8 text.\nimport {DocApi} from './model';\n");let mut api_cases=String::new();
 for (group,entries) in catalog.api.chunks(32).enumerate(){
  let mut group_source=String::from("import {DocApi} from '../model';import {decodeDocApi} from '../decoder';\n");let mut cases=String::new();
  for (offset,entry) in entries.iter().enumerate(){let i=group*32+offset;
   files.insert(format!("api/declaration-{i}.json.txt"),serde_json::to_string_pretty(entry).map_err(|e|e.to_string())?);
   group_source.push_str(&format!("import content{i} from './declaration-{i}.json.txt';\n"));
   cases.push_str(&format!("if(index==={i}){{return decodeDocApi(content{i});}}\n"));
  }
  group_source.push_str(&format!("export function load(index:int):DocApi{{{cases}throw 'Unknown documentation API index';}}\n"));
  files.insert(format!("api/group-{group}.tsr"),tsrust_compiler::format(&group_source).map_err(|e|e.message)?);
  api_source.push_str(&format!("import {{load as Group{group}}} from './api/group-{group}';\n"));
  api_cases.push_str(&format!("if(index<{}){{return Group{group}(index);}}\n",(group+1)*32));
 }
 api_source.push_str(&format!("export function load(index:int):DocApi{{{api_cases}throw 'Unknown documentation API index';}}\n"));
 files.insert("api.tsr".into(),tsrust_compiler::format(&api_source).map_err(|e|e.message)?);source.push_str("import {load as Api} from './api';\n");
 // These data contain only strings and scalars. They never instantiate a component.
 for (name,ty,value) in [("navigation","DocNavigation[]",json!(navigation)),("search","DocSearchEntry[]",json!(search))]{
  data(&mut files,format!("data/{name}.tsr"),ty,value)?;
  files.insert(format!("{name}.tsr"),tsrust_compiler::format(&format!("import {{{element}}} from './model';import {{load as data}} from './data/{name}';export function load():{ty}{{return data();}}",element=ty.trim_end_matches("[]"))).map_err(|e|e.message)?);
  source.push_str(&format!("import {{load as {}Data}} from './{name}';\n",if name=="navigation"{"Navigation"}else{"Search"}));
 }
 source.push_str(&format!("export function pageId(route:ReadOnly<string>):string{{{page_ids}throw 'Unknown documentation route';}}\nexport function loadNavigation():DocNavigation[]{{return NavigationData();}}\nexport function loadSearch():DocSearchEntry[]{{return SearchData();}}\nexport function loadExample(id:ReadOnly<string>):DocExample{{{example_loaders}throw 'Unknown documentation example';}}\nexport function loadPage(route:ReadOnly<string>):DocCatalog{{{page_loaders}throw 'Unknown documentation route';}}\nfunction pageData(index:int):DocPage{{{page_data_cases}throw 'Unknown documentation page index';}}\n/** Explicit full snapshot for tooling; application routes use loadPage instead. */\nexport function loadCatalog():DocCatalog{{let pages:DocPage[]=[];for(let index=0;index<{};index++){{pages.push(pageData(index));}}let api:DocApi[]=[];for(let index=0;index<{};index++){{api.push(Api(index));}}return {{schemaVersion:2,pages,examples:[{}],api,library:LibraryData()}};}}\nexport function renderExample(id:ReadOnly<string>):{}{{{registry}throw 'Unknown documentation example';}}\n",catalog.pages.len(),catalog.api.len(),all_examples.join(","),if components{"Node|null"}else{"null"}));
 files.insert("catalog.tsr".into(),tsrust_compiler::format(&source).map_err(|e|e.message)?);Ok(Publication{files})
}
