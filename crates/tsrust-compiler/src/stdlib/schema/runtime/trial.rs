//! Reversible typed projections bridge readonly callbacks without payload clones.
//! A consuming callback drops restoration state before it receives the value.
use super::issues::{Code, Issues, Kind, Segment};
use super::union::Preparation;

pub(super) trait Restore<I, O> {
    fn restore(self, value: O) -> I;
}
impl<I, O, F: FnOnce(O) -> I> Restore<I, O> for F {
    fn restore(self, value: O) -> I { self(value) }
}

/// A committed stage has no constructible restoration state.
pub(super) enum Committed {}
impl<I, O> Restore<I, O> for Committed {
    fn restore(self, _: O) -> I { match self {} }
}

pub(super) enum Trial<I, O, R, E> {
    Prepared { value: O, restoration: R },
    Rejected(I),
    /// Missing input with no default or presence rule. Only omission trials
    /// produce this state; the enclosing field supplies its missing-field issue.
    Absent,
    Committed(Option<O>),
    Aborted(E),
}

/// Ordinary checks borrow the original input. Only validated data enters a
/// reversible typed projection, using the already prepared defaults/conversions.
pub(super) fn prepare<I, P, O, R: Restore<I, O>, E>(
    input: I, issues: &mut Issues,
    inspect: impl FnOnce(&I, &mut Issues) -> Preparation<P, E>,
    project: impl FnOnce(I, P) -> (O, R),
) -> Trial<I, O, R, E> {
    match inspect(&input, issues) {
        Preparation::Ready(state) => {
            let (value, restoration) = project(input, state);
            Trial::Prepared { value, restoration }
        }
        Preparation::Rejected => Trial::Rejected(input),
        Preparation::Aborted(error) => Trial::Aborted(error),
    }
}

/// Conversion owns raw input immediately; receiving-schema rejection cannot
/// make the enclosing alternative eligible for retry.
pub(super) fn coerce<I, C, O, E>(
    input: I, convert: impl FnOnce(I) -> Result<C, E>,
    validate: impl FnOnce(C) -> Result<Option<O>, E>,
) -> Trial<I, O, Committed, E> {
    match convert(input).and_then(validate) {
        Ok(value) => Trial::Committed(value),
        Err(error) => Trial::Aborted(error),
    }
}

