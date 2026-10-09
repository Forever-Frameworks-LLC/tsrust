//! Nullable callback fields convert their present payload, not the whole union.
use super::*;

impl Emitter<'_> {
    pub(super) fn nullable_callback_replacement_bounds(&self, function: usize) -> Vec<String> {
        let mut bounds = Vec::new();
        let fun = &self.program.functions[function];
        for parameter in 0..fun.parameter_count {
            let Some(layout) = self.record_parameter_layout(fun, parameter) else {
                continue;
            };
            for leaf in &layout.leaves {
                if leaf.ty.union().is_none() {
                    continue;
                }
                let Some(Type::Callback(callback)) = self.callback_payload(leaf.ty) else {
                    continue;
                };
                let mut record = layout.record;
                let mut names = Vec::new();
                for (position, &index) in leaf.path.iter().enumerate() {
                    let field = &self.program.records[record].fields[index];
                    names.push(field.name.text.clone());
                    if position + 1 < leaf.path.len() {
                        let Some(Type::Record(child)) = field.ty else {
                            unreachable!("checked record path");
                        };
                        record = child;
                    }
                }
                let sources = self
                    .program
                    .record_parameter_replacements
                    .get(&function)
                    .into_iter()
                    .flatten()
                    .filter(|(id, path, _)| *id == parameter && *path == names)
                    .filter_map(|(_, _, source)| {
                        self.callback_payload(*source).or_else(|| {
                            matches!(source, Type::Closure(_) | Type::BorrowedCallable(_))
                                .then_some(*source)
                        })
                    })
                    .chain(
                        self.program
                            .record_parameter_borrows
                            .get(&function)
                            .into_iter()
                            .flatten()
                            .filter(|borrow| {
                                borrow.target.binding == parameter && borrow.target.fields == names
                            })
                            .map(|borrow| Type::Closure(borrow.origin)),
                    );
                for payload in sources {
                    if !matches!(payload, Type::Closure(_) | Type::BorrowedCallable(_)) {
                        continue;
                    }
                    for source in
                        self.record_borrow_replacement_types(function, payload, parameter, &names)
                    {
                        let bound = format!("C{callback}: From<{source}>");
                        if !bounds.contains(&bound) {
                            bounds.push(bound);
                        }
                    }
                }
            }
        }
        bounds
    }

    pub(super) fn nullable_callback_parameter_store(
        &self,
        fun: &CheckedFunction,
        target: Type,
        value: &Expr,
        code: &str,
    ) -> Option<String> {
        let Type::Union(destination) = target else {
            return None;
        };
        let Some(Type::Callback(callback)) = self.callback_payload(target) else {
            return None;
        };
        let ExprKind::Inject { value: inner, .. } = &value.kind else {
            return None;
        };
        let source = value.ty?.union()?;
        let targets = &self.program.unions[destination.index()].members;
        let absent = matches!(inner.ty, Some(Type::Null | Type::Undefined));
        let arms = self.program.unions[source.index()]
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let scalar = matches!(member, Type::Null | Type::Undefined);
                let selected = targets
                    .iter()
                    .position(|candidate| {
                        if scalar {
                            candidate == member
                        } else {
                            !matches!(candidate, Type::Null | Type::Undefined)
                        }
                    })
                    .expect("checked nullable callback shape");
                let payload = if scalar {
                    "payload".to_owned()
                } else if absent {
                    "unreachable!(\"absent callback payload\")".to_owned()
                } else {
                    let payload = if let Some(Type::Closure(origin)) = inner.ty {
                        self.tag_owner_constructor(fun, origin, "payload")
                    } else {
                        "payload".to_owned()
                    };
                    if let Some(Type::BorrowedCallable(family)) = inner.ty {
                        self.local_borrowed_family_store(
                            fun,
                            family,
                            &format!("C{callback}"),
                            &payload,
                        )
                        .unwrap_or_else(|| format!("C{callback}::from({payload})"))
                    } else {
                        format!("C{callback}::from({payload})")
                    }
                };
                format!(
                    "Union{}::V{index}(payload)=>Union{}::<C{callback}>::V{selected}({payload})",
                    source.index(),
                    destination.index()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        Some(format!("match ({code}) {{{arms}}}"))
    }
}
