use crate::native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType};
use crate::Mode;
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
pub(crate) const BASENAME: NativeBinding = NativeBinding {
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
pub(crate) const DIRNAME: NativeBinding = NativeBinding {
    module: "std:path", name: "dirname", parameters: PATH_PARAMETER,
    result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_path_dirname" }, features: &["path"],
};
pub(crate) const EXTNAME: NativeBinding = NativeBinding {
    module: "std:path", name: "extname", parameters: PATH_PARAMETER,
    result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_path_extname" }, features: &["path"],
};

pub(crate) const IS_ABSOLUTE: NativeBinding = NativeBinding {
    module: "std:path", name: "isAbsolute", parameters: PATH_PARAMETER,
    result: NativeType::Boolean,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_path_is_absolute" }, features: &["path"],
};
pub(crate) const NORMALIZE: NativeBinding = NativeBinding {
    module: "std:path", name: "normalize", parameters: PATH_PARAMETER,
    result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_path_normalize" }, features: &["path"],
};

pub(crate) const JOIN: NativeBinding = NativeBinding {
    module: "std:path", name: "join",
    parameters: &[NativeParameter {name:"paths",ty:NativeType::String,access:Mode::Read,optional:false,rest:true}],
    result: NativeType::String,
    backend: NativeBackend::Bundled {symbol:"tsrust_std_path_join"}, features:&["path"],
};

pub(crate) const RELATIVE: NativeBinding = NativeBinding {
    module:"std:path",name:"relative",
    parameters:&[
        NativeParameter{name:"from",ty:NativeType::String,access:Mode::Read,optional:false,rest:false},
        NativeParameter{name:"to",ty:NativeType::String,access:Mode::Read,optional:false,rest:false},
    ],result:NativeType::String,backend:NativeBackend::Bundled{symbol:"tsrust_std_path_relative"},features:&["path"],
};
