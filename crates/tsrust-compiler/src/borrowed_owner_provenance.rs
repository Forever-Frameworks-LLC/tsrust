//! Checked storage-owner provenance for the pending native marker representation.
//! This planner does not alter source ownership or claim runtime instance identity.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Owner {
    pub(super) function: usize,
    pub(super) binding: usize,
}

pub(super) type Vector = Vec<(usize, Owner)>;
pub(super) type Plan = BTreeMap<usize, BTreeSet<Vector>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Scope {
    Function(usize),
    Closure(usize),
}
impl Scope {
    pub(super) fn function(self, program: &CheckedProgram) -> &CheckedFunction {
        match self {
            Self::Function(id) => &program.functions[id],
            Self::Closure(id) => &program.closures[id].function,
        }
    }
    pub(super) fn parameters(self, program: &CheckedProgram) -> Vec<usize> {
        match self {
            Self::Function(id) => (0..program.functions[id].parameter_count).collect(),
            Self::Closure(id) => program.closures[id].parameters.clone(),
        }
    }
    pub(super) fn borrows(
        self,
        program: &CheckedProgram,
    ) -> &[crate::analysis::record_parameter_borrows::Borrow] {
        match self {
            Self::Function(id) => program.record_parameter_borrows.get(&id),
            Self::Closure(id) => program.closure_record_parameter_borrows.get(&id),
        }
        .map_or(&[], Vec::as_slice)
    }
    pub(super) fn releases(self, program: &CheckedProgram) -> &[(usize, Vec<String>)] {
        match self {
            Self::Function(id) => program.record_call_releases.get(&id),
            Self::Closure(id) => program.closure_record_call_releases.get(&id),
        }
        .map_or(&[], Vec::as_slice)
    }
    pub(super) fn replacements(self, program: &CheckedProgram) -> &[(usize, Vec<String>, Type)] {
        match self {
            Self::Function(id) => program.record_parameter_replacements.get(&id),
            Self::Closure(_) => None,
        }
        .map_or(&[], Vec::as_slice)
    }
    pub(super) fn call(expr: &Expr) -> Option<Self> {
        match expr.target {
            Some(CallTarget::Function(id) | CallTarget::ForwardFunction(id)) => {
                Some(Self::Function(id))
            }
            Some(CallTarget::Closure(_)) => match expr.callable_origin {
                Some(CallableOrigin::Closure(id)) => Some(Self::Closure(id)),
                _ => None,
            },
            _ => None,
        }
    }
}

fn resolve(
    function: &CheckedFunction,
    scope: Scope,
    inputs: &[Option<Owner>],
    binding: usize,
) -> Option<Owner> {
    let mut binding = binding;
    let mut seen = BTreeSet::new();
    while seen.insert(binding) {
        if let Some(Some(owner)) = inputs.get(binding) {
            return Some(owner.clone());
        }
        let value = function.bindings.get(binding)?;
        if value.parameter {
            return None;
        }
        if !value.rebindable && (value.callback_reference || value.borrowed_place.is_some()) {
            let [root] = value.view_roots.as_slice() else {
                return None;
            };
            binding = *root;
            continue;
        }
        let Scope::Function(id) = scope else {
            return None;
        };
        return Some(Owner {
            function: id,
            binding,
        });
    }
    None
}

