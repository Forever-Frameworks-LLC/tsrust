//! Bind planned named captures to checked defining slots without materializing loans.
use super::*;

#[derive(Clone, Copy)]
struct Request {
    plan: usize,
    node: usize,
    owner: usize,
}
#[derive(Default)]
pub(super) struct Contexts {
    requests: HashMap<usize, Request>,
    prepared: HashMap<usize, Context>,
    completed: HashMap<usize, Context>,
    lexical_owners: std::collections::HashSet<usize>,
    identities: HashMap<usize, Vec<crate::local_capture_plan::BindingId>>,
    /// Lexical owner of each nested function owner.
    enclosing: HashMap<usize, usize>,
    /// Requests by emitted declaration name, for generic instances.
    declared: HashMap<String, Request>,
}
impl Contexts {
    pub(super) fn new(program: &Program, functions: &HashMap<String, usize>) -> Self {
        let mut contexts = Self::default();
        contexts.enclosing = program
            .local_declarations
            .iter()
            .map(|declaration| (declaration.function_owner, declaration.enclosing_owner))
            .collect();
        for (plan_id, plan) in program.local_capture_plans.iter().enumerate() {
            let requirements = plan.group_requirements();
            for declaration in &program.local_declarations {
                if declaration.id < plan.node_start
                    || declaration.id >= plan.node_start + plan.group_of.len()
                {
                    continue;
                }
                let group = plan.group_of[declaration.id - plan.node_start];
                if !requirements[group]
                    .iter()
                    .any(|binding| contexts.encloses(binding.owner, declaration.function_owner))
                {
                    continue;
                }
                let request = Request {
                    plan: plan_id,
                    node: declaration.id,
                    owner: declaration.function_owner,
                };
                contexts.declared.insert(declaration.emitted_name.clone(), request);
                if let Some(&function) = functions.get(&declaration.emitted_name) {
                    contexts.requests.insert(function, request);
                }
            }
        }
        for function in &program.functions {
            if let Some(application) = &function.application
                && let Some(&index) = functions.get(&function.name.text)
            {
                contexts.instance(index, &application.declaration.text);
            }
        }
        for &index in contexts.requests.keys() {
            let mut current = index;
            let mut visited = std::collections::HashSet::new();
            while visited.insert(current) {
                let Some(owner) = contexts.declaring_function(current, program, functions) else { break; };
                contexts.lexical_owners.insert(owner);
                current = owner;
            }
        }
        contexts
    }
    /// Whether `ancestor` lexically contains the nested function `owner`.
    /// A recursive group can name a variable of one of its own members; only
    /// the functions inside that member receive it, each call anew.
    fn encloses(&self, ancestor: usize, mut owner: usize) -> bool {
        while let Some(&parent) = self.enclosing.get(&owner) {
            if parent == ancestor {
                return true;
            }
            owner = parent;
        }
        false
    }
    /// The function whose body declares this capturing function. Checking
    /// it first lets its calls prepare the typed capture context.
    pub(super) fn declaring_function(&self, index: usize, program: &Program, functions: &HashMap<String, usize>) -> Option<usize> {
        let function = &program.functions[index];
        if let Some(owner) = function.application.as_ref().and_then(|application| application.lexical_owner.as_ref()) {
            return functions.get(owner).copied();
        }
        let name = function.application.as_ref().map_or(function.name.text.as_str(), |application| application.declaration.text.as_str());
        let declaration = program.local_declarations.iter().find(|declaration| declaration.emitted_name == name)?;
        let plan = program.local_capture_plans.iter().find(|plan| plan.owners.contains_key(&declaration.enclosing_owner))?;
        let name = plan.owners.get(&declaration.enclosing_owner)?;
        functions.get(name).copied()
    }
    /// A concrete copy of a capturing function with callback parameters
    /// receives the same captured variables as the declaration it copies.
    pub(super) fn specialized(&mut self, copy: usize, declaration: usize) {
        let Some(request) = self.requests.get(&declaration) else {
            return;
        };
        self.requests.insert(copy, *request);
        if let Some(context) = self.prepared.get(&declaration) {
            let context = context.clone();
            self.prepared.insert(copy, context);
            self.identities.insert(copy, self.identities[&declaration].clone());
        }
    }
    /// An instance of a generic nested function captures what its declaration does.
    pub(super) fn instance(&mut self, copy: usize, declaration: &str) {
        if let Some(&request) = self.declared.get(crate::generic_functions::lexical_declaration(declaration)) {
            self.requests.insert(copy, request);
        }
    }
    /// Preserve lexical types for unused declaration checking without imposing
    /// hidden runtime arguments on the function that declares them.
    pub(super) fn complete(&mut self, index: usize, context: &Context) {
        if self.lexical_owners.contains(&index) {
            self.completed.insert(index, context.clone());
        }
    }
    pub(super) fn prepare_completed(
        &mut self, program: &Program, functions: &HashMap<String, usize>,
    ) -> Result<Vec<usize>, Diagnostic> {
        let mut pending = self.requests.keys().copied()
            .filter(|index| !self.prepared.contains_key(index)).collect::<Vec<_>>();
        pending.sort_unstable();
        let mut ready = Vec::new();
        for index in pending {
            let Some(owner) = self.declaring_function(index, program, functions) else { continue; };
            let Some(source) = self.completed.get(&owner) else { continue; };
            let mut outer = source.clone();
            let mut ancestor = owner;
            let mut visited = std::collections::HashSet::new();
            while visited.insert(ancestor) {
                let Some(parent) = self.declaring_function(ancestor, program, functions) else { break; };
                ancestor = parent;
                let Some(source) = self.completed.get(&parent) else { continue; };
                let start = outer.bindings.len();
                for binding in &source.bindings {
                    let mut binding = binding.clone();
                    for root in &mut binding.view_roots { *root += start; }
                    outer.bindings.push(binding);
                }
                for (&source, &slot) in &source.source_bindings {
                    outer.source_bindings.entry(source).or_insert(slot + start);
                }
            }
            let request = self.requests[&index];
            let plan = &program.local_capture_plans[request.plan];
            if plan.requirements(request.node).iter()
                .filter(|capture| self.encloses(capture.binding.owner, request.owner))
                .any(|capture| !outer.source_bindings.contains_key(&capture.binding.declaration_start)) {
                continue;
            }
            self.prepare(index, &outer, program)?;
            ready.push(index);
        }
        Ok(ready)
    }
    pub(super) fn needs_context(&self, index: usize) -> bool {
        self.requests.contains_key(&index)
    }
    pub(super) fn ready(&self, index: usize) -> bool {
        self.prepared.contains_key(&index)
    }
    pub(super) fn prepare(
        &mut self,
        index: usize,
        outer: &Context,
        program: &Program,
    ) -> Result<(), Diagnostic> {
        let Some(&request) = self.requests.get(&index) else {
            return Ok(());
        };
        if self.prepared.contains_key(&index) {
            return Ok(());
        }
        let Some((context, identities)) = self.project_request(request, outer, program)? else { return Ok(()); };
        self.identities.insert(index, identities);
        self.prepared.insert(index, context);
        Ok(())
    }
    /// Project a generic declaration's lexical captures before selecting any
    /// invocation types. This does not allocate instances or materialize loans.
    pub(super) fn source_context(
        &self, declaration: &str, outer: &Context, program: &Program,
    ) -> Result<Option<(Context, Vec<crate::local_capture_plan::BindingId>)>, Diagnostic> {
        let Some(&request) = self.declared.get(crate::generic_functions::lexical_declaration(declaration)) else { return Ok(None); };
        self.project_request(request, outer, program)
    }

