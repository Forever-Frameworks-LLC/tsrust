//! Declaration context and ownership commit separated by initializer work.
use super::super::*;

pub(super) struct Declaration {
    pub(super) expected: Option<Type>,
    pub(super) mode: Mode,
    pub(super) retained_sequence: bool,
    pub(super) removal: Option<Box<crate::capture_loan_owners::Owners>>,
    view: bool,
    place_view: bool,
    mutable_view: bool,
}

impl Analyzer {
    pub(super) fn prepare_declaration(&mut self, context: &mut Context, name: &Token, annotation: &Option<crate::type_syntax::ValueAnnotation>, value: &mut Expr, binding: &mut Option<usize>) -> Result<Option<Box<Declaration>>, Diagnostic> {
        let expected = annotation
            .as_ref()
            .map(|a| match crate::callable_annotations::callback(a) {
                Some(callback) => self.resolve_callback(&callback),
                None => self.callable_resolver().resolve_value_annotation(a, false),
            })
            .transpose()?;
        let expected = self.absent_nullable_record_annotation(context,expected,value)?;
        if value.is_deferred()
            && self.deferred_let(context, name, expected, value, binding)?
        {
            return Ok(None);
        }
        // Infer the concrete environment on a declaration move. Keep
        // callback contracts intact; this does not create an alias.
        let expected = expected.or_else(|| match &value.kind {
            ExprKind::Name(source) => context.names.get(source).and_then(|&id| {
                let ty = context.bindings[id].ty;
                matches!(ty, Type::Closure(_) | Type::Callback(_)).then_some(ty)
            }),
            _ => None,
        });
        let view = self
            .expression_type_hint(context, value)
            .is_some_and(|ty| matches!(ty, Type::BorrowedUnion(_)));
        let place_view = self.borrowed_place_hint(context, value)
            || self.indexed_element_hint(context, value);
        let mutable_view = Self::mutable_match_initializer(context, name, value)
            && !self.expression_type_hint(context, value).is_some_and(Type::is_copy);
        let removal = Self::array_removal_snapshot(context, value);
        let retained_sequence = self.prepare_bound_removal_sequence(context, value);
        let mode = if retained_sequence { Mode::Consume } else if mutable_view { Mode::Mutate } else if view || place_view { Mode::Read } else { Mode::Consume };
        Ok(Some(Box::new(Declaration { expected, mode, retained_sequence, removal, view, place_view, mutable_view })))
    }

    pub(super) fn finish_declaration(&mut self, context: &mut Context, rebindable: bool, name: &Token, value: &mut Expr, binding: &mut Option<usize>, frame: Box<Declaration>, mut actual: Type) -> Result<(), Diagnostic> {
        let Declaration { expected, retained_sequence, removal, view, place_view, mutable_view, .. } = *frame;
        if retained_sequence { context.storage_exits.record_bound_sequence(value.span); }
        if actual == Type::Void {
            return Err(Diagnostic::new(
                "type",
                "Cannot bind a function with no return value.",
                value.span,
            ));
        }
        self.require(actual, expected, value.span)?;
        if let Some(Type::Callback(contract)) = expected
            && self.callables.callbacks[contract].kind.is_once()
            && let Type::Closure(origin) = actual
            && self.closures[origin].invocation != Mode::Consume
        {
            let wrapper = self.once_return(origin, context, value.span);
            value.wrap_once(wrapper);
            actual = Type::Closure(wrapper);
        }
        let nullable_callback = expected
            .and_then(|ty| self.unions.payload(ty))
            .and_then(|ty| {
                if let Type::Callback(id) = ty {
                    Some(self.callables.callbacks[id].kind)
                } else {
                    None
                }
            });
        if nullable_callback.is_some() {
            if let Some(concrete) = self.nullable_origin(value)? {
                let members = self
                    .unions
                    .members(actual.union().expect("nullable callable"));
                let specialized = self.unions.intern(
                    members.into_iter().map(|member| {
                        if matches!(member, Type::Callback(_)) {
                            concrete
                        } else {
                            member
                        }
                    }),
                    value.span,
                )?;
                Self::retag_nullable_origin(
                    value,
                    actual.union().expect("nullable callable"),
                    specialized.union().expect("nullable callable"),
                );
                actual = specialized;
            }
        }
        let callback = Self::conditional_callback_metadata(context, value)?;
        // A borrowed owned union is held as its readonly view.
        if place_view && !mutable_view && let Type::Union(union) = actual && !union.copy {
            actual = Type::BorrowedUnion(union);
        }
        let native_loans = Self::native_view_value_loans(context, value)?;
        let id = Self::bind(context, name, actual, false)?;
        if self.program.module_owner_bindings.contains(&name.span) {
            context.bindings[id].declaration_span = Some(name.span);
        }
        if !native_loans.is_empty() || (self.native_borrow_types && self.native_shape_contains_view(actual)) {
            self.reject_recursive_native_view_storage(actual, name.span)?;
            context.native_view_declaration_depths.insert(id, context.lexical_scopes.len());
        }
        Self::set_native_view_loans(context, id, native_loans);
        if mutable_view {
            context.bindings[id].mutable_place = true;
            context.mutable_match_borrows.push(id);
            context.bindings[id].borrowed_place = Some(value.span);
            context.bindings[id].view_roots = Self::view_sources(context, value);
            context.bindings[id].used = Some(value.span);
        }
        self.attach_binding_capture_loans(context, value, id, !(view || place_view || mutable_view));
        Self::finish_bound_array_removal(context, value, removal, id);
        if let Some(kind) = nullable_callback {
            context.bindings[id].callback_kind = Some(kind);
            context.bindings[id].callback_invocation = Some(kind.invocation());
        }
        context.bindings[id].rebindable =
            rebindable && !matches!(actual, Type::BorrowedUnion(_)) && !place_view;
        if !mutable_view && (matches!(actual, Type::BorrowedUnion(_)) || (place_view && !actual.is_copy()))
        {
            if place_view && !matches!(actual, Type::BorrowedUnion(_)) {
                context.bindings[id].borrowed_place = Some(value.span);
            }
            context.bindings[id].view_roots = Self::view_sources(context, value);
            context.bindings[id].used = Some(value.span);
        }
        if let Some((invocation, reference, kind)) = callback {
            context.bindings[id].callback_invocation = invocation;
            context.bindings[id].callback_reference = reference;
            context.bindings[id].callback_kind = kind;
            if reference { context.bindings[id].view_roots = Self::view_sources(context, value); }
        }
        if !context.bindings[id].rebindable && let Some(ty)=crate::flow::sentinel(value) {
            context.constant_absence.insert(id,ty);
        }
        *binding = Some(id);
        Ok(())
    }
}
