//! Explicit locale owners share native bindings and ordinary Result transport.
pub(crate) const RUNTIME:&str=include_str!("runtime.rs");
#[cfg(test)]#[allow(dead_code)]mod tests{include!("runtime.rs");include!("runtime_tests.rs");}
use crate::{Mode,native_bindings::{NativeBinding,NativeBackend,NativeParameter,NativeType},native_signatures::{Signature,NativeTypeId}};
const fn parameter(name:&'static str,ty:&'static Signature,optional:bool)->NativeParameter{NativeParameter{name,ty:NativeType::Structured(ty),access:Mode::Read,optional,rest:false}}
const fn binding(name:&'static str,parameters:&'static[NativeParameter],result:&'static Signature)->NativeBinding{NativeBinding{module:"std:locale",name,parameters,result:NativeType::Structured(result),returned_borrow:None,backend:NativeBackend::Bundled{symbol:"locale_intrinsic"},features:&["locale"]}}
const LOCALE:Signature=Signature::Named(NativeTypeId::Locale);
const COLLATOR:Signature=Signature::Named(NativeTypeId::Collator);
const SEGMENTER:Signature=Signature::Named(NativeTypeId::Segmenter);
const NUMBER:Signature=Signature::Named(NativeTypeId::NumberFormat);
const ERROR:Signature=Signature::Named(NativeTypeId::LocaleError);
pub(crate) const PARSE:NativeBinding=binding("parseLocale",&[parameter("text",&Signature::String,false)],&Signature::Result(&LOCALE,&ERROR));
pub(crate) const FORMAT_LOCALE:NativeBinding=binding("formatLocale",&[parameter("locale",&LOCALE,false)],&Signature::String);
pub(crate) const CREATE_COLLATOR:NativeBinding=binding("createCollator",&[parameter("locale",&LOCALE,false),parameter("options",&Signature::Optional(&Signature::Named(NativeTypeId::CollatorOptions)),true)],&Signature::Result(&COLLATOR,&ERROR));
pub(crate) const COMPARE:NativeBinding=binding("__compare",&[parameter("value",&COLLATOR,false),parameter("left",&Signature::String,false),parameter("right",&Signature::String,false)],&Signature::Int);
pub(crate) const STRING_COMPARE:NativeBinding=binding("__stringCompare",&[parameter("left",&Signature::String,false),parameter("right",&Signature::String,false),parameter("collator",&COLLATOR,false)],&Signature::Int);
pub(crate) const CREATE_SEGMENTER:NativeBinding=binding("createSegmenter",&[],&SEGMENTER);
pub(crate) const SEGMENT:NativeBinding=binding("__segment",&[parameter("value",&SEGMENTER,false),parameter("text",&Signature::String,false)],&Signature::Array(&Signature::Named(NativeTypeId::SegmentRange)));
pub(crate) const CREATE_NUMBER:NativeBinding=binding("createNumberFormat",&[parameter("locale",&LOCALE,false),parameter("options",&Signature::Optional(&Signature::Named(NativeTypeId::NumberFormatOptions)),true)],&Signature::Result(&NUMBER,&ERROR));
pub(crate) const FORMAT_NUMBER:NativeBinding=binding("__formatNumber",&[parameter("formatter",&NUMBER,false),parameter("value",&Signature::Union(&[Signature::Int,Signature::Named(NativeTypeId::Decimal)]),false)],&Signature::String);
pub(crate) const OPERATIONS:&[NativeBinding]=&[PARSE,FORMAT_LOCALE,CREATE_COLLATOR,COMPARE,STRING_COMPARE,CREATE_SEGMENTER,SEGMENT,CREATE_NUMBER,FORMAT_NUMBER];
pub(crate) fn method(identity:Option<NativeTypeId>,name:&str)->Option<&'static NativeBinding>{Some(match(identity,name){(Some(NativeTypeId::Collator),"compare")=>&COMPARE,(Some(NativeTypeId::Segmenter),"segment")=>&SEGMENT,(Some(NativeTypeId::NumberFormat),"format")=>&FORMAT_NUMBER,_=>return None})}
pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic>{if program.records.iter().any(|r|matches!(r.native_type,Some(crate::native_signatures::NativeTypeId::Locale|crate::native_signatures::NativeTypeId::Collator|crate::native_signatures::NativeTypeId::Segmenter|crate::native_signatures::NativeTypeId::NumberFormat))){for binding in OPERATIONS{crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;}}Ok(())}
