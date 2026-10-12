//! Callable contracts, capture ownership, storage, and dispatch.
pub(crate) mod aggregate_callable_facts;
pub(crate) mod borrowed_callable_contracts;
pub(crate) mod borrowed_callable_dispatch;
pub(crate) mod annotations;
pub(crate) mod compatibility;
pub(crate) mod contracts;
pub(crate) mod dispatch;
pub(crate) mod environment_demand;
pub(crate) mod layout;
pub(crate) mod resolution;
pub(crate) mod size;
pub(crate) mod storage;
pub(crate) mod capture_loan_identity;
pub(crate) mod capture_loan_owners;
pub(crate) mod local_capture_plan;
pub(crate) mod local_function_values;
pub(crate) mod local_functions;
pub(crate) mod local_readiness;
pub(crate) mod named_capture_lowering;
pub(crate) mod native_view_callable_storage;
pub(crate) mod native_view_family_registry;
pub(crate) mod native_view_origin_remap;
pub(crate) mod polymorphic_callable_storage;
pub(crate) mod polymorphic_invocation_plans;
pub(crate) mod environment_lifetimes;

pub(crate) mod module_state;
