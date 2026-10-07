//! Ordered parameter entry preserves default effects and exceptional completion.
use super::*;
pub(super) struct Entry {
    pub normal: bool,
    pub failures: Vec<Vec<Binding>>,
    pub facts: HashMap<(usize, usize), Option<Vec<Contract>>>,
}
fn install(binding: &mut Binding, value: &expression_flow::Value) {
    binding.origins = body_origins::canonical(value.origins.clone());
    binding.selection = value.selection();
    binding.targets = value.contracts.clone().map_or_else(|| vec![Target::Unknown], |contracts| contracts.into_iter().map(Target::Contract).collect());
}
pub(super) fn prepare(declaration: Span, arguments: &[expression_flow::Value], state: &mut Vec<Binding>, declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index, dispatch: &mut dyn expression_flow::Dispatch) -> Option<Entry> {
    let parameters = effects.parameters(declaration)?;
    // Aggregate element provenance is not represented yet. Refuse a proof rather
    // than treating a callable rest array as an effect-free parameter.
    if parameters.iter().any(|parameter| parameter.rest) { return None; }
    let mut entry = Entry { normal: true, failures: Vec::new(), facts: HashMap::new() };
    for parameter in parameters {
        let index = state.iter().position(|binding| binding.identity == parameter.binding.identity)?;
        install(&mut state[index], &expression_flow::Value::default());
    }
    for (position, parameter) in parameters.iter().enumerate() {
        let index = state.iter().position(|binding| binding.identity == parameter.binding.identity)?;
        let supplied = arguments.get(position);
        let value = supplied.cloned().unwrap_or_else(expression_flow::Value::undefined);
        match &parameter.initializer {
            ParameterInitializer::Default { value: default, .. } if value.selection().undefined != Some(false) => {
                let mut default_state = state.clone();
                let evaluation = expression_flow::evaluate_with_dispatch(default, &mut default_state, &mut entry.facts, declarations, true, effects, dispatch);
                entry.failures.extend(evaluation.failures);
                if evaluation.normal { install(&mut default_state[index], &evaluation.value); }
                if value.selection().undefined == Some(true) {
                    if !evaluation.normal { entry.normal = false; return Some(entry); }
                    *state = default_state;
                } else {
                    install(&mut state[index], &value);
                    if evaluation.normal { join(state, &default_state, declarations); }
                }
            }
            _ => install(&mut state[index], &value),
        }
    }
    Some(entry)
}
