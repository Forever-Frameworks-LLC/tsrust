//! Schema plan construction, validation and export emission.
use super::*;

#[path = "collection_trials.rs"]
mod schema_collection_trials;
#[path = "decode.rs"]
mod schema_decode;
#[path = "decode_call.rs"]
mod schema_decode_call;
#[path = "decode_input.rs"]
mod schema_decode_input;
#[path = "decode_pipelines.rs"]
mod schema_decode_pipelines;
#[path = "diagnostic_options.rs"]
mod schema_diagnostic_options;
#[path = "export.rs"]
mod schema_export;
#[path = "json_projection.rs"]
mod schema_json_projection;
#[path = "owned_projection.rs"]
mod schema_owned_projection;
#[path = "pipeline_array_trials.rs"]
mod schema_pipeline_array_trials;
#[path = "pipeline_object_trials.rs"]
mod schema_pipeline_object_trials;
#[path = "pipeline_trials.rs"]
mod schema_pipeline_trials;
#[path = "pipeline_wrappers.rs"]
mod schema_pipeline_wrappers;
#[path = "pipelines.rs"]
mod schema_pipelines;
#[path = "plans.rs"]
mod schema_plans;
#[path = "prepare_composite.rs"]
mod schema_prepare_composite;
#[path = "prepare_json.rs"]
mod schema_prepare_json;
#[path = "prepare_owned.rs"]
mod schema_prepare_owned;
#[path = "recursive_output.rs"]
mod schema_recursive_output;
#[path = "validation.rs"]
mod schema_validation;
