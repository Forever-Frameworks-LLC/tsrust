//! Restricted local Rust exports become ordinary checked source functions.
use super::*;
use syn::{spanned::Spanned, visit::Visit};

#[derive(Clone, Debug)]
pub enum Type { Int, Number, Boolean, Void, String, Array(Box<Type>), Borrow(Box<Type>, bool), Context, Result(Box<Type>, Box<Type>) }
impl Type {
    fn source(&self) -> String {
        match self {
            Self::Int=>"int".into(), Self::Number=>"number".into(), Self::Boolean=>"boolean".into(), Self::Void=>"void".into(), Self::String=>"string".into(), Self::Context=>"TaskContext".into(),
            Self::Array(value)=>format!("{}[]",value.source()),
            Self::Borrow(value,mutable)=>format!("{}<{}>",if *mutable {"Mutable"}else{"ReadOnly"},value.source()),
            Self::Result(ok,err)=>format!("Result<{},{}>",ok.source(),err.source()),
        }
    }
    fn seed(&self)->String {match self {Self::Int|Self::Number=>"0".into(),Self::Boolean=>"false".into(),Self::String=>"''".into(),Self::Array(_)=>"[]".into(),Self::Void=>"undefined".into(),Self::Result(ok,err)=>format!("Ok<{},{}>({})",ok.source(),err.source(),if matches!(ok.as_ref(),Self::Void){String::new()}else{ok.seed()}),_=>unreachable!()}}
    fn owned(&self)->bool {matches!(self,Self::String|Self::Array(_))}
}
#[derive(Clone, Debug)]
pub struct Export {pub name:String,pub parameters:Vec<Type>,pub result:Type,pub declaration:Span,pub native:Span}
#[derive(Clone, Debug)]
pub struct Module {pub exports:Vec<Export>,pub context:bool}
fn range(text:&str,span:proc_macro2::Span)->Span {
    let offset=|position:proc_macro2::LineColumn|text.split_inclusive('\n').take(position.line.saturating_sub(1)).map(str::len).sum::<usize>()+position.column;
    Span::new(offset(span.start()).min(text.len()),offset(span.end()).min(text.len()))
}
fn shape(ty:&syn::Type, output:bool)->Result<Type,String> {
    match ty {
        syn::Type::Tuple(value) if value.elems.is_empty()=>Ok(Type::Void),
        syn::Type::Reference(value) if !output && value.lifetime.is_none()=>{
            let inner=match value.elem.as_ref(){syn::Type::Slice(slice)=>Type::Array(Box::new(shape(&slice.elem,true)?)),syn::Type::Path(path) if path.path.is_ident("str")=>Type::String,other=>shape(other,true)?};
            if !matches!(inner,Type::String|Type::Array(_)|Type::Context) || (value.mutability.is_some() && !matches!(inner,Type::Array(_))) {return Err("Borrowed Rust inputs support &str, primitive slices, mutable primitive slices and &TaskContext.".into());}
            Ok(Type::Borrow(Box::new(inner),value.mutability.is_some()))
        },
        syn::Type::Path(path) if path.qself.is_none() && path.path.segments.len()==1=>{
            let segment=&path.path.segments[0];
            if matches!(segment.ident.to_string().as_str(),"i64"|"f64"|"bool"|"String") && !matches!(segment.arguments,syn::PathArguments::None){return Err("Scalar and string boundary types do not accept generic arguments.".into());}
            match segment.ident.to_string().as_str(){
                "i64"=>Ok(Type::Int),"f64"=>Ok(Type::Number),"bool"=>Ok(Type::Boolean),"String"=>Ok(Type::String),"TaskContext" if !output=>Err("TaskContext must be borrowed as &TaskContext.".into()),
                "Vec"|"Result"=>{
                    let syn::PathArguments::AngleBracketed(args)=&segment.arguments else{return Err("Expected explicit Rust type arguments.".into())};
                    let args=args.args.iter().map(|arg|if let syn::GenericArgument::Type(ty)=arg{shape(ty,true)}else{Err("Only concrete type arguments are supported.".into())}).collect::<Result<Vec<_>,_>>()?;
                    if segment.ident=="Vec" && args.len()==1 && matches!(args[0],Type::Int|Type::Number|Type::Boolean){return Ok(Type::Array(Box::new(args[0].clone())));}
                    if segment.ident=="Result" && output && args.len()==2 && !args.iter().any(|arg|matches!(arg,Type::Result(..)|Type::Borrow(..)|Type::Context)) && !matches!(args[1],Type::Void){return Ok(Type::Result(Box::new(args[0].clone()),Box::new(args[1].clone())));}
                    Err("Supported buffers are Vec<i64|f64|bool>; output Result needs two owned payload types.".into())
                },
                _=>Err("Unsupported Rust boundary type. Use i64, f64, bool, String, primitive Vec/slices or an owned Result output.".into())
            }
        },
        _=>Err("Rust boundary types cannot contain returned borrows, pointers, generics or traits.".into()),
    }
}
// All Rust bodies are compiled with forbid(unsafe_code). External source inclusion
// is rejected so the project graph owns every byte used by build/dev/export.
struct Policy {failure:Option<(proc_macro2::Span,String)>}
impl Policy {
    fn tokens(&mut self,tokens:proc_macro2::TokenStream){
        for token in tokens {match token {
            proc_macro2::TokenTree::Group(group)=>self.tokens(group.stream()),
            proc_macro2::TokenTree::Ident(name) if matches!(name.to_string().as_str(),"include"|"include_str"|"include_bytes"|"env"|"option_env")=>self.failure=Some((name.span(),"Compile-time external inputs are unsupported in local Rust bridges.".into())),
            _=>{}
        }}
    }
}
impl<'ast> Visit<'ast> for Policy {
    fn visit_ident(&mut self,name:&'ast syn::Ident){
        if matches!(name.to_string().as_str(),"include"|"include_str"|"include_bytes"|"env"|"option_env"){self.failure=Some((name.span(),"Compile-time external input aliases are unsupported in local Rust bridges.".into()));}
    }
    fn visit_item_mod(&mut self,item:&'ast syn::ItemMod){self.failure=Some((item.span(),"Local Rust bridges must be self-contained; nested modules are unsupported.".into()));}
    fn visit_macro(&mut self,item:&'ast syn::Macro){
        self.tokens(item.tokens.clone());
        if !item.path.get_ident().is_some_and(|name|matches!(name.to_string().as_str(),"vec"|"format"|"panic"|"assert"|"assert_eq"|"assert_ne"|"debug_assert"|"debug_assert_eq"|"debug_assert_ne"|"matches"|"unreachable"|"todo")) {self.failure=Some((item.span(),"Only standard value/assertion macros are supported in local Rust bridges.".into()));}
    }
    fn visit_item_extern_crate(&mut self,item:&'ast syn::ItemExternCrate){self.failure=Some((item.span(),"External crate declarations are unsupported; std and pinned Rayon are available.".into()));}
}
pub(super) fn module(source:SourceFile)->Result<ModuleSource,ProjectDiagnostic>{
    let fail=|span,message|error(&source.id,"rust-bridge",message,range(&source.text,span));
    let parsed=syn::parse_file(&source.text).map_err(|value|fail(value.span(),value.to_string()))?;
    if !parsed.attrs.is_empty(){return Err(fail(parsed.attrs[0].span(),"Rust module attributes are unsupported.".into()));}
    let mut policy=Policy{failure:None};policy.visit_file(&parsed);if let Some((span,message))=policy.failure{return Err(fail(span,message));}
    let mut declarations=String::new();let mut exports=Vec::new();let mut context=false;
    for item in &parsed.items {
        let syn::Item::Fn(function)=item else{continue};
        if !matches!(function.vis,syn::Visibility::Public(_)){continue;}
        let sig=&function.sig;
        if sig.asyncness.is_some()||matches!(sig.safety,syn::Safety::Unsafe(_))||sig.abi.is_some()||sig.variadic.is_some()||!sig.generics.params.is_empty()||sig.generics.where_clause.is_some()||function.attrs.iter().any(|attr|!attr.path().is_ident("doc")) {return Err(fail(sig.span(),"Exports must be safe, synchronous, concrete Rust functions without conditional attributes.".into()));}
        let name=sig.ident.to_string();if name.starts_with("r#"){return Err(fail(sig.ident.span(),"Raw Rust identifiers are unsupported at the boundary.".into()));}
        let mut parameters=Vec::new();let mut labels=Vec::new();let mut consume=String::new();
        for (index,input) in sig.inputs.iter().enumerate(){
            let syn::FnArg::Typed(input)=input else{return Err(fail(input.span(),"Methods are unsupported; export a free function.".into()));};
            let syn::Pat::Ident(pattern)=input.pat.as_ref() else{return Err(fail(input.span(),"Export parameters require simple names.".into()));};
            let ty=if let syn::Type::Reference(reference)=input.ty.as_ref() {if reference.mutability.is_none() && reference.lifetime.is_none() && matches!(reference.elem.as_ref(),syn::Type::Path(path) if path.path.is_ident("TaskContext")){context=true;Type::Borrow(Box::new(Type::Context),false)}else{shape(&input.ty,false).map_err(|message|fail(input.ty.span(),message))?}}else{shape(&input.ty,false).map_err(|message|fail(input.ty.span(),message))?};
            if matches!(ty,Type::Void){return Err(fail(input.span(),"Unit is supported only as a return type.".into()));}
            let label=pattern.ident.to_string();if label.starts_with("r#"){return Err(fail(pattern.ident.span(),"Raw Rust parameter identifiers are unsupported at the boundary.".into()));}labels.push(format!("{label}:{}",if ty.owned(){format!("Owned<{}>",ty.source())}else{ty.source()}));
            if ty.owned(){
                let mut local=format!("rustInput{index}");
                while sig.inputs.iter().any(|input|matches!(input,syn::FnArg::Typed(input) if matches!(input.pat.as_ref(),syn::Pat::Ident(pattern) if pattern.ident==local))){local.push('_');}
                consume.push_str(&format!("const {local}={label};"));
            }
            parameters.push(ty);
        }
        let result=match &sig.output{syn::ReturnType::Default=>Type::Void,syn::ReturnType::Type(_,ty)=>shape(ty,true).map_err(|message|fail(ty.span(),message))?};
        let docs=function.attrs.iter().filter_map(|attr| {
            if let syn::Meta::NameValue(value)=&attr.meta && value.path.is_ident("doc")
                && let syn::Expr::Lit(value)=&value.value && let syn::Lit::Str(value)=&value.lit {Some(value.value().replace("*/","* /"))} else {None}
        }).collect::<Vec<_>>().join("\n");
        let start=declarations.len();
        declarations.push_str(&format!("/** {docs}\nLocal Rust export. Direct calls run on the caller. Owned inputs move; borrowed inputs remain borrowed. Rust panics are reported by worker joins. */\nexport function {name}({}):{}{{{consume}{} }}\n",labels.join(","),result.source(),if matches!(result,Type::Void){String::new()}else{format!("return {};",result.seed())}));
        let name_start=start+declarations[start..].rfind(&format!("function {name}")).unwrap()+9;
        exports.push(Export{name,parameters,result,declaration:Span::new(name_start,name_start+sig.ident.to_string().len()),native:range(&source.text,sig.ident.span())});
    }
    if context{declarations.push_str("import {TaskContext} from 'std:threads';\n");}
    let mut module=super::parse_module(SourceFile{id:SourceId(format!("{}.tsr",source.id.0)),text:declarations},true)?;
    module.source=source;module.rust=Some(Module{exports,context});Ok(module)
}
