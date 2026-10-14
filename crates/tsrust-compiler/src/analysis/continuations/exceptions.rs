//! Exception boundaries survive suspension and restore enclosing frames on errors.
use super::super::*;

pub(in crate::analysis) type Exits = (Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame);

pub(in crate::analysis) struct ExitBoundary {
    outer: usize,
    entry_loans: std::rc::Rc<crate::capture_loan_identity::Boundary>,
    previous_entry_loans: Option<std::rc::Rc<crate::capture_loan_identity::Boundary>>,
    previous_loop_depth: Option<usize>,
    previous_completions: Option<Box<exception_state::CompletionFrame>>,
    previous_errors: Option<Box<exception_state::ErrorFrame>>,
}

pub(in crate::analysis) enum RegionMode { Fixed(crate::error_effects::ErrorEffects), Current, Partial }

impl Analyzer {
    #[cfg(test)]
    pub(in crate::analysis) fn region_work(&mut self, context: &mut Context, region: &mut ExceptionRegion, result: Type, mode: RegionMode, runtime: Option<Type>, unwind: &mut dyn FnMut(Span, &mut exception_state::ExitState)) -> Result<(exception_state::CompletionStates,crate::error_effects::FailureSet),Diagnostic> {
        super::driver::check_region(self,context,region,result,mode,runtime,unwind)
    }

    pub(in crate::analysis) fn prepare_exit_frames(context: &mut Context) -> ExitBoundary {
        let outer=context.bindings.len();
        let entry_loans=std::rc::Rc::new(crate::capture_loan_identity::Boundary::capture(&context.capture_loans));
        let previous_entry_loans=context.region_entry_loans.replace(entry_loans.clone());
        let previous_loop_depth=context.region_loop_depth.replace(context.loops.len());
        let previous_completions=context.completion_frame.replace(Box::new(exception_state::CompletionFrame{outer,exits:vec![]}));
        let previous_errors=context.error_frame.replace(Box::new(exception_state::ErrorFrame{outer,points:vec![]}));
        ExitBoundary { outer, entry_loans, previous_entry_loans, previous_loop_depth, previous_completions, previous_errors }
    }

    pub(in crate::analysis) fn finish_exit_frames(context: &mut Context, boundary: ExitBoundary, checked: Result<bool, Diagnostic>) -> Result<Exits, Diagnostic> {
        let ExitBoundary { outer, entry_loans, previous_entry_loans, previous_loop_depth, previous_completions, previous_errors } = boundary;
        let mut completions=*std::mem::replace(&mut context.completion_frame,previous_completions).expect("region completion frame");
        let mut errors=*std::mem::replace(&mut context.error_frame,previous_errors).expect("region error frame");
        context.region_loop_depth=previous_loop_depth;
        context.region_entry_loans=previous_entry_loans;
        let exited=checked?;
        // These exits carry no borrowed payload; region-local loans end here.
        for (kind,_,state) in &mut completions.exits {
            if matches!(kind,crate::completion::CompletionKind::Break(_)|crate::completion::CompletionKind::Continue(_)) {
                state.unwind_capture_loans(&entry_loans,&[]);
            }
        }
        for (_,state) in &mut errors.points {state.unwind_capture_loans(&entry_loans,&[]);}
        let normal=(!exited).then(||exception_state::ExitState::capture(context,outer));
        Ok((normal,completions,errors))
    }

    pub(in crate::analysis) fn prepare_throw_value(&self, context: &Context, value: &Expr) -> Result<Option<Type>, Diagnostic> {
        if matches!(self.expression_type_hint(context,value),Some(Type::BorrowedCallable(_))) {
            return Err(Diagnostic::new("closure-escape","A thrown callable must own its captured storage.",value.span));
        }
        let expected=self.expression_type_hint(context,value).filter(|ty|matches!(ty,Type::Closure(_)));
        if let Some(Type::Closure(origin))=expected
            && self.closures[origin].borrowed
        {
            return Err(Diagnostic::new("closure-escape","A thrown callable must own its captured storage.",value.span)
                .related(self.closures[origin].span,"Borrowed callable is created here."));
        }
        Ok(expected)
    }

    pub(in crate::analysis) fn finish_throw_value(&self, context: &mut Context, value: &Expr, ty: Type) -> Result<Type, Diagnostic> {
        self.thread_scope_exit(context, 0, value.span)?;
        self.thread_discard(ty, value.span)?;
        let facts = self.aggregate_callable_facts(ty);
        if facts.scoped || facts.unknown {return Err(Diagnostic::new("closure-escape","A thrown value must own any callable storage it contains.",value.span));}
        if ty==Type::Void {return Err(Diagnostic::new("type","A thrown expression must produce a value.",value.span));}
        capture_restoration::record(context);
        self.record_typed_completion(context,crate::completion::CompletionKind::Throw,ty,value.span);
        Ok(ty)
    }
}

pub(super) struct CallableBoundary {
    exits: ExitBoundary,
    environment: Option<std::rc::Rc<exception_state::CheckEnvironment>>,
}

impl Analyzer {
    pub(in crate::analysis) fn exception_callable_work(&mut self, context: &mut Context, body: &mut [Stmt], result: Type) -> Result<bool, Diagnostic> {
        super::driver::check_exception_callable(self,context,body,result)
    }

    pub(super) fn prepare_exception_callable(&mut self, context: &mut Context) -> CallableBoundary {
        let effects = self.discovery_error_effects();
        let environment = context.exception_environment.replace(std::rc::Rc::new(exception_state::CheckEnvironment {
            effects, runtime: None, refresh_contracts: true, partial_contracts: true,
        }));
        CallableBoundary { exits: Self::prepare_exit_frames(context), environment }
    }

    pub(super) fn finish_exception_callable(context: &mut Context, boundary: CallableBoundary, checked: Result<bool, Diagnostic>) -> Result<bool, Diagnostic> {
        let exits = Self::finish_exit_frames(context, boundary.exits, checked);
        context.exception_environment = boundary.environment;
        let (normal, completions, _) = exits?;
        if let Some((_,span,_)) = completions.exits.iter().find(|(kind,_,_)| matches!(kind, crate::completion::CompletionKind::Break(_)|crate::completion::CompletionKind::Continue(_))) {
            return Err(Diagnostic::new("loop-control", "Completion cannot leave its callable's control boundary.", *span));
        }
        Ok(normal.is_none())
    }
}
