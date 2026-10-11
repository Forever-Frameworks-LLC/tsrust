use crate::native_bindings::{NativeBackend,NativeBinding,NativeType};

pub(crate) const MONOTONIC_NOW:NativeBinding=NativeBinding {
    module:"std:time",name:"monotonicNow",returned_borrow:None,
    parameters:&[],result:NativeType::Number,
    backend:NativeBackend::Bundled{symbol:"tsrust_std_time_monotonic_now"},
    features:&["monotonic-time"],
};

pub(crate) const RUNTIME:&str=include_str!("runtime.rs");
