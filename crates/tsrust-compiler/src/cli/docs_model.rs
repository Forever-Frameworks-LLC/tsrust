//! Small checked loaders and page-sized data factories; no escaped catalog JSON program.
use std::collections::BTreeMap;
use tsrust_compiler::documentation::{self as docs,MODEL_SOURCE as MODEL};
use serde_json::json;
pub(super) struct Publication {pub files:BTreeMap<String,String>}
fn data(files:&mut BTreeMap<String,String>,path:String,ty:&str,value:impl serde::Serialize)->Result<(),String>{
 let literal=serde_json::to_string_pretty(&value).map_err(|e|e.to_string())?;
 let source=format!("// Generated documentation data. Edit the declared source instead.\nimport {{{ty}}} from '../model';\nexport function load():{ty}{{return {literal};}}\n");
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
 let types=model.lines().filter_map(|line|line.strip_prefix("export interface ")).filter_map(|line|line.split_whitespace().next()).collect::<Vec<_>>();
 let imports=types.iter().map(|name|format!("{name} as Stored{name}")).collect::<Vec<_>>().join(",");
 let aliases=types.iter().map(|name|format!("export type {name}=Stored{name};\n")).collect::<String>();
 let mut source=format!("// Generated checked documentation loaders. Content lives in page-sized modules.\nimport {{{imports}}} from './model';\n{aliases}\n");
 data(&mut files,"data/library.tsr".into(),"DocLibrary",&catalog.library)?;
 source.push_str("import {load as LibraryData} from './data/library';\n");
 let mut registry=String::new();let mut all_pages=Vec::new();let mut all_examples=Vec::new();let mut all_api=Vec::new();let mut page_loaders=String::new();let mut example_loaders=String::new();let mut search=Vec::new();let mut navigation=Vec::new();let mut page_ids=String::new();
 for (i,page) in catalog.pages.iter().enumerate(){data(&mut files,format!("pages/{}.tsr",page.metadata.id),"DocPage",page)?;source.push_str(&format!("import {{load as Page{i}}} from './pages/{}';\n",page.metadata.id));all_pages.push(format!("Page{i}()"));navigation.push(json!({"id":page.metadata.id,"group":page.metadata.group,"route":page.route,"title":page.document.title}));
  page_ids.push_str(&format!("if(route==={}){{return {};}}\n",tsrust_compiler::json_string(&page.route),tsrust_compiler::json_string(&page.metadata.id)));
  let selected_api=catalog.api.iter().enumerate().filter(|(_,entry)|belongs(page,entry)).collect::<Vec<_>>();
  let selected_examples=catalog.examples.iter().enumerate().filter(|(_,example)|page.metadata.examples.contains(&example.id)).collect::<Vec<_>>();
  let mut text=format!("{} {}",page.document.title,page.markdown);
  for (_,entry) in &selected_api {text.push_str(&format!(" {} {} {}",entry.name,entry.signature,entry.documentation));for field in &entry.fields{text.push_str(&format!(" {} {} {}",field.name,field.signature,field.documentation));}}
  for (_,example) in &selected_examples{text.push(' ');text.push_str(&example.code);}
  search.push(json!({"route":page.route,"text":text}));
  let api=selected_api.iter().map(|(i,_)|format!("Api{i}()")).collect::<Vec<_>>().join(",");let examples=selected_examples.iter().map(|(i,_)|format!("Example{i}()")).collect::<Vec<_>>().join(",");
  page_loaders.push_str(&format!("if(route==={}){{return {{schemaVersion:2,pages:[Page{i}()],examples:[{examples}],api:[{api}],library:{{components:[],recipes:[],blocks:[]}}}};}}\n",tsrust_compiler::json_string(&page.route)));
 }
 let components=catalog.examples.iter().any(|e|e.kind==docs::ExampleKind::Component);if components{source.push_str("import {Node} from 'std:ui';\n");}
 for (i,example) in catalog.examples.iter().enumerate(){data(&mut files,format!("examples/{}.tsr",example.id),"DocExample",example)?;source.push_str(&format!("import {{load as Example{i}}} from './examples/{}';\n",example.id));all_examples.push(format!("Example{i}()"));example_loaders.push_str(&format!("if(id==={}){{return Example{i}();}}\n",tsrust_compiler::json_string(&example.id)));
  let preview=if example.kind==docs::ExampleKind::Component{source.push_str(&format!("import {{{} as Preview{i}}} from {};\n",example.export,tsrust_compiler::json_string(&format!("../../{}",example.source))));format!("Preview{i}()")}else{"null".into()};registry.push_str(&format!("if(id==={}){{return {preview};}}\n",tsrust_compiler::json_string(&example.id)));
 }
 for (i,entry) in catalog.api.iter().enumerate(){data(&mut files,format!("api/declaration-{i}.tsr"),"DocApi",entry)?;source.push_str(&format!("import {{load as Api{i}}} from './api/declaration-{i}';\n"));all_api.push(format!("Api{i}()"));}
 // These data contain only strings and scalars. They never instantiate a component.
 for (name,ty,value) in [("navigation","DocNavigation[]",json!(navigation)),("search","DocSearchEntry[]",json!(search))]{
  let element=ty.trim_end_matches("[]");let literal=serde_json::to_string_pretty(&value).map_err(|e|e.to_string())?;
  files.insert(format!("{name}.tsr"),tsrust_compiler::format(&format!("import {{{element}}} from './model';\nexport function load():{ty}{{return {literal};}}" )).map_err(|e|e.message)?);
  source.push_str(&format!("import {{load as {}Data}} from './{name}';\n",if name=="navigation"{"Navigation"}else{"Search"}));
 }
 source.push_str(&format!("export function pageId(route:ReadOnly<string>):string{{{page_ids}throw 'Unknown documentation route';}}\nexport function loadNavigation():DocNavigation[]{{return NavigationData();}}\nexport function loadSearch():DocSearchEntry[]{{return SearchData();}}\nexport function loadExample(id:ReadOnly<string>):DocExample{{{example_loaders}throw 'Unknown documentation example';}}\nexport function loadPage(route:ReadOnly<string>):DocCatalog{{{page_loaders}throw 'Unknown documentation route';}}\n/** Explicit full snapshot for tooling; application routes use loadPage instead. */\nexport function loadCatalog():DocCatalog{{return {{schemaVersion:2,pages:[{}],examples:[{}],api:[{}],library:LibraryData()}};}}\nexport function renderExample(id:ReadOnly<string>):{}{{{registry}throw 'Unknown documentation example';}}\n",all_pages.join(","),all_examples.join(","),all_api.join(","),if components{"Node|null"}else{"null"}));
 files.insert("catalog.tsr".into(),tsrust_compiler::format(&source).map_err(|e|e.message)?);Ok(Publication{files})
}
