//! Function body scheduling and lexical capture dependencies.
use super::*;
mod dependencies;

struct DependencyGroup {
    members: Vec<usize>,
    completion: Vec<usize>,
    recursive: bool,
}

impl Analyzer {
    /// Schedule source dependencies with heap continuations. Runtime syntax and
    /// source function identities remain unchanged. Capturing declarations wait
    /// for a real defining context, and active recursive edges retain inference.
    pub(super) fn function(&mut self, index: usize) -> Result<(), Diagnostic> {
        if self.function_request_ready(index)? { return Ok(()); }
        continuations::driver::check_function(self, index)
    }

    pub(super) fn function_request_ready(&mut self, index: usize) -> Result<bool, Diagnostic> {
        if self.discovering_results || self.checked[index].is_some() { return Ok(true); }
        if self.active_recursive_functions.contains(&index) {
            if self.signatures[index].inferred_result && !self.provisional_results.contains_key(&index) {
                return Err(self.missing_recursive_result(index));
            }
            return Ok(true);
        }
        if self.visiting[index] { return self.prepare_function_body(index).map(|_| true); }
        Ok(false)
    }

    fn recursive_summary_is_complete(&self, index: usize, data_types: &mut HashMap<Type, bool>) -> bool {
        let signature = &self.signatures[index];
        if self.callable_result_type(signature.result) {
            // The initial pass requests recursive inference. Its discovery
            // worklist resolves concrete environments before body scheduling.
            return self.assumed_modes.is_none() || self.known_results.contains_key(&index);
        }
        if signature.inferred_result {
            // These contexts contain only the declared scalar parameters.
            // Defaults, destructuring, captures and concrete environments still
            // require a suspended full function context, not a guessed seed.
            return !self.named_captures.needs_context(index) && !self.templates.contains_key(&index)
                && self.program.functions[index].parameter_initializers.iter().all(|initializer| matches!(initializer, ParameterInitializer::Required))
                && signature.body_params.iter().all(|ty| matches!(ty, Type::Int | Type::Number | Type::Boolean | Type::String));
        }
        self.dependency_data_type(signature.result, data_types)
    }

    /// Closed data graphs need no caller-created callback environment. Traverse
    /// recursive record/union graphs on the heap; cache only within one unchanged
    /// graph inspection, since checking can publish new scoped record fields.
    fn dependency_data_type(&self, root: Type, cache: &mut HashMap<Type, bool>) -> bool {
        let mut pending = vec![root];
        let mut seen = HashSet::new();
        let valid = loop {
            let Some(ty) = pending.pop() else { break true; };
            if !seen.insert(ty) { continue; }
            if let Some(&valid) = cache.get(&ty) {
                if !valid { break false; }
                continue;
            }
            match ty {
                Type::Void | Type::String | Type::Int | Type::Number | Type::Boolean
                | Type::Null | Type::Undefined | Type::Enum(..) | Type::Literal(..) => {},
                Type::Array(element) => pending.push(element.ty()),
                Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(self.unions.members(id)),
                Type::Set(id) => pending.push(self.program.records[id].set_definition.as_ref().expect("set type").key),
                Type::Map(id) => {
                    let map = self.program.records[id].map_definition.as_ref().expect("map type");
                    pending.extend([map.key, map.value]);
                }
                Type::Record(id) => {
                    let record = &self.program.records[id];
                    if record.native_type.is_some() || record.fields.iter().any(|field| field.ty.is_none()) { break false; }
                    pending.extend(record.fields.iter().map(|field| field.ty.expect("resolved data field")));
                }
                _ => break false,
            }
        };
        if valid { cache.extend(seen.into_iter().map(|ty| (ty, true))); }
        else { cache.insert(root, false); }
        valid
    }

    fn seed_recursive_results(&mut self, members: &[usize]) -> Result<(), Diagnostic> {
        for &index in members {
            if !self.signatures[index].inferred_result || self.provisional_results.contains_key(&index) { continue; }
            let function = &self.program.functions[index];
            let mut context = Context::new(function.name.text.clone());
            for ((parameter, _, _), &ty) in function.params.iter().zip(&self.signatures[index].body_params) {
                Self::bind(&mut context, parameter, ty, true)?;
            }
            if let Some(result) = self.provisional_result(&context, &function.body) {
                self.provisional_results.insert(index, result);
                self.signatures[index].result = result;
            }
        }
        Ok(())
    }

    fn missing_recursive_result(&self, index: usize) -> Diagnostic {
        Diagnostic::new("result-inference", "The result type of this recursive function cannot be inferred: every return depends on the recursive call or on a local variable. Add a result annotation.", self.program.functions[index].name.span)
    }

