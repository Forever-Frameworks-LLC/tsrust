//! Concrete callback environments give local record containers a shared layout.
//! Declaration-only layouts remain rejected until their environments are planned.
use super::*;

impl Analyzer {
    /// Array storage needs one concrete record environment for all elements.
    /// Preserve the nominal contract while joining each checked field carrier.
    pub(super) fn scoped_record_array_element(
        &mut self,
        values: &mut [Expr],
        element: ArrayElement,
        span: Span,
    ) -> Result<ArrayElement, Diagnostic> {
        let ArrayElement::Record(declaration, depth) = element else { return Ok(element); };
        if !self.aggregate_callable_facts(Type::Record(declaration)).scoped { return Ok(element); }
        let mut common = None;
        for value in values.iter() {
            let actual = match value.ty {
                Some(Type::Record(actual)) if depth == 0 => actual,
                Some(Type::Array(ArrayElement::Record(actual, nested))) if nested + 1 == depth => actual,
                _ => return Ok(element),
            };
            common = Some(match common {
                None => actual,
                Some(previous) if previous == actual => actual,
                Some(previous) => match self.join_record_types(previous, actual, span)? {
                    Some(Type::Record(joined)) => joined,
                    // Elements can differ in a nullable callback-record field only.
                    _ => match self.join_sticky_record_types(previous, actual, span)? {
                        Some(Type::Record(joined)) => joined,
                        _ => return Err(Diagnostic::new("type", "Array elements require compatible concrete callback field contracts.", value.span)),
                    },
                },
            });
        }
        let Some(common) = common else { return Ok(element); };
        Self::apply_scoped_array_carrier(values, common, depth);
        Ok(ArrayElement::Record(common, depth))
    }

    pub(super) fn apply_scoped_array_carrier(values: &mut [Expr], common: usize, depth: u8) {
        let mut pending = values.iter_mut().map(|value| (value, depth)).collect::<Vec<_>>();
        while let Some((value, depth)) = pending.pop() {
            if depth == 0 {
                if value.ty != Some(Type::Record(common)) { value.wrap_record_carrier(Type::Record(common)); }
            } else if !matches!(value.kind, ExprKind::Array(_)) {
                let target = Type::Array(ArrayElement::Record(common, depth - 1));
                if value.ty != Some(target) { value.wrap_record_carrier(target); }
            } else if let ExprKind::Array(elements) = &mut value.kind {
                value.ty = Some(Type::Array(ArrayElement::Record(common, depth - 1)));
                pending.extend(elements.iter_mut().map(|element| (element, depth - 1)));
            }
        }
    }

