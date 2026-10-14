//! Project facts retain nominal source identity instead of linked display names.
use crate::Mode;
use crate::project::{ProjectAnalysis, SourceLocation};
use crate::syntax::{CallbackKind, Type};
use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectLocalFunctionInfo {
    pub id: usize,
    pub scope: usize,
    pub name: String,
    pub declaration: SourceLocation,
    pub function_scope: SourceLocation,
    pub result: Option<ProjectType>,
}

/// Concrete type with nominal declaration identity; no linked display-name parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectType {
    Native(crate::native_signatures::NativeTypeId),
    AppliedNative(crate::native_signatures::NativeTypeId, Vec<ProjectType>),
    SchemaPipeline { identity: usize, input: Box<ProjectType>, output: Box<ProjectType>, json_input: bool, text_input: bool, object_policy: bool, scalar_constraints: bool },
    Named {
        name: String,
        declaration: Option<SourceLocation>,
        arguments: Vec<ProjectType>,
    },
    Structural { identity: usize, retained: bool, fields: Vec<(String, bool, ProjectType)> },
    StructuralReference { identity: usize, retained: bool },
    Recursive { identity: usize, body: Box<ProjectType> },
    RecursiveReference { identity: usize },
    Array(Box<ProjectType>),
    Union(Vec<ProjectType>),
    Tuple(Vec<ProjectType>),
    Variant { name: String, payload: Box<ProjectType> },
    Callable {
        ownership: crate::type_syntax::CallableOwnership,
        parameters: Vec<ProjectParameterInfo>,
        result: Box<ProjectType>,
    },
}
impl ProjectType {
    /// Expanded union aliases can close a cycle without carrying the original
    /// reserved union's recursive flag. Retain the actual back-edge identity.
    fn references_recursive(&self, target: usize) -> bool {
        match self {
            Self::RecursiveReference { identity } => *identity == target,
            Self::Array(inner) | Self::Recursive { body: inner, .. } | Self::Variant { payload: inner, .. } => inner.references_recursive(target),
            Self::Union(members) | Self::Tuple(members) | Self::AppliedNative(_, members) | Self::Named { arguments: members, .. } => members.iter().any(|member| member.references_recursive(target)),
            Self::Structural { fields, .. } => fields.iter().any(|(_, _, ty)| ty.references_recursive(target)),
            Self::SchemaPipeline { input, output, .. } => input.references_recursive(target) || output.references_recursive(target),
            Self::Callable { parameters, result, .. } => result.references_recursive(target) || parameters.iter().any(|parameter| parameter.ty.as_ref().is_some_and(|ty| ty.references_recursive(target)) || parameter.body_ty.as_ref().is_some_and(|ty| ty.references_recursive(target))),
            Self::Native(_) | Self::StructuralReference { .. } => false,
        }
    }
}
/// Checked argument contract. Unresolved callable shapes remain unknown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectParameterInfo {
    pub name: String,
    pub ty: Option<ProjectType>,
    pub body_ty: Option<ProjectType>,
    pub omission: crate::ParameterOmission,
    pub rest: bool,
    pub access: Mode,
}
/// Call, callee and declaration ranges each carry their own original source identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectGenericCallInfo {
    pub location: SourceLocation,
    pub callee: SourceLocation,
    pub declaration: SourceLocation,
    pub parameters: Vec<ProjectParameterInfo>,
    pub result: ProjectType,
}
/// Known generic parameters with no asserted concrete result shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectGenericParameterCallInfo {
    pub location: SourceLocation,
    pub callee: SourceLocation,
    pub declaration: SourceLocation,
    pub parameters: Vec<ProjectParameterInfo>,
}
/// Checked expression type in its original module's UTF-8 source coordinates.
/// Distinct specializations at a shared span remain separate facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectExpressionTypeInfo {
    pub location: SourceLocation,
    pub ty: ProjectType,
}

