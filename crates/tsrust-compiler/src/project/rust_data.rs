//! Plain owned Rust data is projected field by field, without serialization.
use super::*;
#[derive(Clone,Debug)]
pub enum Data {Struct(Vec<(String,Type)>),Enum(Vec<Variant>)}
#[derive(Clone,Debug)]
pub struct Variant {pub name:String,pub fields:Vec<(String,Type)>,pub tuple:bool,pub named:bool}
fn fields(fields:&syn::Fields,public:bool,options:&str)->Result<Vec<(String,Type)>,(proc_macro2::Span,String)> {
 fields.iter().enumerate().map(|(index,field)| {
  if public && !matches!(field.vis,syn::Visibility::Public(_)){return Err((field.span(),"Boundary struct fields must be public.".into()));}
  if field.attrs.iter().any(|attr|!attr.path().is_ident("doc")){return Err((field.span(),"Boundary fields accept only documentation attributes.".into()));}
  let name=field.ident.as_ref().map(ToString::to_string).unwrap_or_else(||if fields.len()==1{"value".into()}else{format!("item{index}")});
  if name.starts_with("r#"){return Err((field.span(),"Raw field identifiers are unsupported at the boundary.".into()));}
  let ty=shape(&field.ty,true,options).map_err(|message|(field.span(),message))?;
  if matches!(ty,Type::Void){return Err((field.span(),"Boundary fields need concrete owned data types.".into()));}
  Ok((name,ty))
 }).collect()
}
fn imported_paths(file:&syn::File)->BTreeMap<String,Vec<String>>{
 let mut aliases=BTreeMap::new();let mut pending=file.items.iter().filter_map(|item|if let syn::Item::Use(item)=item{Some((&item.tree,Vec::<String>::new()))}else{None}).collect::<Vec<_>>();
 while let Some((tree,mut prefix))=pending.pop(){match tree{
  syn::UseTree::Path(path)=>{prefix.push(path.ident.to_string());pending.push((&path.tree,prefix));},
  syn::UseTree::Group(group)=>for tree in &group.items{pending.push((tree,prefix.clone()));},
  syn::UseTree::Name(name)=>{if name.ident!="self"{prefix.push(name.ident.to_string());}if let Some(name)=prefix.last(){aliases.insert(name.clone(),prefix);}},
  syn::UseTree::Rename(name)=>{if name.ident!="self"{prefix.push(name.ident.to_string());}aliases.insert(name.rename.to_string(),prefix);},
  syn::UseTree::Glob(_)=>{}
 }}aliases
}
fn standard_drop(path:&syn::Path,aliases:&BTreeMap<String,Vec<String>>)->bool{
 let mut names=path.segments.iter().map(|segment|segment.ident.to_string()).collect::<Vec<_>>();let mut seen=BTreeSet::new();
 if path.leading_colon.is_none(){while let Some(first)=names.first(){let Some(prefix)=aliases.get(first)else{break};if !seen.insert(first.clone()){return false;}let mut resolved=prefix.clone();resolved.extend(names.into_iter().skip(1));names=resolved;}}
 matches!(names.as_slice(),[name]if name=="Drop") || matches!(names.as_slice(),[root,ops,name]if (root=="std"||root=="core")&&ops=="ops"&&name=="Drop")
}
pub(super) fn collect(file:&syn::File,options:&str)->Result<BTreeMap<String,Data>,(proc_macro2::Span,String)> {
 let mut types=BTreeMap::new();
 for item in &file.items {
  let (name,generics,attrs,data)=match item {
   syn::Item::Struct(item) if matches!(item.vis,syn::Visibility::Public(_))=> {
    if !matches!(item.fields,syn::Fields::Named(_)){return Err((item.span(),"Boundary structs require named fields.".into()));}
    (&item.ident,&item.generics,&item.attrs,Data::Struct(fields(&item.fields,true,options)?))
   },
   syn::Item::Enum(item) if matches!(item.vis,syn::Visibility::Public(_))=> {
    if item.variants.is_empty(){return Err((item.span(),"Boundary enums require at least one variant.".into()));}
    let variants=item.variants.iter().map(|variant| {
     if variant.discriminant.is_some() || variant.attrs.iter().any(|attr|!attr.path().is_ident("doc")){return Err((variant.span(),"Boundary enum variants accept only documentation attributes and no explicit discriminants.".into()));}
     Ok(Variant{name:variant.ident.to_string(),fields:fields(&variant.fields,false,options)?,tuple:matches!(variant.fields,syn::Fields::Unnamed(_)),named:matches!(variant.fields,syn::Fields::Named(_))})
    }).collect::<Result<Vec<_>,_>>()?;
    (&item.ident,&item.generics,&item.attrs,Data::Enum(variants))
   },_=>continue
  };
  if !generics.params.is_empty() || generics.where_clause.is_some() || attrs.iter().any(|attr|!attr.path().is_ident("doc") && !attr.path().is_ident("derive")) || name.to_string().starts_with("r#") {return Err((name.span(),"Boundary data must be concrete and have only documentation or derive attributes.".into()));}
  types.insert(name.to_string(),data);
 }
 let aliases=imported_paths(file);
 for item in &file.items {
  let syn::Item::Impl(item)=item else{continue};
  let destructor=item.trait_.as_ref().is_some_and(|(path,_)|standard_drop(path,&aliases));
  if !destructor{continue;}
  let syn::Type::Path(owner)=item.self_ty.as_ref()else{continue};if owner.qself.is_some(){continue;}
  let path=&owner.path;let local=path.leading_colon.is_none() && (path.segments.len()==1 || path.segments.len()==2 && path.segments[0].ident=="self");
  let crate_qualified=path.leading_colon.is_none() && path.segments.len()==2 && path.segments[0].ident=="crate";
  if (local||crate_qualified) && path.segments.last().is_some_and(|segment|types.contains_key(&segment.ident.to_string())) {
   return Err((item.span(),"Boundary data cannot implement Drop; keep resource owners private to the Rust adapter. Qualified crate boundary owners require an explicit supported local declaration.".into()));
  }
 }
 for name in types.keys(){validate(&Type::Named(name.clone()),&types,&mut BTreeSet::new()).map_err(|message|(proc_macro2::Span::call_site(),message))?;}
 Ok(types)
}
pub(super) fn validate(ty:&Type,types:&BTreeMap<String,Data>,active:&mut BTreeSet<String>)->Result<(),String> {
 match ty {
  Type::Named(name)=> {
   let data=types.get(name).ok_or_else(||format!("Unsupported boundary type {name}; declare a public struct or enum in the binding module."))?;
   if active.len()>=128{return Err("Boundary type nesting exceeds 128.".into());}
   if !active.insert(name.clone()){return Err(format!("Recursive boundary data {name} is unsupported."));}
   let fields=match data {Data::Struct(fields)=>fields.iter().collect::<Vec<_>>(),Data::Enum(variants)=>variants.iter().flat_map(|variant|variant.fields.iter()).collect()};
   for (_,ty) in fields{validate(ty,types,active)?;}active.remove(name);
  },Type::Array(inner)|Type::Option(inner,_)|Type::Borrow(inner,_)=>validate(inner,types,active)?,Type::Result(ok,err)=>{validate(ok,types,active)?;validate(err,types,active)?;},_=>{}
 }Ok(())
}
pub(super) fn declarations(types:&BTreeMap<String,Data>,file:&syn::File)->String {declarations_impl(types,file,true)}
pub(super) fn declarations_only(types:&BTreeMap<String,Data>,file:&syn::File)->String {declarations_impl(types,file,false)}
fn declarations_impl(types:&BTreeMap<String,Data>,file:&syn::File,seeds:bool)->String {
 let mut source=String::new();
 let mut docs=BTreeMap::new();
 let mut field_docs=BTreeMap::new();
 for item in &file.items {
  let records=match item {
   syn::Item::Struct(item)=>vec![(item.ident.to_string(),&item.fields)],
   syn::Item::Enum(item)=>item.variants.iter().map(|variant|(format!("{}{}",item.ident,variant.ident),&variant.fields)).collect(),_=>vec![]
  };
  for (name,fields) in records {for (index,field) in fields.iter().enumerate(){let field_name=field.ident.as_ref().map(ToString::to_string).unwrap_or_else(||if fields.len()==1{"value".into()}else{format!("item{index}")});field_docs.insert((name.clone(),field_name),documentation(&field.attrs));}}
 }
 for item in &file.items {match item {
  syn::Item::Struct(item)=>{docs.insert(item.ident.to_string(),documentation(&item.attrs));},
  syn::Item::Enum(item)=>{docs.insert(item.ident.to_string(),documentation(&item.attrs));for variant in &item.variants{docs.insert(format!("{}{}",item.ident,variant.ident),documentation(&variant.attrs));}},_=>{}
 }}
 let record=|source:&mut String,name:&str,fields:&[(String,Type)]| {
  source.push_str(&format!("/** {}\nOwned Rust data. Fields move across the binding boundary. */\nexport type {name} = {{\n{}}};\n",docs.get(name).map(String::as_str).unwrap_or(""),fields.iter().map(|(field,ty)|{let docs=field_docs.get(&(name.to_owned(),field.clone())).map(String::as_str).unwrap_or("");format!("{}  {field}: {};\n",if docs.is_empty(){String::new()}else{format!("  /** {docs} */\n")},ty.source())}).collect::<String>()));
 };
 for (name,data) in types {
  let (seed_name,fields)=match data {
   Data::Struct(fields)=>{record(&mut source,name,fields);(name.clone(),fields)},
   Data::Enum(variants)=> {
    for variant in variants{record(&mut source,&format!("{name}{}",variant.name),&variant.fields);}
    source.push_str(&format!("/** {} */\nexport type {name} = {};\n",docs.get(name).map(String::as_str).unwrap_or(""),variants.iter().map(|variant|format!("{name}{}",variant.name)).collect::<Vec<_>>().join(" | ")));
    (format!("{name}{}",variants[0].name),&variants[0].fields)
   }
  };
  if seeds{source.push_str(&format!("function rustSeed{name}():{name}{{const seed:{seed_name}={{ {} }};return seed;}}\n",fields.iter().map(|(name,ty)|format!("{name}:{},",ty.seed())).collect::<String>()));}
 }
 source
}