    fn holds_scoped_record(&self, container: Type) -> bool {
        let mut pending = match container {
            Type::Array(element) => vec![element.ty()],
            Type::Map(id) | Type::Set(id) => {
                let record = &self.program.records[id];
                record.map_definition.iter().flat_map(|map| [map.key, map.value])
                    .chain(record.set_definition.iter().map(|set| set.key))
                    .collect()
            }
            // A value that is either such a record or something else has no
            // single layout either.
            Type::Union(id) | Type::BorrowedUnion(id) => self.unions.members(id),
            _ => return false,
        };
        let mut seen = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) {
                continue;
            }
            match ty {
                Type::Record(id) => {
                    // Concrete instances retain checked callback environments.
                    // Declaration-only container layouts still have no native
                    // environment and must remain rejected.
                    if self.aggregate_callable_facts(ty).scoped
                        && self.scoped_records.declaration(id) == id
                    {
                        return true;
                    }
                }
                Type::Array(element) => pending.push(element.ty()),
                Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(self.unions.members(id)),
                _ => {}
            }
        }
        false
    }

    /// Callback contracts reachable from a type through records, nullable
    /// unions and arrays.
    fn reachable_callbacks(&self, ty: Type, found: &mut HashSet<usize>) {
        let mut pending = vec![ty];
        let mut seen = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) { continue; }
            match ty {
                Type::Callback(id) => { found.insert(id); }
                Type::Record(id) => pending.extend(self.program.records[id].fields.iter().filter_map(|field| field.ty)),
                Type::Array(element) => pending.push(element.ty()),
                Type::Map(id) => pending.extend(self.program.records[id].map_definition.as_ref().map(|map| map.value)),
                Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(self.unions.members(id)),
                _ => {}
            }
        }
    }
    /// A nullable callback record whose callbacks all come from the function's
    /// own parameters has a native layout: the function is generic over them.
    fn parameter_bound_nullable_record(&self, ty: Type, declared: &HashSet<usize>) -> bool {
        // An array of capturing records borrowed from the caller is generic in the same way.
        let array = matches!(ty, Type::Array(ArrayElement::Record(_, _)))
            || matches!(ty, Type::Array(ArrayElement::Union(id, _)) if self.nullable_scoped_record_declaration(Type::Union(id)).is_some())
            || matches!(ty, Type::Map(id) if self.program.records[id].map_definition.as_ref().is_some_and(|map| matches!(map.value, Type::Record(_)) && !matches!(map.key, Type::Record(_))));
        let nullable = matches!(ty, Type::Union(id) | Type::BorrowedUnion(id) if {
            let members = self.unions.members(id);
            members.iter().filter(|member| matches!(member, Type::Record(_))).count() == 1 && members.iter().all(|member| matches!(member, Type::Record(_) | Type::Null | Type::Undefined))
        });
        if !array && !nullable { return false; }
        let mut callbacks = HashSet::new();
        self.reachable_callbacks(ty, &mut callbacks);
        !callbacks.is_empty() && callbacks.is_subset(declared)
    }

    pub(super) fn scoped_record_containers(&self) -> Result<(), Diagnostic> {
        let mut found: Option<Span> = None;
        let mut declared = Vec::new();
        for (index, function) in self.checked.iter().enumerate() {
            let mut callbacks = HashSet::new();
            if let Some(function) = function.as_ref() {
                for binding in &function.bindings[..function.parameter_count] { self.reachable_callbacks(binding.ty, &mut callbacks); }
            }
            declared.push(callbacks);
            let Some(function) = function.as_ref().filter(|function| !function.template) else { continue };
            for binding in &function.bindings[..function.parameter_count] {
                if self.holds_scoped_record(binding.ty) && !self.parameter_bound_nullable_record(binding.ty, &declared[index]) {
                    let span = binding.declaration_span.unwrap_or(self.program.functions[index].name.span);
                    if found.is_none_or(|earlier| span.start < earlier.start) {
                        found = Some(span);
                    }
                }
            }
        }
        let none = HashSet::new();
        let bodies = self.checked.iter().enumerate().filter_map(|(index, function)| Some((function.as_ref()?, &declared[index]))).filter(|(function, _)| !function.template)
            .chain(self.closures.iter().map(|closure| (&closure.function, &none)).filter(|(function, _)| !function.template));
        for (function, declared) in bodies {
            crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
                // An argument viewed as a nullable parameter keeps its own concrete environment.
                let viewed_argument = matches!(&expr.kind, ExprKind::Inject { value, .. } if matches!(expr.ty, Some(Type::BorrowedUnion(_)))
                    && value.ty.is_some_and(|ty| !self.holds_scoped_record(ty) && self.nullable_scoped_record_declaration(ty.owning()).is_some()));
                if !viewed_argument && expr.ty.is_some_and(|ty| self.holds_scoped_record(ty) && !self.parameter_bound_nullable_record(ty, declared))
                    && found.is_none_or(|span| expr.span.start < span.start)
                {
                    found = Some(expr.span);
                }
            });
        }
        match found {
            Some(span) => Err(Diagnostic::new(
                "unsupported",
                "An array, Map, Set or union type cannot hold a record whose callback borrows local variables yet. Keep the record in a variable or in a field of another record.",
                span,
            )),
            None => Ok(()),
        }
    }
}