/// Project-aware metadata from one successful checked-body walk. Optional
/// receiver facts describe successful hops; they do not authorize ordinary access.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectCheckedTypeFacts {
    /// Program-owned declarations, with source identity preserved across imports.
    pub module_values: Vec<ProjectExpressionTypeInfo>,
    /// Enum identities retain their declaration across import aliases.
    pub enum_scalars: Vec<(ProjectType, String)>,
    pub json_string_receivers: Vec<SourceLocation>,
    /// Fixed nominal annotation use and declaration locations in generic headers.
    pub generic_nominals: Vec<(SourceLocation, SourceLocation)>,
    /// Checked generic value binding and original declaration source locations.
    pub generic_values: Vec<(SourceLocation, SourceLocation)>,
    pub parameter_types: Vec<ProjectExpressionTypeInfo>,
    pub projections: Vec<ProjectExpressionTypeInfo>,
    pub optional_receivers: Vec<ProjectExpressionTypeInfo>,
    pub method_results: Vec<ProjectExpressionTypeInfo>,
    pub generic_calls: Vec<ProjectGenericCallInfo>,
    pub generic_parameter_calls: Vec<ProjectGenericParameterCallInfo>,
}

impl ProjectAnalysis {
    pub fn checked_local_functions(&self) -> Vec<ProjectLocalFunctionInfo> {
        if self.analysis.program.local_declarations.is_empty() { return Vec::new(); }
        let functions = self.analysis.program.functions.iter().filter(|function| !function.template)
            .map(|function| (function.name.as_str(), function)).collect::<HashMap<_, _>>();
        let names = self.analysis.program.records.iter().enumerate()
            .map(|(id, record)| (record.name.text.as_str(), id)).collect();
        self.analysis.program.local_declarations.iter().filter_map(|declaration| {
            let result = functions.get(declaration.emitted_name.as_str())
                .and_then(|function| self.project_type(function.result, &names));
            Some(ProjectLocalFunctionInfo { id: declaration.id, scope: declaration.scope,
                name: declaration.source_name.text.clone(),
                declaration: self.source_location(declaration.source_name.span)?,
                function_scope: self.source_location(declaration.function_scope)?, result })
        }).collect()
    }

    pub fn checked_control_labels(&self)->Vec<(SourceLocation,SourceLocation)>{
        self.analysis.checked_control_labels().into_iter().filter_map(|(span,definition)|Some((self.source_location(span)?,self.source_location(definition)?))).collect()
    }