impl<I, O, R: Restore<I, O>, E> Trial<I, O, R, E> {
    /// Type-erased restoration is uniquely owned. Lifetimes remain bounded by
    /// the actual input, output and restoration state; no lifetime is extended.
    pub(super) fn boxed<'a>(self) -> Trial<I, O, Box<dyn FnOnce(O) -> I + 'a>, E>
    where I: 'a, O: 'a, R: 'a {
        match self {
            Self::Prepared { value, restoration } => Trial::Prepared {
                value, restoration: Box::new(move |value| restoration.restore(value)),
            },
            Self::Rejected(input) => Trial::Rejected(input),
            Self::Absent => Trial::Absent,
            Self::Committed(value) => Trial::Committed(value),
            Self::Aborted(error) => Trial::Aborted(error),
        }
    }

    /// Compiler-only representation injection, not a user transformation.
    /// Both directions move values; branch identity retains the exact inverse.
    pub(super) fn inject<'a, N: 'a>(
        self, inject: impl FnOnce(O) -> N,
        project: impl FnOnce(N) -> O + 'a,
    ) -> Trial<I, N, Box<dyn FnOnce(N) -> I + 'a>, E>
    where I: 'a, O: 'a, R: 'a {
        match self {
            Self::Prepared { value, restoration } => Trial::Prepared {
                value: inject(value), restoration: Box::new(move |value| restoration.restore(project(value))),
            },
            Self::Rejected(input) => Trial::Rejected(input),
            Self::Absent => Trial::Absent,
            Self::Committed(value) => Trial::Committed(value.map(inject)),
            Self::Aborted(error) => Trial::Aborted(error),
        }
    }

    /// Change only the enclosing input representation. This is a move-only
    /// compiler adapter, so it does not commit a user-visible conversion.
    pub(super) fn remap_input<'a, N: 'a>(
        self, wrap: impl FnOnce(I) -> N + 'a,
    ) -> Trial<N, O, Box<dyn FnOnce(O) -> N + 'a>, E>
    where I: 'a, O: 'a, R: 'a {
        match self {
            Self::Prepared { value, restoration } => Trial::Prepared {
                value, restoration: Box::new(move |value| wrap(restoration.restore(value))),
            },
            Self::Rejected(input) => Trial::Rejected(wrap(input)),
            Self::Absent => Trial::Absent,
            Self::Committed(value) => Trial::Committed(value),
            Self::Aborted(error) => Trial::Aborted(error),
        }
    }

    /// Apply one built-in constraint without committing a reversible branch.
    /// Only this check's issue receives its message; earlier failures pass through.
    pub(super) fn constraint(
        self, path: &[Segment<'_>], issues: &mut Issues, kind: Kind,
        message: Option<&str>, check: impl FnOnce(&O) -> Option<Code>,
    ) -> Self {
        let mut reject = |code| {
            let start = issues.items.len();
            issues.push(path, code, kind, kind);
            if let Some(message) = message { issues.message_since(start, message); }
        };
        match self {
            Self::Prepared { value, restoration } => match check(&value) {
                None => Self::Prepared { value, restoration },
                Some(code) => {
                    reject(code);
                    Self::Rejected(restoration.restore(value))
                }
            },
            Self::Committed(Some(value)) => match check(&value) {
                None => Self::Committed(Some(value)),
                Some(code) => { reject(code); Self::Committed(None) }
            },
            other => other,
        }
    }

    pub(super) fn refine(
        self, path: &[Segment<'_>], issues: &mut Issues, kind: Kind,
        predicate: impl FnOnce(&O) -> Result<bool, E>,
    ) -> Self {
        match self {
            Self::Prepared { value, restoration } => match predicate(&value) {
                Ok(true) => Self::Prepared { value, restoration },
                Ok(false) => {
                    issues.push(path, Code::Refinement, kind, kind);
                    Self::Rejected(restoration.restore(value))
                }
                Err(error) => Self::Aborted(error),
            },
            Self::Committed(Some(value)) => match predicate(&value) {
                Ok(true) => Self::Committed(Some(value)),
                Ok(false) => {
                    issues.push(path, Code::Refinement, kind, kind);
                    Self::Committed(None)
                }
                Err(error) => Self::Aborted(error),
            },
            other => other,
        }
    }

    /// A multi-issue refinement preserves restoration until its callback succeeds.
    pub(super) fn refine_issues(
        self, predicate: impl FnOnce(&O) -> Result<bool, E>,
    ) -> Self {
        match self {
            Self::Prepared { value, restoration } => match predicate(&value) {
                Ok(true) => Self::Prepared { value, restoration },
                Ok(false) => Self::Rejected(restoration.restore(value)),
                Err(error) => Self::Aborted(error),
            },
            Self::Committed(Some(value)) => match predicate(&value) {
                Ok(true) => Self::Committed(Some(value)),
                Ok(false) => Self::Committed(None),
                Err(error) => Self::Aborted(error),
            },
            other => other,
        }
    }

    pub(super) fn transform<N>(self, convert: impl FnOnce(O) -> Result<N, E>) -> Trial<I, N, Committed, E> {
        let value = match self {
            Self::Prepared { value, restoration } => {
                // No rollback after user code gains ownership, including throw.
                drop(restoration);
                value
            }
            Self::Committed(Some(value)) => value,
            Self::Committed(None) => return Trial::Committed(None),
            Self::Rejected(input) => return Trial::Rejected(input),
            Self::Absent => return Trial::Absent,
            Self::Aborted(error) => return Trial::Aborted(error),
        };
        match convert(value) {
            Ok(value) => Trial::Committed(Some(value)),
            Err(error) => Trial::Aborted(error),
        }
    }

    pub(super) fn finish(self) -> Result<Option<O>, E> {
        match self {
            Self::Prepared { value, .. } | Self::Committed(Some(value)) => Ok(Some(value)),
            Self::Rejected(_) | Self::Committed(None) | Self::Absent => Ok(None),
            Self::Aborted(error) => Err(error),
        }
    }
}

/// Validate a key before its value without cloning either input. A consuming
/// key callback commits the pair even if the value subsequently rejects.
pub(super) fn pair<'a, K:'a, V:'a, A:'a, B:'a, RK:Restore<K,A>+'a, RV:Restore<V,B>+'a, E>(
    left:Trial<K,A,RK,E>,
    right:V,
    value:impl FnOnce(V)->Trial<V,B,RV,E>,
)->Trial<(K,V),(A,B),Box<dyn FnOnce((A,B))->(K,V)+'a>,E>{
    let (left,restore_left)=match left{
        Trial::Prepared{value,restoration}=>(value,Some(restoration)),
        Trial::Committed(Some(value))=>(value,None),
        Trial::Rejected(left)=>return Trial::Rejected((left,right)),
        Trial::Committed(None)=>return Trial::Committed(None),
        Trial::Aborted(error)=>return Trial::Aborted(error),
        Trial::Absent=>unreachable!("collection keys are present"),
    };
    match value(right){
        Trial::Prepared{value:right,restoration:restore_right}=>match restore_left{
            Some(restore_left)=>Trial::Prepared{value:(left,right),restoration:Box::new(move |(left,right)|(restore_left.restore(left),restore_right.restore(right)))},
            None=>Trial::Committed(Some((left,right))),
        },
        Trial::Committed(right)=>Trial::Committed(right.map(|right|(left,right))),
        Trial::Rejected(right)=>match restore_left{
            Some(restore)=>Trial::Rejected((restore.restore(left),right)),
            None=>Trial::Committed(None),
        },
        Trial::Aborted(error)=>Trial::Aborted(error),
        Trial::Absent=>unreachable!("collection values are present"),
    }
}

