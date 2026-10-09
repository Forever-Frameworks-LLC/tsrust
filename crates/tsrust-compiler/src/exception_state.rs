//! Checker state at an error/completion exit, independent of scheduling.
use super::*;
use crate::completion::CompletionKind;

pub(super) struct CheckEnvironment {
    pub(super) refresh_contracts:bool,
    pub(super) partial_contracts:bool,
    pub(super) effects:crate::error_effects::ErrorEffects,
    pub(super) runtime:Option<Type>,
}

#[derive(Clone)]
pub(super) struct CompletionFrame {
    pub(super) outer:usize,
    pub(super) exits:Vec<(CompletionKind,Span,ExitState)>,
}

#[inline(never)]
pub(super) fn record_completion(context:&mut Context,kind:CompletionKind,span:Span) {
    let Some(outer)=context.completion_frame.as_ref().map(|frame|frame.outer) else {return;};
    let state=ExitState::capture(context,outer);
    context.completion_frame.as_mut().expect("active completion frame").exits.push((kind,span,state));
}

#[inline(never)]
pub(super) fn consume_target(context:&mut Context,target:ControlTarget) {
    if let Some(frame)=&mut context.completion_frame {frame.exits.retain(|(kind,_,_)|!kind.targets(target));}
}

#[derive(Clone)]
pub(super) struct ErrorFrame {
    pub(super) outer:usize,
    pub(super) points:Vec<(Span,ExitState)>,
}
impl ErrorFrame {
    /// Remove only operations proven infallible; unclassified points stay conservative.
    pub(super) fn refine(&mut self,own_effect:&mut impl FnMut(Span)->Option<bool>) {
        self.points.retain(|(span,_)|own_effect(*span)!=Some(false));
    }
}

#[inline(never)]
pub(super) fn record_point(context:&mut Context,span:Span) {
    super::capture_restoration::record(context);
    let Some(outer)=context.error_frame.as_ref().map(|frame|frame.outer) else {return;};
    let state=ExitState::capture(context,outer);
    context.error_frame.as_mut().expect("active error frame").points.push((span,state));
}

#[derive(Default)]
pub(super) struct CompletionStates {
    normal:Option<ExitState>,
    exits:Vec<(CompletionKind,ExitState)>,
}
impl CompletionStates {
    #[inline(never)]
    pub(super) fn record_map_payloads(&self,exits:&mut storage_exit_availability::Exits,boundary:usize,phase:map_completion_payloads::Phase) {
        if let Some(state)=&self.normal {
            let payloads=state.capture_loan_owners.map_payload_snapshot();
            if !payloads.is_empty() || exits.has_map_entry_survivors(){exits.record_map_completion_phase(boundary,phase,None,payloads);}
        }
        for (kind,state) in &self.exits {
            let payloads=state.capture_loan_owners.map_payload_snapshot();
            if !payloads.is_empty() || exits.has_map_entry_survivors(){exits.record_map_completion_phase(boundary,phase,Some(*kind),payloads);}
        }
    }
    pub(super) fn joined_entry(&self)->Option<ExitState> {
        self.normal.iter().chain(self.exits.iter().map(|(_,state)|state)).cloned().reduce(ExitState::join)
    }
    /// Commit cleaned control destinations and forward already-unwound exits.
    #[inline(never)]
    pub(super) fn forward_exits(self,analyzer:&mut Analyzer,context:&mut Context,span:Span)->Result<bool,Diagnostic> {
        let abrupt=self.normal.is_none();
        self.restore_normal(context);
        for (kind,mut state) in self.exits {
            if let CompletionKind::Break(Some(target))|CompletionKind::Continue(Some(target))=kind {
                analyzer.commit_region_control(context,target,kind.is_continue(),state.clone(),span)?;
            }
            let frame=context.completion_frame.as_mut().expect("enclosing completion frame");
            state.project_outer(frame.outer);
            frame.exits.push((kind,span,state));
        }
        Ok(abrupt)
    }
    /// Restore only normal fallthrough. Abrupt states belong to their destinations.
    pub(super) fn restore_normal(&self,context:&mut Context)->bool {
        let Some(state)=&self.normal else {return false;};
        state.clone().restore(context);
        true
    }
    pub(super) fn project_outer(&mut self,outer:usize) {
        if let Some(state)=&mut self.normal {state.project_outer(outer);}
        for (_,state) in &mut self.exits {state.project_outer(outer);}
    }
    #[inline(never)]
    pub(super) fn from_completions(normal:Option<ExitState>,frame:CompletionFrame,unwind:&mut impl FnMut(Span,&mut ExitState))->Self {
        if let Some(state)=&normal {assert_eq!(state.owners.len(),frame.outer,"normal and abrupt states must share their binding prefix");}
        let mut output=Self{normal,exits:vec![]};
        for (kind,span,mut state) in frame.exits {
            assert_eq!(state.owners.len(),frame.outer,"completion states must share their frame's binding prefix");
            unwind(span,&mut state);
            output.record(kind,state);
        }
        output
    }
    #[inline(never)]
    pub(super) fn record_failures(&mut self,frame:ErrorFrame,unwind:&mut impl FnMut(Span,&mut ExitState)) {
        for (span,mut state) in frame.points {
            assert_eq!(state.owners.len(),frame.outer,"failure states must share their frame's binding prefix");
            unwind(span,&mut state);
            self.record(CompletionKind::Throw,state);
        }
    }
    pub(super) fn catch(mut self,handler:&mut impl FnMut(ExitState)->Result<Self,Diagnostic>)->Result<Self,Diagnostic> {
        let Some(state)=self.take_throw() else {return Ok(self);};
        let handled=handler(state)?;
        if let Some(state)=handled.normal {self.normal=Some(match self.normal.take(){Some(previous)=>previous.join(state),None=>state});}
        for (kind,state) in handled.exits {self.record(kind,state);}
        Ok(self)
    }
    pub(super) fn take_throw(&mut self)->Option<ExitState> {
        let index=self.exits.iter().position(|(kind,_)|*kind==CompletionKind::Throw)?;
        Some(self.exits.remove(index).1)
    }
    pub(super) fn record(&mut self,kind:CompletionKind,state:ExitState) {
        if let Some((_,previous))=self.exits.iter_mut().find(|(existing,_)|*existing==kind) {*previous=previous.clone().join(state);}else{self.exits.push((kind,state));}
    }
    #[inline(never)]
    pub(super) fn finally(self,cleanup:&mut impl FnMut(ExitState)->Result<Self,Diagnostic>)->Result<Self,Diagnostic> {
        let mut output=Self::default();
        let pending=self.normal.into_iter().map(|state|(None,state)).chain(self.exits.into_iter().map(|(kind,state)|(Some(kind),state)));
        for (kind,state) in pending {
            let cleaned=cleanup(state)?;
            if let Some(state)=cleaned.normal {
                if let Some(kind)=kind {output.record(kind,state);}
                else {output.normal=Some(match output.normal.take(){Some(previous)=>previous.join(state),None=>state});}
            }
            for (kind,state) in cleaned.exits {output.record(kind,state);}
        }
        Ok(output)
    }
}