    /// Preserve successful checked views/results with nominal source identities.
    /// Unknown variants suppress their shared-span group; callers must also keep
    /// multiple distinct known variants conservative when selecting editor help.
    pub fn checked_type_facts(&self) -> ProjectCheckedTypeFacts {
        let names = self
            .analysis
            .program
            .records
            .iter()
            .enumerate()
            .map(|(id, record)| (record.name.text.as_str(), id))
            .collect();
        let facts = self.analysis.checked_type_facts();
        let generic_values = facts.generic_values.iter().filter_map(|(span, declaration)|
            Some((self.source_location(*span)?, self.source_location(*declaration)?))).collect();
        // Legacy display metadata may omit an abstract result specialization.
        // Keep its known sibling from authorizing project signature help.
        let unresolved_results = facts
            .identities
            .method_results
            .iter()
            .filter(|(_, types)| {
                types
                    .iter()
                    .any(|ty| ty.and_then(|ty| self.project_type(ty, &names)).is_none())
            })
            .map(|(&span, _)| span)
            .collect::<HashSet<_>>();
        let parameter_types = self.project_expression_types(facts.identities.parameter_types, &names);
        let projections = self.project_expression_types(facts.identities.projections, &names);
        let optional_receivers =
            self.project_expression_types(facts.identities.optional_receivers, &names);
        let method_results = self.project_expression_types(facts.identities.method_results, &names);
        let generic_calls = facts
            .generic_calls
            .into_iter()
            .filter(|fact| !unresolved_results.contains(&(fact.span.start, fact.span.end)))
            .filter_map(|fact| {
                Some(ProjectGenericCallInfo {
                    location: self.source_location(fact.span)?,
                    callee: self.source_location(fact.callee_span)?,
                    declaration: self.source_location(fact.declaration_span)?,
                    parameters: fact
                        .parameters
                        .into_iter()
                        .zip(fact.parameter_types)
                        .zip(fact.body_parameter_types)
                        .map(|((parameter, ty), body_ty)| ProjectParameterInfo {
                            name: parameter.name,
                            access: parameter.access,
                            omission: parameter.omission,
                            rest: parameter.rest,
                            body_ty: self.project_type(body_ty, &names),
                            ty: self.project_type(ty, &names),
                        })
                        .collect(),
                    result: self.project_type(fact.result_type, &names)?,
                })
            })
            .collect();
        let generic_parameter_calls = facts.generic_parameter_calls.into_iter().filter_map(|fact| {
            Some(ProjectGenericParameterCallInfo {
                location: self.source_location(fact.span)?,
                callee: self.source_location(fact.callee_span)?,
                declaration: self.source_location(fact.declaration_span)?,
                parameters: fact.parameters.into_iter().zip(fact.parameter_types).zip(fact.body_parameter_types)
                    .map(|((parameter, ty), body_ty)| ProjectParameterInfo {
                        name: parameter.name, access: parameter.access, omission: parameter.omission,
                        rest: parameter.rest, ty: self.project_type(ty, &names), body_ty: self.project_type(body_ty, &names),
                    }).collect(),
            })
        }).collect();
        let enum_scalars=self.analysis.program.records.iter().enumerate().filter_map(|(id,record)|{let definition=record.enum_definition.as_ref()?;let ty=self.project_type(Type::Enum(id,definition.kind),&names)?;Some((ty,if definition.kind==crate::syntax::EnumKind::String{"string"}else{"number"}.to_owned()))}).collect();
        let module_values = self.analysis.program.functions.iter().find(|function| function.name == "main")
            .into_iter().flat_map(|function| &function.bindings).filter_map(|binding| {
                let span = binding.declaration_span?;
                if !self.analysis.program.module_owner_bindings.contains(&span) { return None; }
                Some(ProjectExpressionTypeInfo { location: self.source_location(span)?, ty: self.project_type(binding.ty, &names)? })
            }).collect();
        ProjectCheckedTypeFacts {
            module_values,
            enum_scalars,
            json_string_receivers: facts.json_string_receivers.iter().filter_map(|span|self.source_location(*span)).collect(),
            generic_nominals: self.analysis.program.deferred_generic_nominals.iter().filter_map(|(span, declaration)|
                Some((self.source_location(*span)?, self.source_location(*declaration)?))).collect(),
            generic_values,
            parameter_types,
            projections,
            optional_receivers,
            method_results,
            generic_calls,
            generic_parameter_calls,
        }
    }

    /// Compatibility accessor for project-aware generic call facts.
    pub fn generic_calls(&self) -> Vec<ProjectGenericCallInfo> {
        self.checked_type_facts().generic_calls
    }

    fn project_expression_types(
        &self,
        groups: std::collections::BTreeMap<(usize, usize), Vec<Option<Type>>>,
        names: &HashMap<&str, usize>,
    ) -> Vec<ProjectExpressionTypeInfo> {
        let mut output = Vec::new();
        for ((start, end), types) in groups {
            let Some(location) = self.source_location(crate::Span::new(start, end)) else {
                continue;
            };
            let Some(types) = types
                .into_iter()
                .map(|ty| self.project_type(ty?, names))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let mut unique = Vec::new();
            for ty in types {
                if !unique.contains(&ty) {
                    unique.push(ty);
                }
            }
            output.extend(unique.into_iter().map(|ty| ProjectExpressionTypeInfo {
                location: location.clone(),
                ty,
            }));
        }
        output
    }

