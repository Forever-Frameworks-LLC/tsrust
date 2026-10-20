//! Consume successful project facts; nominal aliases resolve by source declaration.
use super::*;
use tsrust_compiler::project::{
    ProjectCheckedTypeFacts, ProjectGenericCallInfo, ProjectGenericParameterCallInfo, ProjectGraph, ProjectType, SourceId,
};

impl SourceIndex {
    pub(super) fn project_type_spelling(
        &mut self,
        graph: &ProjectGraph,
        current: &SourceId,
        ty: &ProjectType,
        dependencies: &std::collections::BTreeMap<SourceId, SourceIndex>,
    ) -> Option<String> {
        match ty {
            ProjectType::SchemaPipeline { identity, input, output, json_input, text_input, object_policy, scalar_constraints } => {
                let input = self.project_type_spelling(graph, current, input, dependencies)?;
                let output = self.project_type_spelling(graph, current, output, dependencies)?;
                let source = current.0.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                let owner = format!("__tsrust_private_pipeline_{source}_{identity}");
                self.schema_pipeline_types.insert(owner.clone(), (input, output.clone(), *object_policy, *json_input, *text_input, *scalar_constraints));
                let schema = self.native_type_spelling(tsrust_compiler::native_signatures::NativeTypeId::Schema);
                self.generic_display_names.insert(owner.clone(), format!("{schema}<{}>", self.display_type(&output)));
                Some(owner)
            }
            ProjectType::Native(identity) => Some(self.native_type_spelling(*identity)),
            ProjectType::AppliedNative(identity, arguments) => {
                let name = self.native_type_spelling(*identity);
                let arguments = arguments.iter().map(|argument| self.project_type_spelling(graph, current, argument, dependencies)).collect::<Option<Vec<_>>>()?;
                Some(format!("{name}<{}>", arguments.join(", ")))
            },
            ProjectType::Callable {
                ownership,
                parameters,
                result,
            } => {
                use tsrust_compiler::type_syntax::CallableOwnership;
                let prefix = match ownership {
                    CallableOwnership::Task => Some("Task"),
                    CallableOwnership::Default => None,
                    CallableOwnership::Read => Some("ReadOnly"),
                    CallableOwnership::ProgramReusable | CallableOwnership::Reusable => Some("Mutable"),
                    CallableOwnership::Once => Some("Once"),
                    CallableOwnership::Program => None,
                    CallableOwnership::Owned => Some("Owned"),
                    CallableOwnership::OwnedOnce => Some("Once"),
                    CallableOwnership::SharedRead => Some("SharedRead"),
                };
                let parameters = parameters
                    .iter()
                    .enumerate()
                    .map(|(index, parameter)| {
                        let ty = parameter.ty.as_ref()?;
                        let access = if parameter.rest || project_callable_value(ty) {
                            tsrust_compiler::Mode::Copy
                        } else {
                            parameter.access
                        };
                        let required_after = parameters[index + 1..].iter().any(|parameter| {
                            !parameter.rest
                                && parameter.omission
                                    == tsrust_compiler::ParameterOmission::Required
                        });
                        let mut supplied =
                            self.project_type_spelling(graph, current, ty, dependencies)?;
                        if parameter.omission == tsrust_compiler::ParameterOmission::Defaulted
                            && required_after
                        {
                            supplied = super::optional_parameter_type(&supplied);
                        }
                        let supplied = tsrust_compiler::type_syntax::access_type_text(access, &supplied);
                        Some(format!(
                            "{}{}{}: {supplied}",
                            if parameter.rest { "..." } else { "" },
                            parameter.name,
                            if !parameter.rest
                                && parameter.omission
                                    != tsrust_compiler::ParameterOmission::Required
                                && !required_after
                            {
                                "?"
                            } else {
                                ""
                            },
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(tsrust_compiler::type_syntax::function_type_text(
                    prefix,
                    &parameters,
                    &self.project_type_spelling(graph, current, result, dependencies)?,
                ))
            }
            ProjectType::StructuralReference { identity, retained } => {
                let source=current.0.as_bytes().iter().map(|byte|format!("{byte:02x}")).collect::<String>();
                let flavor=if *retained {"retained_"}else{""};
                Some(format!("__tsrust_private_schema_{flavor}{source}_{identity}"))
            }
            ProjectType::RecursiveReference { identity } => {
                let source=current.0.as_bytes().iter().map(|byte|format!("{byte:02x}")).collect::<String>();
                Some(format!("__tsrust_private_recursive_{source}_{identity}"))
            }
            ProjectType::Recursive { identity, body } => {
                let source=current.0.as_bytes().iter().map(|byte|format!("{byte:02x}")).collect::<String>();
                let owner=format!("__tsrust_private_recursive_{source}_{identity}");
                // Register the back edge before visiting fields. Hover uses a finite
                // self label; access expands the alias only at the requested hop.
                self.generic_display_names.insert(owner.clone(), "self".into());
                let body=self.project_type_spelling(graph,current,body,dependencies)?;
                self.type_aliases.insert(owner, body.clone());
                Some(body)
            }
            ProjectType::Structural { identity, retained, fields } => {
                let source=current.0.as_bytes().iter().map(|byte|format!("{byte:02x}")).collect::<String>();
                let flavor=if *retained {"retained_"}else{""};
                let owner=format!("__tsrust_private_schema_{flavor}{source}_{identity}");
                self.generic_display_names.insert(owner.clone(), "self".into());
                let fields=fields.iter().map(|(name,optional,ty)|Some((name.clone(),*optional,self.project_type_spelling(graph,current,ty,dependencies)?))).collect::<Option<Vec<_>>>()?;
                let display=format!("{{ {} }}",fields.iter().map(|(name,optional,ty)|format!("{name}{}: {};",if *optional {"?"}else{""},self.display_type(ty))).collect::<Vec<_>>().join(" "));
                self.generic_display_names.insert(owner.clone(),display);
                if !self.symbols.iter().any(|symbol|symbol.owner.as_ref()==Some(&owner)) {
                    for (name,optional,ty) in fields {
                        self.symbols.push(Symbol { signature:format!("{name}{}: {ty}",if optional {"?"}else{""}),name,kind:8,span:Span::new(0,0),extent:Span::new(0,0),scope:0,ty,docs:"Inferred from checked object fields.".into(),parameters:vec![],owner:Some(owner.clone()) });
                    }
                }
                Some(owner)
            }
            ProjectType::Array(inner) => {
                let spelling = self.project_type_spelling(graph, current, inner, dependencies)?;
                Some(if matches!(inner.as_ref(), ProjectType::Callable { .. } | ProjectType::Union(_) | ProjectType::Recursive { .. }) {
                    format!("({spelling})[]")
                } else {
                    format!("{spelling}[]")
                })
            }
            ProjectType::Tuple(positions) => Some(format!("[{}]", positions.iter().map(|position|self.project_type_spelling(graph,current,position,dependencies)).collect::<Option<Vec<_>>>()?.join(","))),
            ProjectType::Union(members) => Some(
                members
                    .iter()
                    .map(|member| {
                        let spelling =
                            self.project_type_spelling(graph, current, member, dependencies)?;
                        Some(if matches!(member, ProjectType::Callable { .. }) {
                            format!("({spelling})")
                        } else {
                            spelling
                        })
                    })
                    .collect::<Option<Vec<_>>>()?
                    .join(" | "),
            ),
            ProjectType::Variant { name, payload } => Some(format!("{name}({})", self.project_type_spelling(graph, current, payload, dependencies)?)),
            ProjectType::Named {
                name,
                declaration,
                arguments,
            } => {
                let base = if let Some(declaration) = declaration {
                    let module=&graph.modules[&declaration.source];
                    let source=module.checked_source();
                    let uri = crate::project_diagnostics::file_uri(&declaration.source).ok()?;
                    let range = LineIndex::new(source)
                        .range(source, declaration.span)
                        .ok()?;
                    let location = json!({"uri":uri,"range":range});
                    let visible = self
                        .symbols
                        .iter()
                        .enumerate()
                        .find(|(id, symbol)| {
                            matches!(symbol.kind, 10 | 23)
                                && symbol.owner.is_none()
                                && ((current == &declaration.source
                                    && symbol.span == declaration.span)
                                    || self.foreign_definitions.get(id) == Some(&location))
                        })
                        .map(|(_, symbol)| symbol.name.clone());
                    if let Some(visible) = visible {
                        visible
                    } else if let Some((identity, _)) = self
                        .nominal_definitions
                        .iter()
                        .find(|(_, origin)| **origin == location)
                    {
                        identity.clone()
                    } else {
                        let dependency = dependencies.get(&declaration.source)?;
                        let original = dependency
                            .symbols
                            .iter()
                            .find(|symbol| matches!(symbol.kind, 10 | 23) && symbol.span == declaration.span)?;
                        let encoded = uri
                            .as_str()
                            .as_bytes()
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>();
                        let identity = format!("__tsrust_private_{encoded}_{}", original.name);
                        self.generic_display_names
                            .insert(identity.clone(), name.clone());
                        self.import_record_fields(
                            dependency,
                            &original.name,
                            &identity,
                            Span::new(0, 0),
                            uri.as_str(),
                            &Default::default(),
                        );
                        identity
                    }
                } else {
                    name.clone()
                };
                if (base.starts_with("__tsrust_private_")||base.starts_with('@')) && let Some(declaration)=declaration && let Some(dependency)=dependencies.get(&declaration.source) && let Some(alias)=dependency.public_record_alias(name){self.generic_display_names.insert(base.clone(),alias);}
                if arguments.is_empty() {
                    return Some(base);
                }
                let arguments = arguments
                    .iter()
                    .map(|argument| {
                        self.project_type_spelling(graph, current, argument, dependencies)
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(canonical_data_type(&format!(
                    "{base}<{}>",
                    arguments.join(",")
                )))
            }
        }
    }
    pub(crate) fn apply_project_checked_facts(
        &mut self,
        graph: &ProjectGraph,
        current: &SourceId,
        facts: &ProjectCheckedTypeFacts,
        dependencies: &std::collections::BTreeMap<SourceId, SourceIndex>,
    ) {
        for (owner,name,location,ty,optional) in &facts.record_fields {
            let Some(owner)=self.project_type_spelling(graph,current,owner,dependencies) else{continue;};
            let Some(ty)=self.project_type_spelling(graph,current,ty,dependencies) else{continue;};
            let signature=format!("{name}{}: {}",if *optional{"?"}else{""},self.display_type(&ty));
            let template=owner.split('<').next().unwrap_or(&owner);let docs=self.symbols.iter().find(|symbol|symbol.kind==8&&symbol.owner.as_deref()==Some(template)&&symbol.name==*name).map(|symbol|symbol.docs.clone()).unwrap_or_default();
            if let Some(symbol)=self.symbols.iter_mut().find(|symbol|symbol.kind==8 && symbol.owner.as_deref()==Some(owner.as_str()) && symbol.name==*name){symbol.ty=ty;symbol.signature=signature;if symbol.docs.is_empty(){symbol.docs=docs;}}
            else {
                let span=if location.source==*current{location.span}else{Span::new(0,0)};
                let id=self.symbols.len();
                self.symbols.push(Symbol{name:name.clone(),kind:8,span,extent:span,scope:0,ty,signature,docs,parameters:Vec::new(),owner:Some(owner)});
                if location.source!=*current && let Ok(uri)=crate::project_diagnostics::file_uri(&location.source){
                    let source=graph.modules[&location.source].checked_source();
                    if let Ok(range)=LineIndex::new(source).range(source,location.span){self.foreign_definitions.insert(id,json!({"uri":uri,"range":range}));}
                }
            }
        }
        for (ty,base) in &facts.enum_scalars {if let Some(name)=self.project_type_spelling(graph,current,ty,dependencies){self.enum_scalar_types.insert(name,base.clone());}}
        self.apply_project_generic_facts(graph, current, &facts.generic_calls, &facts.generic_parameter_calls, dependencies);
        for (location, declaration) in facts.generic_values.iter().filter(|(location, _)| location.source == *current) {
            if facts.generic_values.iter().any(|(other, target)| other == location && target != declaration) { continue; }
            let source_index = if declaration.source == *current { &*self } else {
                let Some(index) = dependencies.get(&declaration.source) else { continue; }; index
            };
            let Some(mut source) = source_index.symbols.iter().find(|symbol| symbol.span == declaration.span && symbol.kind == 12).cloned() else { continue; };
            let header_start = source_index.tokens.iter().position(|token| token.span == declaration.span)
                .and_then(|id| id.checked_sub(1)).map(|id| source_index.tokens[id].span.start);
            if let Some(header_start) = header_start {
                let original = source.signature.clone();
                let mut edits = Vec::new();
                for (use_site, target) in &facts.generic_nominals {
                    if use_site.source != declaration.source || use_site.span.start < header_start
                        || use_site.span.end > header_start + original.len() { continue; }
                    let module=&graph.modules[&target.source];
                    let text=module.checked_source();
                    let name = text[target.span.start..target.span.end].to_owned();
                    let ty = ProjectType::Named { name, declaration: Some(target.clone()), arguments: Vec::new() };
                    if let Some(spelling) = self.project_type_spelling(graph, current, &ty, dependencies) {
                        edits.push((use_site.span.start - header_start, use_site.span.end - header_start, self.display_type(&spelling)));
                    }
                }
                edits.sort_by_key(|edit| edit.0);
                edits.dedup();
                let mut parameter_start = 0;
                for parameter in &mut source.parameters {
                    if let Some(offset) = original[parameter_start..].find(parameter.as_str()) {
                        let start = parameter_start + offset;
                        let stop = start + parameter.len();
                        parameter_start = stop;
                        for (begin, end, spelling) in edits.iter().rev().filter(|(begin,end,_)| *begin >= start && *end <= stop) {
                            parameter.replace_range(begin-start..end-start, spelling);
                        }
                    }
                }
                for (start,end,spelling) in edits.into_iter().rev() { source.signature.replace_range(start..end,&spelling); }
            }
            for symbol in self.symbols.iter_mut().filter(|symbol| symbol.span == location.span) {
                symbol.kind = 12;
                symbol.signature = source.signature.replacen(&format!("function {}", source.name), &format!("function {}", symbol.name), 1);
                symbol.parameters = source.parameters.clone();
                symbol.docs = source.docs.clone();
            }
        }
        self.json_string_receivers=facts.json_string_receivers.iter().filter(|location|location.source==*current).map(|location|location.span).collect();
        let mut views = std::collections::BTreeMap::<(usize, usize), Option<String>>::new();
        let mut optional = std::collections::BTreeMap::<(usize, usize), Option<String>>::new();
        // Successful project metadata is authoritative, including suppressed
        // unknown specialization groups. Do not retain provisional local facts.
        let mut results = std::collections::BTreeMap::<(usize, usize), Option<String>>::new();
        let mut parameter_types =
            std::collections::BTreeMap::<(usize, usize), Option<String>>::new();
        for (kind, group) in [
            (0, &facts.projections),
            (1, &facts.optional_receivers),
            (2, &facts.method_results),
            (3, &facts.parameter_types),
        ] {
            for fact in group.iter().filter(|fact| fact.location.source == *current) {
                let span = fact.location.span;
                let spelling = self.project_type_spelling(graph, current, &fact.ty, dependencies);
                let target = match kind {
                    0 => &mut views,
                    1 => &mut optional,
                    3 => &mut parameter_types,
                    _ => &mut results,
                };
                target
                    .entry((span.start, span.end))
                    .and_modify(|existing| {
                        if *existing != spelling {
                            *existing = None;
                        }
                    })
                    .or_insert(spelling);
            }
        }
        let finish = |map: std::collections::BTreeMap<(usize, usize), Option<String>>| {
            map.into_iter()
                .filter_map(|((start, end), ty)| ty.map(|ty| (Span::new(start, end), ty)))
                .collect()
        };
        self.apply_checked_parameter_types(parameter_types);
        self.checked_views = finish(views);
        self.checked_optional_receivers = finish(optional);
        self.checked_method_results = finish(results);
        self.refresh_destructured_names();
        self.refresh_result_match_bindings();
        self.refresh_checked_initializers();
        self.apply_module_value_facts(graph, current, facts, dependencies);
        self.materialize_generic_fields();
        self.recover_result_payloads();
    }

    /// Use the owning declaration's checked type for both local and imported
    /// bindings. Do not infer imported schema values from a textual type label.
    fn apply_module_value_facts(
        &mut self,
        graph: &ProjectGraph,
        current: &SourceId,
        facts: &ProjectCheckedTypeFacts,
        dependencies: &std::collections::BTreeMap<SourceId, SourceIndex>,
    ) {
        for fact in &facts.module_values {
            let mut locations = Vec::new();
            if fact.location.source == *current { locations.push(fact.location.span); }
            for import in &graph.modules[current].imports {
                if import.target.as_ref() != Some(&fact.location.source) { continue; }
                let dependency = &graph.modules[&fact.location.source];
                for binding in &import.bindings {
                    let declared_name = dependency.source.text.get(fact.location.span.start..fact.location.span.end);
                    if declared_name == Some(dependency.exported_local(&binding.exported)) {
                        locations.push(binding.span);
                    }
                }
            }
            if locations.is_empty() { continue; }
            let Some(ty) = self.project_type_spelling(graph, current, &fact.ty, dependencies) else { continue; };
            let display = self.display_type(&ty);
            for symbol in self.symbols.iter_mut().filter(|symbol| symbol.kind == 13 && locations.contains(&symbol.span)) {
                let keyword = if symbol.signature.starts_with("let ") { "let" } else { "const" };
                symbol.signature = format!("{keyword} {}: {display}", symbol.name);
                symbol.ty = ty.clone();
            }
        }
    }

    pub(crate) fn apply_project_generic_facts(
        &mut self,
        graph: &ProjectGraph,
        current: &SourceId,
        facts: &[ProjectGenericCallInfo],
        partial: &[ProjectGenericParameterCallInfo],
        dependencies: &std::collections::BTreeMap<SourceId, SourceIndex>,
    ) {
        let mut calls =
            std::collections::BTreeMap::<(usize, usize), Option<CheckedGenericCall>>::new();
        let mut results = std::collections::BTreeMap::<(usize, usize), Option<String>>::new();
        for (location, callee, declaration, parameters, result) in facts.iter()
            .map(|fact| (&fact.location, &fact.callee, &fact.declaration, &fact.parameters, Some(&fact.result)))
            .chain(partial.iter().map(|fact| (&fact.location, &fact.callee, &fact.declaration, &fact.parameters, None)))
            .filter(|(location, callee, _, _, _)| location.source == *current && callee.source == *current)
        {
            let result = result.and_then(|ty| self.project_type_spelling(graph, current, ty, dependencies));
            let parameters = parameters
                .iter()
                .map(|parameter| tsrust_compiler::GenericParameterInfo {
                    name: parameter.name.clone(),
                    access: parameter.access,
                    omission: parameter.omission,
                    rest: parameter.rest,
                    body_ty: parameter.body_ty.as_ref().and_then(|ty| {
                        self.project_type_spelling(graph, current, ty, dependencies)
                    }),
                    ty: parameter.ty.as_ref().and_then(|ty| {
                        self.project_type_spelling(graph, current, ty, dependencies)
                    }),
                })
                .collect();
            let call = CheckedGenericCall {
                span: location.span,
                callee_span: callee.span,
                declaration_span: declaration.span,
                parameters,
                result: result.clone(),
            };
            let call_span = call.span;
            calls
                .entry((call.callee_span.start, call.callee_span.end))
                .and_modify(|existing| {
                    if existing.as_ref() != Some(&call) {
                        *existing = None;
                    }
                })
                .or_insert(Some(call));
            if let Some(result) = result {
                results
                    .entry((call_span.start, call_span.end))
                    .and_modify(|existing| {
                        if existing.as_ref() != Some(&result) {
                            *existing = None;
                        }
                    })
                    .or_insert(Some(result));
            }
        }
        self.checked_generic_calls = calls.into_values().flatten().collect();
        self.checked_method_results.extend(
            results
                .into_iter()
                .filter_map(|((start, end), ty)| ty.map(|ty| (Span::new(start, end), ty))),
        );
        self.refresh_checked_initializers();
        self.materialize_generic_fields();
    }
}

// Access is carried by the callable contract, including nullable transports.
fn project_callable_value(ty: &ProjectType) -> bool {
    match ty {
        ProjectType::Callable { .. } => true,
        ProjectType::Union(members) => members.iter().any(project_callable_value),
        _ => false,
    }
}


impl SourceIndex {
    pub(crate) fn recover_project_member_receivers(&mut self,graph:&ProjectGraph,id:&SourceId) {
        let Some((probe,start,added,original_end))=self.receiver_probe().or_else(||self.unfinished_member_probe()) else{return};
        let check = |probe: String| {
            let mut graph=graph.clone();let module=graph.modules.get_mut(id)?;
            // Keep import masking and source positions from this snapshot.
            module.declaration_source=format!("{}{}",&module.declaration_source[..start],&probe[start..]);
            module.source.text=probe;
            tsrust_compiler::project::check_graph(graph).ok()
        };
        let analysis=check(probe.clone()).or_else(|| {
            // An unrelated broken declaration after the cursor must not erase
            // facts from a complete prefix. Missing dependencies still fail checking.
            let end=original_end.checked_add(added+1)?;
            let mut prefix=probe.get(..end)?.to_owned();
            let mut blocks=0usize;
            for token in self.tokens.iter().take_while(|token|token.span.end<=original_end) {
                if token.text=="{" {blocks+=1;} else if token.text=="}" {blocks=blocks.checked_sub(1)?;}
            }
            prefix.extend(std::iter::repeat_n('}',blocks));
            check(prefix)
        });
        let Some(analysis)=analysis else{return};
        let facts=analysis.checked_type_facts();
        // Only use facts preceding the edit: their source positions and complete
        // declarations are unchanged. Do not expose invented probe expressions.
        let mut preceding=tsrust_compiler::project::ProjectCheckedTypeFacts::default();
        let before=|fact:&&tsrust_compiler::project::ProjectExpressionTypeInfo|fact.location.source==*id && fact.location.span.end<=start;
        preceding.record_fields=facts.record_fields.iter().filter(|(_,_,location,_,_)|location.source!=*id || location.span.end<=start).cloned().collect();
        preceding.module_values=facts.module_values.iter().filter(before).cloned().collect();
        preceding.method_results=facts.method_results.iter().filter(before).cloned().collect();
        preceding.parameter_types=facts.parameter_types.iter().filter(before).cloned().collect();
        preceding.projections=facts.projections.iter().filter(before).cloned().collect();
        self.apply_project_checked_facts(&analysis.graph,id,&preceding,&Default::default());
        self.recover_result_payloads();
        self.json_string_receivers=facts.json_string_receivers.into_iter().filter_map(|location|{
            if location.source!=*id{return None;}let mut span=location.span;
            if span.end<=start{}else if span.start>=start+added && span.end<=original_end+added{span.start-=added;span.end-=added;}else{return None;}
            Some(span)
        }).collect();
    }
}
