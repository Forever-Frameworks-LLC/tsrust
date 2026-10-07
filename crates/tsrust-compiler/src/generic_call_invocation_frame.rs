//! Contextual invocation slots stay separate from captured caller bindings.
use super::*;
pub(super) struct Frame {
    pub state: Vec<Binding>,
    caller: Vec<Binding>,
    owned: HashSet<crate::local_capture_plan::BindingId>,
}
impl Frame {
    pub(super) fn build(declaration: Span, caller: &[Binding], arguments: &[expression_flow::Value], effects: &effect_bodies::Index) -> Option<Self> {
        let parameters = effects.parameters(declaration)?;
        let body = effects.body(declaration)?;
        let mut slots = parameters.iter().enumerate().map(|(position, parameter)| {
            let mut binding = parameter.binding.clone();
            if !parameter.rest && let Some(argument) = arguments.get(position) {
                binding.origins = body_origins::canonical(argument.origins.clone());
                binding.selection = argument.selection();
                binding.elements = argument.elements.clone();
                binding.targets = argument.contracts.clone().map_or_else(|| vec![Target::Unknown], |contracts| contracts.into_iter().map(Target::Contract).collect());
            }
            binding
        }).collect::<Vec<_>>();
        collect(body, &mut slots, effects);
        for local in &mut slots[parameters.len()..] {
            local.origins = vec![body_origins::Origin::Unknown];
            local.selection = Selection::default();
        local.elements = None;
            local.targets = vec![Target::Unknown];
        }
        let owned = slots.iter().filter_map(|binding| binding.identity).collect::<HashSet<_>>();
        let mut state = caller.iter().filter(|binding| !binding.identity.is_some_and(|identity| owned.contains(&identity))).cloned().collect::<Vec<_>>();
        state.extend(slots);
        Some(Self { state, caller: caller.to_vec(), owned })
    }
    pub(super) fn project(&self, outgoing: &[Binding]) -> Vec<Binding> {
        let mut updated_by_identity = HashMap::new();
        for binding in outgoing {
            if let Some(identity) = binding.identity { updated_by_identity.entry(identity).or_insert(binding); }
        }
        let mut caller = self.caller.clone();
        for binding in &mut caller {
            let Some(identity) = binding.identity else { continue; };
            if self.owned.contains(&identity) { continue; }
            if let Some(updated) = updated_by_identity.get(&identity) {
                binding.origins = updated.origins.clone();
                binding.selection = updated.selection;
                binding.elements = updated.elements.clone();
                binding.targets = updated.targets.clone();
            }
        }
        caller
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn supplied_callback_bodies_do_not_replace_formal_signatures_or_caller_slots() {
        let source=crate::format("function one():int{return 1;}function two():int{return 2;}function relay(action:Function<int>):int{const local=action;return local();}function main(){let local=one;let shared=one;}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let main=program.functions.iter().find(|function|function.name.text=="main").unwrap();
        let relay=program.functions.iter().find(|function|function.name.text=="relay").unwrap();
        let one=program.functions.iter().find(|function|function.name.text=="one").unwrap().name.span;
        let two=program.functions.iter().find(|function|function.name.text=="two").unwrap().name.span;
        let mut caller=Vec::new();collect(&main.body,&mut caller,&effects);
        let argument=expression_flow::Value::from_origins(vec![body_origins::Origin::Body(one)]);
        let frame=Frame::build(relay.name.span,&caller,&[argument],&effects).unwrap();
        let parameter=frame.state.iter().find(|binding|binding.name=="action").unwrap();
        assert_eq!(parameter.origins,vec![body_origins::Origin::Body(one)]);
        assert_eq!(parameter.contract.as_ref().unwrap().required,0);
        let own=frame.state.iter().find(|binding|binding.name=="local" && binding.identity!=caller[0].identity).unwrap();
        assert_eq!(own.origins,vec![body_origins::Origin::Unknown]);
        let mut outgoing=frame.state.clone();
        for binding in &mut outgoing {
            if binding.name=="action" || binding.identity==own.identity || binding.name=="shared" { binding.origins=vec![body_origins::Origin::Body(two)]; }
        }
        let projected=frame.project(&outgoing);
        assert_eq!(projected.len(),caller.len());
        assert_eq!(projected[0].origins,caller[0].origins);
        assert_eq!(projected[1].origins,vec![body_origins::Origin::Body(two)]);
        let recursive=Frame::build(relay.name.span,&frame.state,&[expression_flow::Value::from_origins(vec![body_origins::Origin::Body(two)])],&effects).unwrap();
        let active=recursive.state.iter().find(|binding|binding.name=="action").unwrap();
        assert_eq!(active.origins,vec![body_origins::Origin::Body(two)]);
        let restored=recursive.project(&recursive.state);
        assert_eq!(restored.iter().find(|binding|binding.name=="action").unwrap().origins,vec![body_origins::Origin::Body(one)]);
    }
}

#[cfg(test)]
mod parameter_metadata_tests {
    use super::*;
    #[test]
    fn arrows_defaults_and_rest_keep_body_and_parameter_metadata_distinct() {
        let source=crate::format("function relay(action:Function<int>=():int=>7,...extra:Function<int>[]):int{return action();}function main(){const arrow=(action:Function<int>):int=>action();}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let relay=program.functions.iter().find(|function|function.name.text=="relay").unwrap();
        let parameters=effects.parameters(relay.name.span).unwrap();
        assert_eq!(parameters.len(),2);
        assert!(parameters[1].rest);
        let ParameterInitializer::Default {value,..}=&parameters[0].initializer else {panic!("default metadata")};
        assert!(effects.body(value.span).is_some());
        let omitted=Frame::build(relay.name.span,&[],&[],&effects).unwrap();
        assert!(omitted.state.iter().all(|binding|binding.origins==vec![body_origins::Origin::Unknown]));
        let arguments=vec![expression_flow::Value::from_origins(vec![body_origins::Origin::Body(value.span)]);2];
        let supplied=Frame::build(relay.name.span,&[],&arguments,&effects).unwrap();
        assert_eq!(supplied.state[0].origins,arguments[0].origins);
        assert_eq!(supplied.state[1].origins,vec![body_origins::Origin::Unknown]);
        let main=program.functions.iter().find(|function|function.name.text=="main").unwrap();
        let StmtKind::Let {value:arrow,..}=&main.body[0].kind else {panic!("arrow")};
        let arrow_frame=Frame::build(arrow.span,&[],&arguments,&effects).unwrap();
        assert_eq!(arrow_frame.state[0].contract.as_ref().unwrap().required,0);
        assert_eq!(arrow_frame.state[0].origins,arguments[0].origins);
    }
}