    fn function_dependency_order(&self, root: usize) -> Vec<DependencyGroup> {
        enum Work { Enter(usize), Finish(usize) }
        let mut pending = vec![Work::Enter(root)];
        let mut discovery = HashMap::new();
        let mut finish = Vec::new();
        let mut parents = HashMap::<usize, Vec<usize>>::new();
        let mut data_types = HashMap::new();
        while let Some(work) = pending.pop() {
            match work {
                Work::Enter(index) => {
                    if self.checked[index].is_some() || self.scheduled_functions.contains(&index)
                        || discovery.contains_key(&index) { continue; }
                    discovery.insert(index, discovery.len());
                    pending.push(Work::Finish(index));
                    let dependencies = dependencies::references(&self.program.functions[index], &self.function_names)
                        .into_iter().filter(|&dependency| {
                            self.checked[dependency].is_none() && !self.scheduled_functions.contains(&dependency)
                                // Record callback contracts depend on environments
                                // constructed by the caller. Keep their original
                                // checking point until context suspension is explicit.
                                && self.signatures[dependency].params.iter().all(|&ty|
                                    !matches!(ty, Type::Record(_)) || self.dependency_data_type(ty, &mut data_types))
                                && (!self.named_captures.needs_context(dependency) || self.named_captures.ready(dependency))
                        }).collect::<Vec<_>>();
                    for &dependency in &dependencies { parents.entry(dependency).or_default().push(index); }
                    pending.extend(dependencies.into_iter().rev().map(Work::Enter));
                }
                Work::Finish(index) => finish.push(index),
            }
        }
        // Collapse recursive groups before reversing dependency order. Within
        // a group keep its first source traversal entry first: provisional
        // inferred results must be seeded by the same entry as direct checking.
        let mut groups = Vec::new();
        let finish_rank = finish.iter().enumerate().map(|(rank, &index)| (index, rank)).collect::<HashMap<_, _>>();
        let mut seen = HashSet::new();
        for &root in finish.iter().rev() {
            if !seen.insert(root) { continue; }
            let mut group = Vec::new();
            let mut pending = vec![root];
            while let Some(index) = pending.pop() {
                group.push(index);
                for &parent in parents.get(&index).into_iter().flatten() {
                    if seen.insert(parent) { pending.push(parent); }
                }
            }
            group.sort_unstable_by_key(|index| discovery[index]);
            let recursive = group.len() > 1 || parents.get(&root).is_some_and(|parents| parents.contains(&root));
            let mut completion = group.clone();
            completion.sort_unstable_by_key(|index| finish_rank[index]);
            groups.push(DependencyGroup { members: group, completion, recursive });
        }
        groups.into_iter().rev().collect()
    }

    /// Resolve lexical owners before installing an unused-body placeholder.
    /// The continuation list preserves the original owner-first checks; owner
    /// completion can prepare or even check a waiting descendant.
    pub(super) fn capturing_function_is_unused(&mut self, index: usize) -> Result<bool, Diagnostic> {
        let mut pending = Vec::new();
        let mut current = index;
        let mut unused = loop {
            if !self.named_captures.needs_context(current) || self.named_captures.ready(current) {
                break false;
            }
            if let Some(owner) = self.named_captures.declaring_function(current, &self.program, &self.function_names)
                && owner != current
                && self.checked[owner].is_none()
                && !self.visiting[owner]
            {
                pending.push((current, owner));
                current = owner;
            } else {
                break self.defer_unused_capturing_function(current);
            }
        };
        while let Some((current, owner)) = pending.pop() {
            if !unused {
                self.function(owner)?;
            }
            unused = self.defer_unused_capturing_function(current);
        }
        Ok(unused)
    }

    fn defer_unused_capturing_function(&mut self, index: usize) -> bool {
        if self.named_captures.ready(index) || self.checked[index].is_some() {
            return false;
        }
        self.checked[index] = Some(CheckedFunction {
            storage_exits: Default::default(), native_returns: None, native_mutations: None,
            template: true,
            specialization: false,
            name: self.program.functions[index].name.text.clone(),
            bindings: Vec::new(),
            parameter_count: 0,
            result: self.signatures[index].result,
            body: Vec::new(),
        });
        true
    }

