pub(crate) mod emit;
pub(crate) const RUNTIME:&str=include_str!("runtime.rs");
#[cfg(test)] mod tests {include!("runtime.rs");include!("runtime_tests.rs");}
use crate::{Mode,native_bindings::{NativeBinding,NativeBackend,NativeParameter,NativeType},native_signatures::{Signature,NativeTypeId}};
const DECIMAL:Signature=Signature::Named(NativeTypeId::Decimal);
const ERROR:Signature=Signature::Named(NativeTypeId::DecimalError);
const fn arg(name:&'static str,ty:NativeType)->NativeParameter{NativeParameter{name,ty,access:Mode::Read,optional:false,rest:false}}
const fn binding(name:&'static str,symbol:&'static str,parameters:&'static [NativeParameter],result:NativeType)->NativeBinding{NativeBinding{module:"std:decimal",name,parameters,result,returned_borrow:None,backend:NativeBackend::Bundled{symbol},features:&["decimal"]}}
const VALUE:NativeParameter=arg("value",NativeType::Structured(&DECIMAL));
const BINARY:&[NativeParameter]=&[arg("left",NativeType::Structured(&DECIMAL)),arg("right",NativeType::Structured(&DECIMAL))];
pub(crate) const PARSE_DECIMAL:NativeBinding=binding("parseDecimal","tsrust_decimal_parse_decimal",&[arg("text",NativeType::String)],NativeType::Structured(&Signature::Result(&DECIMAL,&ERROR)));
pub(crate) const FROM_INT:NativeBinding=binding("fromInt","tsrust_decimal_from_int",&[arg("value",NativeType::Int)],NativeType::Structured(&DECIMAL));
pub(crate) const FORMAT:NativeBinding=binding("format","tsrust_decimal_format",&[VALUE],NativeType::String);
pub(crate) const FORMAT_FIXED:NativeBinding=binding("formatFixed","tsrust_decimal_format_fixed",&[VALUE,arg("scale",NativeType::Int)],NativeType::Structured(&Signature::Result(&Signature::String,&ERROR)));
pub(crate) const ADD:NativeBinding=binding("add","tsrust_decimal_add",BINARY,NativeType::Structured(&Signature::Result(&DECIMAL,&ERROR)));
pub(crate) const SUBTRACT:NativeBinding=binding("subtract","tsrust_decimal_subtract",BINARY,NativeType::Structured(&Signature::Result(&DECIMAL,&ERROR)));
pub(crate) const MULTIPLY:NativeBinding=binding("multiply","tsrust_decimal_multiply",BINARY,NativeType::Structured(&Signature::Result(&DECIMAL,&ERROR)));
pub(crate) const DIVIDE_EXACT:NativeBinding=binding("divideExact","tsrust_decimal_divide_exact",BINARY,NativeType::Structured(&Signature::Result(&DECIMAL,&ERROR)));
pub(crate) const COMPARE:NativeBinding=binding("compare","tsrust_decimal_compare",BINARY,NativeType::Int);
pub(crate) const TO_INT:NativeBinding=binding("toInt","tsrust_decimal_to_int",&[VALUE],NativeType::Structured(&Signature::Result(&Signature::Int,&ERROR)));
pub(crate) const TO_NUMBER:NativeBinding=binding("toNumber","tsrust_decimal_to_number",&[VALUE],NativeType::Structured(&Signature::Result(&Signature::Number,&ERROR)));
