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

pub(crate) const OPEN_JSONL:NativeBinding=NativeBinding {
 module:"std:json",name:"openJsonl",returned_borrow:None,
 parameters:&[
  NativeParameter{name:"path",ty:NativeType::String,access:Mode::Read,optional:false,rest:false},
  NativeParameter{name:"limits",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonLimits)),access:Mode::Read,optional:true,rest:false},
  NativeParameter{name:"capacity",ty:NativeType::Int,access:Mode::Copy,optional:true,rest:false},
 ],
 result:NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::JsonlReader),&Signature::Named(NativeTypeId::JsonlError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_open_jsonl"},features:&["json","fs"],
};
pub(crate) const READ_JSONL:NativeBinding=NativeBinding {
 name:"readJsonl",parameters:&[NativeParameter{name:"reader",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonlReader)),access:Mode::Mutate,optional:false,rest:false}],
 result:NativeType::Structured(&Signature::Result(&Signature::Optional(&Signature::Named(NativeTypeId::JsonValue)),&Signature::Named(NativeTypeId::JsonlError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_read_jsonl"},..OPEN_JSONL
};
pub(crate) const CLOSE_JSONL:NativeBinding=NativeBinding {
 name:"closeJsonl",parameters:&[NativeParameter{name:"reader",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonlReader)),access:Mode::Consume,optional:false,rest:false}],
 result:NativeType::Structured(&Signature::Result(&Signature::Void,&Signature::Named(NativeTypeId::JsonlError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_close_jsonl"},..OPEN_JSONL
};

pub(crate) const AS_BOOLEAN:NativeBinding=NativeBinding {
 module:"std:json",name:"asBoolean",returned_borrow:None,
 parameters:&[NativeParameter{name:"value",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonValue)),access:Mode::Read,optional:false,rest:false}],
 result:NativeType::Structured(&Signature::Result(&Signature::Boolean,&Signature::Named(NativeTypeId::JsonError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_asBoolean"},features:&["json"],
};

pub(crate) const AS_INT:NativeBinding=NativeBinding {
 module:"std:json",name:"asInt",returned_borrow:None,
 parameters:&[NativeParameter{name:"value",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonValue)),access:Mode::Read,optional:false,rest:false}],
 result:NativeType::Structured(&Signature::Result(&Signature::Int,&Signature::Named(NativeTypeId::JsonError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_asInt"},features:&["json"],
};

pub(crate) const AS_NUMBER:NativeBinding=NativeBinding {
 module:"std:json",name:"asNumber",returned_borrow:None,
 parameters:&[NativeParameter{name:"value",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonValue)),access:Mode::Read,optional:false,rest:false}],
 result:NativeType::Structured(&Signature::Result(&Signature::Number,&Signature::Named(NativeTypeId::JsonError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_asNumber"},features:&["json"],
};

pub(crate) const TAKE_STRING:NativeBinding=NativeBinding {
 module:"std:json",name:"takeString",returned_borrow:None,
 parameters:&[NativeParameter{name:"value",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonValue)),access:Mode::Consume,optional:false,rest:false}],
 result:NativeType::Structured(&Signature::Result(&Signature::String,&Signature::Named(NativeTypeId::JsonError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_takeString"},features:&["json"],
};