    pub(super) fn prepare_function_body(&mut self, index: usize) -> Result<Option<(PreparedFunction, super::function_parameters::FunctionParameters)>, Diagnostic> {
        if self.discovering_results {
            return Ok(None);
        }
        if self.checked[index].is_some() {
            return Ok(None);
        }
        if self.named_captures.needs_context(index) && !self.named_captures.ready(index) {
            return Err(self.program.local_capture_plans.iter().find_map(crate::local_capture_plan::Plan::pending_diagnostic).expect("pending named capture"));
        }
        let name = self.program.functions[index].name.clone();
        if self.visiting[index] {
            if self.signatures[index].inferred_result {
                // A return that does not depend on the recursive call fixes the
                // type; the finished body is then checked against it as usual.
                let Some(&provisional) = self.provisional_results.get(&index) else {
                    return Err(self.missing_recursive_result(index));
                };
                self.signatures[index].result = provisional;
            }
            let start = self
                .function_stack
                .iter()
                .position(|&id| id == index)
                .expect("active function");
            for &id in &self.function_stack[start..] {
                if self.signatures[id]
                    .params
                    .iter()
                    .any(|ty| matches!(ty, Type::Callback(_)))
                {
                    self.recursive_callbacks.insert(id);
                }
            }
            return if self.assumed_modes.is_some() {
                Ok(None)
            } else {
                Err(Diagnostic::new(
                    "recursion-inference",
                    "Recursive access inference is required.",
                    name.span,
                ))
            };
        }
        if self.callable_result_type(self.signatures[index].result)
            && self.assumed_modes.is_some()
            && !self.known_results.contains_key(&index)
        {
            self.discover_results()?;
        }
        self.visiting[index] = true;
        self.function_stack.push(index);
        self.depth += 1;
        let body = std::mem::take(&mut self.program.functions[index].body);
        let (context, parameters) = self.reserve_function_parameters(index)?;
        let result_type = self.signatures[index].result;
        Ok(Some((PreparedFunction { index, body, context, result_type }, parameters)))
    }

    pub(super) fn complete_named_parameters(&mut self, index: usize, context: &mut Context, body: &[Stmt]) -> Type {
        for (position, access) in self.program.functions[index].parameter_access.iter().enumerate() {
            if matches!(access, Some((Mode::Read, _))) && !context.bindings[position].ty.is_copy() {
                context.readonly_parameters.insert(position);
            }
        }
        if self.signatures[index].inferred_result && !self.provisional_results.contains_key(&index)
            && let Some(provisional) = self.provisional_result(context, body)
        {
            // Type hints inside the body read the signature before the
            // recursive call is checked, so publish the result now.
            self.provisional_results.insert(index, provisional);
            self.signatures[index].result = provisional;
        }
        // Move the body into analysis rather than clone the syntax tree.
        let result_type = self.signatures[index].result;
        context.inferred_body_result = self.signatures[index].inferred_result;
        result_type
    }

    pub(super) fn finish_function_body(&mut self, prepared: PreparedFunction, returned: Result<bool, Diagnostic>) -> Result<Vec<usize>, Diagnostic> {
        let PreparedFunction { index, mut body, mut context, mut result_type } = prepared;
        let name = self.program.functions[index].name.clone();
        let returned = returned.map_err(|error| self.filter_ownership_fixes(&context, error))?;
        if context.inferred_body_result {
            result_type = self.inferred_body_result(&body)?;
            self.signatures[index].result = result_type;
        }
        // Replay skipped this function's failed return; a declared data result
        // still types its callers. A callable result needs its checked closure.
        let contract=self.program.diagnostic_contracts.contains(&name.span.start) && !self.callable_result_type(result_type);
        if result_type != Type::Void && !returned && !contract {
            return Err(Diagnostic::new(
                "type",
                format!(
                    "Function {:?} must return {}.",
                    self.program.functions[index]
                        .application
                        .as_ref()
                        .map(|application| application.declaration.text.as_str())
                        .unwrap_or(&name.text),
                    self.type_name(result_type)
                ),
                name.span,
            ));
        }
        self.finalize_nullable_locals(&mut context.bindings, &mut body)?;
        // A record annotation names its declaration; the checked result must
        // retain the concrete callback environment selected by its returns.
        if let Type::Array(ArrayElement::Record(base, depth)) = result_type
            && let Some(joined) = self.joined_array_result(&mut body, base, depth)?
        {
            result_type = joined;
        }
        if let Type::Record(base) = result_type
            && let Some(joined) = self.joined_record_result(&mut body, base)?
        {
            result_type = joined;
        } else if let Type::Record(base) = result_type {
            let actual = self.inferred_body_result(&body)?;
            if let Type::Record(instance) = actual {
                if self.scoped_records.declaration(instance) == base {
                    result_type = actual;
                }
            }
        }
        let result_type = if self.callable_result_type(result_type) {
            context.returned_callable.expect("checked callable return")
        } else {
            result_type
        };
        let result_type = self.nullable_record_result(&mut body, result_type)?;
        self.finalize_nullable_returns(&mut body, result_type)?;
        self.declared_parameter_access(index, &mut context)?;
        self.validate_native_returns(&context)?;
        if self.assumed_modes.is_some() && self.callable_result_type(self.signatures[index].result)
        {
            self.known_results.insert(index, result_type);
        }
        let uncalled = self.prepare_uncalled_capturing_functions(index, &context)?;
        self.checked[index] = Some(CheckedFunction {
                storage_exits: context.storage_exits.clone(), native_returns: context.native_returns.clone(), native_mutations: context.native_mutations.clone(),
            template: context.template,
            specialization: index >= self.original_count,
            name: name.text,
            bindings: context.bindings,
            parameter_count: self.signatures[index].params.len(),
            result: result_type,
            body,
        });
        self.visiting[index] = false;
        self.function_stack.pop();
        self.depth -= 1;
        Ok(uncalled)
    }

}