    fn project_type(&self, ty: Type, names: &HashMap<&str, usize>) -> Option<ProjectType> {
        self.project_type_inner(ty, names, &mut HashSet::new())
    }
    fn project_type_inner(
        &self,
        ty: Type,
        names: &HashMap<&str, usize>,
        active: &mut HashSet<Type>,
    ) -> Option<ProjectType> {
        if !active.insert(ty) {
            return match ty {
                Type::Record(id) => Some(ProjectType::StructuralReference {
                    identity: id, retained: self.analysis.program.records[id].name.text.starts_with("@schema:retained:"),
                }),
                Type::Union(id) | Type::BorrowedUnion(id) =>
                    Some(ProjectType::RecursiveReference { identity: id.index() }),
                Type::Array(element) => Some(ProjectType::Array(Box::new(self.project_type_inner(element.ty(), names, active)?))),
                _ => None,
            };
        }
        let result = self.project_type_value(ty, names, active);
        active.remove(&ty);
        result
    }
    fn project_type_value(
        &self,
        ty: Type,
        names: &HashMap<&str, usize>,
        active: &mut HashSet<Type>,
    ) -> Option<ProjectType> {
        let primitive = match ty {
            Type::Int => "int",
            Type::Number => "number",
            Type::String => "string",
            Type::Boolean => "boolean",
            Type::Null => "null",
            Type::Undefined => "undefined",
            Type::Void => "void",
            Type::Closure(id) => {
                if self.analysis.program.deferred_generic_values.contains(&id) { return None; }
                let closure = &self.analysis.program.closures[id];
                if closure.function.template {
                    return None;
                }
                let ownership = if closure.invocation == Mode::Consume {
                    crate::type_syntax::CallableOwnership::Once
                } else {
                    crate::type_syntax::CallableOwnership::Default
                };
                return Some(ProjectType::Callable {
                    ownership,
                    parameters: closure
                        .parameters
                        .iter()
                        .map(|id| {
                            let binding = &closure.function.bindings[*id];
                            Some(ProjectParameterInfo {
                                name: binding.name.clone(),
                                ty: Some(self.project_type_inner(binding.input_ty.unwrap_or(binding.ty), names, active)?),
                                body_ty: Some(self.project_type_inner(binding.ty, names, active)?),
                                omission: binding.omission,
                                    rest: binding.rest,
                                access: binding.mode,
                            })
                        })
                        .collect::<Option<Vec<_>>>()?,
                    result: Box::new(self.project_type_inner(
                        closure.function.result,
                        names,
                        active,
                    )?),
                });
            }
            Type::Callback(id) => {
                let contract = &self.analysis.program.callback_contracts[id];
                let ownership = match self
                    .analysis
                    .program
                    .callable_shapes
                    .get(contract.structural)
                    .kind
                    .expect("callback structural contract")
                {
                    CallbackKind::Read => crate::type_syntax::CallableOwnership::Read,
                    CallbackKind::Reusable => crate::type_syntax::CallableOwnership::Reusable,
                    CallbackKind::Once => crate::type_syntax::CallableOwnership::Once,
                    CallbackKind::ProgramReusable => crate::type_syntax::CallableOwnership::ProgramReusable,
                    CallbackKind::Program => crate::type_syntax::CallableOwnership::Program,
                    CallbackKind::Owned => crate::type_syntax::CallableOwnership::Owned,
                    CallbackKind::SharedRead => crate::type_syntax::CallableOwnership::SharedRead,
                    CallbackKind::OwnedOnce => crate::type_syntax::CallableOwnership::OwnedOnce,
                };
                return Some(ProjectType::Callable {
                    ownership,
                    parameters: contract
                        .params
                        .iter()
                        .enumerate()
                        .map(|(index, parameter)| {
                            Some(ProjectParameterInfo {
                                name: contract.parameter_names[index].clone(),
                                ty: Some(self.project_type_inner(parameter.ty, names, active)?),
                                body_ty: None,
                                omission: parameter.omission,
                            rest: parameter.rest,
                                access: parameter.mode,
                            })
                        })
                        .collect::<Option<Vec<_>>>()?,
                    result: Box::new(self.project_type_inner(contract.result, names, active)?),
                });
            }
            Type::StoredCallable(id) => {
                let declaration = self.analysis.program.stored_contracts.declaration(id)?;
                return self.project_type_inner(Type::Callback(declaration), names, active);
            }
            Type::Map(id) => {
                if self.analysis.program.records[id].native_type.is_some(){return self.project_record_inner(id,names,active);}
                let map=self.analysis.program.records[id].map_definition.as_ref()?;
                return Some(ProjectType::Named {name:"Map".into(),declaration:None,arguments:vec![self.project_type_inner(map.key,names,active)?,self.project_type_inner(map.value,names,active)?]});
            },
            Type::Set(id) => return Some(ProjectType::Named {
                name: "Set".into(), declaration: None,
                arguments: vec![self.project_type_inner(self.analysis.program.records[id].set_definition.as_ref()?.key, names, active)?],
            }),
            Type::Literal(id,_)=>return Some(ProjectType::Named{name:self.analysis.program.records[id].literal_definition.as_ref()?.text(),declaration:None,arguments:vec![]}),
            Type::Record(id) | Type::Enum(id, _) => return self.project_record_inner(id, names, active),
            Type::Array(element) => {
                return Some(ProjectType::Array(Box::new(self.project_type_inner(
                    element.ty(),
                    names,
                    active,
                )?)));
            }
            Type::Union(id) | Type::BorrowedUnion(id) => {
                if let Some(variants) = crate::result_identity::variants(&self.analysis.program.records, &self.analysis.program.unions[id.index()].members) {
                    let arguments = variants.into_iter().map(|record| self.project_type_inner(crate::result_identity::payload(&self.analysis.program.records[record])?, names, active)).collect::<Option<_>>()?;
                    return Some(ProjectType::Named { name: "Result".into(), declaration: None, arguments });
                }
                let body = ProjectType::Union(
                    self.analysis.program.unions[id.index()].members.iter()
                        .map(|ty| self.project_type_inner(*ty, names, active))
                        .collect::<Option<_>>()?,
                );
                return Some(if self.analysis.program.unions[id.index()].recursive || body.references_recursive(id.index()) {
                    ProjectType::Recursive { identity: id.index(), body: Box::new(body) }
                } else { body });
            }
            _ => return None,
        };
        Some(ProjectType::Named {
            name: primitive.into(),
            declaration: None,
            arguments: vec![],
        })
    }
    fn project_record(&self, id: usize, names: &HashMap<&str, usize>) -> Option<ProjectType> {
        self.project_type(Type::Record(id), names)
    }
    fn project_record_inner(&self, id: usize, names: &HashMap<&str, usize>, active: &mut HashSet<Type>) -> Option<ProjectType> {
        let record = &self.analysis.program.records[id];
        if let Some((input, output, json_input)) = self.analysis.program.schema_pipelines.types(id) {
            return Some(ProjectType::SchemaPipeline {
                identity: id, input: Box::new(self.project_type_inner(input, names, active)?),
                output: Box::new(self.project_type_inner(output, names, active)?), json_input,
                text_input: self.analysis.program.schema_pipelines.accepts_text(id),
                scalar_constraints: self.analysis.program.schema_pipelines.wrapper(id).is_some_and(|plan|
                    matches!(output, Type::Int | Type::Number | Type::String)
                    && (matches!(plan.kind, crate::stdlib::schema::pipelines::WrapperKind::Constraint(_))
                        || (plan.kind == crate::stdlib::schema::pipelines::WrapperKind::Message
                            && self.analysis.program.records[plan.source].native_type == Some(crate::native_signatures::NativeTypeId::Schema)))),
                object_policy: {
                    let plans = &self.analysis.program.schema_pipelines;
                    let source = plans.wrapper(id).filter(|plan| plan.kind == crate::stdlib::schema::pipelines::WrapperKind::Message)
                        .map_or(id, |plan| plan.source);
                    plans.object(source).is_some_and(|plan| plan.policy != crate::stdlib::schema::pipelines::ObjectPolicy::Retain)
                        || (self.analysis.program.records[source].native_type == Some(crate::native_signatures::NativeTypeId::Schema)
                            && matches!(output, Type::Record(record) if !self.analysis.program.records[record].name.text.starts_with("@schema:retained:")))
                },
            });
        }
        if let Some(identity) = record.native_type {
            if let Some(application) = &record.application {
                let arguments = application.resolved_arguments.iter().map(|ty| self.project_type_inner(*ty, names, active)).collect::<Option<Vec<_>>>()?;
                if !arguments.is_empty() { return Some(ProjectType::AppliedNative(identity, arguments)); }
            }
            return Some(ProjectType::Native(identity));
        }
        if let Some(application) = &record.application {
            let variant = match application.declaration.text.as_str() { "Result#Ok" => Some("Ok"), "Result#Err" => Some("Err"), _ => None };
            if let Some(name) = variant {
                return Some(ProjectType::Variant { name: name.into(), payload: Box::new(self.project_type_inner(crate::result_identity::payload(record)?, names, active)?) });
            }
            // Schema outcome alternatives are declared by std:schema, not a project module.
            let outcome = application.declaration.text.as_str();
            if outcome == crate::stdlib::schema::builders::SUCCESS || outcome == crate::stdlib::schema::builders::FAILURE {
                let arguments = application.resolved_arguments.iter().map(|ty| self.project_type_inner(*ty, names, active)).collect::<Option<Vec<_>>>()?;
                return Some(ProjectType::Named { name: outcome.to_owned(), declaration: None, arguments });
            }
        }
        if record.is_tuple() {
            return Some(ProjectType::Tuple(record.fields.iter().map(|field| self.project_type_inner(field.ty?,names,active)).collect::<Option<_>>()?));
        }
        if record.name.text.starts_with("@module:object:")||record.name.text.starts_with("@schema:decode-error-")||record.name.text.starts_with("@schema:object:")||record.name.text.starts_with("@schema:input:")||record.name.text.starts_with("@schema:retained:") {
            return Some(ProjectType::Structural { identity: id, retained: record.name.text.starts_with("@schema:retained:"), fields: record.fields.iter().map(|field| Some((field.name.text.clone(), field.optional, self.project_type_inner(field.ty?, names, active)?))).collect::<Option<_>>()? });
        }
        let declaration = self.source_location(
            record
                .application
                .as_ref()
                .map_or(record.name.span, |application| application.declaration.span),
        )?;
        let source = &self.graph.modules[&declaration.source].source.text;
        let name = if self.graph.modules[&declaration.source].data.is_some() {
            format!("JsonData{}", declaration.span.start)
        } else { source.get(declaration.span.start..declaration.span.end)?.to_owned() };
        let arguments = record
            .application
            .as_ref()
            .map(|application| {
                application
                    .arguments
                    .iter()
                    .map(|argument| self.project_annotation(argument, names))
                    .collect::<Option<Vec<_>>>()
            })
            .unwrap_or(Some(vec![]))?;
        Some(ProjectType::Named {
            name,
            declaration: Some(declaration),
            arguments,
        })
    }
    fn project_annotation(
        &self,
        annotation: &ValueAnnotation,
        names: &HashMap<&str, usize>,
    ) -> Option<ProjectType> {
        match &annotation.kind {
            Kind::Literal(value) => Some(ProjectType::Named {name:value.text(),declaration:None,arguments:vec![]}),
            Kind::Named(name) => {
                if let Some(id) = names.get(name.text.as_str()) {
                    return self.project_record(*id, names);
                }
                Some(ProjectType::Named {
                    name: name.text.clone(),
                    declaration: None,
                    arguments: vec![],
                })
            }
            Kind::Group(inner) => self.project_annotation(inner, names),
            Kind::Array(inner) => Some(ProjectType::Array(Box::new(
                self.project_annotation(inner, names)?,
            ))),
            Kind::Union(members) => Some(ProjectType::Union(
                members
                    .iter()
                    .map(|member| self.project_annotation(member, names))
                    .collect::<Option<_>>()?,
            )),
            Kind::Tuple(positions) => Some(ProjectType::Tuple(positions.iter().map(|position|self.project_annotation(position,names)).collect::<Option<_>>()?)),
            Kind::Applied { .. } | Kind::Callable { .. } | Kind::Polymorphic { .. } => None,
        }
    }
}

