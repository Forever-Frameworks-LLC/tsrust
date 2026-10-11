//! Stored parameter views retain source provenance after the call returns.
use super::*;
use super::native_view_loans::{Loan, Segment};

enum Destination {
    Argument(usize),
    Capture { origin: usize, inner: usize, family: bool },
}

impl Analyzer {
    fn remember_native_parameter_store(&self, context: &mut Context, destination: usize, incoming: Vec<Loan>, span: Span) -> Result<(), Diagnostic> {
        if context.effects_only || !context.bindings[destination].parameter || incoming.is_empty() { return Ok(()); }
        let sources = Self::native_escape_sources(context, incoming, span)?;
        let stores = context.native_mutations.get_or_insert_with(Default::default);
        for source in sources {
            let entry = (destination, source);
            if !stores.contains(&entry) { stores.push(entry); }
        }
        Ok(())
    }

    pub(super) fn record_native_parameter_store(&self, context: &mut Context, target: &Expr, value: &Expr) -> Result<(), Diagnostic> {
        if !self.native_borrow_types || context.effects_only { return Ok(()); }
        let Some(place) = crate::flow::place(target, &context.names) else { return Ok(()); };
        if !context.bindings[place.root].parameter { return Ok(()); }
        let mut incoming = Self::native_view_value_loans(context, value)?;
        for loan in &mut incoming {
            for field in place.fields.iter().rev() { native_view_projections::wrap(loan, &Segment::Field(field.clone())); }
        }
        native_view_recursive::normalize(context, context.bindings[place.root].ty, &mut incoming);
        self.remember_native_parameter_store(context, place.root, incoming, value.span)
    }

    pub(super) fn apply_native_call_stores(&self, context: &mut Context, expression: &Expr) -> Result<(), Diagnostic> {
        if !self.native_borrow_types || context.effects_only { return Ok(()); }
        let mut call = expression;
        while let ExprKind::Inject { value, .. } | ExprKind::Unary { op: UnaryOp::RecordCarrier, value, .. } = &call.kind { call = value; }
        let ExprKind::Call(_, args) = &call.kind else { return Ok(()); };
        let Some(target) = native_view_calls::target(call) else { return Ok(()); };
        let mut stores = Vec::new();
        let mut closure_stores = |origin: usize, method: Option<usize>, family: bool| -> Result<(), Diagnostic> {
            let closure = &self.closures[origin];
            let (function, parameters) = method.map_or((&closure.function, &closure.parameters), |method| (&closure.methods[method].function, &closure.methods[method].parameters));
            for (destination, output) in function.native_mutations.as_deref().into_iter().flatten() {
                if function.bindings[*destination].mode != Mode::Mutate { continue; }
                let destination = if let Some(position) = parameters.iter().position(|root| root == destination) {
                    Destination::Argument(position)
                } else {
                    let Some(capture) = closure.captures.iter().enumerate().find(|(position,capture)| method.map_or(capture.inner,|method|closure.methods[method].captures[*position]) == *destination).map(|(_,capture)|capture) else { continue; };
                    Destination::Capture { origin, inner: capture.inner, family }
                };
                let mut mapped = self.native_closure_outputs(origin, method, std::slice::from_ref(output), call.span)?;
                if family {
                    for returned in &mut mapped {
                        if let native_view_calls::Source::Capture(capture) = returned.source { returned.source = native_view_calls::Source::FamilyCapture(origin, capture); }
                    }
                }
                stores.push((destination, mapped));
            }
            Ok(())
        };
        match target {
            native_view_calls::Target::Function(id) => {
                let Some(function) = self.checked[id].as_ref() else { return Ok(()); };
                for (destination, loan) in function.native_mutations.as_deref().into_iter().flatten() {
                    if function.bindings[*destination].mode == Mode::Mutate {
                        stores.push((Destination::Argument(*destination), vec![native_view_calls::Returned { source: native_view_calls::Source::Argument(loan.root), loan: loan.clone() }]));
                    }
                }
            }
            native_view_calls::Target::Closure(origin) => closure_stores(origin, None, false)?,
            native_view_calls::Target::Method(origin, method) => closure_stores(origin, Some(method), false)?,
            native_view_calls::Target::Family(family) => {
                for &origin in self.callables.storage.origins(family).unwrap_or(&[]) { closure_stores(origin, None, true)?; }
            }
        }
        if stores.is_empty() { return Ok(()); }
        context.native_view_calls = true;
        let mut updates = Vec::new();
        for (destination, outputs) in stores {
            let (root, path) = match destination {
                Destination::Argument(position) => {
                    let target = args.get(position).ok_or_else(|| Diagnostic::new("native-view-call", "Stored view destination has no caller argument.", call.span))?;
                    let place = crate::flow::place(target, &context.names).ok_or_else(|| Diagnostic::new("native-view-call", "Stored views require a named caller destination.", target.span))?;
                    (place.root, place.fields.into_iter().map(Segment::Field).collect::<Vec<_>>())
                }
                Destination::Capture { origin, inner, family } => {
                    let closure = &self.closures[origin];
                    let capture = closure.captures.iter().find(|capture|capture.inner==inner).expect("checked capture destination");
                    if matches!(capture.storage, Mode::Read | Mode::Mutate) {
                        let outer = context.native_view_environments.as_ref().and_then(|environments|environments.get(&origin)).and_then(|captures|captures.iter().find(|(captured,_)|*captured==inner)).map(|(_,outer)|*outer)
                            .ok_or_else(||Diagnostic::new("native-view-call", "A captured view destination has no caller storage provenance. Pass the record as a mutable argument.", call.span))?;
                        (outer, Vec::new())
                    } else {
                        if closure.invocation == Mode::Consume { continue; }
                        let ExprKind::Call(callee,_) = &call.kind else { unreachable!() };
                        let place = crate::flow::place(callee, &context.names).ok_or_else(||Diagnostic::new("native-view-call", "Updating captured view storage requires a named callable.", call.span))?;
                        let mut path = place.fields.into_iter().map(Segment::Field).collect::<Vec<_>>();
                        if family { path.push(Segment::CallableAlternative(origin)); }
                        path.push(Segment::Field(format!("capture{inner}")));
                        (place.root,path)
                    }
                }
            };
            let mut incoming = Self::native_view_value_outputs(context, call, Some(&outputs))?;
            for loan in &mut incoming {
                for segment in path.iter().rev() { native_view_projections::wrap(loan, segment); }
            }
            native_view_recursive::normalize(context, context.bindings[root].ty, &mut incoming);
            updates.push((root, incoming));
        }
        // Resolve all sources before applying any destination update.
        for (destination, incoming) in updates {
            self.remember_native_parameter_store(context, destination, incoming.clone(), call.span)?;
            let mut retained = context.bindings[destination].native_view_loans.as_deref().cloned().unwrap_or_default();
            retained.extend(incoming);
            Self::set_native_view_loans(context, destination, retained);
        }
        Ok(())
    }
}