#[derive(Clone)]
pub(super) struct ExitState {
    owners:Vec<Availability>,
    flow:crate::flow::Facts,
    capture_loan_owners:crate::capture_loan_owners::Owners,
    capture_loan_ids:crate::capture_loan_identity::Registry,
    capture_loans:Vec<(usize,Mode,Span)>,
}
impl ExitState {
    /// Project an inner region exit onto its enclosing region's binding prefix.
    /// Payload loans must be checked and lexical loans unwound separately.
    pub(super) fn project_outer(&mut self,outer:usize) {
        assert!(outer<=self.owners.len(),"exit projection cannot create missing owner states");
        self.owners.truncate(outer);
        self.flow.retain(|place,_|place.root<outer);
    }
    /// Release loans of discarded locals, retaining loans carried by the pending payload.
    /// The caller must identify payload loans before removing its local bindings.
    pub(super) fn unwind_capture_loans(&mut self,entry:&crate::capture_loan_identity::Boundary,payload_loans:&[(usize,Mode,Span)]) {
        entry.unwind(&mut self.capture_loans);
        self.capture_loan_owners.replace_payload(payload_loans.iter().map(|loan| self.capture_loan_ids.intern(*loan)));
        let _ = self.capture_loan_owners.leave_scope(self.owners.len());
        for loan in payload_loans {
            if !self.capture_loans.contains(loan) {self.capture_loans.push(*loan);}
        }
    }
    pub(super) fn capture(context:&Context,outer:usize)->Self {
        Self {
            owners:context.bindings[..outer].iter().map(Availability::of).collect(),
            flow:context.flow.iter().filter(|(place,_)|place.root<outer).map(|(place,fact)|(place.clone(),fact.clone())).collect(),
            capture_loan_owners:context.capture_loan_owners.clone(),
            capture_loan_ids:context.capture_loan_ids.clone(),
            capture_loans:context.capture_loans.clone(),
        }
    }
    pub(super) fn join(mut self,other:Self)->Self {
        assert_eq!(self.owners.len(),other.owners.len(),"exit states must share an outer binding prefix");
        self.owners=self.owners.into_iter().zip(other.owners).map(|(left,right)|left.join(right)).collect();
        self.flow=crate::flow::join(&self.flow,&other.flow);
        self.capture_loan_owners.join(&other.capture_loan_owners);
        for loan in other.capture_loans {if !self.capture_loans.contains(&loan) {self.capture_loans.push(loan);}}
        self
    }
    pub(super) fn restore(self,context:&mut Context) {
        assert!(context.bindings.len()>=self.owners.len(),"exit binding prefix must still exist");
        for (state,binding) in self.owners.into_iter().zip(&mut context.bindings) {state.restore(binding);}
        context.flow=self.flow;context.capture_loans=self.capture_loans;
        context.capture_loan_owners=self.capture_loan_owners;
        Analyzer::restore_live_capture_loans(context);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn unwound_tracked_payload_keeps_its_loan_after_local_owner_disposal() {
        let mut context = Context::new("main".into());
        let payload = (0, Mode::Read, Span::new(1, 2));
        let id = context.capture_loan_ids.intern(payload);
        context.capture_loan_owners.attach(crate::capture_loan_owners::Place { binding: 1, fields: vec![] }, [id]);
        context.capture_loans.push(payload);
        let mut state = ExitState::capture(&context, 0);
        state.unwind_capture_loans(&crate::capture_loan_identity::Boundary::capture(&[]), &[payload]);
        state.restore(&mut context);
        assert_eq!(context.capture_loans, vec![payload]);
        assert!(context.capture_loan_owners.live().contains(&id));
    }

    #[test]
    fn restoring_exception_owner_snapshot_recovers_missing_live_capture_loan() {
        let mut context = Context::new("main".into());
        let retained = (0, Mode::Mutate, Span::new(1, 2));
        let unknown = (1, Mode::Read, Span::new(3, 4));
        let id = context.capture_loan_ids.intern(retained);
        context.capture_loan_owners.attach(crate::capture_loan_owners::Place { binding: 2, fields: vec![] }, [id]);
        context.capture_loans = vec![unknown];
        let state = ExitState::capture(&context, 0);
        context.capture_loan_owners = Default::default();
        state.restore(&mut context);
        assert_eq!(context.capture_loans, vec![unknown, retained]);
    }

    #[test]
    fn post_test_loop_keeps_break_and_continue_owners_but_not_return_exits() {
        use crate::capture_loan_owners::Place;
        for (body_source, normal) in [("break;", true), ("continue;", true), ("return;", false)] {
            let source = format!("function main(){{ do {{ {body_source} }} while(false); }}");
            let mut analyzer = Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
            let mut context = Context::new("main".into());
            Analyzer::bind(&mut context, &Token { text: "outer".into(), span: Span::new(0, 1), kind: TokenKind::Name }, Type::String, false).unwrap();
            context.capture_loan_owners.attach(Place { binding: 0, fields: vec![] }, [10]);
            let mut statements = std::mem::take(&mut analyzer.program.functions[0].body);
            let statement = &mut statements[0];
            let StmtKind::Loop { condition, body, update, post_test, tail_reachable } = &mut statement.kind else { panic!("post-test loop"); };
            let abrupt = analyzer.tail_loop(&mut context, condition, body, update, *post_test,
                tail_reachable, Type::Void, statement.span, &statement.labels).unwrap();
            assert_eq!(!abrupt, normal, "{body_source}");
            assert_eq!(context.capture_loan_owners.live(), if normal { std::collections::BTreeSet::from([10]) } else { Default::default() }, "{body_source}");
        }
    }

    #[test]
    fn control_destinations_keep_break_and_continue_owner_sets_separate() {
        use crate::capture_loan_owners::Place;
        let mut analyzer = Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context = Context::new("main".into());
        context.loops.push(LoopFrame { labels: vec![], target: ControlTarget::Loop(0),
            flow_entry: HashMap::new(), flow_required: HashMap::new(), break_flow: None,
            continue_flow: None, entry: vec![], breaks: vec![], break_seen: false,
            break_owners: None, continue_owners: None,scope_start:usize::MAX, defer_continue: true, continues: None,
            span: Span::new(0, 1) });
        context.capture_loan_owners.attach(Place { binding: 0, fields: vec![] }, [10]);
        analyzer.control_exit(&mut context, &[], &mut None, true, Span::new(2, 3)).unwrap();
        context.capture_loan_owners = Default::default();
        context.capture_loan_owners.attach(Place { binding: 0, fields: vec![] }, [11]);
        analyzer.control_exit(&mut context, &[], &mut None, false, Span::new(4, 5)).unwrap();
        let frame = &context.loops[0];
        assert_eq!(frame.continue_owners.as_ref().expect("continue owners").live(), std::collections::BTreeSet::from([10]));
        assert_eq!(frame.break_owners.as_ref().expect("break owners").live(), std::collections::BTreeSet::from([11]));
    }
    #[test]
    fn capture_owner_snapshots_restore_and_join_every_reaching_owner() {
        use crate::capture_loan_owners::Place;
        let mut context = Context::new("main".into());
        context.capture_loan_owners.attach(Place { binding: 0, fields: vec!["left".into()] }, [10]);
        let left = ExitState::capture(&context, 0);
        context.capture_loan_owners = Default::default();
        context.capture_loan_owners.attach(Place { binding: 1, fields: vec!["right".into()] }, [11]);
        let right = ExitState::capture(&context, 0);
        context.capture_loan_owners = Default::default();
        left.join(right).restore(&mut context);
        assert_eq!(context.capture_loan_owners.live(), std::collections::BTreeSet::from([10, 11]));
        assert_eq!(context.capture_loan_owners.leave_scope(1), std::collections::BTreeSet::from([11]));
        assert_eq!(context.capture_loan_owners.live(), std::collections::BTreeSet::from([10]));
    }
    use super::*;
    #[test]
    fn region_control_exits_do_not_commit_outer_loop_before_cleanup() {
        for continuing in [false,true] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".into());
            Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
            context.loops.push(LoopFrame{labels:vec![],target:ControlTarget::Loop(0),flow_entry:HashMap::new(),flow_required:HashMap::new(),break_flow:None,continue_flow:None,entry:vec![(0,Availability::of(&context.bindings[0]))],breaks:vec![Availability::default()],break_seen:false, break_owners:None, continue_owners:None,scope_start:usize::MAX,defer_continue:true,continues:None,span:Span::new(0,1)});
            let source=if continuing {"function main(){continue;}"}else{"function main(){break;}"};
            let mut parsed=crate::syntax::parse(source).unwrap();
            let (normal,frame,_)=analyzer.check_region_body(&mut context,&mut parsed.functions[0].body,Type::Void).unwrap();
            assert!(normal.is_none());
            assert_eq!(frame.exits.len(),1);
            assert!(!context.loops[0].break_seen);
            assert!(context.loops[0].continues.is_none());
            assert!(context.loops[0].break_flow.is_none() && context.loops[0].continue_flow.is_none());
        }
    }
    #[test]
    fn cleaned_control_state_commits_without_changing_fallthrough() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        context.loops.push(LoopFrame{labels:vec![],target:ControlTarget::Loop(0),flow_entry:HashMap::new(),flow_required:HashMap::new(),break_flow:None,continue_flow:None,entry:vec![(0,Availability::of(&context.bindings[0]))],breaks:vec![Availability::default()],break_seen:false, break_owners:None, continue_owners:None,scope_start:usize::MAX,defer_continue:true,continues:None,span:Span::new(0,1)});
        let original=Availability::of(&context.bindings[0]);
        let state=ExitState::capture(&context,1);
        context.region_loop_depth=Some(1);
        assert!(!analyzer.commit_region_control(&mut context,ControlTarget::Loop(0),false,state.clone(),Span::new(0,1)).unwrap());
        assert!(!context.loops[0].break_seen);
        context.region_loop_depth=None;
        assert!(analyzer.commit_region_control(&mut context,ControlTarget::Loop(0),false,state,Span::new(0,1)).unwrap());
        assert!(context.loops[0].break_seen);
        assert_eq!(context.bindings[0].transferred,original.transferred);
        assert_eq!(context.bindings[0].moved_fields,original.fields);
    }
    #[test]
    fn forwarding_commits_cleaned_break_only_after_outer_boundary() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        context.loops.push(LoopFrame{labels:vec![],target:ControlTarget::Loop(0),flow_entry:HashMap::new(),flow_required:HashMap::new(),break_flow:None,continue_flow:None,entry:vec![(0,Availability::of(&context.bindings[0]))],breaks:vec![Availability::default()],break_seen:false, break_owners:None, continue_owners:None,scope_start:usize::MAX,defer_continue:true,continues:None,span:Span::new(0,1)});
        context.completion_frame=Some(Box::new(CompletionFrame{outer:1,exits:vec![]}));
        let normal=ExitState::capture(&context,1);
        context.bindings[0].transferred=Some(Span::new(10,17));
        let cleaned=ExitState::capture(&context,1);
        normal.clone().restore(&mut context);
        context.region_loop_depth=Some(1);
        let pending=CompletionStates{normal:Some(normal.clone()),exits:vec![(CompletionKind::Break(Some(ControlTarget::Loop(0))),cleaned.clone())]};
        assert!(!pending.forward_exits(&mut analyzer,&mut context,Span::new(20,30)).unwrap());
        assert!(!context.loops[0].break_seen);
        context.region_loop_depth=None;
        let pending=CompletionStates{normal:Some(normal),exits:vec![(CompletionKind::Break(Some(ControlTarget::Loop(0))),cleaned)]};
        assert!(!pending.forward_exits(&mut analyzer,&mut context,Span::new(30,40)).unwrap());
        assert!(context.loops[0].break_seen);
        assert_eq!(context.loops[0].breaks[0].transferred,Some(Span::new(10,17)));
        assert!(context.bindings[0].transferred.is_none());
    }
    #[test]
    fn source_cleanup_controls_continue_validation_and_replacement() {
        for (source,rejected,replaced) in [
            ("try{const moved=payload;continue;}finally{payload=\"repaired\";}",false,false),
            ("try{continue;}finally{const moved=payload;}",true,false),
            ("try{try{const moved=payload;continue;}finally{const done=1;}}finally{payload=\"repaired\";}",false,false),
            ("try{try{continue;}finally{const done=1;}}finally{const moved=payload;}",true,false),
            ("try{const owned=\"value\";const action=()=>owned;const read=()=>payload.length;throw action;}finally{payload=\"repaired\";}",false,true),
            ("try{const action=()=>1;const read=()=>payload.length;throw action;}finally{payload=\"repaired\";}",false,true),
            ("try{const read=()=>payload.length;throw 7;}finally{payload=\"repaired\";}",false,true),
            ("try{const read=()=>payload.length;throw 7;}catch{payload=\"repaired\";throw 8;}",false,true),
            ("try{const read=()=>payload.length;return;}finally{payload=\"repaired\";}",false,true),
            ("try{const read=()=>payload.length;continue;}finally{payload=\"repaired\";}",false,false),
            ("try{const moved=payload;continue;}finally{return;}",false,true),
            ("try{const moved=payload;break;}finally{throw 7;}",false,true),
        ] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".into());
            Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
            context.bindings[0].rebindable=true;
            context.loops.push(LoopFrame{labels:vec![],target:ControlTarget::Loop(0),flow_entry:HashMap::new(),flow_required:HashMap::new(),break_flow:None,continue_flow:None,entry:vec![(0,Availability::of(&context.bindings[0]))],breaks:vec![Availability::default()],break_seen:false, break_owners:None, continue_owners:None,scope_start:usize::MAX,defer_continue:false,continues:None,span:Span::new(0,1)});
            context.completion_frame=Some(Box::new(CompletionFrame{outer:1,exits:vec![]}));
            let mut parsed=crate::syntax::parse(&format!("function main(){{{source}}}")).unwrap();
            let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
            let (completed,_)=analyzer.check_typed_exception_region(&mut context,region,Type::Void,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).unwrap();
            if source.contains("const read=") {
                assert!(analyzer.closures.iter().any(|closure|closure.borrowed));
                assert!(completed.exits.iter().all(|(_,state)|state.capture_loans.is_empty()));
            }
            let result=completed.forward_exits(&mut analyzer,&mut context,Span::new(0,source.len()));
            if rejected {assert_eq!(result.unwrap_err().code,"loop-transfer");}
            else {assert!(result.unwrap());}
            assert!(!context.loops[0].break_seen);
            if replaced {assert!(context.loops[0].continue_flow.is_none());}
        }
    }
    #[test]
    fn isolated_region_unwinds_payload_free_exit_loans() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let outer=(0,Mode::Read,Span::new(1,2));
        let local=(0,Mode::Read,Span::new(3,4));
        context.capture_loans.push(outer);
        let (_,frame,errors)=analyzer.check_with_exit_frames(&mut context,|_,context|{
            context.capture_loans.push(local);
            record_completion(context,CompletionKind::Continue(Some(ControlTarget::Loop(0))),Span::new(5,6));
            record_point(context,Span::new(7,8));
            Ok(true)
        }).unwrap();
        assert_eq!(frame.exits[0].2.capture_loans,vec![outer]);
        assert_eq!(errors.points[0].1.capture_loans,vec![outer]);
    }
    #[test]
    fn scalar_throw_preserves_outer_closure_loan_during_cleanup() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let mut setup=crate::syntax::parse("function main(){let payload=\"outer\";const read=()=>payload.length;}").unwrap();
        analyzer.block_with_scope(&mut context,&mut setup.functions[0].body,Type::Void,true).unwrap();
        assert!(!context.capture_loans.is_empty());
        let before=context.capture_loans.clone();
        let mut parsed=crate::syntax::parse("function main(){try{throw 7;}finally{payload=\"unsafe\";}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let error=analyzer.check_typed_exception_region(&mut context,region,Type::Void,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).err().unwrap();
        assert_eq!(error.code,"capture-conflict");
        assert_eq!(context.capture_loans,before);
        assert!(context.region_entry_loans.is_none());
    }
    #[test]
    fn nested_region_boundaries_restore_after_inner_diagnostic() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let previous = std::rc::Rc::new(crate::capture_loan_identity::Boundary::capture(&[(9,Mode::Read,Span::new(9,10))]));
        context.region_entry_loans=Some(previous.clone());
        context.region_loop_depth=Some(7);
        context.capture_loans.push((0,Mode::Read,Span::new(1,2)));
        analyzer.check_with_exit_frames(&mut context,|analyzer,context|{
            assert_eq!(context.region_entry_loans.as_deref(),Some(&crate::capture_loan_identity::Boundary::capture(&[(0,Mode::Read,Span::new(1,2))])));
            assert_eq!(context.region_loop_depth,Some(0));
            context.capture_loans.push((0,Mode::Read,Span::new(3,4)));
            let diagnostic=analyzer.check_with_exit_frames(context,|_,context|{
                assert_eq!(context.region_entry_loans.as_deref(),Some(&crate::capture_loan_identity::Boundary::capture(&[(0,Mode::Read,Span::new(1,2)),(0,Mode::Read,Span::new(3,4))])));
                Err(Diagnostic::new("type","inner failure",Span::new(5,6)))
            }).err().unwrap();
            assert_eq!(diagnostic.code,"type");
            assert_eq!(context.region_entry_loans.as_deref(),Some(&crate::capture_loan_identity::Boundary::capture(&[(0,Mode::Read,Span::new(1,2))])));
            assert_eq!(context.region_loop_depth,Some(0));
            record_completion(context,CompletionKind::Return,Span::new(7,8));
            assert_eq!(context.completion_frame.as_ref().unwrap().exits.len(),1);
            Ok(true)
        }).unwrap();
        assert_eq!(context.region_entry_loans,Some(previous));
        assert_eq!(context.region_loop_depth,Some(7));
        assert!(context.completion_frame.is_none());
        assert!(context.error_frame.is_none());
    }
    #[test]
    fn checking_environment_restores_after_nested_diagnostic() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let effects=crate::error_effects::ErrorEffects{functions:vec![true],..Default::default()};
        analyzer.with_exception_environment(&mut context,&effects,Some(Type::Int),|analyzer,context|{
            let outer=context.exception_environment.as_ref().unwrap().clone();
            assert_eq!(outer.runtime,Some(Type::Int));
            assert_eq!(outer.effects.functions,vec![true]);
            let error=analyzer.with_exception_environment(context,&crate::error_effects::ErrorEffects::default(),None,|_,_|->Result<(),Diagnostic>{Err(Diagnostic::new("type","inner",Span::new(1,2)))}).unwrap_err();
            assert_eq!(error.code,"type");
            assert!(std::rc::Rc::ptr_eq(context.exception_environment.as_ref().unwrap(),&outer));
            Ok(())
        }).unwrap();
        assert!(context.exception_environment.is_none());
    }
    #[test]
    fn nested_typed_source_regions_apply_outer_cleanup_precedence() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let mut parsed=crate::syntax::parse("function main(){try{try{throw 7;}catch(error){throw error;}finally{return;}}finally{throw \"outer\";}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_typed_exception_region(&mut context,region,Type::Void,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).unwrap();
        assert!(completed.normal.is_none());
        assert_eq!(completed.exits.len(),1);
        assert_eq!(completed.exits[0].0,CompletionKind::Throw);
        assert_eq!(contract.explicit,vec![Type::String]);
        assert!(!context.names.contains_key("error"));
        assert!(context.exception_environment.is_none());
    }
    #[test]
    fn outer_catch_observes_owner_transferred_by_inner_finally() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{try{throw 7;}finally{const moved=payload;}}catch{const length=payload.length;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let error=analyzer.check_typed_exception_region(&mut context,region,Type::Void,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).err().unwrap();
        assert_eq!(error.code,"transferred");
        assert!(context.exception_environment.is_none());
    }
    #[test]
    fn nested_runtime_rethrow_preserves_failure_point_ownership() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        for (name,ty) in [("values",Type::Array(ArrayElement::Int(0))),("payload",Type::String)] {
            Analyzer::bind(&mut context,&Token{text:name.into(),span:Span::new(0,name.len()),kind:TokenKind::Name},ty,false).unwrap();
        }
        let runtime=analyzer.materialize_discovery_runtime_error();
        let mut parsed=crate::syntax::parse("function main(){try{try{const value=values[9];const moved=payload;}catch(inner){throw inner;}}catch(outer){const length=payload.length;return outer.code.length;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_typed_exception_region(&mut context,region,Type::Int,&crate::error_effects::ErrorEffects::default(),Some(runtime),&mut |_,_|{}).unwrap();
        assert!(completed.normal.as_ref().unwrap().owners[1].transferred.is_some());
        assert_eq!(completed.exits.len(),1);
        assert_eq!(completed.exits[0].0,CompletionKind::Return);
        assert!(completed.exits[0].1.owners[1].transferred.is_none());
        assert_eq!(region.catch.as_ref().unwrap().error_type,Some(runtime));
        assert!(contract.explicit.is_empty() && !contract.runtime && !contract.opaque);
        assert!(context.exception_environment.is_none());
    }
    #[test]
    fn nested_result_discovery_bootstraps_direct_throw_contract() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let parsed=crate::syntax::parse("function main(){try{try{throw \"failure\";}finally{const done=1;}}catch(error){return error.length;}}").unwrap();
        assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),Some(Type::Int));
        assert!(context.names.is_empty());
        assert!(context.exception_environment.is_none());
        assert_eq!(analyzer.depth,0);
    }
    #[test]
    fn nested_runtime_result_discovery_materializes_schema_lazily() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"values".into(),span:Span::new(0,6),kind:TokenKind::Name},Type::Array(ArrayElement::Int(0)),false).unwrap();
        let parsed=crate::syntax::parse("function main(){try{try{const value=values[9];}catch(inner){throw inner;}}catch(outer){return outer.code.length;}}").unwrap();
        for _ in 0..2 {
            assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),Some(Type::Int));
            assert_eq!(analyzer.program.records.len(),1);
            assert_eq!(analyzer.program.records[0].name.text,"#RuntimeError");
            assert!(context.exception_environment.is_none());
            assert_eq!(context.bindings.len(),1);
        }
    }
    #[test]
    fn finally_metadata_covers_all_pending_flow_types() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let union=analyzer.unions.intern([Type::Int,Type::Boolean],Span::new(0,1)).unwrap();
        Analyzer::bind(&mut context,&Token{text:"value".into(),span:Span::new(0,5),kind:TokenKind::Name},union,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{if(typeof value===\"number\"){return;}}finally{const copy=value;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        analyzer.check_typed_exception_region(&mut context,region,Type::Void,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).unwrap();
        let StmtKind::Let{value,..}=&region.finally.as_ref().unwrap()[0].kind else {panic!("copy");};
        assert_eq!(value.ty,Some(union));
        let local=*context.bindings.iter().enumerate().find(|(_,binding)|binding.name=="copy").map(|(id,_)|id).as_ref().unwrap();
        assert_eq!(context.bindings[local].ty,union);
    }
    #[test]
    fn canonical_generic_cleanup_keeps_stable_target_across_replays() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function identity<T>(value:Owned<T>):T{return value;}function main(){}").unwrap()).unwrap();
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"flag".into(),span:Span::new(0,4),kind:TokenKind::Name},Type::Boolean,false).unwrap();
        let mut previous=None;
        for _ in 0..2 {
            let mut parsed=crate::syntax::parse("function main(){try{if(flag){return;}}finally{const value=identity<string>(\"cleanup\");}}").unwrap();
            let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
            let (completed,contract)=analyzer.check_current_exception_region(&mut context,region,Type::Void,None,&mut |_,_|{}).unwrap();
            assert!(completed.normal.is_some());
            assert_eq!(completed.exits.len(),1);
            assert_eq!(completed.exits[0].0,CompletionKind::Return);
            assert!(contract.explicit.is_empty() && !contract.runtime && !contract.opaque);
            let StmtKind::Let{value,..}=&region.finally.as_ref().unwrap()[0].kind else {panic!("value");};
            assert_eq!(value.ty,Some(Type::String));
            let target=value.target.expect("canonical call target");
            if let Some((before,count))=previous {assert_eq!(target,before);assert_eq!(analyzer.checked.len(),count);}
            previous=Some((target,analyzer.checked.len()));
        }
    }
    #[test]
    fn canonical_cleanup_call_uses_joined_union_specialization() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function identity<T>(value:T):T{return value;}function main(){}").unwrap()).unwrap();
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let mut context=Context::new("main".into());
        let union=analyzer.unions.intern([Type::Int,Type::Boolean],Span::new(0,1)).unwrap();
        Analyzer::bind(&mut context,&Token{text:"value".into(),span:Span::new(0,5),kind:TokenKind::Name},union,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{if(typeof value===\"number\"){return;}}finally{const copy=identity(value);}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (_,contract)=analyzer.check_current_exception_region(&mut context,region,Type::Void,None,&mut |_,_|{}).unwrap();
        let StmtKind::Let{value,binding,..}=&region.finally.as_ref().unwrap()[0].kind else {panic!("copy");};
        assert_eq!(value.ty,Some(union));
        assert_eq!(context.bindings[binding.unwrap()].ty,union);
        let Some(CallTarget::Function(target))=value.target else {panic!("specialized target");};
        assert_eq!(analyzer.checked[target].as_ref().unwrap().bindings[0].ty,union);
        assert!(contract.explicit.is_empty() && !contract.runtime && !contract.opaque);
    }
    #[test]
    fn canonical_cleanup_closure_keeps_joined_capture_layout() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let union=analyzer.unions.intern([Type::Int,Type::Boolean],Span::new(0,1)).unwrap();
        Analyzer::bind(&mut context,&Token{text:"value".into(),span:Span::new(0,5),kind:TokenKind::Name},union,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{if(typeof value===\"number\"){return;}}finally{const action=()=>value;const copy=action();}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        analyzer.check_typed_exception_region(&mut context,region,Type::Void,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).unwrap();
        let body=region.finally.as_ref().unwrap();
        let StmtKind::Let{value,binding,..}=&body[0].kind else {panic!("action");};
        let Some(Type::Closure(origin))=value.ty else {panic!("closure");};
        assert_eq!(context.bindings[binding.unwrap()].ty,Type::Closure(origin));
        let closure=&analyzer.closures[origin];
        assert_eq!(closure.function.result,union);
        assert_eq!(closure.captures.len(),1);
        assert_eq!(closure.function.bindings[closure.captures[0].inner].ty,union);
        let StmtKind::Let{value,binding,..}=&body[1].kind else {panic!("copy");};
        assert_eq!(value.ty,Some(union));
        assert_eq!(context.bindings[binding.unwrap()].ty,union);
    }
    #[test]
    fn nested_current_regions_refresh_new_generic_contracts() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function identity<T>(value:Owned<T>):T{return value;}function main(){}").unwrap()).unwrap();
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let mut context=Context::new("main".into());
        let mut parsed=crate::syntax::parse("function main(){try{try{const value=identity<string>(\"safe\");}catch(error){return error.length;}}finally{const done=1;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_current_exception_region(&mut context,region,Type::Int,None,&mut |_,_|{}).unwrap();
        assert!(completed.normal.is_some() && completed.exits.is_empty());
        assert!(contract.explicit.is_empty() && !contract.runtime && !contract.opaque);
        assert!(context.exception_environment.is_none());
    }
    #[test]
    fn nested_current_failing_generic_preserves_argument_transfer() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function fail<T>(value:Owned<T>):T{const zero=0;const quotient=1/zero;return value;}function main(){}").unwrap()).unwrap();
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{try{const result=fail<string>(payload);}catch{const length=payload.length;}}finally{const done=1;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let error=analyzer.check_current_exception_region(&mut context,region,Type::Void,None,&mut |_,_|{}).err().unwrap();
        assert_eq!(error.code,"transferred");
        assert!(context.bindings[0].transferred.is_some());
        assert!(context.exception_environment.is_none());
        assert!(context.completion_frame.is_none());
        assert!(context.error_frame.is_none());
    }
    #[test]
    fn nested_current_failing_generic_routes_runtime_payload() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function fail<T>(value:Owned<T>):T{const zero=0;const quotient=1/zero;return value;}function main(){}").unwrap()).unwrap();
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{try{const result=fail<string>(payload);}catch(error){return error.code.length;}}finally{const done=1;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_current_exception_region(&mut context,region,Type::Int,None,&mut |_,_|{}).unwrap();
        assert_eq!(completed.exits.len(),1);
        assert_eq!(completed.exits[0].0,CompletionKind::Return);
        assert!(completed.exits[0].1.owners[0].transferred.is_some());
        assert!(contract.explicit.is_empty() && !contract.runtime && !contract.opaque);
        assert_eq!(analyzer.program.records.len(),1);
        assert!(context.exception_environment.is_none());
    }
    #[test]
    fn nested_discovery_refreshes_new_specialization_with_incomplete_cache() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function identity<T>(value:Owned<T>):T{return value;}function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let parsed=crate::syntax::parse("function main(){try{try{const value=identity<string>(\"safe\");}catch(inner){throw inner;}throw 7;}catch(outer){return outer;}}").unwrap();
        assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),Some(Type::Int));
        assert!(context.exception_environment.is_none());
        assert!(context.names.is_empty());
        assert_eq!(analyzer.depth,0);
    }
    #[test]
    fn forwarding_restores_fallthrough_and_keeps_existing_abrupt_states() {
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let normal=ExitState::capture(&context,1);
        context.bindings[0].transferred=Some(Span::new(10,17));
        let abrupt=ExitState::capture(&context,1);
        context.completion_frame=Some(Box::new(CompletionFrame{outer:1,exits:vec![(CompletionKind::Throw,Span::new(1,2),normal.clone())]}));
        let pending=CompletionStates{normal:Some(normal),exits:vec![(CompletionKind::Return,abrupt)]};
        assert!(!pending.forward_exits(&mut Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap(),&mut context,Span::new(20,30)).unwrap());
        assert!(context.bindings[0].transferred.is_none());
        let frame=context.completion_frame.as_ref().unwrap();
        assert_eq!(frame.exits.len(),2);
        assert_eq!(frame.exits[0].0,CompletionKind::Throw);
        assert_eq!(frame.exits[0].1,Span::new(1,2));
        assert_eq!(frame.exits[1].0,CompletionKind::Return);
        assert!(frame.exits[1].2.owners[0].transferred.is_some());
    }
    #[test]
    fn nested_completions_forward_outer_prefix_without_reviving_owner() {
        let mut context=Context::new("main".into());
        for name in ["outer","inner"] {
            Analyzer::bind(&mut context,&Token{text:name.into(),span:Span::new(0,5),kind:TokenKind::Name},Type::String,false).unwrap();
        }
        context.bindings[0].transferred=Some(Span::new(10,15));
        context.completion_frame=Some(Box::new(CompletionFrame{outer:1,exits:vec![]}));
        let state=ExitState::capture(&context,2);
        let pending=CompletionStates{normal:None,exits:vec![(CompletionKind::Return,state.clone()),(CompletionKind::Throw,state)]};
        assert!(pending.forward_exits(&mut Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap(),&mut context,Span::new(20,40)).unwrap());
        let frame=context.completion_frame.as_ref().unwrap();
        assert_eq!(frame.exits.len(),2);
        for (_,span,state) in &frame.exits {
            assert_eq!(*span,Span::new(20,40));
            assert_eq!(state.owners.len(),1);
            assert!(state.owners[0].transferred.is_some());
        }
    }
    #[test]
    fn discovery_false_branch_narrows_mixed_catch_without_changing_outer_flow() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        for (name,ty) in [("values",Type::Array(ArrayElement::Int(0))),("flag",Type::Boolean)] {
            Analyzer::bind(&mut context,&Token{text:name.into(),span:Span::new(0,name.len()),kind:TokenKind::Name},ty,false).unwrap();
        }
        let parsed=crate::syntax::parse("function main(){try{if(flag){throw 7;}const value=values[9];}catch(error){if(typeof error===\"number\"){throw error;}else{return error.code.length;}}}").unwrap();
        context.flow.insert(crate::flow::Place{root:1,fields:vec![]},crate::flow::Fact{members:vec![Type::Boolean],origin:None});
        let before=context.flow.clone();
        assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),Some(Type::Int));
        assert_eq!(context.flow,before);
        assert_eq!(context.bindings.len(),2);
        assert!(!context.names.contains_key("error"));
        assert!(!context.returning_closure);
        assert_eq!(analyzer.depth,0);
    }
    #[test]
    fn mixed_runtime_and_scalar_catch_discovery_narrows_payload() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        for (name,ty) in [("values",Type::Array(ArrayElement::Int(0))),("flag",Type::Boolean)] {
            Analyzer::bind(&mut context,&Token{text:name.into(),span:Span::new(0,name.len()),kind:TokenKind::Name},ty,false).unwrap();
        }
        let parsed=crate::syntax::parse("function main(){try{if(flag){throw 7;}const value=values[9];}catch(error){if(typeof error===\"number\"){return error;}return error.code.length;}}").unwrap();
        assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),Some(Type::Int));
        assert_eq!(analyzer.program.records.len(),1);
        assert_eq!(context.bindings.len(),2);
    }
    #[test]
    fn ordinary_runtime_catch_discovery_materializes_internal_record_once() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("type RuntimeError={value:int;};function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"values".into(),span:Span::new(0,6),kind:TokenKind::Name},Type::Array(ArrayElement::Int(0)),false).unwrap();
        let parsed=crate::syntax::parse("function main(){try{const value=values[9];}catch(error){return error.code.length;}}").unwrap();
        for _ in 0..2 {
            assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),Some(Type::Int));
            assert_eq!(analyzer.program.records.len(),2);
            assert_eq!(analyzer.program.records[0].name.text,"RuntimeError");
            assert_eq!(analyzer.program.records[1].name.text,"#RuntimeError");
            assert_eq!(analyzer.fields[1]["code"],Type::String);
            assert_eq!(analyzer.fields[1]["start"],Type::Int);
        }
    }
    #[test]
    fn ordinary_result_discovery_infers_direct_throw_catch_payload() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let parsed=crate::syntax::parse("function main(){try{throw \"failure\";}catch(error){return error.length;}}").unwrap();
        assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),Some(Type::Int));
        assert!(context.names.is_empty());
    }
    #[test]
    fn incoming_discovery_handles_unchecked_enclosing_function() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let context=Context::new("main".into());
        let mut parsed=crate::syntax::parse("function main(){try{throw \"failure\";}catch(error){return error.length;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        assert_eq!(analyzer.discover_checked_exception_result(&context,region,Type::Int,None).unwrap(),Some(Type::Int));
        assert!(analyzer.checked[0].is_none());
    }
    #[test]
    fn partial_discovery_contracts_keep_unchecked_functions_opaque() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function fail(value:int):int{return value/0;}function main(){}").unwrap()).unwrap();
        let before=analyzer.discovery_error_effects();
        assert!(before.function_errors.iter().all(|contract|contract.opaque));
        analyzer.function(0).unwrap();
        let after=analyzer.discovery_error_effects();
        assert!(after.function_errors[0].runtime && !after.function_errors[0].opaque);
        assert!(after.function_errors[1].opaque);
        assert!(analyzer.checked_error_effects().is_err());
    }
    #[test]
    fn checked_incoming_discovery_isolates_owned_payload_transfer_and_failure() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        analyzer.function(0).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        for (tail,valid) in [("return error.length;",true),("return missing;",false)] {
            let mut parsed=crate::syntax::parse(&format!("function main(){{try{{throw payload;}}catch(error){{{tail}}}}}")).unwrap();
            let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
            let result=analyzer.discover_checked_exception_result(&context,region,Type::Int,None);
            assert_eq!(result.is_ok(),valid);
            assert!(context.bindings[0].transferred.is_none());
            assert_eq!(context.bindings.len(),1);
            assert_eq!(context.names["payload"],0);
            assert!(!context.names.contains_key("error"));
            assert_eq!(analyzer.depth,0);
            assert!(region.catch.as_ref().unwrap().error_type.is_none());
        }
    }
    #[test]
    fn checked_incoming_contract_drives_catch_result_discovery() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        analyzer.function(0).unwrap();
        let context=Context::new("main".into());
        let mut parsed=crate::syntax::parse("function main(){try{throw \"failure\";}catch(error){return error.length;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        assert!(region.catch.as_ref().unwrap().error_type.is_none());
        assert_eq!(analyzer.discover_checked_exception_result(&context,region,Type::Int,None).unwrap(),Some(Type::Int));
        assert!(context.names.is_empty());
        assert!(region.catch.as_ref().unwrap().error_type.is_none());
    }
    #[test]
    fn failed_return_discovery_restores_callable_permission() {
        for permission in [false,true] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".into());
            context.returning_closure=permission;
            let parsed=crate::syntax::parse("function main(){return missing;}").unwrap();
            assert!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).is_err());
            assert_eq!(context.returning_closure,permission);
            assert_eq!(analyzer.depth,0);
        }
    }
    #[test]
    fn nested_finally_discovery_preserves_outer_cleanup_precedence() {
        for (source,expected) in [
            ("try{try{return 1;}finally{throw 7;}}finally{return 2;}",Some(Type::Int)),
            ("try{try{return 1;}finally{return 2;}}finally{throw 7;}",None),
            ("try{try{throw 7;}finally{return 2;}}finally{const local=3;}",Some(Type::Int)),
        ] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".into());
            let parsed=crate::syntax::parse(&format!("function main(){{{source}}}")).unwrap();
            assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),expected);
            assert_eq!(analyzer.depth,0);
            assert!(context.names.is_empty());
        }
    }
    #[test]
    fn unreachable_catch_does_not_seed_result_discovery() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let mut parsed=crate::syntax::parse("function main(){try{}catch(error){return 7;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        region.catch.as_mut().unwrap().error_contract=Some(Box::default());
        assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),None);
    }
    #[test]
    fn catch_result_discovery_uses_payload_type_and_restores_outer_scope() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"error".into(),span:Span::new(0,5),kind:TokenKind::Name},Type::Int,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{throw \"failure\";}catch(error){return error.length;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        region.catch.as_mut().unwrap().error_type=Some(Type::String);
        assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),Some(Type::Int));
        assert_eq!(context.names["error"],0);
        assert_eq!(context.bindings.len(),1);
    }
    #[test]
    fn finally_result_discovery_respects_abrupt_cleanup_and_local_scope() {
        for (source,expected) in [
            ("try{return 1;}finally{const local=2;return local;}",Some(Type::Int)),
            ("try{return 1;}finally{throw 7;}",None),
            ("try{return 1;}finally{throw 7;}return 3;",None),
            ("try{throw 7;}finally{return 2;}",Some(Type::Int)),
            ("try{return 1;}finally{const local=2;}",Some(Type::Int)),
        ] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".into());
            let parsed=crate::syntax::parse(&format!("function main(){{{source}}}")).unwrap();
            assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),expected,"{source}");
            assert!(context.names.is_empty());
        }
    }
    #[test]
    fn result_discovery_checks_throw_payload_without_seeding_from_unreachable_return() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let parsed=crate::syntax::parse("function main(){throw \"failure\";return 7;}").unwrap();
        assert_eq!(analyzer.discover_body(&mut context,&parsed.functions[0].body,Type::Int).unwrap(),None);
        let invalid=crate::syntax::parse("function main(){throw console.log(7);}").unwrap();
        let error=analyzer.discover_body(&mut context,&invalid.functions[0].body,Type::Int).unwrap_err();
        assert_eq!(error.code,"type");
        assert!(error.message.contains("must produce a value"));
    }
    #[test]
    fn refreshed_handler_contracts_keep_real_failure_after_argument_transfer() {
        let source="function fail<T>(value:Owned<T>):int{let n:int=1;return n/0;}function main(){}";
        let mut analyzer=Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{throw 7;}catch{const value=fail<string>(payload);}finally{const done=1;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_current_exception_region(&mut context,region,Type::Void,None,&mut |_,_|{}).unwrap();
        assert!(completed.normal.is_some());
        assert_eq!(completed.exits.len(),1);
        assert_eq!(completed.exits[0].0,CompletionKind::Throw);
        assert!(completed.exits[0].1.owners[0].transferred.is_some());
        assert!(contract.runtime && !contract.opaque && contract.explicit.is_empty());
    }
    #[test]
    fn refreshed_handler_contracts_remove_spurious_failure_before_owner_transfer() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function identity<T>(value:Owned<T>):T{return value;}function main(){}").unwrap()).unwrap();
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{throw 7;}catch{const value=identity<string>(\"value\");const moved=payload;}finally{const done=1;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_current_exception_region(&mut context,region,Type::Void,None,&mut |_,_|{}).unwrap();
        assert!(completed.normal.is_some() && completed.exits.is_empty());
        assert!(context.bindings[0].transferred.is_some());
        assert!(!contract.runtime && !contract.opaque && contract.explicit.is_empty());
    }
    #[test]
    fn current_region_refreshes_handler_and_finally_specializations() {
        for tail in ["catch{const value=identity<string>(\"caught\");}","finally{const value=identity<string>(\"cleanup\");return;}"] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function identity<T>(value:Owned<T>):T{return value;}function main(){}").unwrap()).unwrap();
            for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
            let mut context=Context::new("main".into());
            let mut parsed=crate::syntax::parse(&format!("function main(){{try{{throw 7;}}{tail}}}")).unwrap();
            let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
            let (_,contract)=analyzer.check_current_exception_region(&mut context,region,Type::Void,None,&mut |_,_|{}).unwrap();
            assert!(!contract.opaque && !contract.runtime && contract.explicit.is_empty(),"{tail}");
        }
    }
    #[test]
    fn current_region_driver_refreshes_generic_body_before_catch_routing() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function identity<T>(value:Owned<T>):T{return value;}function main(){}").unwrap()).unwrap();
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let mut context=Context::new("main".into());
        let mut parsed=crate::syntax::parse("function main(){try{const value=identity<string>(\"value\");}catch{}finally{const done=1;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_current_exception_region(&mut context,region,Type::Void,None,&mut |_,_|{}).unwrap();
        assert!(completed.normal.is_some() && completed.exits.is_empty());
        assert!(!contract.opaque && !contract.runtime && contract.explicit.is_empty());
        let incoming=region.catch.as_ref().unwrap().error_contract.as_ref().unwrap();
        assert!(!incoming.opaque && !incoming.runtime && incoming.explicit.is_empty());
    }
    #[test]
    fn current_body_contracts_refresh_after_checking_new_generic_calls() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function identity<T>(value:Owned<T>):T{return value;}function main(){}").unwrap()).unwrap();
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let mut context=Context::new("main".into());
        let mut parsed=crate::syntax::parse("function main(){const value=identity<string>(\"value\");}").unwrap();
        let (normal,frame,errors,contract,effects)=analyzer.check_current_region_body(&mut context,&mut parsed.functions[0].body,Type::Void).unwrap();
        assert!(normal.is_some() && frame.exits.is_empty() && errors.points.is_empty());
        assert!(!contract.runtime && !contract.opaque && contract.explicit.is_empty());
        assert_eq!(effects.function_errors.len(),analyzer.checked.len());
    }
    #[test]
    fn checked_contract_provider_refreshes_after_named_generic_materialization() {
        let source="function identity<T>(value:Owned<T>):T{return value;}function main(){}";
        let mut analyzer=Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let previous=analyzer.checked_error_effects().unwrap();
        let mut context=Context::new("main".into());
        let mut parsed=crate::syntax::parse("function main(){const value=identity<string>(\"value\");}").unwrap();
        analyzer.check_region_body(&mut context,&mut parsed.functions[0].body,Type::Void).unwrap();
        let refreshed=analyzer.checked_error_effects().unwrap();
        let StmtKind::Let{value,..}=&mut parsed.functions[0].body[0].kind else {panic!("call");};
        let CallTarget::Function(target)=value.target.unwrap() else {panic!("function target");};
        assert!(target>=previous.function_errors.len());
        assert_eq!(refreshed.function_errors.len(),analyzer.checked.len());
        let stale=crate::error_effects::annotate_region_body(&mut parsed.functions[0].body,&previous);
        assert!(stale.opaque);
        let contract=crate::error_effects::annotate_region_body(&mut parsed.functions[0].body,&refreshed);
        assert!(!contract.runtime && !contract.opaque && contract.explicit.is_empty());
    }
    #[test]
    fn checked_contract_provider_preserves_function_identity_for_region_calls() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function fail(value:int):int{return value/0;}function main(){}").unwrap()).unwrap();
        assert_eq!(analyzer.checked_error_effects().err().expect("unchecked functions rejected").code,"error-effect");
        for index in 0..analyzer.original_count {analyzer.function(index).unwrap();}
        let effects=analyzer.checked_error_effects().unwrap();
        let fail=analyzer.function_names["fail"];
        assert!(effects.function_errors[fail].runtime);
        let mut context=Context::new("main".into());
        let mut parsed=crate::syntax::parse("function main(){try{const result=fail(7);}catch{}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_typed_exception_region(&mut context,region,Type::Void,&effects,None,&mut |_,_|{}).unwrap();
        assert!(completed.normal.is_some() && completed.exits.is_empty());
        assert!(region.catch.as_ref().unwrap().error_contract.as_ref().unwrap().runtime);
        assert!(!contract.runtime && !contract.opaque && contract.explicit.is_empty());
    }
    #[test]
    fn runtime_bounds_snapshot_reaches_catch_before_later_owner_transfer() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        for (name,ty) in [("values",Type::Array(ArrayElement::Int(0))),("payload",Type::String)] {
            Analyzer::bind(&mut context,&Token{text:name.into(),span:Span::new(0,name.len()),kind:TokenKind::Name},ty,false).unwrap();
        }
        let mut parsed=crate::syntax::parse("function main(){try{const value=values[4];const moved=payload;}catch{const length=payload.length;}finally{const done=1;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_typed_exception_region(&mut context,region,Type::Void,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).unwrap();
        assert!(completed.normal.is_some() && completed.exits.is_empty());
        assert!(completed.normal.unwrap().owners[1].transferred.is_some());
        assert!(contract.explicit.is_empty() && !contract.runtime && !contract.opaque);
        assert!(region.catch.as_ref().unwrap().error_contract.as_ref().unwrap().runtime);
        assert!(context.bindings[1].transferred.is_some());
    }
    #[test]
    fn conditional_finally_preserves_pending_error_and_adds_replacement_contract() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"flag".into(),span:Span::new(0,4),kind:TokenKind::Name},Type::Boolean,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{throw 7;}finally{if(flag){throw \"replacement\";}}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_typed_exception_region(&mut context,region,Type::Void,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).unwrap();
        assert!(completed.normal.is_none());
        assert_eq!(completed.exits.len(),1);
        assert_eq!(completed.exits[0].0,CompletionKind::Throw);
        assert!(contract.explicit.contains(&Type::Int));
        assert!(contract.explicit.contains(&Type::String));
        assert_eq!(contract.explicit.len(),2);
        assert!(!contract.runtime && !contract.opaque);
    }
    #[test]
    fn source_region_driver_restores_fallthrough_after_abrupt_cleanup_replay() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        for (name,ty) in [("payload",Type::String),("flag",Type::Boolean)] {
            Analyzer::bind(&mut context,&Token{text:name.into(),span:Span::new(0,name.len()),kind:TokenKind::Name},ty,false).unwrap();
        }
        let mut parsed=crate::syntax::parse("function main(){try{if(flag){const moved=payload;return 1;}}finally{const local=2;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,_)=analyzer.check_typed_exception_region(&mut context,region,Type::Int,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).unwrap();
        assert!(completed.normal.is_some());
        assert!(completed.exits[0].1.owners[0].transferred.is_some());
        assert!(context.bindings[0].transferred.is_none());
    }
    #[test]
    fn normal_continuation_restoration_does_not_use_last_abrupt_replay() {
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let normal=ExitState::capture(&context,1);
        context.bindings[0].transferred=Some(Span::new(10,17));
        let abrupt=ExitState::capture(&context,1);
        let pending=CompletionStates{normal:Some(normal),exits:vec![(CompletionKind::Return,abrupt.clone())]};
        assert!(pending.restore_normal(&mut context));
        assert!(context.bindings[0].transferred.is_none());
        assert!(pending.exits[0].1.owners[0].transferred.is_some());
        abrupt.restore(&mut context);
        let unreachable=CompletionStates{normal:None,exits:vec![]};
        assert!(!unreachable.restore_normal(&mut context));
        assert!(context.bindings[0].transferred.is_some());
    }
    #[test]
    fn source_region_recheck_clears_unreachable_handler_metadata() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let mut parsed=crate::syntax::parse("function main(){try{throw 7;}catch(error){}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let effects=crate::error_effects::ErrorEffects::default();
        analyzer.check_typed_exception_region(&mut context,region,Type::Void,&effects,None,&mut |_,_|{}).unwrap();
        assert_eq!(region.catch.as_ref().unwrap().error_type,Some(Type::Int));
        region.body.clear();
        analyzer.check_typed_exception_region(&mut context,region,Type::Void,&effects,None,&mut |_,_|{}).unwrap();
        let handler=region.catch.as_ref().unwrap();
        assert!(handler.binding.is_none() && handler.error_type.is_none());
        assert!(handler.error_contract.as_ref().unwrap().explicit.is_empty());
    }
    #[test]
    fn source_region_driver_preserves_normal_and_throw_contracts_and_finally_precedence() {
        for (source,normal,exit,payload) in [
            ("try{}finally{const local=1;}",true,None,None),
            ("try{throw 7;}finally{const local=1;}",false,Some(CompletionKind::Throw),Some(Type::Int)),
            ("try{throw 7;}catch(error){const local=error;}",true,None,None),
            ("try{return 1;}finally{throw \"replacement\";}",false,Some(CompletionKind::Throw),Some(Type::String)),
            ("try{throw 7;}finally{return 2;}",false,Some(CompletionKind::Return),None),
        ] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".into());
            let mut parsed=crate::syntax::parse(&format!("function main(){{{source}}}")).unwrap();
            let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
            let (completed,contract)=analyzer.check_typed_exception_region(&mut context,region,Type::Int,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).unwrap();
            assert_eq!(completed.normal.is_some(),normal,"{source}");
            assert_eq!(completed.exits.iter().map(|(kind,_)|*kind).collect::<Vec<_>>(),exit.into_iter().collect::<Vec<_>>(),"{source}");
            assert_eq!(contract.explicit,payload.into_iter().collect::<Vec<_>>(),"{source}");
            assert!(!contract.runtime && !contract.opaque,"{source}");
            assert!(context.names.is_empty(),"{source}");
        }
    }
    #[test]
    fn source_region_driver_routes_rethrow_through_finally_return() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{let local=1;throw payload;}catch(error){throw error;}finally{const answer=2;return answer;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (completed,contract)=analyzer.check_typed_exception_region(&mut context,region,Type::Int,&crate::error_effects::ErrorEffects::default(),None,&mut |_,_|{}).unwrap();
        assert!(completed.normal.is_none());
        assert_eq!(completed.exits.len(),1);
        assert_eq!(completed.exits[0].0,CompletionKind::Return);
        assert!(completed.exits[0].1.owners[0].transferred.is_some());
        assert_eq!(completed.exits[0].1.owners.len(),1);
        assert!(contract.explicit.is_empty() && !contract.runtime && !contract.opaque);
        assert_eq!(region.catch.as_ref().unwrap().error_type,Some(Type::String));
        assert!(context.error_frame.is_none() && context.completion_frame.is_none());
        for name in ["local","error","answer"] {assert!(!context.names.contains_key(name));}
    }
    #[test]
    fn finally_replay_reuses_generic_specialization_and_preserves_argument_transfer() {
        let source="function take<T>(value:Owned<T>):T{return value;}function main(){}";
        let mut analyzer=Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let entry=context.clone();
        let state=ExitState::capture(&context,1);
        let mut parsed=crate::syntax::parse("function main(){const moved=take<string>(payload);}").unwrap();
        let source_body=parsed.functions[0].body.clone();
        let mut specialization=None;
        for _ in 0..2 {
            analyzer.check_region_body_at_state(&entry,&mut context,state.clone(),&source_body,&mut parsed.functions[0].body,Type::Void).unwrap();
            assert!(context.bindings[0].transferred.is_some());
            assert_eq!(context.bindings[1].ty,Type::String);
            let StmtKind::Let{value,..}=&parsed.functions[0].body[0].kind else {panic!("call");};
            let current=value.target.expect("resolved generic call");
            if let Some(previous)=specialization {assert_eq!(current,previous);}else{specialization=Some(current);}
        }
        context=entry.clone();
        context.bindings[0].transferred=Some(Span::new(50,57));
        let transferred=ExitState::capture(&context,1);
        let error=analyzer.check_region_body_at_state(&entry,&mut context,transferred,&source_body,&mut parsed.functions[0].body,Type::Void).err().expect("transferred argument rejected");
        assert_eq!(error.code,"transferred");
    }
    #[test]
    fn finally_replay_reuses_captured_closure_identity_and_checks_each_owner_path() {
        let source="function main(){let text=\"payload\";}";
        let mut analyzer=Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"text".into(),span:Span::new(0,4),kind:TokenKind::Name},Type::String,false).unwrap();
        let entry=context.clone();
        let state=ExitState::capture(&context,1);
        let mut parsed=crate::syntax::parse("function main(){const action=():int=>text.length;const length=action();}").unwrap();
        let source_body=parsed.functions[0].body.clone();
        let mut origin=None;
        for _ in 0..2 {
            analyzer.check_region_body_at_state(&entry,&mut context,state.clone(),&source_body,&mut parsed.functions[0].body,Type::Void).unwrap();
            let Type::Closure(current)=context.bindings[1].ty else {panic!("closure");};
            if let Some(previous)=origin {assert_eq!(current,previous);}else{origin=Some(current);}
            assert_eq!(context.bindings.len(),3);
            assert!(analyzer.closures[current].borrowed);
            assert_eq!(analyzer.closures[current].captures[0].outer,0);
            assert!(context.capture_loans.is_empty());
        }
        assert_eq!(analyzer.closures.len(),1);
        context=entry.clone();
        context.bindings[0].transferred=Some(Span::new(50,54));
        let transferred=ExitState::capture(&context,1);
        let error=analyzer.check_region_body_at_state(&entry,&mut context,transferred,&source_body,&mut parsed.functions[0].body,Type::Void).err().expect("transferred capture rejected");
        assert_eq!(error.code,"transferred");
    }
    #[test]
    fn source_finally_replay_keeps_local_slots_and_outer_owner_changes() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        Analyzer::bind(&mut context,&Token{text:"payload".into(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let entry=context.clone();
        let state=ExitState::capture(&context,1);
        let mut pending=CompletionStates{normal:Some(state.clone()),exits:vec![(CompletionKind::Return,state)]};
        let mut parsed=crate::syntax::parse("function main(){const local=7;const moved=payload;}").unwrap();
        let source_body=parsed.functions[0].body.clone();
        let mut passes=0;
        pending=pending.finally(&mut |state| {
            passes+=1;
            let (normal,frame,errors)=analyzer.check_region_body_at_state(&entry,&mut context,state,&source_body,&mut parsed.functions[0].body,Type::Void)?;
            assert!(errors.points.is_empty());
            assert_eq!(context.bindings.len(),3);
            let StmtKind::Let{value,..}=&parsed.functions[0].body[1].kind else {panic!("local");};
            assert_eq!(value.binding,Some(0));
            assert!(context.bindings[0].transferred.is_some());
            Ok(CompletionStates::from_completions(normal,frame,&mut |_,_|{}))
        }).unwrap();
        assert_eq!(passes,2);
        assert!(pending.normal.unwrap().owners[0].transferred.is_some());
        assert_eq!(pending.exits[0].0,CompletionKind::Return);
        assert!(pending.exits[0].1.owners[0].transferred.is_some());
        assert!(!context.names.contains_key("local") && !context.names.contains_key("moved"));
    }
    #[test]
    fn throwing_a_prechecked_borrowed_callable_cannot_escape_its_storage() {
        let source="function main(){let text=\"payload\";const action=()=>text.length;}";
        let mut analyzer=Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        let mut parsed=crate::syntax::parse(source).unwrap();
        analyzer.check_region_body(&mut context,&mut parsed.functions[0].body,Type::Void).unwrap();
        let Type::Closure(origin)=context.bindings[1].ty else {panic!("closure");};
        assert!(analyzer.closures[origin].borrowed);
        context.names.insert("action".into(),1);
        let mut throwing=crate::syntax::parse("function main(){throw action;}").unwrap();
        let StmtKind::Throw(value)=&mut throwing.functions[0].body[0].kind else {panic!("throw");};
        let error=analyzer.check_throw_value(&mut context,value).unwrap_err();
        assert_eq!(error.code,"closure-escape");
        assert!(context.bindings[1].transferred.is_none());
    }
    #[test]
    fn source_thrown_callable_owns_captured_storage_before_scope_exit() {
        let source="function main(){let text=\"payload\";const action=()=>text;throw action;}";
        let mut analyzer=Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        let mut parsed=crate::syntax::parse(source).unwrap();
        let effects=crate::error_effects::ErrorEffects::default();
        let (normal,frame,errors,contract)=analyzer.check_typed_region_body(&mut context,&mut parsed.functions[0].body,Type::Void,&effects).unwrap();
        assert!(normal.is_none() && errors.points.is_empty());
        let Type::Closure(origin)=contract.explicit[0] else {panic!("callable payload");};
        let closure=&analyzer.closures[origin];
        assert!(!closure.borrowed);
        assert!(closure.captures.iter().all(|capture|capture.storage==Mode::Consume));
        assert!(context.bindings[0].transferred.is_some());
        assert!(context.bindings[1].transferred.is_some());
        assert!(context.capture_loans.is_empty());
        assert_eq!(frame.exits.len(),1);
        assert_eq!(frame.exits[0].0,CompletionKind::Throw);
    }
    #[test]
    fn typed_source_catch_routing_projects_dead_locals_and_propagates_rethrow() {
        for rethrow in [true,false] {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        Analyzer::bind(&mut context,&Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let tail=if rethrow {"throw error;"}else{""};
        let mut parsed=crate::syntax::parse(&format!("function main(){{try{{let extra=1;throw payload;}}catch(error){{let count=1;{tail}}}}}")).unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let effects=crate::error_effects::ErrorEffects::default();
        let (normal,frame,errors,incoming)=analyzer.check_typed_region_body(&mut context,&mut region.body,Type::Void,&effects).unwrap();
        assert!(errors.points.is_empty());
        let pending=CompletionStates::from_completions(normal,frame,&mut |_,_|{});
        let mut checked=0;
        let outgoing=pending.catch(&mut |state|{
            checked+=1;state.restore(&mut context);
            let (normal,frame,errors,contract)=analyzer.check_typed_catch_exits(&mut context,region.catch.as_mut().unwrap(),Type::Void,&incoming,&effects,None)?;
            assert!(errors.points.is_empty());
            assert_eq!(contract.explicit,if rethrow {vec![Type::String]}else{vec![]});
            let mut result=CompletionStates::from_completions(normal,frame,&mut |_,_|{});
            result.project_outer(1);
            Ok(result)
        }).unwrap();
        assert_eq!(checked,1);
        assert_eq!(outgoing.exits.len(),usize::from(rethrow));
        let state=if rethrow {
            assert!(outgoing.normal.is_none());
            assert_eq!(outgoing.exits[0].0,CompletionKind::Throw);
            &outgoing.exits[0].1
        }else {outgoing.normal.as_ref().unwrap()};
        assert_eq!(state.owners.len(),1);
        assert!(state.owners[0].transferred.is_some());
        for name in ["extra","error","count"] {assert!(!context.names.contains_key(name));}
        }
    }
    #[test]
    fn rejected_catch_recheck_clears_prior_binding_and_payload_type() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        let mut parsed=crate::syntax::parse("function main(){try{}catch(error){}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let handler=region.catch.as_mut().unwrap();
        let effects=crate::error_effects::ErrorEffects::default();
        let incoming=crate::error_effects::FailureSet{explicit:vec![Type::String],..Default::default()};
        analyzer.check_typed_catch_exits(&mut context,handler,Type::Void,&incoming,&effects,None).unwrap();
        assert!(handler.binding.is_some());
        assert_eq!(handler.error_type,Some(Type::String));
        let unresolved=crate::error_effects::FailureSet{opaque:true,..Default::default()};
        assert!(analyzer.check_typed_catch_exits(&mut context,handler,Type::Void,&unresolved,&effects,None).is_err());
        assert!(handler.binding.is_none());
        assert!(handler.error_type.is_none());
        assert!(handler.error_contract.as_ref().unwrap().opaque);
        assert!(!context.names.contains_key("error"));
    }
    #[test]
    fn checked_throw_contract_drives_catch_binding_and_rethrow_contract() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        Analyzer::bind(&mut context,&Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{throw payload;}catch(error){throw error;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let effects=crate::error_effects::ErrorEffects::default();
        let (_,frame,_,incoming)=analyzer.check_typed_region_body(&mut context,&mut region.body,Type::Void,&effects).unwrap();
        frame.exits[0].2.clone().restore(&mut context);
        let handler=region.catch.as_mut().unwrap();
        assert!(handler.error_type.is_none());
        let (normal,frame,errors,outgoing)=analyzer.check_typed_catch_exits(&mut context,handler,Type::Void,&incoming,&effects,None).unwrap();
        assert_eq!(handler.error_type,Some(Type::String));
        assert_eq!(handler.error_contract.as_ref().unwrap().explicit,vec![Type::String]);
        assert_eq!(outgoing.explicit,vec![Type::String]);
        assert!(normal.is_none() && errors.points.is_empty());
        assert_eq!(frame.exits[0].0,CompletionKind::Throw);
        assert!(frame.exits[0].2.owners[0].transferred.is_some());
        assert!(!context.names.contains_key("error"));
    }
    #[test]
    fn typed_region_body_refines_errors_only_after_checking_and_annotation() {
        let source="function first():int{return 1/0;}function outer(n:int,value:Owned<string>):int{return n;}function main(){let payload=\"x\";outer(first(),payload);}";
        let mut checked=crate::check(source).unwrap().program;
        let effects=crate::error_effects::annotate(&mut checked);
        let mut analyzer=Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        Analyzer::bind(&mut context,&Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let mut parsed=crate::syntax::parse(source).unwrap();
        let main=parsed.functions.iter_mut().find(|function|function.name.text=="main").unwrap();
        main.body.remove(0);
        let (normal,completions,errors,contract)=analyzer.check_typed_region_body(&mut context,&mut main.body,Type::Void,&effects).unwrap();
        assert!(normal.unwrap().owners[0].transferred.is_some());
        assert!(completions.exits.is_empty());
        assert_eq!(errors.points.len(),1);
        assert!(errors.points[0].1.owners[0].transferred.is_none());
        assert!(contract.runtime && !contract.opaque && contract.explicit.is_empty());
        assert!(context.error_frame.is_none() && context.completion_frame.is_none());
        let mut context=Context::new("main".to_owned());
        Analyzer::bind(&mut context,&Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let mut throwing=crate::syntax::parse("function main(){throw payload;}").unwrap();
        let (normal,completions,errors,contract)=analyzer.check_typed_region_body(&mut context,&mut throwing.functions[0].body,Type::Void,&effects).unwrap();
        assert!(normal.is_none() && errors.points.is_empty());
        assert_eq!(completions.exits.len(),1);
        assert_eq!(contract.explicit,vec![Type::String]);
        assert!(!contract.runtime && !contract.opaque);
    }
    #[test]
    fn refined_nested_call_failures_do_not_consume_later_owned_arguments() {
        let source="function first():int{return 1/0;}function outer(n:int,value:Owned<string>):int{return n;}function main(){let payload=\"x\";outer(first(),payload);}";
        let mut checked=crate::check(source).unwrap().program;
        crate::error_effects::annotate(&mut checked);
        let main=checked.functions.iter().find(|function|function.name=="main").unwrap();
        let own=crate::error_effects::OperationEffects::for_body(&main.body);
        let mut analyzer=Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        Analyzer::bind(&mut context,&Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        context.error_frame=Some(Box::new(ErrorFrame{outer:1,points:vec![]}));
        let mut parsed=crate::syntax::parse(source).unwrap();
        let main=parsed.functions.iter_mut().find(|function|function.name.text=="main").unwrap();
        let StmtKind::Expr(call)=&mut main.body[1].kind else {panic!("call");};
        analyzer.expression(&mut context,call,Mode::Copy,&Loans::new(),None).unwrap();
        let mut frame=*context.error_frame.take().unwrap();
        assert_eq!(frame.points.len(),2);
        frame.refine(&mut |span|own.get(span));
        assert_eq!(frame.points.len(),1);
        frame.points.remove(0).1.restore(&mut context);
        assert!(context.bindings[0].transferred.is_none());
    }
    #[test]
    fn checked_source_finally_return_replaces_throw_without_restoring_owner() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        Analyzer::bind(&mut context,&Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{throw payload;}finally{return 2;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let (normal,frame,errors)=analyzer.check_region_body(&mut context,&mut region.body,Type::Int).unwrap();
        assert!(errors.points.is_empty());
        let pending=CompletionStates::from_completions(normal,frame,&mut |_,state|state.unwind_capture_loans(&crate::capture_loan_identity::Boundary::capture(&[]),&[]));
        let mut cleaned=0;
        let completed=pending.finally(&mut |state| {
            cleaned+=1;state.restore(&mut context);
            let (normal,frame,errors)=analyzer.check_region_body(&mut context,region.finally.as_mut().unwrap(),Type::Int)?;
            assert!(errors.points.is_empty());
            Ok(CompletionStates::from_completions(normal,frame,&mut |_,state|state.unwind_capture_loans(&crate::capture_loan_identity::Boundary::capture(&[]),&[])))
        }).unwrap();
        assert_eq!(cleaned,1);
        assert!(completed.normal.is_none());
        assert_eq!(completed.exits.len(),1);
        assert_eq!(completed.exits[0].0,CompletionKind::Return);
        assert!(completed.exits[0].1.owners[0].transferred.is_some());
    }
    #[test]
    fn typed_catch_exits_preserve_incoming_transfers_and_shadow_scope() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        for (name,ty) in [("payload",Type::String),("error",Type::Int)] {
            Analyzer::bind(&mut context,&Token{text:name.to_owned(),span:Span::new(0,7),kind:TokenKind::Name},ty,false).unwrap();
        }
        context.bindings[0].transferred=Some(Span::new(10,17));
        let mut parsed=crate::syntax::parse("function main(){try{}catch(error){throw error;}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let handler=region.catch.as_mut().unwrap();handler.error_type=Some(Type::String);
        let (normal,frame,errors)=analyzer.check_catch_exits(&mut context,handler,Type::Void).unwrap();
        assert!(normal.is_none());
        assert!(errors.points.is_empty());
        assert_eq!(frame.outer,2);
        assert_eq!(frame.exits.len(),1);
        assert_eq!(frame.exits[0].0,CompletionKind::Throw);
        assert_eq!(frame.exits[0].2.owners[0].transferred,Some(Span::new(10,17)));
        assert_eq!(context.names["error"],1);
        assert!(context.completion_frame.is_none() && context.error_frame.is_none());
        let binding=handler.binding.unwrap();
        assert_eq!(context.bindings[binding].ty,Type::String);
        assert!(context.bindings[binding].transferred.is_some());
    }
    #[test]
    fn region_body_collects_exits_in_isolated_frames_and_restores_outer_frames() {
        for source in ["throw payload;","clone(payload);","throw nothing();"] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function nothing(){}function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".to_owned());
            Analyzer::bind(&mut context,&Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
            context.completion_frame=Some(Box::new(CompletionFrame{outer:0,exits:vec![]}));
            context.error_frame=Some(Box::new(ErrorFrame{outer:0,points:vec![]}));
            let mut parsed=crate::syntax::parse(&format!("function main(){{{source}}}")).unwrap();
            let checked=analyzer.check_region_body(&mut context,&mut parsed.functions[0].body,Type::Void);
            assert_eq!(context.completion_frame.as_ref().unwrap().outer,0);
            assert!(context.completion_frame.as_ref().unwrap().exits.is_empty());
            assert_eq!(context.error_frame.as_ref().unwrap().outer,0);
            assert!(context.error_frame.as_ref().unwrap().points.is_empty());
            if source=="throw nothing();" {assert!(checked.is_err());continue;}
            let (normal,completions,errors)=checked.unwrap();
            assert_eq!(completions.outer,1);
            assert_eq!(errors.outer,1);
            if source=="throw payload;" {
                assert!(normal.is_none());
                assert_eq!(completions.exits.len(),1);
                assert!(completions.exits[0].2.owners[0].transferred.is_some());
                assert!(errors.points.is_empty());
            }else {
                assert!(normal.is_some());
                assert!(completions.exits.is_empty());
                assert_eq!(errors.points.len(),1);
            }
        }
    }
    #[test]
    fn source_throw_uses_active_completion_frame_and_remains_guarded_without_one() {
        for enabled in [false,true] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".to_owned());
            if enabled {context.completion_frame=Some(Box::new(CompletionFrame{outer:0,exits:vec![]}));}
            let mut parsed=crate::syntax::parse("function main(){throw 7;}").unwrap();
            let checked=analyzer.block(&mut context,&mut parsed.functions[0].body,Type::Void);
            if enabled {
                assert!(checked.unwrap());
                assert_eq!(context.completion_frame.as_ref().unwrap().exits[0].0,CompletionKind::Throw);
            } else {assert_eq!(checked.unwrap_err().code,"unsupported");}
        }
    }
    #[test]
    fn checked_throw_records_transferred_payload_but_invalid_throw_does_not() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function nothing(){}function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        Analyzer::bind(&mut context,&Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        context.completion_frame=Some(Box::new(CompletionFrame{outer:1,exits:vec![]}));
        let mut parsed=crate::syntax::parse("function main(){throw payload;}").unwrap();
        let StmtKind::Throw(value)=&mut parsed.functions[0].body[0].kind else {panic!("throw");};
        analyzer.check_throw_value(&mut context,value).unwrap();
        let frame=context.completion_frame.as_ref().unwrap();
        assert_eq!(frame.exits.len(),1);
        assert_eq!(frame.exits[0].0,CompletionKind::Throw);
        assert_eq!(frame.exits[0].2.owners[0].transferred,Some(value.span));
        let mut invalid=crate::syntax::parse("function main(){throw nothing();}").unwrap();
        let StmtKind::Throw(value)=&mut invalid.functions[0].body[0].kind else {panic!("throw");};
        assert_eq!(analyzer.check_throw_value(&mut context,value).unwrap_err().code,"type");
        assert_eq!(context.completion_frame.as_ref().unwrap().exits.len(),1);
    }
    #[test]
    fn both_returning_branches_keep_distinct_owner_snapshots() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        for (name,ty) in [("payload",Type::String),("flag",Type::Boolean)] {
            Analyzer::bind(&mut context,&Token{text:name.to_owned(),span:Span::new(0,7),kind:TokenKind::Name},ty,false).unwrap();
        }
        context.completion_frame=Some(Box::new(CompletionFrame{outer:2,exits:vec![]}));
        let mut parsed=crate::syntax::parse("function main(){if(flag){return payload;}else{return clone(payload);}}").unwrap();
        assert!(analyzer.block(&mut context,&mut parsed.functions[0].body,Type::String).unwrap());
        let frame=context.completion_frame.as_ref().unwrap();
        assert_eq!(frame.exits.len(),2);
        assert!(frame.exits.iter().all(|(kind,_,_)|*kind==CompletionKind::Return));
        assert!(frame.exits[0].2.owners[0].transferred.is_some());
        assert!(frame.exits[1].2.owners[0].transferred.is_none());
    }
    #[test]
    fn checked_control_destinations_are_consumed_at_their_enclosing_boundary() {
        for (source,continuing) in [("outer: {break outer;}",false),("outer: while(true){continue outer;}",true)] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".to_owned());
            context.completion_frame=Some(Box::new(CompletionFrame{outer:0,exits:vec![]}));
            let mut parsed=crate::syntax::parse(&format!("function main(){{{source}}}")).unwrap();
            analyzer.block(&mut context,&mut parsed.functions[0].body,Type::Void).unwrap();
            let frame=context.completion_frame.as_ref().unwrap();
            assert!(frame.exits.is_empty(),"{source}");
            let body=match &parsed.functions[0].body[0].kind {StmtKind::Block(body)|StmtKind::While{body,..}=>body,_=>panic!("target")};
            match body[0].control_target {
                Some(crate::syntax::ControlTarget::Block(_)) if !continuing=>{},
                Some(crate::syntax::ControlTarget::Loop(_)) if continuing=>{},
                kind=>panic!("unexpected destination {kind:?}"),
            }
        }
    }
    #[test]
    fn checked_return_records_owner_transfer_before_scope_cleanup() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        Analyzer::bind(&mut context,&Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        context.completion_frame=Some(Box::new(CompletionFrame{outer:1,exits:vec![]}));
        let mut parsed=crate::syntax::parse("function main(){return payload;}").unwrap();
        assert!(analyzer.block(&mut context,&mut parsed.functions[0].body,Type::String).unwrap());
        let frame=context.completion_frame.as_ref().unwrap();
        assert_eq!(frame.exits.len(),1);
        assert_eq!(frame.exits[0].0,CompletionKind::Return);
        assert!(frame.exits[0].2.owners[0].transferred.is_some());
    }
    #[test]
    fn nested_region_projection_keeps_outer_moves_and_discards_inner_flow() {
        let mut context=Context::new("main".to_owned());
        for name in ["outer","inner"] {
            Analyzer::bind(&mut context,&Token{text:name.to_owned(),span:Span::new(0,5),kind:TokenKind::Name},Type::String,false).unwrap();
        }
        context.bindings[0].transferred=Some(Span::new(10,15));
        for root in [0,1] {
            context.flow.insert(crate::flow::Place{root,fields:vec![]},crate::flow::Fact{members:vec![Type::String],origin:None});
        }
        let mut nested=ExitState::capture(&context,2);
        nested.project_outer(1);
        let entry=ExitState::capture(&context,1);
        let joined=entry.join(nested);
        assert_eq!(joined.owners.len(),1);
        assert_eq!(joined.flow.len(),1);
        joined.restore(&mut context);
        assert_eq!(context.bindings[0].transferred,Some(Span::new(10,15)));
        assert!(context.flow.keys().all(|place|place.root==0));
    }
    #[test]
    fn recorded_failures_join_after_lexical_cleanup_without_restoring_arguments() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function first():int{return 1/0;}function outer(n:int,value:Owned<string>):int{return n;}function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        Analyzer::bind(&mut context,&Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name},Type::String,false).unwrap();
        Analyzer::bind(&mut context,&Token{text:"stable".to_owned(),span:Span::new(8,14),kind:TokenKind::Name},Type::Int,false).unwrap();
        context.error_frame=Some(Box::new(ErrorFrame{outer:2,points:vec![]}));
        context.capture_loans.push((1,Mode::Read,Span::new(30,31)));
        let mut parsed=crate::syntax::parse("function main(){outer(first(),payload);}").unwrap();
        let StmtKind::Expr(value)=&mut parsed.functions[0].body[0].kind else {panic!("call");};
        analyzer.expression(&mut context,value,Mode::Copy,&Loans::new(),None).unwrap();
        let frame=*context.error_frame.take().unwrap();
        assert_eq!(frame.points.len(),2);
        let mut completions=CompletionStates::default();
        let mut visited=vec![];
        completions.record_failures(frame,&mut |span,state|{visited.push(span);state.unwind_capture_loans(&crate::capture_loan_identity::Boundary::capture(&[]),&[]);});
        assert_eq!(visited.len(),2);
        let mut handled=0;
        let output=completions.catch(&mut |state|{
            handled+=1;
            state.restore(&mut context);
            assert!(context.bindings[0].transferred.is_some());
            assert!(context.capture_loans.is_empty());
            Ok(CompletionStates::default())
        }).unwrap();
        assert_eq!(handled,1);
        assert!(output.exits.is_empty());
    }
    #[test]
    fn exception_unwind_does_not_keep_local_loan_in_removed_entry_position() {
        let mut context = Context::new("main".into());
        let first = (0, Mode::Read, Span::new(1, 2));
        let outer = (1, Mode::Read, Span::new(3, 4));
        let local = (2, Mode::Mutate, Span::new(5, 6));
        let payload = (3, Mode::Read, Span::new(7, 8));
        context.capture_loans = vec![outer, local, payload];
        let mut state = ExitState::capture(&context, 0);
        let entry = crate::capture_loan_identity::Boundary::capture(&[first, outer]);
        state.unwind_capture_loans(&entry, &[payload]);
        assert_eq!(state.capture_loans, vec![outer, payload]);
    }
    #[test]
    fn lexical_unwind_releases_local_capture_loans_but_preserves_payload_loans() {
        let mut context=Context::new("main".to_owned());
        Analyzer::bind(&mut context,&Token{text:"owner".to_owned(),span:Span::new(0,5),kind:TokenKind::Name},Type::String,false).unwrap();
        let outer=(0,Mode::Read,Span::new(1,2));
        let local=(0,Mode::Mutate,Span::new(3,4));
        let payload=(0,Mode::Read,Span::new(5,6));
        context.capture_loans=vec![outer,local,payload];
        context.bindings[0].transferred=Some(Span::new(7,8));
        let mut state=ExitState::capture(&context,1);
        state.unwind_capture_loans(&crate::capture_loan_identity::Boundary::capture(&[outer]),&[outer,payload,payload]);
        state.restore(&mut context);
        assert_eq!(context.capture_loans,vec![outer,payload]);
        assert_eq!(context.bindings[0].transferred,Some(Span::new(7,8)));
        let mut discarded=ExitState::capture(&context,1);
        discarded.unwind_capture_loans(&crate::capture_loan_identity::Boundary::capture(&[outer]),&[]);
        discarded.restore(&mut context);
        assert_eq!(context.capture_loans,vec![outer]);
    }
    #[test]
    fn integer_mutations_record_arithmetic_failure_after_rhs_evaluation() {
        for (source,count) in [("left++;",1),("left += right;",1),("left += -right;",2)] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".to_owned());
            for name in ["left","right"] {
                let id=Analyzer::bind(&mut context,&Token{text:name.to_owned(),span:Span::new(0,1),kind:TokenKind::Name},Type::Int,false).unwrap();
                context.bindings[id].rebindable=true;
            }
            context.error_frame=Some(Box::new(ErrorFrame{outer:2,points:vec![]}));
            let mut parsed=crate::syntax::parse(&format!("function main(){{{source}}}")).unwrap();
            let statement=&mut parsed.functions[0].body[0];
            match &mut statement.kind {
                StmtKind::Expr(value)=>{analyzer.expression(&mut context,value,Mode::Copy,&Loans::new(),None).unwrap();},
                StmtKind::Assign{target,value,operator:Some((op,span))}=>{analyzer.compound_assign(&mut context,target,value,*op,*span).unwrap();},
                _=>panic!("mutation"),
            }
            let points=&context.error_frame.as_ref().unwrap().points;
            assert_eq!(points.len(),count,"{source}");
            assert!(points.iter().all(|(_,state)|state.owners.iter().all(|owner|owner.transferred.is_none())));
        }
    }
    #[test]
    fn bounds_failure_precedes_later_owned_argument_transfer() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function outer(n:int,value:Owned<string>):int{return n;}function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        for (name,ty) in [("values",Type::Array(ArrayElement::Int(0))),("payload",Type::String)] {
            Analyzer::bind(&mut context,&Token{text:name.to_owned(),span:Span::new(0,1),kind:TokenKind::Name},ty,false).unwrap();
        }
        context.error_frame=Some(Box::new(ErrorFrame{outer:2,points:vec![]}));
        let mut parsed=crate::syntax::parse("function main(){outer(values[0],payload);}").unwrap();
        let StmtKind::Expr(value)=&mut parsed.functions[0].body[0].kind else {panic!("call");};
        analyzer.expression(&mut context,value,Mode::Copy,&Loans::new(),None).unwrap();
        let frame=context.error_frame.as_ref().unwrap();
        assert_eq!(frame.points.len(),2);
        assert!(frame.points[0].1.owners[1].transferred.is_none());
        assert!(frame.points[1].1.owners[1].transferred.is_some());
    }
    #[test]
    fn checked_integer_operations_record_but_ieee_arithmetic_does_not() {
        for (ty,count) in [(Type::Int,2),(Type::Number,0)] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".to_owned());
            for name in ["left","right"] {Analyzer::bind(&mut context,&Token{text:name.to_owned(),span:Span::new(0,1),kind:TokenKind::Name},ty,false).unwrap();}
            context.error_frame=Some(Box::new(ErrorFrame{outer:2,points:vec![]}));
            let mut parsed=crate::syntax::parse("function main(){-left+right;}").unwrap();
            let StmtKind::Expr(value)=&mut parsed.functions[0].body[0].kind else {panic!("arithmetic");};
            analyzer.expression(&mut context,value,Mode::Copy,&Loans::new(),None).unwrap();
            assert_eq!(context.error_frame.as_ref().unwrap().points.len(),count);
        }
    }
    #[test]
    fn nested_argument_failure_precedes_later_owned_argument_transfer() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function first():int{return 1/0;}function outer(n:int,value:Owned<string>):int{return 1/0;}function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        let token=Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name};
        Analyzer::bind(&mut context,&token,Type::String,false).unwrap();
        context.error_frame=Some(Box::new(ErrorFrame{outer:1,points:vec![]}));
        let mut parsed=crate::syntax::parse("function main(){outer(first(),payload);}").unwrap();
        let StmtKind::Expr(value)=&mut parsed.functions[0].body[0].kind else {panic!("call");};
        analyzer.expression(&mut context,value,Mode::Copy,&Loans::new(),None).unwrap();
        let frame=context.error_frame.as_ref().unwrap();
        assert_eq!(frame.points.len(),2);
        assert!(frame.points[0].1.owners[0].transferred.is_none());
        assert!(frame.points[1].1.owners[0].transferred.is_some());
        assert!(frame.points[0].0.start>frame.points[1].0.start);
    }
    #[test]
    fn catch_routing_removes_only_throw_and_retains_other_destinations() {
        let context=Context::new("main".to_owned());
        let state=ExitState::capture(&context,0);
        let mut flow=CompletionStates::default();
        for kind in [CompletionKind::Return,CompletionKind::Throw,CompletionKind::Break(Some(ControlTarget::Block(7)))] {flow.record(kind,state.clone());}
        assert!(flow.take_throw().is_some());
        assert!(flow.take_throw().is_none());
        assert_eq!(flow.exits.iter().map(|(kind,_)|*kind).collect::<Vec<_>>(),vec![CompletionKind::Return,CompletionKind::Break(Some(ControlTarget::Block(7)))]);
    }
    #[test]
    fn handler_rethrow_escapes_instead_of_reentering_the_same_handler() {
        let context=Context::new("main".to_owned());let state=ExitState::capture(&context,0);
        let mut pending=CompletionStates::default();
        pending.record(CompletionKind::Throw,state.clone());pending.record(CompletionKind::Return,state);
        let mut calls=0;
        let output=pending.catch(&mut |state|{calls+=1;let mut handler=CompletionStates::default();handler.record(CompletionKind::Throw,state);Ok(handler)}).unwrap();
        assert_eq!(calls,1);assert_eq!(output.exits.len(),2);
        assert!(output.exits.iter().any(|(kind,_)|*kind==CompletionKind::Throw));
        assert!(output.exits.iter().any(|(kind,_)|*kind==CompletionKind::Return));
    }
    #[test]
    fn error_join_never_restores_a_transferred_throw_payload() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        let token=Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name};
        Analyzer::bind(&mut context,&token,Type::String,false).unwrap();
        let before=ExitState::capture(&context,1);
        let mut value=Expr::new(ExprKind::Name("payload".to_owned()),Span::new(10,17));
        analyzer.check_throw_value(&mut context,&mut value).unwrap();
        let after=ExitState::capture(&context,1);
        before.join(after).restore(&mut context);
        assert_eq!(context.bindings[0].transferred,Some(value.span));
    }
    #[test]
    fn error_join_preserves_partial_moves_and_widens_flow_and_capture_loans() {
        let mut context=Context::new("main".to_owned());
        let token=Token{text:"owner".to_owned(),span:Span::new(0,5),kind:TokenKind::Name};
        Analyzer::bind(&mut context,&token,Type::Record(0),false).unwrap();
        let place=crate::flow::Place{root:0,fields:vec!["name".to_owned()]};
        context.flow.insert(place.clone(),crate::flow::Fact{members:vec![Type::Int],origin:None});
        let first=ExitState::capture(&context,1);
        let moved=Span::new(10,15);
        context.bindings[0].moved_fields.insert(vec!["name".to_owned()],moved);
        context.flow.insert(place.clone(),crate::flow::Fact{members:vec![Type::String],origin:None});
        context.capture_loans.push((0,Mode::Read,moved));
        let second=ExitState::capture(&context,1);
        first.join(second).restore(&mut context);
        assert_eq!(context.bindings[0].moved_fields.get(&vec!["name".to_owned()]),Some(&moved));
        assert_eq!(context.flow[&place].members,vec![Type::Int,Type::String]);
        assert_eq!(context.capture_loans,vec![(0,Mode::Read,moved)]);
    }
    #[test]
    fn argument_transfer_survives_a_potentially_failing_call_snapshot() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function consume(value:Owned<string>):int{return 1/0;}function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        let token=Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name};
        Analyzer::bind(&mut context,&token,Type::String,false).unwrap();
        let before=ExitState::capture(&context,1);
        let mut parsed=crate::syntax::parse("function main(){throw consume(payload);}").unwrap();
        let StmtKind::Throw(value)=&mut parsed.functions[0].body[0].kind else {panic!("throw");};
        assert_eq!(analyzer.check_throw_value(&mut context,value).unwrap(),Type::Int);
        let ExprKind::Call(_,args)=&value.kind else {panic!("call");};
        let transferred=args[0].span;
        let after=ExitState::capture(&context,1);
        before.join(after).restore(&mut context);
        assert_eq!(context.bindings[0].transferred,Some(transferred));
    }
    #[test]
    fn completion_joins_keep_return_throw_and_control_targets_distinct() {
        let context=Context::new("main".to_owned());
        let state=ExitState::capture(&context,0);
        let mut flow=CompletionStates::default();
        for kind in [CompletionKind::Return,CompletionKind::Throw,CompletionKind::Break(Some(ControlTarget::Block(10))),CompletionKind::Break(Some(ControlTarget::Block(20))),CompletionKind::Return] {
            flow.record(kind,state.clone());
        }
        assert_eq!(flow.exits.len(),4);
    }
    #[test]
    fn abrupt_cleanup_replaces_pending_exit_while_normal_cleanup_preserves_it() {
        let context=Context::new("main".to_owned());
        let state=ExitState::capture(&context,0);
        let mut pending=CompletionStates::default();pending.record(CompletionKind::Throw,state.clone());
        let output=pending.finally(&mut |state| {
            let mut cleanup=CompletionStates::default();cleanup.record(CompletionKind::Return,state);Ok(cleanup)
        }).unwrap();
        assert_eq!(output.exits[0].0,CompletionKind::Return);
        let target=CompletionKind::Continue(Some(ControlTarget::Tail(42)));
        let mut pending=CompletionStates::default();pending.record(target,state);
        let output=pending.finally(&mut |state|Ok(CompletionStates{normal:Some(state),exits:vec![]})).unwrap();
        assert_eq!(output.exits[0].0,target);
        assert!(output.normal.is_none());
    }
    #[test]
    fn cleanup_owner_changes_reach_normal_return_and_targeted_paths() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function sink(value:Owned<string>){}function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        let token=Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name};
        Analyzer::bind(&mut context,&token,Type::String,false).unwrap();
        let state=ExitState::capture(&context,1);
        let mut pending=CompletionStates{normal:Some(state.clone()),exits:vec![]};
        pending.record(CompletionKind::Return,state.clone());
        pending.record(CompletionKind::Break(Some(ControlTarget::Block(42))),state);
        let mut parsed=crate::syntax::parse("function main(){sink(payload);}").unwrap();
        let StmtKind::Expr(template)=parsed.functions[0].body.remove(0).kind else {panic!("call");};
        let mut count=0;
        let output=pending.finally(&mut |state| {
            count+=1;state.restore(&mut context);
            let mut value=template.clone();
            assert_eq!(analyzer.expression(&mut context,&mut value,Mode::Copy,&Loans::new(),None)?,Type::Void);
            Ok(CompletionStates{normal:Some(ExitState::capture(&context,1)),exits:vec![]})
        }).unwrap();
        assert_eq!(count,3);
        assert!(output.normal.unwrap().owners[0].transferred.is_some());
        assert_eq!(output.exits.len(),2);
        assert!(output.exits.iter().all(|(_,state)|state.owners[0].transferred.is_some()));
    }
}
