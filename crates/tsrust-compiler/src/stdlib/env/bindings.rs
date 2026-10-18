use crate::{Mode,native_bindings::{NativeBinding,NativeBackend,NativeParameter,NativeType},native_signatures::{Signature,NativeTypeId}};
pub(crate) const GET:NativeBinding=NativeBinding {
 module:"std:env",name:"get",returned_borrow:None,
 parameters:&[NativeParameter{name:"name",ty:NativeType::String,access:Mode::Read,optional:false,rest:false}],
 result:NativeType::Structured(&Signature::Result(&Signature::Optional(&Signature::String),&Signature::Named(NativeTypeId::EnvError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_env_get"},features:&["env"],
};

pub(crate) const ARGS:NativeBinding=NativeBinding {
 name:"args",parameters:&[],
 result:NativeType::Structured(&Signature::Result(&Signature::Array(&Signature::String),&Signature::Named(NativeTypeId::EnvError))),
 backend:NativeBackend::Bundled{symbol:"tsrust_std_env_args"},..GET
};
