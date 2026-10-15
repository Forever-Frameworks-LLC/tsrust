//! Plain owned Rust data is projected field by field, without serialization.
use super::*;
#[derive(Clone,Debug)]
pub enum Data {Struct(Vec<(String,Type)>),Enum(Vec<Variant>)}
#[derive(Clone,Debug)]
pub struct Variant {pub name:String,pub fields:Vec<(String,Type)>,pub tuple:bool,pub named:bool}
fn fields(fields:&syn::Fields,public:bool)->Result<Vec<(String,Type)>,(proc_macro2::Span,String)> {
 fields.iter().enumerate().map(|(index,field)| {
  if public && !matches!(field.vis,syn::Visibility::Public(_)){return Err((field.span(),"Boundary struct fields must be public.".into()));}
  if field.attrs.iter().any(|attr|!attr.path().is_ident("doc")){return Err((field.span(),"Boundary fields accept only documentation attributes.".into()));}
  let name=field.ident.as_ref().map(ToString::to_string).unwrap_or_else(||if fields.len()==1{"value".into()}else{format!("item{index}")});
  if name.starts_with("r#"){return Err((field.span(),"Raw field identifiers are unsupported at the boundary.".into()));}
  let ty=shape(&field.ty,true).map_err(|message|(field.span(),message))?;
  if matches!(ty,Type::Void|Type::Result(..)){return Err((field.span(),"Boundary fields need concrete owned data types.".into()));}
  Ok((name,ty))
 }).collect()
}
pub(super) fn collect(file:&syn::File)->Result<BTreeMap<String,Data>,(proc_macro2::Span,String)> {
 let mut types=BTreeMap::new();
 for item in &file.items {
  let (name,generics,attrs,data)=match item {
   syn::Item::Struct(item) if matches!(item.vis,syn::Visibility::Public(_))=> {
    if !matches!(item.fields,syn::Fields::Named(_)){return Err((item.span(),"Boundary structs require named fields.".into()));}
    (&item.ident,&item.generics,&item.attrs,Data::Struct(fields(&item.fields,true)?))
   },
   syn::Item::Enum(item) if matches!(item.vis,syn::Visibility::Public(_))=> {
    if item.variants.is_empty(){return Err((item.span(),"Boundary enums require at least one variant.".into()));}
    let variants=item.variants.iter().map(|variant| {
     if variant.discriminant.is_some() || variant.attrs.iter().any(|attr|!attr.path().is_ident("doc")){return Err((variant.span(),"Boundary enum variants accept only documentation attributes and no explicit discriminants.".into()));}
     Ok(Variant{name:variant.ident.to_string(),fields:fields(&variant.fields,false)?,tuple:matches!(variant.fields,syn::Fields::Unnamed(_)),named:matches!(variant.fields,syn::Fields::Named(_))})
    }).collect::<Result<Vec<_>,_>>()?;
    (&item.ident,&item.generics,&item.attrs,Data::Enum(variants))
   },_=>continue
  };
  if !generics.params.is_empty() || generics.where_clause.is_some() || attrs.iter().any(|attr|!attr.path().is_ident("doc") && !attr.path().is_ident("derive")) || name.to_string().starts_with("r#") {return Err((name.span(),"Boundary data must be concrete and have only documentation or derive attributes.".into()));}
  types.insert(name.to_string(),data);
 }
 for item in &file.items {
  if let syn::Item::Impl(item)=item && item.trait_.as_ref().is_some_and(|(path,_)|path.is_ident("Drop")) && let syn::Type::Path(path)=item.self_ty.as_ref() && path.path.get_ident().is_some_and(|name|types.contains_key(&name.to_string())) {
   return Err((item.span(),"Boundary data cannot implement Drop; keep resource owners private to the Rust adapter.".into()));
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
  },Type::Array(inner)|Type::Borrow(inner,_)=>validate(inner,types,active)?,Type::Result(ok,err)=>{validate(ok,types,active)?;validate(err,types,active)?;},_=>{}
 }Ok(())
}
pub(super) fn declarations(types:&BTreeMap<String,Data>,file:&syn::File)->String {
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
  source.push_str(&format!("function rustSeed{name}():{name}{{const seed:{seed_name}={{ {} }};return seed;}}\n",fields.iter().map(|(name,ty)|format!("{name}:{},",ty.seed())).collect::<String>()));
 }
 source
}
