//! Reusable regex owners with independently owned UTF-8 range snapshots.
pub(crate) const RUNTIME:&str=include_str!("runtime.rs");
#[cfg(test)] mod tests {include!("runtime.rs");include!("runtime_tests.rs");}
use crate::{Mode,native_bindings::{NativeBinding,NativeBackend,NativeParameter,NativeType},native_signatures::{Signature,NativeTypeId}};
const fn parameter(name:&'static str,ty:&'static Signature,optional:bool)->NativeParameter {NativeParameter{name,ty:NativeType::Structured(ty),access:Mode::Read,optional,rest:false}}
const fn binding(name:&'static str,parameters:&'static [NativeParameter],result:&'static Signature)->NativeBinding {NativeBinding{module:"std:regex",name,parameters,result:NativeType::Structured(result),returned_borrow:None,backend:NativeBackend::Bundled{symbol:"regex_intrinsic"},features:&["regex"]}}
pub(crate) const COMPILE:NativeBinding=binding("compile",&[parameter("pattern",&Signature::String,false),parameter("options",&Signature::Optional(&Signature::Named(NativeTypeId::RegexOptions)),true)],&Signature::Result(&Signature::Named(NativeTypeId::Regex),&Signature::Named(NativeTypeId::RegexError)));
pub(crate) const ESCAPE:NativeBinding=binding("escape",&[parameter("text",&Signature::String,false)],&Signature::String);
pub(crate) const IS_MATCH:NativeBinding=binding("__isMatch",&[parameter("value",&Signature::Named(NativeTypeId::Regex),false),parameter("text",&Signature::String,false)],&Signature::Boolean);
pub(crate) const FIND:NativeBinding=binding("__find",&[parameter("value",&Signature::Named(NativeTypeId::Regex),false),parameter("text",&Signature::String,false)],&Signature::Optional(&Signature::Named(NativeTypeId::RegexMatch)));
pub(crate) const FIND_ALL:NativeBinding=binding("__findAll",&[parameter("value",&Signature::Named(NativeTypeId::Regex),false),parameter("text",&Signature::String,false)],&Signature::Array(&Signature::Named(NativeTypeId::RegexMatch)));
pub(crate) const CAPTURES:NativeBinding=binding("__captures",&[parameter("value",&Signature::Named(NativeTypeId::Regex),false),parameter("text",&Signature::String,false)],&Signature::Optional(&Signature::Named(NativeTypeId::RegexCaptures)));
pub(crate) const CAPTURES_ALL:NativeBinding=binding("__capturesAll",&[parameter("value",&Signature::Named(NativeTypeId::Regex),false),parameter("text",&Signature::String,false)],&Signature::Array(&Signature::Named(NativeTypeId::RegexCaptures)));
pub(crate) const REPLACE:NativeBinding=binding("__replace",&[parameter("value",&Signature::Named(NativeTypeId::Regex),false),parameter("text",&Signature::String,false),parameter("replacementTemplate",&Signature::String,false)],&Signature::String);
pub(crate) const REPLACE_ALL:NativeBinding=binding("__replaceAll",&[parameter("value",&Signature::Named(NativeTypeId::Regex),false),parameter("text",&Signature::String,false),parameter("replacementTemplate",&Signature::String,false)],&Signature::String);
pub(crate) const SPLIT:NativeBinding=binding("__split",&[parameter("value",&Signature::Named(NativeTypeId::Regex),false),parameter("text",&Signature::String,false)],&Signature::Array(&Signature::Named(NativeTypeId::RegexMatch)));
pub(crate) const STRING_MATCH:NativeBinding=binding("__stringMatch",&[parameter("text",&Signature::String,false),parameter("regex",&Signature::Named(NativeTypeId::Regex),false)],&Signature::Optional(&Signature::Named(NativeTypeId::RegexCaptures)));
pub(crate) const STRING_MATCH_ALL:NativeBinding=binding("__stringMatchAll",&[parameter("text",&Signature::String,false),parameter("regex",&Signature::Named(NativeTypeId::Regex),false)],&Signature::Array(&Signature::Named(NativeTypeId::RegexCaptures)));
pub(crate) const STRING_SEARCH:NativeBinding=binding("__stringSearch",&[parameter("text",&Signature::String,false),parameter("regex",&Signature::Named(NativeTypeId::Regex),false)],&Signature::Int);
pub(crate) const OPERATIONS:&[NativeBinding]=&[COMPILE,ESCAPE,IS_MATCH,FIND,FIND_ALL,CAPTURES,CAPTURES_ALL,REPLACE,REPLACE_ALL,SPLIT,STRING_MATCH,STRING_MATCH_ALL,STRING_SEARCH];
pub(crate) fn method(identity:Option<NativeTypeId>,name:&str)->Option<&'static NativeBinding>{if identity!=Some(NativeTypeId::Regex){return None;}Some(match name{"isMatch"=>&IS_MATCH,"find"=>&FIND,"findAll"=>&FIND_ALL,"captures"=>&CAPTURES,"capturesAll"=>&CAPTURES_ALL,"replace"=>&REPLACE,"replaceAll"=>&REPLACE_ALL,"split"=>&SPLIT,_=>return None})}
pub(crate) fn string_method(name:&str)->Option<&'static NativeBinding>{Some(match name{"match"=>&STRING_MATCH,"matchAll"=>&STRING_MATCH_ALL,"search"=>&STRING_SEARCH,_=>return None})}
pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic>{
 if program.records.iter().any(|record|record.native_type==Some(NativeTypeId::Regex)) {for binding in OPERATIONS {crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;}} Ok(())
}
