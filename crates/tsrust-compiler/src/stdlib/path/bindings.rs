use crate::native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType};
use crate::Mode;
use crate::native_signatures::{NativeTypeId,Signature};
const BASENAME_PARAMETERS: &[NativeParameter] = &[
    NativeParameter {
        name: "path",
        ty: NativeType::String,
        access: Mode::Read,
        optional: false, rest: false,
    },
    NativeParameter {
        name: "suffix",
        ty: NativeType::String,
        access: Mode::Read,
        optional: true, rest: false,
    },
];
pub(crate) const BASENAME: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:path",
    name: "basename",
    parameters: BASENAME_PARAMETERS,
    result: NativeType::String,
    backend: NativeBackend::Bundled {
        symbol: "tsrust_std_path_basename",
    },
    features: &["path"],
};

const PATH_PARAMETER: &[NativeParameter] = &[NativeParameter {
    name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false,
}];
pub(crate) const DIRNAME: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:path", name: "dirname", parameters: PATH_PARAMETER,
    result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_path_dirname" }, features: &["path"],
};
pub(crate) const EXTNAME: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:path", name: "extname", parameters: PATH_PARAMETER,
    result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_path_extname" }, features: &["path"],
};

pub(crate) const IS_ABSOLUTE: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:path", name: "isAbsolute", parameters: PATH_PARAMETER,
    result: NativeType::Boolean,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_path_is_absolute" }, features: &["path"],
};
pub(crate) const NORMALIZE: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:path", name: "normalize", parameters: PATH_PARAMETER,
    result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_path_normalize" }, features: &["path"],
};

pub(crate) const JOIN: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:path", name: "join",
    parameters: &[NativeParameter {name:"paths",ty:NativeType::String,access:Mode::Read,optional:false,rest:true}],
    result: NativeType::String,
    backend: NativeBackend::Bundled {symbol:"tsrust_std_path_join"}, features:&["path"],
};

pub(crate) const RELATIVE: NativeBinding = NativeBinding { returned_borrow: None,
    module:"std:path",name:"relative",
    parameters:&[
        NativeParameter{name:"from",ty:NativeType::String,access:Mode::Read,optional:false,rest:false},
        NativeParameter{name:"to",ty:NativeType::String,access:Mode::Read,optional:false,rest:false},
    ],result:NativeType::String,backend:NativeBackend::Bundled{symbol:"tsrust_std_path_relative"},features:&["path"],
};
pub(crate) const PARSE:NativeBinding=NativeBinding{module:"std:path",name:"parse",parameters:PATH_PARAMETER,result:NativeType::Structured(&Signature::Named(NativeTypeId::PathParts)),returned_borrow:None,backend:NativeBackend::Bundled{symbol:"tsrust_std_path_parse"},features:&["path"]};
pub(crate) const FORMAT:NativeBinding=NativeBinding{module:"std:path",name:"format",parameters:&[NativeParameter{name:"parts",ty:NativeType::Structured(&Signature::Named(NativeTypeId::PathFormatOptions)),access:Mode::Read,optional:false,rest:false}],result:NativeType::String,returned_borrow:None,backend:NativeBackend::Bundled{symbol:"tsrust_std_path_format"},features:&["path"]};
pub(crate) const RESOLVE:NativeBinding=NativeBinding{module:"std:path",name:"resolve",parameters:JOIN.parameters,result:NativeType::Structured(&Signature::Result(&Signature::String,&Signature::Named(NativeTypeId::PathError))),returned_borrow:None,backend:NativeBackend::Bundled{symbol:"tsrust_std_path_resolve"},features:&["path"]};
pub(crate) const SEPARATOR:NativeBinding=NativeBinding{module:"std:path",name:"separator",parameters:&[],result:NativeType::String,returned_borrow:None,backend:NativeBackend::Bundled{symbol:"tsrust_std_path_separator"},features:&["path"]};
pub(crate) const DELIMITER:NativeBinding=NativeBinding{name:"delimiter",backend:NativeBackend::Bundled{symbol:"tsrust_std_path_delimiter"},..SEPARATOR};
