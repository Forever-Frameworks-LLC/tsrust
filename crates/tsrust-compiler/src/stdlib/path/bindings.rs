use crate::native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType};
use crate::Mode;
const BASENAME_PARAMETERS: &[NativeParameter] = &[
    NativeParameter {
        name: "path",
        ty: NativeType::String,
        access: Mode::Read,
        optional: false,
    },
    NativeParameter {
        name: "suffix",
        ty: NativeType::String,
        access: Mode::Read,
        optional: true,
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
    name: "path", ty: NativeType::String, access: Mode::Read, optional: false,
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
