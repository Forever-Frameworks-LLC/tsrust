//! A record whose callback borrows local variables lives in a local variable,
//! a parameter or a field of such a record. Arrays, Maps, Sets and unions have
//! one element layout and cannot yet hold these records.
use super::*;

impl Analyzer {
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
                Type::Record(_) => {
                    if self.aggregate_callable_facts(ty).scoped {
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

    pub(super) fn scoped_record_containers(&self) -> Result<(), Diagnostic> {
        let mut found: Option<Span> = None;
        for (index, function) in self.checked.iter().enumerate() {
            let Some(function) = function.as_ref().filter(|function| !function.template) else { continue };
            for binding in &function.bindings[..function.parameter_count] {
                if self.holds_scoped_record(binding.ty) {
                    let span = binding.declaration_span.unwrap_or(self.program.functions[index].name.span);
                    if found.is_none_or(|earlier| span.start < earlier.start) {
                        found = Some(span);
                    }
                }
            }
        }
        let bodies = self.checked.iter().flatten().filter(|function| !function.template)
            .chain(self.closures.iter().map(|closure| &closure.function).filter(|function| !function.template));
        for function in bodies {
            crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
                if expr.ty.is_some_and(|ty| self.holds_scoped_record(ty))
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