fn walk(program: &CheckedProgram, mut pending: Vec<(Scope, Vec<Option<Owner>>)>) -> Plan {
    let mut result = Plan::new();
    if program.record_parameter_borrows.is_empty() {
        return result;
    }
    let mut incomplete = BTreeSet::new();
    // Concrete static forwarding calls are grounded below. Abstract uses and
    // closure bodies still require a complete environment-owner interface.
    let unresolved_forward = forwarded_owner_uses::incomplete_targets(program);
    for target in closure_owner_uses::incomplete_targets(program) {
        incomplete.extend(
            program
                .closure_record_capture_borrows
                .get(&target)
                .into_iter()
                .flatten()
                .map(|borrow| borrow.origin),
        );
        incomplete.extend(
            Scope::Closure(target)
                .borrows(program)
                .iter()
                .map(|borrow| borrow.origin),
        );
    }
    for target in unresolved_forward {
        incomplete.extend(
            program
                .record_parameter_borrows
                .get(&target)
                .into_iter()
                .flatten()
                .map(|borrow| borrow.origin),
        );
    }
    let mut seen = BTreeSet::new();
    while let Some((id, inputs)) = pending.pop() {
        if !seen.insert((id, inputs.clone())) {
            continue;
        }
        let function = id.function(program);
        let parameters = id.parameters(program);
        if function.template {
            continue;
        }
        let mut owned = HashSet::new();
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if matches!(expr.kind, ExprKind::Closure { .. })
                && let Some(Type::Closure(origin)) = expr.ty
            {
                owned.insert(origin);
            }
            if let (Some(callee), ExprKind::Call(_, arguments)) = (Scope::call(expr), &expr.kind) {
                let values = arguments
                    .iter()
                    .map(|argument| {
                        if !matches!(argument.kind, ExprKind::Name(_)) {
                            return None;
                        }
                        resolve(function, id, &inputs, argument.binding?)
                    })
                    .collect::<Vec<_>>();
                let values = match callee {
                    Scope::Function(_) => values,
                    Scope::Closure(origin) => {
                        let closure = &program.closures[origin];
                        let mut mapped = vec![None; closure.function.bindings.len()];
                        for (&parameter, owner) in closure.parameters.iter().zip(values) {
                            mapped[parameter] = owner;
                        }
                        for capture in &closure.captures {
                            mapped[capture.inner] = resolve(function, id, &inputs, capture.outer);
                        }
                        mapped
                    }
                };
                pending.push((callee, values));
            }
        });
        let mut vectors = BTreeMap::<usize, Vector>::new();
        let mut unresolved = BTreeSet::new();
        for borrow in id.borrows(program) {
            if !owned.contains(&borrow.origin) {
                continue;
            }
            let capture = &program.closures[borrow.origin].captures[borrow.capture];
            match resolve(function, id, &inputs, parameters[borrow.source]) {
                Some(owner) => vectors
                    .entry(borrow.origin)
                    .or_default()
                    .push((capture.inner, owner)),
                None => {
                    unresolved.insert(borrow.origin);
                }
            }
        }
        if inputs.iter().any(Option::is_some) {
            incomplete.extend(unresolved.iter().copied());
        }
        for (origin, mut vector) in vectors {
            if unresolved.contains(&origin) {
                continue;
            }
            vector.sort();
            vector.dedup();
            result.entry(origin).or_default().insert(vector);
        }
    }
    result.retain(|origin, _| !incomplete.contains(origin));
    result
}

pub(super) fn plan(program: &CheckedProgram) -> Plan {
    let pending = program
        .functions
        .iter()
        .enumerate()
        .filter(|(_, function)| !function.template)
        .map(|(id, function)| (Scope::Function(id), vec![None; function.parameter_count]))
        .collect::<Vec<_>>();
    walk(program, pending)
}
pub(super) fn parameter_vectors(program: &CheckedProgram, function: usize) -> Plan {
    let inputs = (0..program.functions[function].parameter_count)
        .map(|binding| Some(Owner { function, binding }))
        .collect();
    walk(program, vec![(Scope::Function(function), inputs)])
}

