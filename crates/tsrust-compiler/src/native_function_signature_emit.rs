//! Shared native function signatures preserve record and capture obligations.
use super::*;

pub(super) struct NativeSignature {
    pub(super) params: String,
    pub(super) generics: String,
    pub(super) replacement_where: String,
}

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn native_function_signature(
        &self,
        index: usize,
        parameter_shapes: &[Option<crate::fixed_storage::Shape>],
    ) -> NativeSignature {
        let fun = &self.program.functions[index];
        // Hidden capture parameters of a nested function follow its
        // source parameters and use the same access forms.
        let hidden = self
            .program
            .named_capture_access
            .get(&index)
            .map_or(0, Vec::len);
        let borrowed_inputs = self.record_borrow_inputs(index);
        let borrow_lifetimes = self.record_borrow_lifetime_groups(index);
        let params = fun
            .bindings
            .iter()
            .take(fun.parameter_count + hidden)
            .enumerate()
            .map(|(index, b)| {
                let input_ty = b.input_ty.unwrap_or(b.ty);
                let qualifier = match b.mode {
                    Mode::Read if matches!(input_ty,Type::Union(id) if !id.copy) => "",
                    Mode::Read => "&",
                    Mode::Mutate
                        if (b.omission.is_omittable() || self.multiple_data_union(input_ty))
                            && matches!(input_ty,Type::Union(id) if !id.copy) =>
                    {
                        ""
                    }
                    Mode::Mutate => "&mut ",
                    Mode::Consume | Mode::Copy => "",
                };
                let mutable = if b.mutable
                    && (b.mode == Mode::Consume
                        || ((b.omission.is_omittable() || self.multiple_data_union(input_ty))
                            && b.mode == Mode::Mutate))
                {
                    "mut "
                } else {
                    ""
                };
                let parameter_type = if let Some((union, _)) =
                    self.nullable_record_parameter(fun, index)
                {
                    format!("ViewUnion{}<N{index}>", union.index())
                } else if self.record_parameter_entry_view(fun, index).is_some() {
                    format!("R{index}")
                } else if matches!(input_ty, Type::Callback(_)) {
                    format!("F{index}")
                } else {
                    parameter_shapes
                        .get(index)
                        .and_then(Option::as_ref)
                        .map_or_else(
                            || {
                                if (b.omission.is_omittable() || self.multiple_data_union(input_ty))
                                    && b.mode == Mode::Mutate
                                {
                                    self.native_argument_type(ArgumentContract::from(b), "")
                                } else if let Type::Union(id) = input_ty
                                    && !id.copy
                                    && b.mode == Mode::Read
                                {
                                    self.native_type(Type::BorrowedUnion(id))
                                } else {
                                    self.native_type(input_ty)
                                }
                            },
                            |shape| self.shape_type(shape),
                        )
                };
                let qualifier = if borrowed_inputs.contains(&index) && !qualifier.is_empty() {
                    match b.mode {
                        Mode::Read => format!("&'record_input{} ", borrow_lifetimes[&index]),
                        Mode::Mutate => format!("&'record_input{} mut ", borrow_lifetimes[&index]),
                        _ => qualifier.to_owned(),
                    }
                } else {
                    qualifier.to_owned()
                };
                format!("{mutable}value{index}: {qualifier}{parameter_type}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        let params = format!("{params}{}", self.owner_marker_parameters(index));
        let generics = fun
            .bindings
            .iter()
            .take(fun.parameter_count)
            .enumerate()
            .filter_map(|(index, b)| match b.input_ty.unwrap_or(b.ty) {
                Type::Callback(id) => {
                    let contract = &self.program.callback_contracts[id];
                    let trait_name = if matches!(
                        contract.kind,
                        crate::syntax::CallbackKind::Read | crate::syntax::CallbackKind::SharedRead
                    ) {
                        "ReadCallback"
                    } else if contract.kind.is_once() {
                        "OnceCallback"
                    } else {
                        "Callback"
                    };
                    let (args, lifetimes) = self.native_argument_tuple(&contract.params);
                    let bound = if lifetimes.is_empty() {
                        String::new()
                    } else {
                        format!("for{} ", lifetime_parameters(&lifetimes))
                    };
                    Some(format!(
                        "F{index}: {bound}{trait_name}<{args}, {}>",
                        self.callback_result_type(id, contract.result)
                    ))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut lifetime_generics = borrow_lifetimes
            .values()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|id| format!("'record_input{id}"))
            .collect::<Vec<_>>();
        lifetime_generics.extend(generics);
        lifetime_generics.extend(
            self.owner_inputs(index)
                .iter()
                .map(|source| format!("O{source}")),
        );
        let mut generics = lifetime_generics;
        generics.extend(
            (0..fun.parameter_count).filter_map(|id| self.nullable_record_parameter_bound(fun, id)),
        );
        generics.extend(
            (0..fun.parameter_count + hidden)
                .filter_map(|id| self.record_parameter_bound(fun, id, index)),
        );
        generics.extend(
            self.nullable_callback_bounds(
                fun.bindings
                    .iter()
                    .take(fun.parameter_count)
                    .enumerate()
                    .filter(|(id, _)| self.record_parameter_entry_view(fun, *id).is_none())
                    .map(|(_, binding)| binding.input_ty.unwrap_or(binding.ty))
                    .chain(std::iter::once(fun.result)),
            ),
        );
        // A record parameter names nullable callback-record leaves by their
        // full type; the callbacks inside them are generic parameters too.
        let leaves = (0..fun.parameter_count + hidden)
            .filter_map(|id| self.record_parameter_layout(fun, id))
            .flat_map(|layout| {
                layout
                    .leaves
                    .into_iter()
                    .map(|leaf| leaf.ty)
                    .filter(|ty| matches!(ty, Type::Union(_) | Type::BorrowedUnion(_)))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        for bound in self.nullable_callback_bounds(leaves) {
            if !generics.contains(&bound) {
                generics.push(bound);
            }
        }
        let generics = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };
        let mut replacements = self.nullable_parameter_replacement_bounds(index);
        replacements.extend(self.nullable_callback_replacement_bounds(index));
        let replacement_where = if replacements.is_empty() {
            String::new()
        } else {
            format!(" where {}", replacements.join(", "))
        };
        NativeSignature {
            params,
            generics,
            replacement_where,
        }
    }
}
