//! Explicit checked API contracts. Ordinary exports never publish an operation.
mod typescript;
mod react_query;
mod native_codecs;
mod compatibility;
mod native;
pub use native::{NativeImport, NativeServer};
use crate::{analysis::CheckedProgram, project::{ProjectAnalysis, SourceId}, syntax::{CallbackKind, Type}};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub name: String,
    pub module: String,
    pub export: String,
    pub kind: Kind,
    /// Explicitly hide the first borrowed native HTTP Context from wire data.
    #[serde(default)]
    pub context: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Kind { Query, Mutation }

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Schema {
    String, Integer, Number, Boolean, Null, Void,
    Literal { value: serde_json::Value },
    Array { items: Box<Schema> },
    Tuple { items: Vec<Schema> },
    Object { properties: BTreeMap<String, Schema>, required: Vec<String> },
    Union { members: Vec<Schema> },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub name: String,
    pub kind: Kind,
    pub input: Schema,
    pub output: Schema,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub operations: Vec<Operation>,
}

/// Checked public contracts plus private dispatch facts from the same functions.
pub struct CheckedApi {
    pub manifest: Manifest,
    endpoints: BTreeMap<String, Endpoint>,
}

struct Endpoint {
    selection: Selection,
    asynchronous: bool,
}