#[cfg(test)]
#[test]
fn project_metadata_keeps_conflicts_and_suppresses_unknown_shared_spans() {
    struct Source;
    impl crate::project::SourceProvider for Source {
        fn resolve(
            &self,
            _: Option<&crate::project::SourceId>,
            _: &str,
        ) -> Result<crate::project::SourceId, String> {
            Ok(crate::project::SourceId("main.tsr".into()))
        }
        fn read(&self, _: &crate::project::SourceId) -> Result<String, String> {
            Ok("function main() {}".into())
        }
    }
    let checked =
        crate::project::check_graph(crate::project::load_project(&Source, "main.tsr").unwrap())
            .unwrap();
    let offset = (0..32)
        .find(|&offset| {
            checked
                .source_location(crate::Span::new(offset, offset))
                .is_some_and(|location| location.span.start == 0)
        })
        .expect("linked source origin");
    let groups = [
        (
            (offset, offset + 8),
            vec![Some(Type::Int), Some(Type::Number)],
        ),
        ((offset + 9, offset + 13), vec![Some(Type::Int), None]),
    ]
    .into_iter()
    .collect();
    let facts = checked.project_expression_types(groups, &HashMap::new());
    assert_eq!(
        facts.len(),
        2,
        "unknown variant must suppress the entire second span"
    );
    assert!(
        facts
            .iter()
            .all(|fact| fact.location.span == crate::Span::new(0, 8))
    );
    assert_ne!(
        facts[0].ty, facts[1].ty,
        "known conflicts must remain explicit"
    );
}

