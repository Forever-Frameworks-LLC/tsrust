//! Checked storage-owner provenance for the pending native marker representation.
//! This planner does not alter source ownership or claim runtime instance identity.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Owner {
    pub(super) scope: Scope,
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
    pub(super) fn retained_borrows(
        self,
        program: &CheckedProgram,
    ) -> Vec<crate::analysis::record_parameter_borrows::Borrow> {
        let mut borrows = self.borrows(program).to_vec();
        if let Self::Closure(origin) = self {
            let closure = &program.closures[origin];
            borrows.extend(
                program
                    .closure_record_capture_borrows
                    .get(&origin)
                    .into_iter()
                    .flatten()
                    .map(|borrow| crate::analysis::record_parameter_borrows::Borrow {
                        target: borrow.target.clone(),
                        source: closure.parameters.len() + borrow.source_capture,
                        source_projection: borrow.source_projection.clone(),
                        mode: borrow.mode,
                        origin: borrow.origin,
                        capture: borrow.capture,
                    }),
            );
        }
        borrows
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
        if value.parameter || value.capture_reference {
            return None;
        }
        if !value.rebindable && (value.callback_reference || value.borrowed_place.is_some()) {
            let [root] = value.view_roots.as_slice() else {
                return None;
            };
            binding = *root;
            continue;
        }
        return Some(Owner { scope, binding });
    }
    None
}

/// Owners of the inputs of one closure value, resolved where that value was
/// constructed. Capture `outer` bindings belong to the construction context,
/// so a caller that only holds the value must receive them, not resolve them.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Environment {
    inputs: Vec<Option<Owner>>,
    nested: BTreeMap<usize, Environment>,
}

/// The environment of a closure literal's origin constructed in `function`.
fn constructed_environment(
    program: &CheckedProgram,
    function: &CheckedFunction,
    scope: Scope,
    inputs: &[Option<Owner>],
    environments: &BTreeMap<usize, Environment>,
    owned: &HashSet<usize>,
    origin: usize,
    depth: usize,
) -> Option<Environment> {
    if depth > program.closures.len() {
        return None;
    }
    let closure = &program.closures[origin];
    let mut result = Environment {
        inputs: vec![None; closure.function.bindings.len()],
        nested: BTreeMap::new(),
    };
    for capture in &closure.captures {
        result.inputs[capture.inner] = resolve(function, scope, inputs, capture.outer);
        if let Some(Type::Closure(inner)) = function.bindings.get(capture.outer).map(|binding| binding.ty) {
            let environment = if owned.contains(&inner) {
                constructed_environment(program, function, scope, inputs, environments, owned, inner, depth + 1)
            } else {
                environments.get(&capture.outer).cloned()
            };
            if let Some(environment) = environment {
                result.nested.insert(capture.inner, environment);
            }
        }
    }
    Some(result)
}

fn walk(program: &CheckedProgram, pending: Vec<(Scope, Vec<Option<Owner>>)>) -> Plan {
    let mut pending = pending
        .into_iter()
        .map(|(scope, inputs)| (scope, inputs, BTreeMap::<usize, Environment>::new()))
        .collect::<Vec<_>>();
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
    // A closure reached through a held value whose environment is unknown here.
    let mut unresolved_receivers = BTreeSet::new();
    while let Some((id, inputs, environments)) = pending.pop() {
        if !seen.insert((id, inputs.clone(), environments.clone())) {
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
        });
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            // A call through a held closure value can lack origin metadata on
            // the call; the receiver binding's checked type still names it.
            let callee = Scope::call(expr).or_else(|| match expr.target {
                Some(CallTarget::Closure(binding)) => match function.bindings.get(binding)?.ty {
                    Type::Closure(origin) => Some(Scope::Closure(origin)),
                    _ => None,
                },
                _ => None,
            });
            if let (Some(callee), ExprKind::Call(_, arguments)) = (callee, &expr.kind) {
                let values = arguments
                    .iter()
                    .map(|argument| {
                        if !matches!(argument.kind, ExprKind::Name(_)) {
                            return None;
                        }
                        resolve(function, id, &inputs, argument.binding?)
                    })
                    .collect::<Vec<_>>();
                let (values, nested) = match callee {
                    Scope::Function(_) => (values, BTreeMap::new()),
                    Scope::Closure(origin) => {
                        let closure = &program.closures[origin];
                        let receiver = match expr.target {
                            Some(CallTarget::Closure(binding)) => function.bindings.get(binding).map(|value| (binding, value)),
                            _ => None,
                        };
                        let environment = match receiver {
                            Some((binding, _)) if !parameters.contains(&binding) && owned.contains(&origin) => {
                                constructed_environment(program, function, id, &inputs, &environments, &owned, origin, 0)
                            }
                            Some((binding, _)) => environments.get(&binding).cloned(),
                            None => None,
                        };
                        let mut environment = environment.unwrap_or_else(|| {
                            unresolved_receivers.insert(origin);
                            Environment {
                                inputs: vec![None; closure.function.bindings.len()],
                                nested: BTreeMap::new(),
                            }
                        });
                        for (&parameter, owner) in closure.parameters.iter().zip(values) {
                            environment.inputs[parameter] = owner;
                        }
                        (environment.inputs, environment.nested)
                    }
                };
                pending.push((callee, values, nested));
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
    // A callee whose environment was unknown at some call cannot name the
    // owners of what it installs; neither can any closure it installs through.
    for target in unresolved_receivers {
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
        .map(|binding| {
            Some(Owner {
                scope: Scope::Function(function),
                binding,
            })
        })
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
        scope: Scope::Function(main),
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

#[test]
fn closure_local_storage_uses_its_own_owner_scope() {
    let checked = crate::check(r#"
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void { calls[0] += 1; holder.run = (): int => source[0]; }
        function main() {
            const inspect = (): void => {
                const first = [3]; const second = [10]; const calls = [0]; const holder: Holder = { run: undefined };
                install(holder, first, calls); install(holder, second, calls); console.log(holder.run!());
            };
            inspect();
        }
    "#).unwrap();
    let program = &checked.program;
    let install = program
        .functions
        .iter()
        .position(|function| function.name == "install")
        .unwrap();
    let origin = program.record_parameter_borrows[&install][0].origin;
    let closure = program
        .closures
        .iter()
        .position(|closure| {
            closure
                .function
                .bindings
                .iter()
                .any(|binding| binding.name == "first")
        })
        .unwrap();
    let owners = plan(program)[&origin]
        .iter()
        .flat_map(|vector| vector.iter().map(|(_, owner)| owner.clone()))
        .collect::<BTreeSet<_>>();
    let expected = ["first", "second"]
        .map(|name| Owner {
            scope: Scope::Closure(closure),
            binding: program.closures[closure]
                .function
                .bindings
                .iter()
                .position(|binding| binding.name == name)
                .unwrap(),
        })
        .into_iter()
        .collect();
    assert_eq!(owners, expected);
    assert!(
        owners
            .iter()
            .all(|owner| Emitter::owner_name(owner).starts_with("TsrClosureOwner"))
    );
    assert_eq!(
        Emitter::owner_name(&Owner {
            scope: Scope::Function(closure),
            binding: 0
        }),
        format!("TsrOwner{closure}_0")
    );
}
