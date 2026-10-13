//! Ordered plan identities are separate from normalized schema output types.
use crate::syntax::Type;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Tag { String(String), Integer(i64), Number(u64), Boolean(bool), Null }
impl Tag {
    pub(crate) fn literal(value:&crate::literal_types::LiteralValue)->Self {
        use crate::literal_types::LiteralValue;
        match value {
            LiteralValue::String(value)=>Self::String(value.clone()),
            LiteralValue::Int(value)=>Self::Integer(*value),
            LiteralValue::Number(bits)=>Self::number(f64::from_bits(*bits)),
            LiteralValue::Boolean(value)=>Self::Boolean(*value),
        }
    }
    /// JSON integer and floating spellings can denote the same discriminator.
    pub(crate) fn number(value: f64) -> Self {
        let integer = value as i128;
        if value.is_finite() && integer >= i64::MIN as i128 && integer <= i64::MAX as i128 && integer as f64 == value {
            Self::Integer(integer as i64)
        } else {
            Self::Number(if value == 0.0 { 0.0f64.to_bits() } else { value.to_bits() })
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Selector {
    pub field: String,
    /// One nonempty set per branch, in declaration order.
    pub tags: Vec<Vec<Tag>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Choice {
    pub output: Type,
    /// Do not sort or deduplicate. Two branches may have the same output type.
    pub branches: Vec<Type>,
    pub selector: Option<Selector>,
    /// Presence adapters keep an existing runtime plan intact while adding a tag.
    /// They become active only when the source output can contain a choice plan.
    pub presence: Option<Type>,
}
#[derive(Default)]
pub(crate) struct Choices { plans: Vec<Choice> }
impl Choices {
    pub(crate) fn is_empty(&self) -> bool { !self.plans.iter().any(|plan|plan.presence.is_none()) }
    pub(crate) fn intern(&mut self, plan: Choice) -> usize {
        if let Some(index) = self.plans.iter().position(|existing| existing == &plan) { return index; }
        let index = self.plans.len();
        self.plans.push(plan);
        index
    }
    pub(crate) fn get(&self, index: usize) -> &Choice { &self.plans[index] }
    pub(crate) fn for_output(&self, output: Type) -> impl Iterator<Item = (usize, &Choice)> {
        self.plans.iter().enumerate().filter(move |(_, plan)| plan.output == output && self.active(plan))
    }
    fn active(&self, plan:&Choice)->bool {
        plan.presence.is_none() || self.plans.iter().any(|source|source.output==plan.branches[0] && self.active(source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminator_keys_preserve_exact_integer_boundaries() {
        assert_eq!(Tag::number(-0.0),Tag::Integer(0));
        assert_eq!(Tag::number(7.0),Tag::Integer(7));
        assert_eq!(Tag::number(i64::MIN as f64),Tag::Integer(i64::MIN));
        assert_ne!(Tag::number(i64::MAX as f64),Tag::Integer(i64::MAX));
        let exact=9_007_199_254_740_992i64;
        assert_eq!(Tag::number(exact as f64),Tag::Integer(exact));
        assert_ne!(Tag::number((exact+1) as f64),Tag::Integer(exact+1));
        assert_eq!(Tag::number(1.5),Tag::Number(1.5f64.to_bits()));
    }

    #[test]
    fn plan_identity_retains_branch_order_and_repeated_output_types() {
        let mut choices=Choices::default();
        let forward=Choice{output:Type::String,branches:vec![Type::String,Type::Int,Type::String],selector:None,presence:None};
        let first=choices.intern(forward.clone());
        assert_eq!(choices.intern(forward),first);
        let reverse=choices.intern(Choice{output:Type::String,branches:vec![Type::Int,Type::String,Type::String],selector:None,presence:None});
        assert_ne!(first,reverse);
        assert_eq!(choices.get(first).branches,[Type::String,Type::Int,Type::String]);
        assert_eq!(choices.for_output(Type::String).count(),2);
    }

    #[test]
    fn dormant_presence_adapter_activates_when_source_choice_arrives() {
        // Stand-in types exercise registry activation independently of the arena.
        let mut choices=Choices::default();
        choices.intern(Choice{output:Type::Number,branches:vec![Type::String],selector:None,presence:Some(Type::Null)});
        assert!(choices.is_empty());
        assert_eq!(choices.for_output(Type::Number).count(),0);
        choices.intern(Choice{output:Type::String,branches:vec![Type::String,Type::String],selector:None,presence:None});
        assert!(!choices.is_empty());
        assert_eq!(choices.for_output(Type::Number).count(),1);
    }
}
