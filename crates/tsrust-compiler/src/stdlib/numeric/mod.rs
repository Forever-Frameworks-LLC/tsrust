pub(crate) mod emit;
#[cfg(test)]
mod runtime;
use crate::{Mode,native_bindings::{NativeBinding,NativeBackend,NativeParameter,NativeType},native_signatures::{Signature,NativeTypeId}};
const VALUE: Signature = Signature::Union(&[Signature::Int,Signature::Number]);
const ERROR: Signature = Signature::Named(NativeTypeId::NumericError);
const fn binding(name: &'static str,symbol: &'static str,parameters: &'static [NativeParameter],result: &'static Signature)->NativeBinding {
 NativeBinding{module:"std:numeric",name,returned_borrow:None,parameters,result:NativeType::Structured(result),backend:NativeBackend::Bundled{symbol},features:&[]}
}
const fn parameter(name:&'static str,ty:NativeType)->NativeParameter {NativeParameter{name,ty,access:Mode::Consume,optional:false,rest:false}}
pub(crate) const TO_INT:NativeBinding=binding("toInt","tsrust_numeric_to_int",&[parameter("value",NativeType::Structured(&VALUE))],&Signature::Result(&Signature::Int,&ERROR));
pub(crate) const TO_NUMBER:NativeBinding=binding("toNumber","tsrust_numeric_to_number",&[parameter("value",NativeType::Int)],&Signature::Result(&Signature::Number,&ERROR));
pub(crate) const TO_FLOAT32:NativeBinding=binding("toFloat32","tsrust_numeric_to_float32",&[parameter("value",NativeType::Structured(&VALUE))],&Signature::Result(&Signature::Number,&ERROR));
pub(crate) const CHECK_SIGNED:NativeBinding=binding("checkSigned","tsrust_numeric_check_signed",&[parameter("value",NativeType::Int),parameter("bits",NativeType::Int)],&Signature::Result(&Signature::Int,&ERROR));
pub(crate) const CHECK_UNSIGNED:NativeBinding=binding("checkUnsigned","tsrust_numeric_check_unsigned",&[parameter("value",NativeType::Int),parameter("bits",NativeType::Int)],&Signature::Result(&Signature::Int,&ERROR));
pub(crate) const PARSE_INT:NativeBinding=binding("parseInt","tsrust_numeric_parse_int",&[NativeParameter{name:"text",ty:NativeType::String,access:Mode::Read,optional:false,rest:false}],&Signature::Result(&Signature::Int,&ERROR));