    fn project_request(
        &self, request: Request, outer: &Context, program: &Program,
    ) -> Result<Option<(Context, Vec<crate::local_capture_plan::BindingId>)>, Diagnostic> {
        let plan = &program.local_capture_plans[request.plan];
        let captures = plan
            .requirements(request.node)
            .into_iter()
            .filter(|capture| self.encloses(capture.binding.owner, request.owner))
            .collect::<Vec<_>>();
        if captures.is_empty() {
            return Ok(None);
        }
        let mut slots = Vec::with_capacity(captures.len());
        for capture in &captures {
            let Some(&slot) = outer
                .source_bindings
                .get(&capture.binding.declaration_start)
            else {
                return Err(plan.pending_diagnostic().expect("required capture"));
            };
            slots.push(slot);
        }
        let mut context = capture_context::project(outer, &slots);
        // Transitive dependencies need typed slots, but must not introduce their
        // spellings into this body's lexical name map. A peer can capture a
        // different binding with the same spelling as this function's parameter.
        context.names.clear();
        for capture in &plan.direct[request.node - plan.node_start] {
            if let Some(&slot) = context
                .source_bindings
                .get(&capture.binding.declaration_start)
            {
                context.names.insert(capture.name.clone(), slot);
            }
        }
        let identities = captures.iter().map(|capture| capture.binding).collect();
        Ok(Some((context, identities)))
    }
    pub(super) fn access(
        &self, index: usize, checked: Option<&CheckedFunction>,
        assumed: &named_capture_access::Lattice, parameter_count: usize,
    ) -> Vec<named_capture_access::Access> {
        let Some(source) = self.prepared.get(&index) else { return Vec::new(); };
        self.identities[&index].iter().enumerate().map(|(inner, &binding)| {
            let slot = parameter_count + inner;
            let initial = if source.bindings[inner].ty.is_copy() { Mode::Copy } else { Mode::Read };
            let mode = checked.map_or_else(|| assumed.mode(index, binding, initial), |function| function.bindings[slot].mode);
            named_capture_access::Access { binding, slot, mode }
        }).collect()
    }
    pub(super) fn summaries(&self, checked: &[Option<CheckedFunction>]) -> HashMap<usize, Vec<named_capture_access::Access>> {
        self.prepared.keys().map(|&index| {
            let function = checked[index].as_ref().expect("checked named capture body");
            (index, self.access(index, Some(function), &Default::default(), function.parameter_count))
        }).collect()
    }

