//! Checked flow facts describe stable source places, never native storage types.
use crate::syntax::{Expr, ExprKind, Type};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Place {
    pub root: usize,
    pub fields: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Fact {
    pub members: Vec<Type>,
    pub origin: Option<(usize, usize)>,
}
pub(crate) type Facts = HashMap<Place, Fact>;
pub(crate) type Outcomes = (Facts, Facts);

pub(crate) fn place(expr: &Expr, names: &HashMap<String, usize>) -> Option<Place> {
    match &expr.kind {
        ExprKind::Sequence(values)=>values.last().and_then(|last|place(last,names)),
        ExprKind::Name(name) => Some(Place {
            root: expr.binding.or_else(|| names.get(name).copied())?,
            fields: Vec::new(),
        }),
        ExprKind::Field(base, name) => {
            let mut place = place(base, names)?;
            place.fields.push(name.clone());
            Some(place)
        }
        ExprKind::Project { value, .. } => place(value, names),
        ExprKind::TypeAssertion { value, .. } if expr.ty == value.ty => place(value, names),
        ExprKind::Selected { source, .. } => source.place.as_ref().map(|(root, fields)| Place {
            root: *root,
            fields: fields.clone(),
        }),
        // An injection constructs a new owning tag rather than an alias.
        _ => None,
    }
}
/// Writes through a dynamic index affect its containing array, not disjoint fields.
/// This is an invalidation footprint, never a reusable guard identity.
pub(crate) fn mutation_footprint(expr: &Expr, names: &HashMap<String, usize>) -> Option<Place> {
    if let Some(place) = place(expr, names) {
        return Some(place);
    }
    match &expr.kind {
        ExprKind::Field(base, _)
        | ExprKind::Index(base, _)
        | ExprKind::Project { value: base, .. } => mutation_footprint(base, names),
        _ => None,
    }
}

pub(crate) fn sentinel(expr: &Expr) -> Option<Type> {
    match &expr.kind {
        ExprKind::Sequence(values) => values.last().and_then(sentinel),
        ExprKind::Null => Some(Type::Null),
        ExprKind::Undefined => Some(Type::Undefined),
        ExprKind::Inject { value, .. } => sentinel(value),
        ExprKind::Project { narrowed, .. } if matches!(narrowed, Type::Null | Type::Undefined) => {
            Some(*narrowed)
        }
        _ => None,
    }
}
pub(crate) enum Guard { Absent(Type), Typeof(String) }
impl Guard {
    pub(crate) fn matches(&self, ty: Type) -> bool {
        match self {
            Self::Absent(sentinel) => ty == *sentinel,
            Self::Typeof(label) => label == match ty {
                Type::Literal(_,kind)=>return self.matches(kind.base()),
                Type::Int | Type::Number | Type::Enum(_, crate::syntax::EnumKind::Number) => "number",
                Type::String | Type::Enum(_, crate::syntax::EnumKind::String) => "string",
                Type::Boolean => "boolean",
                Type::Undefined | Type::Void => "undefined",
                Type::Null | Type::Set(_) | Type::Map(_) | Type::Array(_) | Type::Record(_) => "object",
                Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_) | Type::Callable(_) => "function",
                Type::Union(_) | Type::BorrowedUnion(_) => return false,
            },
        }
    }
}
pub(crate) fn guard_pair(left: &Expr, right: &Expr, names: &HashMap<String, usize>) -> Option<(Place, Guard)> {
    let typeof_pair = |value: &Expr, label: &Expr| {
        let mut value=value;
        while let ExprKind::Sequence(values)=&value.kind{value=values.last()?;}
        let mut label=label;
        while let ExprKind::Sequence(values)=&label.kind{label=values.last()?;}
        if let ExprKind::Unary { op: crate::syntax::UnaryOp::Typeof, value, .. } = &value.kind {
            if let ExprKind::String(label) = &label.kind {
                return place(value, names).map(|place| (place, Guard::Typeof(label.clone())));
            }
        }
        None
    };
    place(left, names).zip(sentinel(right)).or_else(|| place(right, names).zip(sentinel(left))).map(|(place,sentinel)| (place,Guard::Absent(sentinel)))
        .or_else(|| typeof_pair(left,right)).or_else(|| typeof_pair(right,left))
}
/// Only facts true on every reaching path survive. Alternatives widen at joins.
pub(crate) fn join(left: &Facts, right: &Facts) -> Facts {
    left.iter()
        .filter_map(|(place, a)| {
            let b = right.get(place)?;
            let mut members = a.members.clone();
            for member in &b.members {
                if !members.contains(member) {
                    members.push(*member);
                }
            }
            Some((
                place.clone(),
                Fact {
                    members,
                    origin: if a.origin == b.origin { a.origin } else { None },
                },
            ))
        })
        .collect()
}
pub(crate) fn invalidate(facts: &mut Facts, root: usize) {
    facts.retain(|place, _| place.root != root);
}

pub(crate) fn invalidate_place(facts: &mut Facts, target: &Place) {
    facts.retain(|place, _| {
        place.root != target.root
            || !(place.fields.starts_with(&target.fields)
                || target.fields.starts_with(&place.fields))
    });
}

pub(crate) fn projected_places(expr: &Expr, names: &HashMap<String, usize>, out: &mut Vec<Place>) {
    match &expr.kind {
        ExprKind::Project { value, union, .. } => {
            if expr.ty != Some(Type::Union(*union))
                && let Some(place) = place(value, names)
            {
                out.push(place);
            }
            projected_places(value, names, out);
        }
        ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value)
        | ExprKind::Unary { value, .. }
        | ExprKind::Inject { value, .. }
        | ExprKind::Update { target: value, .. }
        | ExprKind::Field(value, _) => projected_places(value, names, out),
        ExprKind::Optional {
            base: left,
            body: right,
            ..
        }
        | ExprKind::Binary { left, right, .. }
        | ExprKind::Assign {
            target: left,
            value: right,
            ..
        }
        | ExprKind::CoalesceAssign {
            target: left,
            value: right,
            ..
        }
        | ExprKind::CompoundAssign {
            target: left,
            value: right,
            ..
        }
        | ExprKind::LogicalAssign {
            target: left,
            value: right,
            ..
        }
        | ExprKind::Index(left, right) => {
            projected_places(left, names, out);
            projected_places(right, names, out);
        }
        ExprKind::Conditional {
            predicate,
            consequent,
            alternate,
        } => {
            projected_places(predicate, names, out);
            projected_places(consequent, names, out);
            projected_places(alternate, names, out);
        }
        ExprKind::Call(callee, args) => {
            projected_places(callee, names, out);
            for arg in args {
                projected_places(arg, names, out);
            }
        }
        ExprKind::Array(values) | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) | ExprKind::Template(values) => {
            for value in values {
                projected_places(value, names, out);
            }
        }
        ExprKind::Record(fields) => {
            for (_, value) in fields {
                projected_places(value, names, out);
            }
        }
        _ => {}
    }
}
