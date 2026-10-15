//! Narrow destructor instrumentation, not an external Rust adapter facility.
use crate::{Mode, native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType}, native_signatures::{NativeTypeId, Signature}};
const RESOURCE: NativeType = NativeType::Structured(&Signature::Named(NativeTypeId::TrackedResource));
pub const CREATE: NativeBinding = NativeBinding {
 module: "std:testing-runtime", name: "trackedResource", returned_borrow: None, parameters: &[], result: RESOURCE,
 backend: NativeBackend::Bundled { symbol: "tsr_resource_create" }, features: &["testing-resources"],
};
pub const ID: NativeBinding = NativeBinding {
 name: "resourceId", parameters: &[NativeParameter { name: "resource", ty: RESOURCE, access: Mode::Read, optional: false, rest: false }],
 result: NativeType::Int, backend: NativeBackend::Bundled { symbol: "tsr_resource_id" }, ..CREATE
};
pub const CLOSE: NativeBinding = NativeBinding {
 name: "closeResource", parameters: &[NativeParameter { name: "resource", ty: RESOURCE, access: Mode::Consume, optional: false, rest: false }],
 result: NativeType::Void, backend: NativeBackend::Bundled { symbol: "tsr_resource_close" }, ..CREATE
};
pub const RELEASE_COUNT: NativeBinding = NativeBinding {
 name: "releaseCount", parameters: &[NativeParameter { name: "id", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false }],
 result: NativeType::Int, backend: NativeBackend::Bundled { symbol: "tsr_resource_release_count" }, ..CREATE
};
pub const RELEASE_ORDER: NativeBinding = NativeBinding {
 name: "releaseOrder", backend: NativeBackend::Bundled { symbol: "tsr_resource_release_order" }, ..RELEASE_COUNT
};
pub const LIVE_COUNT: NativeBinding = NativeBinding {
 name: "liveCount", result: NativeType::Int, backend: NativeBackend::Bundled { symbol: "tsr_resource_live_count" }, ..CREATE
};