/// Visit callback-bearing array elements in order. A rejected element stops
/// later callbacks. Earlier readonly projections are restored before fallback;
/// any earlier consuming callback makes the whole array committed instead.
pub(super) fn sequence<'a, I: 'a, O: 'a, R: Restore<I, O> + 'a, E>(
    input: Vec<I>, mut validate: impl FnMut(usize, I) -> Trial<I, O, R, E>,
) -> Trial<Vec<I>, Vec<O>, Box<dyn FnOnce(Vec<O>) -> Vec<I> + 'a>, E> {
    let mut values = Vec::with_capacity(input.len());
    let mut restores = Vec::with_capacity(input.len());
    let mut committed = false;
    let mut remaining = input.into_iter().enumerate();
    while let Some((index, input)) = remaining.next() {
        match validate(index, input) {
            Trial::Prepared { value, restoration } => {
                values.push(value);
                if committed { drop(restoration); } else { restores.push(restoration); }
            }
            Trial::Committed(Some(value)) => {
                committed = true;
                restores.clear();
                values.push(value);
            }
            Trial::Rejected(input) => {
                if committed { return Trial::Committed(None); }
                let mut restored: Vec<I> = values.into_iter().zip(restores)
                    .map(|(value, restore)| restore.restore(value)).collect();
                restored.push(input);
                restored.extend(remaining.map(|(_, input)| input));
                return Trial::Rejected(restored);
            }
            Trial::Absent => unreachable!("array element trials always have supplied input"),
            Trial::Committed(None) => return Trial::Committed(None),
            Trial::Aborted(error) => return Trial::Aborted(error),
        }
    }
    if committed { Trial::Committed(Some(values)) } else {
        Trial::Prepared {
            value: values,
            restoration: Box::new(move |values| values.into_iter().zip(restores)
                .map(|(value, restore)| restore.restore(value)).collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn sequence_restores_earlier_elements_and_skips_callbacks_after_rejection() {
        let input = vec![String::from("first"), String::from("reject"), String::from("unvisited")];
        let pointers = input.iter().map(|value| value.as_ptr()).collect::<Vec<_>>();
        let calls = Cell::new(0);
        let trial: Trial<Vec<String>, Vec<String>, _, ()> = sequence(input, |index, value| {
            calls.set(calls.get() + 1);
            if index == 1 { Trial::Rejected(value) }
            else { Trial::Prepared { value, restoration: |value| value } }
        });
        let Trial::Rejected(restored) = trial else { panic!("readonly sequence committed") };
        assert_eq!(calls.get(), 2);
        assert_eq!(restored.iter().map(|value| value.as_ptr()).collect::<Vec<_>>(), pointers);
    }

    #[test]
    fn sequence_propagates_commitment_and_never_restores_consumed_elements() {
        let restored = Cell::new(0);
        let calls = Cell::new(0);
        let trial: Trial<Vec<i32>, Vec<i32>, _, ()> = sequence(vec![1, 2, 3, 4], |index, value| {
            calls.set(calls.get() + 1);
            match index {
                0 => Trial::Prepared { value, restoration: |value| { restored.set(restored.get() + 1); value } },
                1 => Trial::Committed(Some(value + 1)),
                _ => Trial::Rejected(value),
            }
        });
        assert!(matches!(trial, Trial::Committed(None)));
        assert_eq!(calls.get(), 3);
        assert_eq!(restored.get(), 0);
    }

    #[test]
    fn borrowed_preparation_state_is_not_recomputed_for_projection() {
        let prepared = Cell::new(0);
        let mut issues = Issues::new(4, 4, 32);
        let result: Trial<String, _, _, ()> = prepare(String::from("input"), &mut issues,
            |input, _| { assert_eq!(input, "input"); prepared.set(prepared.get() + 1); Preparation::Ready(String::from("default")) },
            |input, default| ((input, default), |(input, _)| input));
        let result = result.refine(&[], &mut issues, Kind::Object, |(_, default)| {
            assert_eq!(default, "default"); Ok(false)
        });
        assert!(matches!(result, Trial::Rejected(input) if input == "input"));
        assert_eq!(prepared.get(), 1);
    }

    #[test]
    fn coercion_commits_before_validation_and_exceptions_skip_validation() {
        let validated = Cell::new(0);
        let result: Trial<Vec<i32>, usize, _, &str> = coerce(vec![1],
            |mut input| { input.push(2); Ok(input.len()) },
            |value| { validated.set(validated.get() + 1); assert_eq!(value, 2); Ok(None) });
        assert!(matches!(result, Trial::Committed(None)));
        let result: Trial<Vec<i32>, usize, _, &str> = coerce(vec![1],
            |_| Err::<usize, _>("conversion"),
            |_| { validated.set(validated.get() + 1); Ok(Some(0)) });
        assert!(matches!(result, Trial::Aborted("conversion")));
        assert_eq!(validated.get(), 1);
    }

    #[test]
    fn constraint_messages_preserve_codes_storage_and_previous_failures() {
        let input = String::from("original");
        let pointer = input.as_ptr();
        let trial: Trial<String, _, _, ()> = Trial::Prepared { value: input, restoration: |value| value };
        let mut issues = Issues::new(4, 4, 32);
        let trial = trial.constraint(&[], &mut issues, Kind::String, Some("minimum"), |_| Some(Code::TooSmall));
        let trial = trial.constraint(&[], &mut issues, Kind::String, Some("maximum"), |_| panic!("failed input checked again"));
        let Trial::Rejected(input) = trial else { panic!("constraint committed input") };
        assert_eq!(input.as_ptr(), pointer);
        assert_eq!(issues.items.len(), 1);
        assert_eq!(issues.items[0].code, Code::TooSmall);
        assert_eq!(issues.items[0].message.as_deref(), Some("minimum"));
    }

    #[test]
    fn constraint_after_transform_stays_committed_with_zero_issue_budget() {
        let trial: Trial<i64, _, _, ()> = Trial::Prepared { value: 7, restoration: |value| value };
        let mut issues = Issues::new(0, 0, 0);
        let trial = trial.transform(|value| Ok(value + 1))
            .constraint(&[], &mut issues, Kind::Integer, Some("too large"), |_| Some(Code::TooLarge));
        assert!(matches!(trial, Trial::Committed(None)));
        assert!(!issues.is_valid());
        assert!(issues.items.is_empty());
        assert!(issues.truncated);
    }

    #[test]
    fn output_injection_keeps_unique_restoration_and_real_borrow_lifetimes() {
        enum Output<'a> { Text(&'a str) }
        let owner = String::from("scoped input");
        let text = owner.as_str();
        let trial: Trial<&str, _, _, ()> = Trial::Prepared { value: text, restoration: |value| value };
        let trial = trial.inject(Output::Text, |value| match value { Output::Text(text) => text });
        let mut issues = Issues::new(1, 4, 32);
        let trial = trial.refine(&[], &mut issues, Kind::String, |_| Ok(false));
        let Trial::Rejected(restored) = trial else { panic!("injection committed input") };
        assert_eq!(restored.as_ptr(), owner.as_ptr());
    }

    #[test]
    fn rejected_readonly_projection_restores_original_string_allocation_and_exact_number() {
        struct Raw { text: String, number: i64 }
        struct Typed { text: String, number: f64 }
        let raw = Raw { text: "original storage".repeat(8), number: i64::MAX };
        let pointer = raw.text.as_ptr();
        let capacity = raw.text.capacity();
        let original_number = raw.number;
        let value = Typed { text: raw.text, number: raw.number as f64 };
        let restore = move |value: Typed| Raw { text: value.text, number: original_number };
        let trial: Trial<Raw, _, _, ()> = Trial::Prepared { value, restoration: restore };
        let mut issues = Issues::new(1, 4, 32);
        let trial = trial.refine(&[], &mut issues, Kind::Object, |value| {
            assert_eq!(value.text.as_ptr(), pointer);
            assert_eq!(value.number, i64::MAX as f64);
            Ok(false)
        });
        let Trial::Rejected(raw) = trial else { panic!("refinement committed input") };
        assert_eq!(raw.text.as_ptr(), pointer);
        assert_eq!(raw.text.capacity(), capacity);
        assert_eq!(raw.number, i64::MAX);
        assert_eq!(issues.items[0].code, Code::Refinement);
    }

    #[test]
    fn consuming_transform_prevents_restoration_and_retry_after_later_rejection() {
        let calls = Cell::new(0);
        let input = vec![1, 2];
        let trial: Trial<Vec<i32>, _, _, ()> = Trial::Prepared {
            value: input, restoration: |value| { calls.set(calls.get() + 1); value },
        };
        let mut issues = Issues::new(1, 4, 32);
        let result = trial.transform(|mut value| { value.push(3); Ok(value.len()) })
            .refine(&[], &mut issues, Kind::Integer, |_| Ok(false));
        assert!(matches!(result, Trial::Committed(None)));
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn exception_drops_projected_payload_and_restoration_storage_once() {
        struct Payload<'a>(&'a Cell<usize>);
        impl Drop for Payload<'_> { fn drop(&mut self) { self.0.set(self.0.get() + 1); } }
        let payload_drops = Cell::new(0);
        let restoration_drops = Cell::new(0);
        let metadata = Payload(&restoration_drops);
        let trial: Trial<_, _, _, &str> = Trial::Prepared {
            value: Payload(&payload_drops), restoration: move |value| { drop(metadata); value },
        };
        let mut issues = Issues::new(0, 0, 0);
        let result = trial.refine(&[], &mut issues, Kind::Object, |_| Err("callback"));
        assert!(matches!(result, Trial::Aborted("callback")));
        assert_eq!(payload_drops.get(), 1);
        assert_eq!(restoration_drops.get(), 1);
        assert!(issues.is_valid());
    }
}