#[test]
fn independent_equal_type_inputs_remain_distinct_through_recursive_relays() {
    let checked = crate::check(r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function counter(start: int): () => int { return (): int => start; }
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void { holder.run = action; }
        function relay(holder: Mutable<Holder>, action: ReadOnly<Function<int>>, n: int): void {
            if (n > 0) { relay(holder, action, n - 1); }
            const alias = action;
            install(holder, alias);
        }
        function main() {
            const first = counter(3); const second = counter(10);
            const holder: Holder = { run: (): int => 0 };
            relay(holder, first, 2); relay(holder, second, 1);
            console.log(holder.run());
        }
    "#).unwrap();
    let result = plan(&checked.program);
    assert_eq!(result.len(), 1);
    let vectors = result.values().next().unwrap();
    assert_eq!(vectors.len(), 2);
    let owners = vectors
        .iter()
        .map(|vector| {
            assert_eq!(vector.len(), 1);
            vector[0].1.clone()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        owners.len(),
        2,
        "Source closure origin is not storage owner identity"
    );
    assert_eq!(
        result,
        plan(&checked.program),
        "Stable ordering and recursive convergence"
    );
}

#[test]
fn correlated_inputs_do_not_form_a_cartesian_product() {
    let checked = crate::check(r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, left: ReadOnly<int[]>, right: ReadOnly<int[]>): void {
            holder.run = (): int => left[0] * 10 + right[0];
        }
        function main() {
            const first = [3]; const second = [10]; const holder: Holder = { run: (): int => 0 };
            install(holder, first, second); install(holder, second, first); console.log(holder.run());
        }
    "#).unwrap();
    let result = plan(&checked.program);
    let vectors = result.values().next().unwrap().iter().collect::<Vec<_>>();
    assert_eq!(vectors.len(), 2);
    assert_eq!(vectors[0].len(), 2);
    assert_eq!(vectors[0][0].1, vectors[1][1].1);
    assert_eq!(vectors[0][1].1, vectors[1][0].1);
    assert_ne!(vectors[0][0].1, vectors[0][1].1);
}

#[test]
fn an_ungrounded_formal_input_is_not_a_concrete_owner() {
    let checked = crate::check(r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void { holder.run = action; }
        function main() {}
    "#).unwrap();
    assert!(plan(&checked.program).is_empty());
}

#[test]
fn repeating_a_call_does_not_manufacture_another_owner() {
    let checked = crate::check(
        r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function main() {
            const source = [3]; const holder: Holder = { run: (): int => 0 };
            install(holder, source); install(holder, source); console.log(holder.run());
        }
    "#,
    )
    .unwrap();
    let result = plan(&checked.program);
    assert_eq!(result.len(), 1);
    assert_eq!(result.values().next().unwrap().len(), 1);
}

#[test]
fn static_installer_function_values_keep_independent_named_owners() {
    let checked = crate::check(r#"
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            calls[0] += 1; holder.run = (): int => source[0];
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0];
            const holder: Holder = { run: undefined }; const perform = install;
            perform(holder, first, calls); perform(holder, second, calls);
            first[0] += 1; console.log(holder.run!());
        }
    "#).unwrap();
    let result = plan(&checked.program);
    assert_eq!(
        result.len(),
        1,
        "Static forwarding must retain concrete owner provenance: {result:?}"
    );
    assert_eq!(result.values().next().unwrap().len(), 2);
}

#[test]
fn abstract_installer_uses_do_not_invent_concrete_owner_vectors() {
    let checked = crate::check(r#"
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            calls[0] += 1; holder.run = (): int => source[0];
        }
        function invoke(action: ReadOnly<Function<void, [Mutable<Holder>, ReadOnly<int[]>, Mutable<int[]>]>>, holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            action(holder, source, calls);
        }
        function main() {
            const source = [3]; const calls = [0]; const holder: Holder = { run: undefined };
            const perform = install;
            invoke(perform, holder, source, calls);
        }
    "#).unwrap();
    assert!(
        plan(&checked.program).is_empty(),
        "Abstract invocations require their own complete owner interface"
    );
    let install = checked
        .program
        .functions
        .iter()
        .position(|function| function.name == "install")
        .unwrap();
    assert!(
        parameter_vectors(&checked.program, install).is_empty(),
        "Even grounded symbolic inputs must not bypass an abstract-use guard"
    );
}

#[test]
fn ordinary_closure_installers_keep_named_caller_owner_identity() {
    let source = r#"
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void { holder.run = (): int => source[0]; }
        function main() {
            const first = [3]; const second = [10]; const holder: Holder = { run: undefined };
            const perform = (target: Mutable<Holder>, source: ReadOnly<int[]>): void => { install(target, source); };
            perform(holder, first); perform(holder, second); console.log(holder.run!());
        }
    "#;
    let checked = crate::check(source).unwrap();
    let program = &checked.program;
    let install = program
        .functions
        .iter()
        .position(|function| function.name == "install")
        .unwrap();
    let origin = program.record_parameter_borrows[&install][0].origin;
    let vectors = plan(program);
    assert_eq!(vectors[&origin].len(), 2);
    let main = program
        .functions
        .iter()
        .position(|function| function.name == "main")
        .unwrap();
    let expected = ["first", "second"].map(|name| Owner {
        function: main,
        binding: program.functions[main]
            .bindings
            .iter()
            .position(|binding| binding.name == name)
            .unwrap(),
    });
    assert_eq!(
        vectors[&origin]
            .iter()
            .flat_map(|vector| vector.iter().map(|(_, owner)| owner.clone()))
            .collect::<BTreeSet<_>>(),
        expected.into_iter().collect()
    );
    let abstract_use = source
        .replace(
            "perform(holder, first);",
            "const alias = perform; alias(holder, first);",
        )
        .replace("perform(holder, second);", "alias(holder, second);");
    let checked = crate::check(&abstract_use).unwrap();
    assert!(
        plan(&checked.program).is_empty(),
        "alias contexts must not invent concrete owner vectors"
    );
}
