//! Final native capture storage is independent of point-sensitive source types.
use super::*;

pub(super) struct Storage {
    types: HashMap<(*const CheckedFunction, usize), Type>,
    pub(super) lifetimes: Vec<usize>,
}

impl Storage {
    pub(super) fn new(
        program: &CheckedProgram,
        alternatives: &borrowed_payload_emit::Alternatives,
    ) -> Self {
        let mut storage = Self {
            types: HashMap::new(),
            lifetimes: vec![0; program.closures.len()],
        };
        loop {
            let mut updates = Vec::new();
            for function in program
                .functions
                .iter()
                .chain(program.closures.iter().map(|closure| &closure.function))
                .chain(
                    program
                        .closures
                        .iter()
                        .flat_map(|closure| closure.methods.iter().map(|method| &method.function)),
                )
            {
                crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
                    if !matches!(expr.kind, ExprKind::Closure { .. }) {
                        return;
                    }
                    let Some(Type::Closure(origin)) = expr.ty else {
                        return;
                    };
                    let closure = &program.closures[origin];
                    for (position, capture) in closure.captures.iter().enumerate() {
                        let source = storage.ty(function, capture.outer);
                        let current = storage.ty(&closure.function, capture.inner);
                        if source == current
                            || !matches!((source,current),(Type::Record(a),Type::Record(b)) if program.scoped_records.declaration(a)==program.scoped_records.declaration(b))
                        {
                            continue;
                        }
                        updates.push(((&closure.function as *const _, capture.inner), source));
                        for method in &closure.methods {
                            if let Some(&inner) = method.captures.get(position) {
                                updates.push(((&method.function as *const _, inner), source));
                            }
                        }
                    }
                });
            }
            if updates.is_empty() {
                break;
            }
            for (key, ty) in updates {
                storage.types.insert(key, ty);
            }
        }
        // Source capture graphs are finite. Resolve dependencies before assigning
        // slots, including concrete callback inputs in expanded native payloads.
        if storage.types.is_empty() && alternatives.is_empty() {
            storage.lifetimes = program
                .closures
                .iter()
                .map(|closure| closure.lifetime_count)
                .collect();
            return storage;
        }
        let mut counts = program
            .closures
            .iter()
            .map(|closure| closure.function.template.then_some(closure.lifetime_count))
            .collect::<Vec<_>>();
        loop {
            let mut changed = false;
            for (origin, closure) in program.closures.iter().enumerate() {
                if counts[origin].is_some() {
                    continue;
                }
                let mut count = 0;
                let mut resolved = true;
                for capture in &closure.captures {
                    let Some(nested) = Self::count(
                        program,
                        alternatives,
                        |origin| counts[origin],
                        storage.ty(&closure.function, capture.inner),
                    ) else {
                        resolved = false;
                        break;
                    };
                    count +=
                        usize::from(matches!(capture.storage, Mode::Read | Mode::Mutate)) + nested;
                }
                if resolved {
                    counts[origin] = Some(count);
                    changed = true;
                }
            }
            if counts.iter().all(Option::is_some) {
                break;
            }
            assert!(changed, "checked native capture graph must be finite");
        }
        storage.lifetimes = counts.into_iter().map(Option::unwrap).collect();
        storage
    }

    pub(super) fn ty(&self, function: &CheckedFunction, binding: usize) -> Type {
        self.types
            .get(&(function as *const _, binding))
            .copied()
            .unwrap_or(function.bindings[binding].ty)
    }

    fn count(
        program: &CheckedProgram,
        alternatives: &borrowed_payload_emit::Alternatives,
        counts: impl Fn(usize) -> Option<usize>,
        root: Type,
    ) -> Option<usize> {
        let mut pending = vec![(root, false)];
        let mut active = HashSet::new();
        let mut total = 0;
        while let Some((ty, finished)) = pending.pop() {
            if finished {
                active.remove(&ty);
                continue;
            }
            if matches!(
                ty,
                Type::Record(_) | Type::Union(_) | Type::BorrowedUnion(_)
            ) {
                if !active.insert(ty) {
                    return None;
                }
                pending.push((ty, true));
            }
            match ty {
                Type::Closure(origin) => total += counts(origin)?,
                Type::BorrowedCallable(family) => {
                    for &origin in program
                        .stored_contracts
                        .origins(family)
                        .expect("checked borrowed origins")
                    {
                        if let Some(vectors) = alternatives.get(&origin) {
                            for vector in vectors {
                                total += counts(origin)?;
                                for &(_, input) in vector {
                                    total += counts(input)?;
                                }
                            }
                        } else {
                            total += counts(origin)?;
                        }
                    }
                }
                Type::Record(record) if program.records[record].native_type.is_some_and(|id|crate::native_types::declaration(id).borrow_access().is_some()) => total += 1,
                Type::Array(element) if crate::environment_lifetimes::contains_native_view(element.ty(),&program.records,&|id|program.unions[id.index()].members.clone()) => pending.push((element.ty(),false)),
                Type::Record(record) => pending.extend(
                    program.records[record]
                        .fields
                        .iter()
                        .filter_map(|field| field.ty)
                        .map(|ty| (ty, false)),
                ),
                Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(
                    program.unions[id.index()]
                        .members
                        .iter()
                        .copied()
                        .map(|ty| (ty, false)),
                ),
                _ => {}
            }
        }
        Some(total)
    }

    pub(super) fn type_lifetimes(
        &self,
        program: &CheckedProgram,
        alternatives: &borrowed_payload_emit::Alternatives,
        ty: Type,
    ) -> usize {
        Self::count(
            program,
            alternatives,
            |origin| Some(self.lifetimes[origin]),
            ty,
        )
        .expect("resolved native environment")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_capture_storage_does_not_rewrite_checked_point_types() {
        let checked = crate::check(
            r#"
            type Holder = { run: ReadOnly<Function<int>>; };
            function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
                holder.run = (): int => action();
            }
            function main() {
                const values = [3];
                const first = (): int => values[0];
                const second = (): int => 9;
                const holder: Holder = { run: (): int => 0 };
                { const read = (): int => holder.run(); console.log(read()); }
                install(holder, first);
                install(holder, second);
                console.log(holder.run());
            }
        "#,
        )
        .unwrap();
        let program = &checked.program;
        let (origin, closure) = program
            .closures
            .iter()
            .enumerate()
            .find(|(_, closure)| {
                closure
                    .captures
                    .iter()
                    .any(|capture| closure.function.bindings[capture.inner].name == "holder")
            })
            .unwrap();
        let capture = closure
            .captures
            .iter()
            .find(|capture| closure.function.bindings[capture.inner].name == "holder")
            .unwrap();
        let point_type = closure.function.bindings[capture.inner].ty;
        let source_lifetimes = closure.lifetime_count;
        let alternatives = borrowed_payload_emit::capture_alternatives(program);
        let storage = Storage::new(program, &alternatives);
        assert_ne!(storage.ty(&closure.function, capture.inner), point_type);
        assert!(storage.lifetimes[origin] > source_lifetimes);
        assert_eq!(closure.function.bindings[capture.inner].ty, point_type);
        assert_eq!(closure.lifetime_count, source_lifetimes);
    }
}
