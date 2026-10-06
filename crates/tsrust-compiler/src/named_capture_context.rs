//! Bind planned named captures to checked defining slots without materializing loans.
use super::*;

struct Request {
    plan: usize,
    node: usize,
    owner: usize,
}
#[derive(Default)]
pub(super) struct Contexts {
    requests: HashMap<usize, Request>,
    prepared: HashMap<usize, Context>,
    identities: HashMap<usize, Vec<crate::local_capture_plan::BindingId>>,
}
impl Contexts {
    pub(super) fn new(program: &Program, functions: &HashMap<String, usize>) -> Self {
        let mut contexts = Self::default();
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
                    .any(|binding| binding.owner != declaration.function_owner)
                {
                    continue;
                }
                let Some(&function) = functions.get(&declaration.emitted_name) else {
                    continue;
                };
                contexts.requests.insert(
                    function,
                    Request {
                        plan: plan_id,
                        node: declaration.id,
                        owner: declaration.function_owner,
                    },
                );
            }
        }
        contexts
    }
    /// The function whose body declares this capturing function. Checking
    /// it first lets its calls prepare the typed capture context.
    pub(super) fn declaring_function(&self, index: usize, program: &Program, functions: &HashMap<String, usize>) -> Option<usize> {
        let request = self.requests.get(&index)?;
        let declaration = program.local_declarations.iter().find(|declaration| declaration.id == request.node)?;
        let name = program.local_capture_plans[request.plan].owners.get(&declaration.enclosing_owner)?;
        functions.get(name).copied()
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
        let Some(request) = self.requests.get(&index) else {
            return Ok(());
        };
        if self.prepared.contains_key(&index) {
            return Ok(());
        }
        let plan = &program.local_capture_plans[request.plan];
        let captures = plan
            .requirements(request.node)
            .into_iter()
            .filter(|capture| capture.binding.owner != request.owner)
            .collect::<Vec<_>>();
        if captures.is_empty() {
            return Ok(());
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
        self.identities.insert(index, captures.iter().map(|capture| capture.binding).collect());
        self.prepared.insert(index, context);
        Ok(())
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
        analyzer.named_captures.prepare(index, &outer, &analyzer.program).unwrap();
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
