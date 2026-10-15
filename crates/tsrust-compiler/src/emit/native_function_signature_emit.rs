//! Shared native function signatures preserve record and capture obligations.
use super::*;

pub(super) struct NativeSignature {
    pub(super) receiver:Option<Mode>,
    pub(super) storage_lifetimes: std::collections::BTreeMap<usize,String>,
    pub(super) params: String,
    pub(super) result: String,
    pub(super) generics: String,
    pub(super) replacement_where: String,
}

pub(super) struct NativeSignaturePlan<'a> {
    pub(super) native_capture_lifetimes: std::collections::BTreeMap<(usize,crate::analysis::native_view_loans::LifetimeSource),String>,
    pub(super) function: &'a CheckedFunction,
    pub(super) parameters: Vec<usize>,
    pub(super) source_parameters: Vec<usize>,
    pub(super) borrow_lifetimes: std::collections::BTreeMap<usize, usize>,
    pub(super) owner_inputs: Vec<usize>,
    pub(super) owner_parameters: String,
    pub(super) record_bounds: Vec<String>,
    pub(super) replacement_bounds: Vec<String>,
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
        let parameters = (0..fun.parameter_count + hidden).collect::<Vec<_>>();
        let source_parameters = (0..fun.parameter_count).collect::<Vec<_>>();
        let mut record_bounds = source_parameters
            .iter()
            .filter_map(|&id| self.nullable_record_parameter_bound(fun, id))
            .collect::<Vec<_>>();
        record_bounds.extend(
            parameters
                .iter()
                .filter_map(|&id| self.record_parameter_bound(fun, id, index)),
        );
        let mut replacement_bounds = self.nullable_parameter_replacement_bounds(index);
        replacement_bounds.extend(self.nullable_callback_replacement_bounds(index));
        self.native_signature_from_plan(
            NativeSignaturePlan {
                native_capture_lifetimes:Default::default(),
                function: fun,
                parameters,
                source_parameters,
                borrow_lifetimes: self.record_borrow_lifetime_groups(index),
                owner_inputs: self.owner_inputs(index),
                owner_parameters: self.owner_marker_parameters(index),
                record_bounds,
                replacement_bounds,
            },
            parameter_shapes,
        )
    }

    #[inline(never)]
    pub(super) fn native_signature_from_plan(
        &self,
        plan: NativeSignaturePlan<'_>,
        parameter_shapes: &[Option<crate::fixed_storage::Shape>],
    ) -> NativeSignature {
        let fun = plan.function;
        let mut native_lifetimes = native_view_lifetime_emit::Lifetimes::with_captures(fun, &plan.parameters, &plan.native_capture_lifetimes);
        if self.native_view_empty_result(fun) {
            native_lifetimes.empty_output();
        }
        let borrow_lifetimes = &plan.borrow_lifetimes;
        let params = plan
            .parameters
            .iter()
            .map(|&index| {
                let b = &fun.bindings[index];
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
                                    self.native_view_signature_type(Type::BorrowedUnion(id),Some(index),&native_lifetimes)
                                } else {
                                    self.native_view_signature_type(input_ty,Some(index),&native_lifetimes)
                                }
                            },
                            |shape| self.shape_type(shape),
                        )
                };
                let qualifier = if let Some(lifetime)=native_lifetimes.storage(index) {
                    match b.mode {Mode::Read=>format!("&{lifetime} "),Mode::Mutate=>format!("&{lifetime} mut "),_=>qualifier.to_owned()}
                } else if borrow_lifetimes.contains_key(&index) && !qualifier.is_empty() {
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
        let params = format!("{params}{}", plan.owner_parameters);
        let generics = plan
            .source_parameters
            .iter()
            .filter_map(|&index| {
                let b = &fun.bindings[index];
                match b.input_ty.unwrap_or(b.ty) {
                    Type::Callback(id) => {
                        let contract = &self.program.callback_contracts[id];
                        let trait_name = if matches!(contract.kind, crate::syntax::CallbackKind::Program | crate::syntax::CallbackKind::ProgramReusable) { "ProgramCallback" } else if matches!(
                            contract.kind,
                            crate::syntax::CallbackKind::Read
                                | crate::syntax::CallbackKind::SharedRead
                        ) {
                            "ReadCallback"
                        } else if contract.kind==crate::syntax::CallbackKind::Task { "TaskCallback" } else if contract.kind.is_once() {
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
                }
            })
            .collect::<Vec<_>>();
        let mut lifetime_generics = borrow_lifetimes
            .values()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|id| format!("'record_input{id}"))
            .collect::<Vec<_>>();
        lifetime_generics.extend(native_lifetimes.declarations.iter().cloned());
        lifetime_generics.extend(generics);
        lifetime_generics.extend(plan.owner_inputs.iter().map(|source| format!("O{source}")));
        let mut generics = lifetime_generics;
        generics.extend(plan.record_bounds);
        generics.extend(
            self.nullable_callback_bounds(
                plan.source_parameters
                    .iter()
                    .filter(|&&id| self.record_parameter_entry_view(fun, id).is_none())
                    .map(|&id| {
                        let binding = &fun.bindings[id];
                        binding.input_ty.unwrap_or(binding.ty)
                    })
                    .chain(std::iter::once(fun.result)),
            ),
        );
        // A record parameter names nullable callback-record leaves by their
        // full type; the callbacks inside them are generic parameters too.
        let leaves = plan
            .parameters
            .iter()
            .filter_map(|&id| self.record_parameter_layout(fun, id))
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
        let mut replacements = plan.replacement_bounds;
        replacements.extend(native_lifetimes.bounds.iter().cloned());
        let replacement_where = if replacements.is_empty() {
            String::new()
        } else {
            format!(" where {}", replacements.join(", "))
        };
        let mut result=self.native_view_signature_type(fun.result,None,&native_lifetimes);
        let mut params=params;
        let mut generics=generics;
        if self.program.task_type(fun.result) && result.contains("'_") && plan.parameters.iter().any(|&id|matches!(fun.bindings[id].mode,Mode::Read|Mode::Mutate)) {
            result=result.replace("'_","'task");
            params=params.split(": &").enumerate().map(|(index,part)| if index==0 {part.to_owned()} else if part.starts_with("'") {format!(": &{part}")} else {format!(": &'task {part}")}).collect();

            generics=if generics.is_empty(){"<'task>".into()}else{format!("<'task,{}",&generics[1..])};
        }
        NativeSignature {
            receiver:None,
            storage_lifetimes:plan.parameters.iter().filter_map(|&root|native_lifetimes.storage(root).map(|name|(root,name.to_owned()))).collect(),
            result,
            params,
            generics,
            replacement_where,
        }
    }
}
