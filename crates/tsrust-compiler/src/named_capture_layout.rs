//! Finite named-group storage plans; runtime placement and loans are separate.
use super::*;
use crate::local_capture_plan::BindingId;

#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub binding: BindingId,
    pub ty: Type,
    pub storage: Mode,
    pub access: Mode,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Source {
    Field(usize),
    /// This value belongs to a member invocation, not the shared environment.
    Invocation(BindingId),
}
#[derive(Clone, Debug)]
pub(crate) struct Member {
    pub function: usize,
    pub slots: Vec<(usize, Source)>,
}
#[derive(Clone, Debug)]
pub(crate) struct Layout {
    pub plan: usize,
    pub group: usize,
    pub fields: Vec<Field>,
    pub members: Vec<Member>,
}

pub(super) fn build(program: &CheckedProgram) -> Result<Vec<Layout>, Diagnostic> {
    let functions: HashMap<&str, usize> = program.functions.iter().enumerate()
        .map(|(index, function)| (function.name.as_str(), index)).collect();
    let mut layouts = Vec::new();
    for (plan_id, plan) in program.local_capture_plans.iter().enumerate() {
        'groups: for (group_id, group) in plan.groups.iter().enumerate() {
            let owners: HashSet<usize> = group.members.iter()
                .map(|&member| program.local_declarations[member].function_owner).collect();
            let mut fields: BTreeMap<BindingId, Field> = BTreeMap::new();
            let mut members = Vec::new();
            for &node in &group.members {
                let declaration = &program.local_declarations[node];
                // A generic declaration is checked only through its instances;
                // each instance receives its captures as call arguments.
                let Some(&function) = functions.get(declaration.emitted_name.as_str()) else {
                    continue 'groups;
                };
                let accesses = program.named_capture_access.get(&function).map(Vec::as_slice).unwrap_or(&[]);
                for access in accesses {
                    if owners.contains(&access.binding.owner) { continue; }
                    let Some(binding) = program.functions[function].bindings.get(access.slot) else {
                        return Err(Diagnostic::new("capture-layout", "Named capture layout has no checked binding slot.", declaration.source_name.span));
                    };
                    let field = fields.entry(access.binding).or_insert(Field {
                        binding: access.binding, ty: binding.ty, storage: Mode::Copy, access: Mode::Copy,
                    });
                    if field.ty != binding.ty {
                        return Err(Diagnostic::new("capture-layout", "Recursive members disagree about the type of one captured binding.", declaration.source_name.span));
                    }
                    field.access = field.access.max(access.mode);
                    field.storage = if field.ty.is_copy() && field.access != Mode::Mutate { Mode::Copy } else { field.access };
                }
                members.push((function, accesses));
            }
            if members.iter().all(|(_, accesses)| accesses.is_empty()) { continue; }
            let fields: Vec<_> = fields.into_values().collect();
            let positions: HashMap<BindingId, usize> = fields.iter().enumerate().map(|(index, field)| (field.binding, index)).collect();
            let members = members.into_iter().map(|(function, accesses)| Member {
                function,
                slots: accesses.iter().map(|access| (access.slot,
                    if owners.contains(&access.binding.owner) { Source::Invocation(access.binding) }
                    else { Source::Field(positions[&access.binding]) })).collect(),
            }).collect();
            layouts.push(Layout { plan: plan_id, group: group_id, fields, members });
        }
    }
    Ok(layouts)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recursive_invocation_parameters_are_not_lifted_into_shared_storage() {
        let source = "function main(){function outer(n:int):int{function child():int{if(n===0){return n;}return outer(n-1);}return child();}console.log(outer(2));}";
        let program = crate::syntax::parse(source).unwrap();
        let initial = Analyzer::new(program.clone()).unwrap();
        let modes = initial.signatures.iter().map(|signature| signature.params.iter().map(|ty| if ty.is_copy(){Mode::Copy}else{Mode::Read}).collect()).collect();
        let mut analyzer = Analyzer::new(program).unwrap();
        analyzer.assumed_modes = Some(modes);
        analyzer.effects_only = true;
        analyzer.speculative_access_run = true;
        let checked = analyzer.run().unwrap();
        let layouts = build(&checked).unwrap();
        assert_eq!(layouts.len(), 1);
        let layout = &layouts[0];
        assert_eq!(layout.plan, 0);
        assert_eq!(layout.group, checked.local_capture_plans[0].group_of[0]);
        assert!(layout.fields.is_empty());
        assert_eq!(layout.members.len(), 2);
        let slots: Vec<_> = layout.members.iter().flat_map(|member| &member.slots).collect();
        assert_eq!(slots.len(), 1);
        assert!(matches!(slots[0].1, Source::Invocation(_)));
    }

    #[test]
    fn recursive_members_share_unique_typed_fields_with_no_callable_cycle() {
        let source = "function main(){let count=0;const values=[3];function left(n:int):int{if(n===0){count+=1;return values[0]+count;}return right(n-1);}function right(n:int):int{return left(n);}console.log(right(2));}";
        let program = crate::syntax::parse(source).unwrap();
        let initial = Analyzer::new(program.clone()).unwrap();
        let modes: Vec<Vec<Mode>> = initial.signatures.iter().map(|signature| signature.params.iter().map(|ty| if ty.is_copy(){Mode::Copy}else{Mode::Read}).collect()).collect();
        let mut lattice = named_capture_access::Lattice::default();
        let mut result = None;
        for _ in 0..8 {
            let mut analyzer = Analyzer::new(program.clone()).unwrap();
            analyzer.assumed_modes = Some(modes.clone());
            analyzer.assumed_capture_access = lattice.clone();
            analyzer.effects_only = true;
            analyzer.speculative_access_run = true;
            let checked = analyzer.run().unwrap();
            if !lattice.observe(&checked.named_capture_access).0 { result = Some(checked); break; }
        }
        let checked = result.expect("converged access");
        let layouts = build(&checked).unwrap();
        assert_eq!(layouts.len(), 1);
        let layout = &layouts[0];
        assert_eq!(layout.members.len(), 2);
        assert_eq!(layout.fields.len(), 2);
        assert_eq!(layout.fields[0].ty, Type::Int);
        assert_eq!(layout.fields[0].storage, Mode::Mutate);
        assert_eq!(layout.fields[1].ty, Type::Array(ArrayElement::Int(0)));
        assert_eq!(layout.fields[1].storage, Mode::Read);
        for member in &layout.members {
            assert_eq!(member.slots, vec![(1, Source::Field(0)), (2, Source::Field(1))]);
            assert_eq!(checked.functions[member.function].parameter_count, 1);
        }
        assert_ne!(layout.fields[0].binding, layout.fields[1].binding);
    }
}