// Adapter-local tags preserve None / Some(None) / Some(Some(value)); they do
// not reuse the language's flattened undefined unions.
pub(super) fn option_prefix(file:&syn::File)->String{
 let mut names=BTreeSet::new();for item in &file.items{match item{
  syn::Item::Struct(item)=>{names.insert(item.ident.to_string());},
  syn::Item::Enum(item)=>{names.insert(item.ident.to_string());for variant in &item.variants{names.insert(format!("{}{}",item.ident,variant.ident));}},
  syn::Item::Fn(item)=>{names.insert(item.sig.ident.to_string());},syn::Item::Type(item)=>{names.insert(item.ident.to_string());},_=>{}
 }}
 let mut prefix="RustAdapter".to_owned();while ["None","Some","SomeUnit","Option","UnitOption"].iter().any(|suffix|names.contains(&format!("{prefix}{suffix}"))){prefix.push('_');}prefix
}
pub(super) fn option_declarations(prefix:&str)->String{
 format!("/** Rust None. No payload is present. */\nexport type {prefix}None = {{kind:'none'}};\n/** Rust Some. The owned payload moves with this record. */\nexport interface {prefix}Some<T> {{kind:'some';value:T;}}\n/** Explicit Rust Option tags preserve nested absence. */\nexport type {prefix}Option<T> = {prefix}None | {prefix}Some<T>;\n/** Rust Some(()). Unit is represented by its tag, not a void field. */\nexport type {prefix}SomeUnit = {{kind:'some'}};\nexport type {prefix}UnitOption = {prefix}None | {prefix}SomeUnit;\n")
}
pub(super) fn input(ty:&Type,types:&BTreeMap<String,Data>)->Result<(),String>{
 let mut pending=vec![ty];let mut seen=BTreeSet::new();while let Some(ty)=pending.pop(){match ty{
  Type::Result(..)=>return Err("Rust Result is output-only; input containers and record fields cannot contain Result.".into()),
  Type::Array(inner)|Type::Option(inner,_)|Type::Borrow(inner,_)=>pending.push(inner),
  Type::Named(name)if seen.insert(name)=>if let Some(data)=types.get(name){match data{Data::Struct(fields)=>pending.extend(fields.iter().map(|(_,ty)|ty)),Data::Enum(variants)=>pending.extend(variants.iter().flat_map(|variant|variant.fields.iter().map(|(_,ty)|ty)))}}
  _=>{}
 }}Ok(())
}

pub(super) fn uses_option(ty:&Type,types:&BTreeMap<String,Data>)->bool {
 let mut pending=vec![ty];let mut seen=BTreeSet::new();while let Some(ty)=pending.pop(){match ty{
  Type::Option(..)=>return true,
  Type::Array(inner)|Type::Borrow(inner,_)=>pending.push(inner),
  Type::Result(ok,err)=>{pending.push(ok);pending.push(err);},
  Type::Named(name)if seen.insert(name)=>if let Some(data)=types.get(name){match data{Data::Struct(fields)=>pending.extend(fields.iter().map(|(_,ty)|ty)),Data::Enum(variants)=>pending.extend(variants.iter().flat_map(|variant|variant.fields.iter().map(|(_,ty)|ty))) }},
  _=>{}
 }}false
}
