//! Typed lexical context projection shared by callable environment checking.
use super::*;

pub(super) fn project(outer: &Context, slots: &[usize]) -> Context {
    let mut context = Context::new(outer.function_name.clone());
    context.lexical_scopes = outer.lexical_scopes.clone();
    context.effects_only = outer.effects_only;
    context.closure_depth = outer.closure_depth + 1;
    context.template = outer.template;
    let remap: HashMap<usize, usize> = slots
        .iter()
        .enumerate()
        .map(|(inner, &outer)| (outer, inner))
        .collect();
    debug_assert_eq!(remap.len(), slots.len(), "capture slots must be unique");
    for &slot in slots {
        let mut binding = outer.bindings[slot].clone();
        // Capture the initialized body value, not its enclosing supplied-slot ABI.
        binding.input_ty = None;
        binding.initializer = None;
        binding.rest = false;
        binding.declaration_span = None;
        binding.omission = crate::ParameterOmission::Required;
        binding.parameter = true;
        binding.mode = if binding.ty.is_copy() {
            Mode::Copy
        } else {
            Mode::Read
        };
        binding.mutable = false;
        binding.capture_reference = false;
        binding.transferred = None;
        binding.moved_fields.clear();
        binding.used = None;
        binding.view_roots = binding
            .view_roots
            .iter()
            .filter_map(|root| remap.get(root).copied())
            .collect();
        context
            .names
            .insert(binding.name.clone(), context.bindings.len());
        context.bindings.push(binding);
    }
    context.source_bindings = outer
        .source_bindings
        .iter()
        .filter_map(|(&source, slot)| remap.get(slot).map(|&inner| (source, inner)))
        .collect();
    // Parameters and locals added afterward are invocation-local, never captures.
    context.capture_boundary = context.bindings.len();
    context
}

#[cfg(test)]
mod tests {
    use super::*;
    fn binding(context: &mut Context, name: &str, start: usize, ty: Type) -> usize {
        let token = Token {
            text: name.into(),
            span: Span::new(start, start + name.len()),
            kind: TokenKind::Name,
        };
        Analyzer::bind(context, &token, ty, false).unwrap()
    }
    #[test]
    fn declaration_identity_survives_shadowing_and_sparse_projection() {
        let mut outer = Context::new("main".into());
        let original = binding(&mut outer, "value", 10, Type::String);
        outer.names.remove("value");
        let shadow = binding(&mut outer, "value", 30, Type::Int);
        assert_eq!(outer.source_bindings[&10], original);
        assert_eq!(outer.source_bindings[&30], shadow);
        let inner = project(&outer, &[original]);
        assert_eq!(inner.bindings.len(), 1);
        assert_eq!(inner.bindings[0].ty, Type::String);
        assert_eq!(inner.source_bindings[&10], 0);
        assert!(!inner.source_bindings.contains_key(&30));
        assert_eq!(inner.capture_boundary, 1);
        assert_eq!(outer.names["value"], shadow);
        let scalar = project(&outer, &[shadow]);
        assert_eq!(scalar.bindings[0].mode, Mode::Copy);
        assert_eq!(scalar.source_bindings[&30], 0);
    }
    #[test]
    fn projection_keeps_typed_roots_but_discards_outer_call_and_flow_state() {
        let mut outer = Context::new("main".into());
        let root = binding(&mut outer, "root", 10, Type::String);
        let hidden = binding(&mut outer, "hidden", 20, Type::Int);
        let view = binding(&mut outer, "view", 30, Type::String);
        outer.bindings[view].view_roots = vec![root, hidden];
        outer.bindings[view].input_ty = Some(Type::Undefined);
        outer.bindings[view].rest = true;
        outer.bindings[view].omission = crate::ParameterOmission::Optional;
        outer.bindings[view].transferred = Some(Span::new(40, 41));
        outer.bindings[view].used = Some(Span::new(42, 43));
        outer.bindings[view].mode = Mode::Consume;
        outer.bindings[view].mutable = true;
        outer
            .capture_loans
            .push((root, Mode::Read, Span::new(50, 51)));
        let inner = project(&outer, &[view, root]);
        assert_eq!(inner.bindings[0].view_roots, vec![1]);
        assert_eq!(inner.bindings[0].mode, Mode::Read);
        assert!(inner.bindings.iter().all(|b| b.input_ty.is_none()
            && !b.rest
            && b.omission == crate::ParameterOmission::Required
            && b.transferred.is_none()
            && b.used.is_none()
            && !b.mutable));
        assert!(inner.capture_loans.is_empty());
        assert_eq!(inner.closure_depth, outer.closure_depth + 1);
        assert_eq!(outer.bindings[view].mode, Mode::Consume);
        assert_eq!(outer.bindings[view].view_roots, vec![root, hidden]);
    }
}
