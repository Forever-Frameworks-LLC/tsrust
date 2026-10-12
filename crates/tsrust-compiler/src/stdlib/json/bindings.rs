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

pub(crate) const GET:NativeBinding=NativeBinding {
 module:"std:json",name:"get",
 returned_borrow:Some(crate::native_return_borrows::ReturnedBorrow{source_parameter:0,access:crate::native_return_borrows::BorrowAccess::ReadOnly,payload:NativeTypeId::JsonView,slot:crate::native_return_borrows::ReturnSlot::ResultOkOptional}),
 parameters:&[
 NativeParameter{name:"value",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonValue)),access:Mode::Read,optional:false,rest:false},
 NativeParameter{name:"key",ty:NativeType::String,access:Mode::Read,optional:false,rest:false}],
 result:NativeType::Structured(&Signature::Result(&Signature::Optional(&Signature::Named(NativeTypeId::JsonView)),&Signature::Named(NativeTypeId::JsonError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_get"},features:&["json"],
};
pub(crate) const AT:NativeBinding=NativeBinding {
 name:"at",parameters:&[
 NativeParameter{name:"value",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonValue)),access:Mode::Read,optional:false,rest:false},
 NativeParameter{name:"index",ty:NativeType::Int,access:Mode::Copy,optional:false,rest:false}],
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_at"},..GET
};

pub(crate) const AS_STRING:NativeBinding=NativeBinding {
 name:"asString",
 returned_borrow:Some(crate::native_return_borrows::ReturnedBorrow{source_parameter:0,access:crate::native_return_borrows::BorrowAccess::ReadOnly,payload:NativeTypeId::StringView,slot:crate::native_return_borrows::ReturnSlot::ResultOk}),
 result:NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::StringView),&Signature::Named(NativeTypeId::JsonError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_asString"},..AS_BOOLEAN
};

// Construction transfers strings and copies scalar values; no parsing round trip.
pub(crate) const NULL_VALUE:NativeBinding=NativeBinding {
 module:"std:json",name:"nullValue",returned_borrow:None,parameters:&[],
 result:NativeType::Structured(&Signature::Named(NativeTypeId::JsonValue)),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_nullValue"},features:&["json"],
};
pub(crate) const FROM_BOOLEAN:NativeBinding=NativeBinding {
 name:"fromBoolean",parameters:&[NativeParameter{name:"value",ty:NativeType::Boolean,access:Mode::Copy,optional:false,rest:false}],

 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_fromBoolean"},..NULL_VALUE
};
pub(crate) const FROM_INT:NativeBinding=NativeBinding {
 name:"fromInt",parameters:&[NativeParameter{name:"value",ty:NativeType::Int,access:Mode::Copy,optional:false,rest:false}],

 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_fromInt"},..NULL_VALUE
};
pub(crate) const FROM_NUMBER:NativeBinding=NativeBinding {
 name:"fromNumber",parameters:&[NativeParameter{name:"value",ty:NativeType::Number,access:Mode::Copy,optional:false,rest:false}],
 result:NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::JsonValue),&Signature::Named(NativeTypeId::JsonError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_fromNumber"},..NULL_VALUE
};
pub(crate) const FROM_STRING:NativeBinding=NativeBinding {
 name:"fromString",parameters:&[NativeParameter{name:"value",ty:NativeType::String,access:Mode::Consume,optional:false,rest:false}],

 backend:NativeBackend::Bundled{symbol:"tsrust_std_json_fromString"},..NULL_VALUE
};

pub(crate) const FROM_ARRAY:NativeBinding=NativeBinding {
 name:"fromArray",parameters:&[
 NativeParameter{name:"values",ty:NativeType::Structured(&Signature::Array(&Signature::Named(NativeTypeId::JsonValue))),access:Mode::Consume,optional:false,rest:false},
 NativeParameter{name:"limits",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonBuildLimits)),access:Mode::Read,optional:true,rest:false},
 ],backend:NativeBackend::Bundled{symbol:"tsrust_std_json_fromArray"},..FROM_NUMBER
};
pub(crate) const FROM_ENTRIES:NativeBinding=NativeBinding {
 name:"fromEntries",parameters:&[
 NativeParameter{name:"entries",ty:NativeType::Structured(&Signature::Array(&Signature::Named(NativeTypeId::JsonEntry))),access:Mode::Consume,optional:false,rest:false},
 NativeParameter{name:"limits",ty:NativeType::Structured(&Signature::Named(NativeTypeId::JsonBuildLimits)),access:Mode::Read,optional:true,rest:false},
 ],backend:NativeBackend::Bundled{symbol:"tsrust_std_json_fromEntries"},..FROM_NUMBER
};