#[cfg(test)]
#[test]
fn project_generic_facts_do_not_select_a_known_result_over_an_abstract_variant() {
    struct Source;
    impl crate::project::SourceProvider for Source {
        fn resolve(
            &self,
            _: Option<&crate::project::SourceId>,
            _: &str,
        ) -> Result<crate::project::SourceId, String> {
            Ok(crate::project::SourceId("main.tsr".into()))
        }
        fn read(&self, _: &crate::project::SourceId) -> Result<String, String> {
            Ok("function identity<T>(value:T):T{return value;}function main(){identity(1);identity(1.5);}".into())
        }
    }
    let mut checked =
        crate::project::check_graph(crate::project::load_project(&Source, "main.tsr").unwrap())
            .unwrap();
    assert_eq!(
        checked.generic_calls().len(),
        2,
        "both independent checked calls retain metadata"
    );
    let main = checked
        .analysis
        .program
        .functions
        .iter_mut()
        .find(|function| function.name == "main")
        .expect("entry body");
    let first = &main.body[0];
    let crate::syntax::StmtKind::Expr(first) = &first.kind else {
        panic!("first call")
    };
    let crate::syntax::ExprKind::Call(callee, _) = &first.kind else {
        panic!("first callee")
    };
    let (span, callee_span) = (first.span, callee.span);
    let crate::syntax::StmtKind::Expr(second) = &mut main.body[1].kind else {
        panic!("second call")
    };
    // Model a shared source span with one unresolved callable specialization.
    // This metadata-only fixture never emits or executes the altered tree.
    second.span = span;
    second.ty = Some(Type::Callable(usize::MAX));
    let crate::syntax::ExprKind::Call(callee, _) = &mut second.kind else {
        panic!("second callee")
    };
    callee.span = callee_span;
    let facts = checked.checked_type_facts();
    assert!(
        facts.generic_calls.is_empty(),
        "a known legacy signature cannot hide the abstract result variant"
    );
    let location = checked.source_location(span).unwrap();
    assert!(
        !facts
            .method_results
            .iter()
            .any(|fact| fact.location == location)
    );
}
