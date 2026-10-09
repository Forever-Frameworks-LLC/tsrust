//! Closure record signatures use explicit argument bindings and retained lifetimes.
use super::native_function_signature_emit::{NativeSignature, NativeSignaturePlan};
use super::*;

pub(super) struct ClosureRecordSignature {
    pub(super) signature: NativeSignature,
    pub(super) trait_where: String,
    pub(super) receiver_lifetime: Option<&'static str>,
}

impl Emitter<'_> {
    pub(super) fn closure_record_signature(&self, index: usize) -> Option<ClosureRecordSignature> {
        let closure = &self.program.closures[index];
        if closure.forward.is_some() {
            return None;
        }
        let mut borrows = self
            .program
            .closure_record_parameter_borrows
            .get(&index)
            .cloned()
            .unwrap_or_default();
        if borrows.is_empty()
            && !self
                .program
                .closure_record_capture_borrows
                .contains_key(&index)
        {
            return None;
        }
        // Projected environments require their own native lifetime and owner interface.
        if borrows
            .iter()
            .any(|borrow| !borrow.source_projection.is_direct())
            || self
                .program
                .closure_record_capture_borrows
                .get(&index)
                .into_iter()
                .flatten()
                .any(|borrow| !borrow.source_projection.is_direct())
        {
            return None;
        }
        let mut sources = closure.parameters.clone();
        let (capture_lifetimes, capture_types) = self.capture_layout(closure);
        let mut hidden_lifetimes = HashMap::new();
        let mut receiver_lifetime = None;
        for borrow in self
            .program
            .closure_record_capture_borrows
            .get(&index)
            .into_iter()
            .flatten()
        {
            let capture = &closure.captures[borrow.source_capture];
            let field = &capture_types[borrow.source_capture];
            let lifetime = capture_lifetimes.iter().find(|lifetime| {
                field
                    .split(|ch: char| !ch.is_alphanumeric() && ch != '\'' && ch != '_')
                    .any(|word| word == lifetime.as_str())
            })?;
            // Owned capture storage needs a receiver lifetime, not an input lifetime.
            if !matches!(capture.storage, Mode::Read | Mode::Mutate) {
                return None;
            }
            let lifetime = if capture.storage == Mode::Mutate {
                receiver_lifetime = Some("'record_receiver");
                "'record_receiver".to_owned()
            } else {
                lifetime.clone()
            };
            hidden_lifetimes.insert(capture.inner, lifetime);
            let source = sources.len();
            sources.push(capture.inner);
            borrows.push(crate::analysis::record_parameter_borrows::Borrow {
                target: borrow.target.clone(),
                source,
                source_projection: borrow.source_projection.clone(),
                mode: borrow.mode,
                origin: borrow.origin,
                capture: borrow.capture,
            });
        }
        if borrows.is_empty() {
            return None;
        }
        let mut groups = std::collections::BTreeMap::new();
        let mut slots = std::collections::BTreeMap::new();
        for borrow in &borrows {
            let source = sources[borrow.source];
            groups.insert(source, source);
        }
        for borrow in &borrows {
            let source = sources[borrow.source];
            let key = (borrow.target.clone(), borrow.origin, borrow.capture);
            if let Some(previous) = slots.insert(key, source) {
                let left = groups[&previous];
                let right = groups[&source];
                let common = left.min(right);
                for group in groups.values_mut() {
                    if *group == left || *group == right {
                        *group = common;
                    }
                }
            }
        }
        let replacements = |parameter: usize, path: &[usize]| -> Vec<String> {
            let Some(layout) = self.record_parameter_layout(&closure.function, parameter) else {
                return Vec::new();
            };
            let mut record = layout.record;
            let mut names = Vec::new();
            for (position, &field) in path.iter().enumerate() {
                let selected = &self.program.records[record].fields[field];
                names.push(selected.name.text.clone());
                if position + 1 < path.len() {
                    let Some(Type::Record(child)) = selected.ty else {
                        return Vec::new();
                    };
                    record = child;
                }
            }
            let mut result = Vec::new();
            for borrow in borrows.iter().filter(|borrow| {
                closure.parameters[borrow.target.binding] == parameter
                    && borrow.target.fields == names
            }) {
                let installed = &self.program.closures[borrow.origin];
                let (mut lifetimes, fields) = self.capture_layout(installed);
                for source in borrows.iter().filter(|candidate| {
                    candidate.origin == borrow.origin && candidate.target == borrow.target
                }) {
                    let field = &fields[source.capture];
                    for (slot, lifetime) in lifetimes.iter_mut().enumerate() {
                        let token = format!("'c{slot}");
                        if field
                            .split(|ch: char| !ch.is_alphanumeric() && ch != '\'' && ch != '_')
                            .any(|word| word == token)
                        {
                            *lifetime = hidden_lifetimes
                                .get(&sources[source.source])
                                .cloned()
                                .unwrap_or_else(|| {
                                    format!("'record_input{}", groups[&sources[source.source]])
                                });
                        }
                    }
                }
                let rendered =
                    self.named_record_environment_type(Type::Closure(borrow.origin), &lifetimes);
                let mut substitutions = HashMap::new();
                for source in borrows.iter().filter(|candidate| {
                    candidate.origin == borrow.origin && candidate.target == borrow.target
                }) {
                    let capture = &installed.captures[source.capture];
                    if matches!(
                        installed.function.bindings[capture.inner].ty,
                        Type::Callback(_)
                    ) {
                        substitutions.insert(
                            format!("K{}", capture.inner),
                            format!("F{}", sources[source.source]),
                        );
                    }
                }
                let rendered =
                    callback_capture_emit::substitute_parameters(&rendered, &substitutions);
                let rendered = if self.borrowed_owners.contains_key(&borrow.origin) {
                    let markers = installed
                        .captures
                        .iter()
                        .filter(|capture| matches!(capture.storage, Mode::Read | Mode::Mutate))
                        .map(|capture| {
                            let source = borrows
                                .iter()
                                .find(|source| {
                                    source.origin == borrow.origin
                                        && source.target == borrow.target
                                        && installed.captures[source.capture].inner == capture.inner
                                })
                                .expect("retained closure owner input");
                            format!("O{}", sources[source.source])
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    format!("TsrOwnedCapture<{rendered},({markers},)>")
                } else {
                    rendered
                };
                result.push(format!("From<{rendered}>"));
            }
            result.sort();
            result.dedup();
            result
        };
        let mut record_bounds = Vec::new();
        let mut replacement_bounds = Vec::new();
        let mut trait_bounds = Vec::new();
        for &parameter in &closure.parameters {
            if let Some(bound) =
                self.record_parameter_bound_with(&closure.function, parameter, |path| {
                    replacements(parameter, path)
                })
            {
                record_bounds.push(bound);
            }
            if let Some(bound) = self.nullable_record_parameter_bound(&closure.function, parameter)
            {
                record_bounds.push(bound);
            }
            if let Some(layout) = self.record_parameter_layout(&closure.function, parameter) {
                for leaf in &layout.leaves {
                    if let Some(Type::Callback(callback)) =
                        self.callback_payload(leaf.ty).or(Some(leaf.ty))
                    {
                        let bounds = replacements(parameter, &leaf.path)
                            .into_iter()
                            .map(|bound| format!("C{callback}: {bound}"))
                            .collect::<Vec<_>>();
                        trait_bounds.extend(bounds.clone());
                        if leaf.ty.union().is_some() {
                            replacement_bounds.extend(bounds);
                        }
                    }
                }
            }
        }
        replacement_bounds.sort();
        replacement_bounds.dedup();
        for bound in &mut trait_bounds {
            let mut substitutions = groups
                .iter()
                .filter(|(parameter, _)| !hidden_lifetimes.contains_key(parameter))
                .map(|(&parameter, &group)| {
                    let slot = closure
                        .parameters
                        .iter()
                        .position(|id| *id == parameter)
                        .unwrap();
                    (format!("'record_input{group}"), format!("'p{slot}"))
                })
                .collect::<Vec<_>>();
            substitutions.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
            substitutions.dedup_by(|a, b| a.0 == b.0);
            for (source, target) in substitutions {
                *bound = bound.replace(&source, &target);
            }
        }
        for (&parameter, &group) in &groups {
            if parameter != group
                && !hidden_lifetimes.contains_key(&parameter)
                && !hidden_lifetimes.contains_key(&group)
            {
                let source = closure
                    .parameters
                    .iter()
                    .position(|id| *id == parameter)
                    .unwrap();
                let target = closure
                    .parameters
                    .iter()
                    .position(|id| *id == group)
                    .unwrap();
                trait_bounds.push(format!("'p{source}: 'p{target}"));
            }
        }
        let owner_inputs = self.closure_owner_inputs(index);
        let owners = owner_inputs
            .iter()
            .map(|source| (format!("O{source}"), "()".to_owned()))
            .collect();
        for bound in &mut trait_bounds {
            *bound = callback_capture_emit::substitute_parameters(bound, &owners);
        }
        for bound in &mut trait_bounds {
            if bound.contains("'record_receiver") {
                *bound = format!("for<'record_receiver> {bound}");
            }
        }
        trait_bounds.sort();
        trait_bounds.dedup();
        let trait_where = if trait_bounds.is_empty() {
            String::new()
        } else {
            format!(" where {}", trait_bounds.join(", "))
        };
        let mut signature = self.native_signature_from_plan(
            NativeSignaturePlan {
                function: &closure.function,
                parameters: closure.parameters.clone(),
                source_parameters: closure.parameters.clone(),
                borrow_lifetimes: groups
                    .into_iter()
                    .filter(|(binding, _)| !hidden_lifetimes.contains_key(binding))
                    .collect(),
                owner_parameters: owner_inputs
                    .iter()
                    .map(|source| format!(",_owner{source}:std::marker::PhantomData<O{source}>"))
                    .collect(),
                owner_inputs,
                record_bounds,
                replacement_bounds,
            },
            &[],
        );
        if let Some(lifetime) = receiver_lifetime {
            signature.generics = if signature.generics.is_empty() {
                format!("<{lifetime}>")
            } else {
                format!("<{lifetime},{}", &signature.generics[1..])
            };
        }
        Some(ClosureRecordSignature {
            signature,
            trait_where,
            receiver_lifetime,
        })
    }
}
