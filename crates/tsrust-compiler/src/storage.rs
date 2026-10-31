//! Backend storage policy and conservative source record layout accounting.
use crate::syntax::{ArrayElement, Type};

fn scalar_array(value:Type)->Type {
    if let Type::Array(ArrayElement::Literal(_,kind,depth))=value {Type::Array(ArrayElement::from_value_type(kind.base()).unwrap().with_depth(depth))}else{value}
}
pub(crate) fn inline_array(value: Type) -> bool {
    let value=scalar_array(value);
    matches!(
        value,
        Type::Array(
            ArrayElement::Int(0..=1) | ArrayElement::Boolean(0..=1) | ArrayElement::String(0)
        )
    ) || matches!(value,Type::Array(ArrayElement::Union(id,0)) if id.scalar_members)
}

pub(crate) fn numeric_array(value: Type) -> bool {
    let value=scalar_array(value);
    matches!(
        value,
        Type::Array(ArrayElement::Int(0) | ArrayElement::Boolean(0))
    )
}

/// Conservative size for the generated enum, including Option padding and its
/// tag. Record eligibility caps element layout at 64 bytes; recursive records
/// still use Vec, but charging the inline bound is safe for either choice.
pub(crate) fn array_storage_bound(element: ArrayElement) -> usize {
    let Type::Array(element)=scalar_array(Type::Array(element)) else{unreachable!()};
    let alignment = std::mem::align_of::<usize>().max(std::mem::align_of::<i64>());
    let inline =
        |bytes: usize| (bytes + alignment).div_ceil(alignment) * alignment * 4 + alignment * 4;
    // Copy elements need no occupancy tag. Include length, alignment, a tag,
    // and the heap variant without depending on Rust's niche optimization.
    let numeric = |bytes: usize| {
        (bytes * 4 + alignment)
            .div_ceil(alignment)
            .saturating_mul(alignment)
            .max(std::mem::size_of::<Vec<()>>())
            + alignment
    };
    match element {
        ArrayElement::Int(0) => numeric(std::mem::size_of::<i64>()),
        ArrayElement::Boolean(0) => numeric(std::mem::size_of::<bool>()),
        ArrayElement::String(0) => inline(string_storage_bound()),
        // Scalar unions contain at most Cow<str> plus an enum tag. Their
        // members cannot introduce recursive inline array/record storage.
        ArrayElement::Union(id,0) if id.scalar_members => inline(string_storage_bound()+alignment),
        ArrayElement::Int(1) => inline(numeric(std::mem::size_of::<i64>())),
        ArrayElement::Boolean(1) => inline(numeric(std::mem::size_of::<bool>())),
        ArrayElement::Record(_, 0) => inline(MAX_INLINE_RECORD_BYTES),
        _ => std::mem::size_of::<Vec<()>>(),
    }
}

pub(crate) const MAX_INLINE_RECORD_BYTES: usize = 64;

/// Use the actual standard Cow layout for conservative record/array budgets.
pub(crate) fn string_storage_bound() -> usize {
    std::mem::size_of::<std::borrow::Cow<'static, str>>()
}
