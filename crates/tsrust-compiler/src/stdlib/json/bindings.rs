use crate::{Mode,native_bindings::{NativeBinding,NativeBackend,NativeParameter,NativeType},native_signatures::{Signature,NativeTypeId}};
pub(crate) const PARSE:NativeBinding=NativeBinding {
 module:"std:json",name:"parse",returned_borrow:None,
 parameters:&[
  NativeParameter{name:"text",ty:NativeType::String,access:Mode::Read,optional:false,rest:false},
  NativeParameter{name:"limits",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonLimits)),access:Mode::Read,optional:true,rest:false},
 ],
 result:NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::JsonValue),&Signature::Named(NativeTypeId::JsonError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_parse"},features:&["json"],
};
pub(crate) const STRINGIFY:NativeBinding=NativeBinding {
 module:"std:json",name:"stringify",returned_borrow:None,
 parameters:&[
  NativeParameter{name:"value",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonValue)),access:Mode::Read,optional:false,rest:false},
  NativeParameter{name:"maxBytes",ty:NativeType::Int,access:Mode::Copy,optional:true,rest:false},
 ],
 result:NativeType::Structured(&Signature::Result(&Signature::String,&Signature::Named(NativeTypeId::JsonError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_stringify"},features:&["json"],
};
pub(crate) const KIND:NativeBinding=NativeBinding {
 module:"std:json",name:"kind",returned_borrow:None,
 parameters:&[NativeParameter{name:"value",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonValue)),access:Mode::Read,optional:false,rest:false}],
 result:NativeType::String,backend:NativeBackend::Bundled{symbol:"tsrust_std_json_kind"},features:&["json"],
};

pub(crate) const PARSE_JSONC:NativeBinding=NativeBinding {
 name:"parseJsonc",
 parameters:&[
  NativeParameter{name:"text",ty:NativeType::String,access:Mode::Read,optional:false,rest:false},
  NativeParameter{name:"allowTrailingComma",ty:NativeType::Boolean,access:Mode::Copy,optional:true,rest:false},
  NativeParameter{name:"limits",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonLimits)),access:Mode::Read,optional:true,rest:false},
 ],
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_parse_jsonc"},..PARSE
};