    pub(super) fn attach(&self, index: usize, target: &mut Context) {
        let Some(source) = self.prepared.get(&index) else {
            return;
        };
        let start = target.bindings.len();
        for binding in &source.bindings {
            let mut binding = binding.clone();
            for root in &mut binding.view_roots {
                *root += start;
            }
            target.bindings.push(binding);
        }
        target.source_bindings.extend(
            source
                .source_bindings
                .iter()
                .map(|(&source, &slot)| (source, slot + start)),
        );
        target.names.extend(
            source
                .names
                .iter()
                .map(|(name, &slot)| (name.clone(), slot + start)),
        );
        target.lexical_scopes = source.lexical_scopes.clone();
        target.closure_depth = source.closure_depth;
        target.named_capture_range = (start, target.bindings.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_source_context_projects_lexical_identity_without_instances_or_loans() {
        let source = "function main(){let count=0;const label='tag';function leaf<T>(value:T):T{count+=1;console.log(label);return value;}function forward<T>(value:T):T{return leaf(value);}}";
        let analyzer = Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let main = analyzer.function_names["main"];
        let mut outer = Context::new("main".into());
        for (statement, ty) in analyzer.program.functions[main].body.iter().take(2).zip([Type::Int, Type::String]) {
            let StmtKind::Let { name, rebindable, .. } = &statement.kind else { panic!("defining binding"); };
            let slot = Analyzer::bind(&mut outer, name, ty, false).unwrap();
            outer.bindings[slot].rebindable = *rebindable;
        }
        let declaration = analyzer.program.local_declarations.iter().find(|declaration| declaration.source_name.text == "forward").unwrap();
        let before = analyzer.program.functions.len();
        let (context, identities) = analyzer.named_captures.source_context(&declaration.emitted_name, &outer, &analyzer.program).unwrap().unwrap();
        assert_eq!(context.bindings.len(), 2);
        assert_eq!(identities.len(), 2);
        assert!(context.names.is_empty(), "transitive captures must not introduce lexical names");
        for identity in identities {
            let original = outer.source_bindings[&identity.declaration_start];
            let projected = context.source_bindings[&identity.declaration_start];
            assert_eq!(context.bindings[projected].ty, outer.bindings[original].ty);
        }
        assert!(outer.capture_loans.is_empty());
        assert!(outer.bindings.iter().all(|binding| binding.transferred.is_none() && binding.used.is_none()));
        assert_eq!(analyzer.program.functions.len(), before);
        assert!(analyzer.named_captures.prepared.is_empty());
        let missing = Context::new("main".into());
        assert!(analyzer.named_captures.source_context(&declaration.emitted_name, &missing, &analyzer.program).is_err());
        assert!(analyzer.named_captures.source_context("main", &outer, &analyzer.program).unwrap().is_none());
    }

    #[test]
    fn forwarder_inherits_mutating_capture_access_without_adding_source_arguments() {
        let source = "function main(){let value=0;function leaf():int{value+=1;return value;}function forward():int{return leaf();}}";
        let mut analyzer = Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let main = analyzer.function_names["main"];
        let StmtKind::Let { name, .. } = &analyzer.program.functions[main].body[0].kind else { panic!("binding"); };
        let mut outer = Context::new("main".into());
        let slot = Analyzer::bind(&mut outer, name, Type::Int, false).unwrap();
        outer.bindings[slot].rebindable = true;
        let declaration = analyzer.program.local_declarations.iter().find(|declaration| declaration.source_name.text == "forward").unwrap();
        let index = analyzer.function_names[&declaration.emitted_name];
        // Specialized declarations retain their request even when the emitted
        // copy name has no entry in the original lexical declaration catalog.
        let copy = usize::MAX;
        analyzer.named_captures.specialized(copy, index);
        analyzer.named_captures.prepare(copy, &outer, &analyzer.program).unwrap();
        analyzer.named_captures.prepare(index, &outer, &analyzer.program).unwrap();
        assert_eq!(analyzer.named_captures.identities[&copy], analyzer.named_captures.identities[&index]);
        analyzer.function(index).unwrap();
        let checked = analyzer.checked[index].as_ref().unwrap();
        assert_eq!(checked.parameter_count, 0);
        assert_eq!(checked.bindings[0].mode, Mode::Mutate);
        assert!(checked.bindings[0].used.is_some());
        assert!(!outer.bindings[slot].mutable);
        assert!(outer.capture_loans.is_empty());
    }

    #[test]
    fn checked_capture_access_is_separate_from_the_source_parameter_abi() {
        for (source, ty, access) in [
            (
                "function main(){const value=\"hi\";function read(step:int):int{return value.length+step;}}",
                Type::String,
                Mode::Read,
            ),
            (
                "function main(){const value=[0];function read(step:int):int{value[0]+=step;return value[0];}}",
                Type::Array(ArrayElement::Int(0)),
                Mode::Mutate,
            ),
            (
                "function main(){let value=0;function read(step:int):int{value+=step;return value;}}",
                Type::Int,
                Mode::Mutate,
            ),
            (
                "function main(){const value=\"hi\";function read(step:int=value.length):int{return step;}}",
                Type::String,
                Mode::Read,
            ),
        ] {
            let mut analyzer = Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
            let main = analyzer.function_names["main"];
            let StmtKind::Let {
                name, rebindable, ..
            } = &analyzer.program.functions[main].body[0].kind
            else {
                panic!("defining slot");
            };
            let mut outer = Context::new("main".into());
            let slot = Analyzer::bind(&mut outer, name, ty, false).unwrap();
            outer.bindings[slot].rebindable = *rebindable;
            let declaration = &analyzer.program.local_declarations[0];
            let index = analyzer.function_names[&declaration.emitted_name];
            analyzer
                .named_captures
                .prepare(index, &outer, &analyzer.program)
                .unwrap();
            analyzer.function(index).unwrap();
            let checked = analyzer.checked[index].as_ref().unwrap();
            assert_eq!(checked.parameter_count, 1);
            assert_eq!(checked.bindings[0].name, "step");
            if checked.bindings[0].omission == crate::ParameterOmission::Defaulted {
                assert!(checked.bindings[0].initializer.is_some());
                let input = checked.bindings[0].input_ty.unwrap();
                assert_eq!(
                    analyzer.unions.members(input.union().unwrap()),
                    vec![Type::Int, Type::Undefined]
                );
            } else {
                assert_eq!(checked.bindings[0].input_ty, Some(Type::Int));
            }
            assert_eq!(checked.bindings[0].mode, Mode::Copy);
            assert_eq!(checked.bindings[1].name, "value");
            assert_eq!(checked.bindings[1].ty, ty);
            assert_eq!(checked.bindings[1].mode, access);
            assert!(checked.bindings[1].used.is_some());
            assert!(checked.bindings[1].input_ty.is_none());
            assert!(outer.capture_loans.is_empty());
            assert!(outer.bindings[0].transferred.is_none());
        }
    }
}