pub(super) struct PreparedFunction {
    pub(super) index: usize,
    pub(super) body: Vec<Stmt>,
    pub(super) context: Context,
    pub(super) result_type: Type,
}

enum Work { Schedule(usize), Group(DependencyGroup), Check(usize), ReleaseGroup(Vec<usize>), Release(Vec<usize>) }

/// Dependency work survives while a function body runs on the checker stack.
pub(super) struct Schedule {
    pending: Vec<Work>,
    inserted: Vec<usize>,
    recursive_inserted: Vec<usize>,
}

impl Schedule {
    pub(super) fn new(index: usize) -> Self {
        Self { pending: vec![Work::Schedule(index)], inserted: Vec::new(), recursive_inserted: Vec::new() }
    }

    pub(super) fn extend(&mut self, uncalled: Vec<usize>) {
        self.pending.extend(uncalled.into_iter().rev().map(Work::Schedule));
    }

    pub(super) fn next(&mut self, analyzer: &mut Analyzer) -> Result<Option<usize>, Diagnostic> {
        while let Some(work) = self.pending.pop() {
            match work {
                Work::Schedule(index) => {
                    if analyzer.checked[index].is_some() { continue; }
                    if analyzer.scheduled_functions.contains(&index) {
                        self.pending.push(Work::Check(index));
                        continue;
                    }
                    let groups = analyzer.function_dependency_order(index);
                    let order = groups.iter().flat_map(|group| group.members.iter().copied()).collect::<Vec<_>>();
                    for &id in &order {
                        if analyzer.scheduled_functions.insert(id) { self.inserted.push(id); }
                    }
                    self.pending.push(Work::Release(order));
                    self.pending.extend(groups.into_iter().rev().map(Work::Group));
                }
                Work::Group(group) => {
                    let mut data_types = HashMap::new();
                    if group.recursive && group.members.iter().all(|&id| analyzer.recursive_summary_is_complete(id, &mut data_types)) {
                        if analyzer.assumed_modes.is_none() {
                            return Err(Diagnostic::new("recursion-inference", "Recursive access inference is required.", analyzer.program.functions[group.members[0]].name.span));
                        }
                        analyzer.seed_recursive_results(&group.members)?;
                        for &id in &group.members {
                            if analyzer.active_recursive_functions.insert(id) { self.recursive_inserted.push(id); }
                            if analyzer.signatures[id].params.iter().any(|ty| matches!(ty, Type::Callback(_))) {
                                analyzer.recursive_callbacks.insert(id);
                            }
                        }
                        self.pending.push(Work::ReleaseGroup(group.members.clone()));
                        // Actual DFS completion order also handles shared
                        // callees: reversing discovery alone can put a sibling
                        // before the inferred result that it depends on.
                        self.pending.extend(group.completion.into_iter().rev().map(Work::Check));
                    } else {
                        self.pending.extend(group.members.into_iter().rev().map(Work::Check));
                    }
                }
                Work::ReleaseGroup(ids) => {
                    for id in ids { analyzer.active_recursive_functions.remove(&id); }
                }
                Work::Check(index) => return Ok(Some(index)),
                Work::Release(ids) => {
                    for id in ids { analyzer.scheduled_functions.remove(&id); }
                }
            }
        }
        Ok(None)
    }

    pub(super) fn release(self, analyzer: &mut Analyzer) {
        for id in self.inserted { analyzer.scheduled_functions.remove(&id); }
        for id in self.recursive_inserted { analyzer.active_recursive_functions.remove(&id); }
    }
}
