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
pub(crate) const VARS:NativeBinding=NativeBinding{name:"vars",parameters:&[],result:NativeType::Structured(&Signature::Result(&Signature::Array(&Signature::Named(NativeTypeId::EnvVar)),&Signature::Named(NativeTypeId::EnvError))),backend:NativeBackend::Bundled{symbol:"tsrust_std_env_vars"},..GET};
pub(crate) const CURRENT_DIR:NativeBinding=NativeBinding{name:"currentDir",parameters:&[],result:NativeType::Structured(&Signature::Result(&Signature::String,&Signature::Named(NativeTypeId::EnvError))),backend:NativeBackend::Bundled{symbol:"tsrust_std_env_current_dir"},..GET};
pub(crate) const CURRENT_EXE:NativeBinding=NativeBinding{name:"currentExe",backend:NativeBackend::Bundled{symbol:"tsrust_std_env_current_exe"},..CURRENT_DIR};
pub(crate) const TEMP_DIR:NativeBinding=NativeBinding{name:"tempDir",backend:NativeBackend::Bundled{symbol:"tsrust_std_env_temp_dir"},..CURRENT_DIR};
pub(crate) const HOME_DIR:NativeBinding=NativeBinding{name:"homeDir",parameters:&[],backend:NativeBackend::Bundled{symbol:"tsrust_std_env_home_dir"},..GET};