fn valid_segment(name: &str) -> bool {
    !matches!(name, "__proto__" | "prototype" | "constructor" | "then" | "toJSON")
        && name.as_bytes().first().is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

impl Manifest {
    /// Resolve an explicit allowlist against a successfully checked project.
    /// Select zero-argument functions or one required data argument. Async results
    /// are unwrapped to their eventual value. No request is dispatched here.
    pub fn from_project(project: &ProjectAnalysis, selections: &[Selection]) -> Result<Self, String> {
        Ok(CheckedApi::from_project(project, selections)?.manifest)
    }

    /// Stable identity of wire contracts, independent of file paths/package version.
    pub fn identity(&self) -> String {
        use sha2::{Digest, Sha256};
        Sha256::digest(serde_json::to_vec(self).expect("API manifest serialization")).iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// Generate an ordinary dependency-free ESM package with declarations.
    /// The returned files have no server source, absolute paths or framework imports.
    pub fn client_package(&self, name: &str, version: &str) -> Result<BTreeMap<String, String>, String> {
        typescript::package(self, name, version)
    }
}

impl CheckedApi {
    pub fn from_project(project: &ProjectAnalysis, selections: &[Selection]) -> Result<Self, String> {
        let mut names = BTreeSet::new();
        let mut operations = Vec::new();
        let mut endpoints = BTreeMap::new();
        for selection in selections {
            let fail = |message: String| format!("API operation {:?}: {message}", selection.name);
            if selection.name.len() > 200 || !selection.name.split('.').all(valid_segment) {
                return Err(fail("use dot-separated ASCII identifiers; reserved object/promise names are excluded".into()));
            }
            if !names.insert(selection.name.clone()) { return Err(fail("duplicate operation name".into())); }
            let source = SourceId(selection.module.clone());
            let module = project.graph.modules.get(&source).ok_or_else(|| fail("selected module is not in the checked project".into()))?;
            if module.rust.is_some() || module.data.is_some() { return Err(fail("select a Tsrust function export".into())); }
            if !module.exports.contains_key(&selection.export) { return Err(fail(format!("{:?} is not an explicit export", selection.export))); }
            let local = module.exported_local(&selection.export);
            let position = project.graph.order.iter().position(|id| id == &source).ok_or_else(|| fail("missing checked module identity".into()))?;
            let linked = if source == project.graph.entry && local == "main" { "main".into() } else { crate::project_link::symbol(position, local) };
            let program = &project.analysis.program;
            let function = program.functions.iter().find(|function| function.name == linked && !function.specialization)
                .ok_or_else(|| fail("export is not a checked named function".into()))?;
            if function.template { return Err(fail("generic operations need an explicit concrete wrapper function".into())); }
            let private_count = usize::from(selection.context);
            if selection.context {
                let context = function.bindings.first().filter(|_| function.parameter_count > 0)
                    .ok_or_else(|| fail("context:true requires a first borrowed native HTTP Context parameter".into()))?;
                if !matches!(context.ty, Type::Record(id) if program.records[id].native_type == Some(crate::native_signatures::NativeTypeId::HttpContext))
                    || !matches!(context.mode, crate::Mode::Read | crate::Mode::Mutate)
                    || context.rest || context.omission.is_omittable() {
                    return Err(fail("context:true requires a required ReadOnly<Context> or Mutable<Context>; arbitrary resources and owned contexts cannot be hidden".into()));
                }
            }
            if function.parameter_count > 1 + private_count { return Err(fail("use one request record argument, or no arguments, after any explicitly selected Context".into())); }
            let mut builder = Builder { program, active: HashSet::new() };
            let input = if let Some(parameter) = function.bindings.iter().take(function.parameter_count).nth(private_count) {
                if parameter.rest || parameter.omission.is_omittable() { return Err(fail("rest/default/optional parameters need a required request-record wrapper".into())); }
                builder.schema(parameter.input_ty.unwrap_or(parameter.ty), false, 0).map_err(&fail)?
            } else { Schema::Void };
            let result = match function.result {
                Type::Closure(id) if program.task_origins.contains(&id) => program.closures[id].function.result,
                Type::Callback(id) if program.callable_shapes.get(program.callback_contracts[id].structural).kind == Some(CallbackKind::Task) => program.callback_contracts[id].result,
                ty => ty,
            };
            let output = builder.schema(result, true, 0).map_err(&fail)?;
            endpoints.insert(selection.name.clone(), Endpoint { selection: selection.clone(), asynchronous: result != function.result });
            operations.push(Operation { name: selection.name.clone(), kind: selection.kind, input, output });
        }
        for name in &names {
            let mut prefix = name.as_str();
            while let Some((parent, _)) = prefix.rsplit_once('.') {
                if names.contains(parent) { return Err(format!("API operation {parent:?} conflicts with namespace of {name:?}")); }
                prefix = parent;
            }
        }
        operations.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(Self { manifest: Manifest { schema_version: 1, operations }, endpoints })
    }
}

struct Builder<'a> { program: &'a CheckedProgram, active: HashSet<Type> }
impl Builder<'_> {
    fn schema(&mut self, ty: Type, allow_void: bool, depth: usize) -> Result<Schema, String> {
        if depth > 128 || !self.active.insert(ty) { return Err("recursive or excessively deep wire data is unsupported; project it into bounded plain data".into()); }
        let result = self.value(ty, allow_void, depth);
        self.active.remove(&ty);
        result
    }

    fn value(&mut self, ty: Type, allow_void: bool, depth: usize) -> Result<Schema, String> {
        let program = self.program;
        Ok(match ty {
            Type::String => Schema::String,
            Type::Int => Schema::Integer,
            Type::Number => Schema::Number,
            Type::Boolean => Schema::Boolean,
            Type::Null => Schema::Null,
            Type::Void if allow_void => Schema::Void,
            Type::Literal(id, _) => {
                use crate::literal_types::LiteralValue;
                let value = match program.records[id].literal_definition.as_ref().ok_or("missing checked literal identity")? {
                    LiteralValue::String(value) => serde_json::json!(value),
                    LiteralValue::Boolean(value) => serde_json::json!(value),
                    LiteralValue::Int(value) if value.unsigned_abs() <= 9_007_199_254_740_991 => serde_json::json!(value),
                    LiteralValue::Number(bits) => serde_json::json!(f64::from_bits(*bits)),
                    _ => return Err("integer literal is outside JavaScript's safe integer range; use an explicit string projection".into()),
                };
                Schema::Literal { value }
            }
            Type::Array(element) => Schema::Array { items: Box::new(self.schema(element.ty(), false, depth + 1)?) },
            Type::Record(id) => {
                let record = &program.records[id];
                if record.native_type.is_some() || record.enum_definition.is_some() || record.set_definition.is_some() || record.map_definition.is_some() || !record.type_parameters.is_empty() {
                    return Err("native resources, enums, collections and unresolved generic records need an explicit plain-data projection".into());
                }
                if record.is_tuple() {
                    Schema::Tuple { items: record.fields.iter().map(|field| self.schema(field.ty.ok_or("unresolved tuple member")?, false, depth + 1)).collect::<Result<_, _>>()? }
                } else {
                    let mut properties = BTreeMap::new();
                    let mut required = Vec::new();
                    for field in &record.fields {
                        let ty = field.ty.ok_or("unresolved record field")?;
                        let schema = if field.optional {
                            if let Type::Union(id) = ty {
                                let members = program.unions[id.index()].members.clone().into_iter().filter(|ty| *ty != Type::Undefined)
                                    .map(|ty| self.schema(ty, false, depth + 1)).collect::<Result<Vec<_>, _>>()?;
                                union(members)?
                            } else { self.schema(ty, false, depth + 1)? }
                        } else { required.push(field.name.text.clone()); self.schema(ty, false, depth + 1)? };
                        properties.insert(field.name.text.clone(), schema);
                    }
                    required.sort();
                    Schema::Object { properties, required }
                }
            }
            Type::Union(id) => {
                let members = program.unions[id.index()].members.clone().into_iter()
                    .map(|ty| self.schema(ty, false, depth + 1)).collect::<Result<_, _>>()?;
                union(members)?
            }
            _ => return Err("type is not plain wire data; functions, undefined values and native handles cannot be exported".into()),
        })
    }
}

fn union(mut members: Vec<Schema>) -> Result<Schema, String> {
    if members.is_empty() { return Err("an optional field needs a representable non-undefined value".into()); }
    if members.len() == 1 { Ok(members.remove(0)) } else { Ok(Schema::Union { members }) }
}
