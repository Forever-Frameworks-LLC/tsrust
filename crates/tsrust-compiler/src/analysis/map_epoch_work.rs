//! One continuation stack for Map storage bodies, joins and cleanup replay.
use super::*;
use std::collections::{BTreeSet,VecDeque};
type Results=HashMap<usize,Type>;
type Places=BTreeSet<crate::capture_loan_owners::Place>;
type Epochs=Vec<storage_exit_availability::MapIterationEpoch>;
#[derive(Clone,Copy)]
enum Slot {Block,Then,Else,Loop,Update,Try,Catch,Finally,Phase}
impl Slot {
    fn body(self,statement:&mut Stmt)->&mut Vec<Stmt> {
        match (self,&mut statement.kind) {
            (Self::Block,StmtKind::Block(body))=>body,
            (Self::Then,StmtKind::If{then_body,..})=>then_body,
            (Self::Else,StmtKind::If{else_body,..})=>else_body,
            (Self::Loop,StmtKind::While{body,..}|StmtKind::Loop{body,..}|StmtKind::ForOf{body,..})=>body,
            (Self::Update,StmtKind::Loop{update,..})=>update,
            (Self::Try,StmtKind::Try(region))=>&mut region.body,
            (Self::Catch,StmtKind::Try(region))=>&mut region.catch.as_mut().expect("catch").body,
            (Self::Finally,StmtKind::Try(region))=>region.finally.as_mut().expect("finally"),
            _=>unreachable!("scheduled Map body"),
        }
    }
}
enum Return {
    PlainBlock,
    LabelledBlock(Vec<storage_exit_availability::MapEntrySurvivors>,Vec<crate::capture_loan_owners::Place>),
    Branch(bool),IterationBody,IterationUpdate,Discard,
    Phase(usize,Option<crate::completion::CompletionKind>),
}
#[derive(Clone,Copy)]
enum Stage {Begin,Statement,Then,Else,Iteration,Update,IterationEnd,Try,Catch,Finally,Phases,ExceptionEnd,FallbackLoop,Finish,Next,Done}
struct Iteration {epochs:Epochs,inner:Results,local:Places}
struct Frame {
    body:Vec<Stmt>,at:usize,results:Results,local:Places,stage:Stage,end:usize,
    waiting:Option<(Slot,Return)>,iteration:Option<Iteration>,
    phases:Option<VecDeque<map_completion_payloads::Snapshot>>,
    // Cleanup replay swaps a private context in while it runs. Ordinary child
    // frames share the current context and never clone it.
    previous:Option<Box<Context>>,
}
impl Frame {
    fn new(body:Vec<Stmt>,results:Results,local:Places)->Self {
        Self{body,at:0,results,local,stage:Stage::Begin,end:0,waiting:None,iteration:None,phases:None,previous:None}
    }
    fn child(&mut self,slot:Slot,result:Return,results:Results,local:Places)->Self {
        self.waiting=Some((slot,result));
        Self::new(std::mem::take(slot.body(&mut self.body[self.at])),results,local)
    }
    fn restore(&mut self,child:&mut Frame)->Return {
        let (slot,result)=self.waiting.take().expect("suspended Map body");
        if !matches!(slot,Slot::Phase) {*slot.body(&mut self.body[self.at])=std::mem::take(&mut child.body);}
        result
    }
}
impl Analyzer {
    pub(super) fn scoped_map_entry_epochs_with_demand(&mut self,context:&mut Context,body:&mut [Stmt],results:&mut Results,demand:&map_entry_demand::Demand,inherited:&Places)->Result<Places,Diagnostic> {
        // Move source ownership to frames without cloning descendants. Even a
        // diagnostic restores every extracted child before returning the tree.
        let owned=body.iter_mut().map(|statement|{
            let kind=std::mem::replace(&mut statement.kind,StmtKind::Break);
            let mut owned=statement.clone();owned.kind=kind;owned
        }).collect();
        let mut frames=vec![Frame::new(owned,std::mem::take(results),inherited.clone())];
        let outcome=(|| {
            loop {
                let frame=frames.last_mut().expect("root Map frame");
                if matches!(frame.stage,Stage::Done) {
                    if frames.len()==1 {return Ok(());}
                    let mut child=frames.pop().unwrap();
                    let parent=frames.last_mut().unwrap();
                    let result=parent.restore(&mut child);
                    let phase=child.previous.take().map(|mut previous|{std::mem::swap(context,&mut previous);previous});
                    self.resume_map_epoch_child(context,parent,&mut child,result,phase)?;
                } else if let Some(child)=self.step_map_epoch_frame(context,frame,demand)? {
                    frames.push(child);
                }
            }
        })();
        // A failed leaf leaves the pending ancestors intact. Reattach bodies
        // and unwind private cleanup contexts without publishing their plans.
        while frames.len()>1 {
            let mut child=frames.pop().unwrap();
            let parent=frames.last_mut().unwrap();
            let result=parent.restore(&mut child);
            if matches!(result,Return::PlainBlock){parent.results=std::mem::take(&mut child.results);}
            if let Some(mut previous)=child.previous.take(){std::mem::swap(context,&mut previous);}
        }
        let mut frame=frames.pop().unwrap();
        *results=std::mem::take(&mut frame.results);
        for (destination,source) in body.iter_mut().zip(frame.body){*destination=source;}
        outcome.map(|()|frame.local)
    }
    fn resume_map_epoch_child(&mut self,context:&mut Context,parent:&mut Frame,child:&mut Frame,result:Return,phase:Option<Box<Context>>)->Result<(),Diagnostic> {
        let statement=&mut parent.body[parent.at];
        match result {
            Return::PlainBlock=>{parent.results=std::mem::take(&mut child.results);parent.local=std::mem::take(&mut child.local);},
            Return::LabelledBlock(states,eligible)=>self.finish_map_block_storage(context,statement,&mut parent.results,&child.results,&child.local,&states,&eligible)?,
            Return::Branch(right)=>{
                let StmtKind::If{then_body,else_body,..}=&statement.kind else{unreachable!()};
                self.finish_map_branch_entries(context,statement.span,right,if right{else_body}else{then_body},&parent.results,&child.results,&parent.local);
            },
            Return::IterationBody=>{
                let iteration=parent.iteration.as_mut().unwrap();iteration.inner=std::mem::take(&mut child.results);
                let target=statement.span.start;
                self.plan_map_iteration_body_family(context,Slot::Loop.body(statement),target,&mut iteration.inner,&mut iteration.epochs)?;
            },
            Return::IterationUpdate=>parent.iteration.as_mut().unwrap().inner=std::mem::take(&mut child.results),
            Return::Discard=>{},
            Return::Phase(boundary,kind)=>{
                let phase=phase.expect("private cleanup context");
                let binding_types=phase.bindings.iter().zip(&context.bindings).enumerate().filter_map(|(binding,(phase,original))|(phase.ty!=original.ty).then_some((binding,phase.ty))).collect();
                context.storage_exits.record_map_cleanup_plan(boundary,kind,map_cleanup_phase_plan::Plan{body:std::mem::take(&mut child.body),binding_types,exits:Box::new(phase.storage_exits)});
            },
        }
        Ok(())
    }
    fn step_map_epoch_frame(&mut self,context:&mut Context,frame:&mut Frame,demand:&map_entry_demand::Demand)->Result<Option<Frame>,Diagnostic> {
        match frame.stage {
            Stage::Begin=>{
                frame.stage=if self.begin_map_epoch_body(context,&frame.body,&frame.results,&mut frame.local){Stage::Statement}else{Stage::Done};
            },
            Stage::Statement=>{
                if frame.at==frame.body.len(){
                    if let Some(boundary)=frame.body.first().map(|statement|statement.span.start){
                        let types=self.scoped_map_storage_inputs(&frame.results);
                        context.storage_exits.record_map_native_exit(boundary,types);
                    }
                    frame.stage=Stage::Done;return Ok(None);
                }
                let statement=&mut frame.body[frame.at];
                if matches!(statement.kind,StmtKind::Break|StmtKind::Continue){context.storage_exits.record_map_control_inputs(statement.span,&frame.results);}
                self.move_map_entry_places(statement,&mut frame.local);
                if matches!(statement.kind,StmtKind::Block(_)){
                    frame.stage=Stage::Next;
                    let result=if statement.labels.is_empty(){Return::PlainBlock}else{
                        let states=context.storage_exits.map_completion_survivors(statement.span).to_vec();
                        let eligible=states.iter().filter(|state|state.consistent && frame.local.contains(&state.place) && !state.released.is_empty()).map(|state|state.place.clone()).collect::<Vec<_>>();
                        if eligible.is_empty(){return Ok(None);}
                        Return::LabelledBlock(states,eligible)
                    };
                    let results=if matches!(result,Return::PlainBlock){std::mem::take(&mut frame.results)}else{frame.results.clone()};
                    return Ok(Some(frame.child(Slot::Block,result,results,frame.local.clone())));
                }
                frame.end=self.prepare_map_epoch_statement(context,statement,&mut frame.results,&mut frame.local)?;
                frame.stage=Stage::Then;
            },
            Stage::Then|Stage::Else=>{
                let right=matches!(frame.stage,Stage::Else);frame.stage=if right{Stage::Iteration}else{Stage::Else};
                if matches!(frame.body[frame.at].kind,StmtKind::If{..}){
                    return Ok(Some(frame.child(if right{Slot::Else}else{Slot::Then},Return::Branch(right),frame.results.clone(),frame.local.clone())));
                }
            },
            Stage::Iteration=>{
                frame.stage=Stage::Try;
                let statement=&frame.body[frame.at];
                let epochs=context.storage_exits.map_iteration_epochs(statement.span).to_vec();
                if !epochs.is_empty() && matches!(statement.kind,StmtKind::While{..}|StmtKind::Loop{..}|StmtKind::ForOf{..}){
                    let local=epochs.iter().map(|epoch|epoch.entry.place.clone()).collect::<Places>();
                    let mut inner=frame.results.clone();
                    for epoch in &epochs {
                        let root=inner[&epoch.entry.place.binding];
                        let working=self.scoped_array_field_type(epoch.working,&epoch.entry.place.fields).expect("working Map path");
                        inner.insert(epoch.entry.place.binding,self.scoped_array_replace_field(root,&epoch.entry.place.fields,working).expect("working Map path"));
                    }
                    frame.iteration=Some(Iteration{epochs,inner:Results::new(),local:local.clone()});frame.stage=Stage::Update;
                    return Ok(Some(frame.child(Slot::Loop,Return::IterationBody,inner,local)));
                }
            },
            Stage::Update=>{
                frame.stage=Stage::IterationEnd;
                if matches!(frame.body[frame.at].kind,StmtKind::Loop{tail_reachable:true,..}){
                    let iteration=frame.iteration.as_mut().unwrap();let inner=std::mem::take(&mut iteration.inner);let local=iteration.local.clone();
                    return Ok(Some(frame.child(Slot::Update,Return::IterationUpdate,inner,local)));
                }
            },
            Stage::IterationEnd=>{
                let iteration=frame.iteration.take().unwrap();
                self.finish_map_iteration_body(context,&frame.body[frame.at],&frame.results,&iteration.inner,iteration.epochs)?;
                frame.stage=Stage::Try;
            },
            Stage::Try=>{
                frame.stage=Stage::Catch;
                if matches!(frame.body[frame.at].kind,StmtKind::Try(_)){
                    return Ok(Some(frame.child(Slot::Try,Return::Discard,frame.results.clone(),frame.local.clone())));
                }
            },
            Stage::Catch=>{
                frame.stage=Stage::Finally;
                if let StmtKind::Try(region)=&frame.body[frame.at].kind && region.catch.is_some(){
                    let mut inner=frame.results.clone();
                    if let Some(layout)=region.body.first().and_then(|statement|context.storage_exits.map_completion_layout(statement.span.start,map_completion_payloads::Phase::Body,Some(crate::completion::CompletionKind::Throw))){
                        for (place,leaf) in layout {
                            if let Some(root)=inner.get(&place.binding).copied() && let Some(output)=self.scoped_array_replace_field(root,&place.fields,*leaf){inner.insert(place.binding,output);}
                        }
                    }
                    return Ok(Some(frame.child(Slot::Catch,Return::Discard,inner,frame.local.clone())));
                }
            },
            Stage::Finally=>{
                frame.stage=Stage::Phases;
                if let StmtKind::Try(region)=&frame.body[frame.at].kind && region.finally.is_some(){
                    return Ok(Some(frame.child(Slot::Finally,Return::Discard,frame.results.clone(),frame.local.clone())));
                }
            },
            Stage::Phases=>{
                let StmtKind::Try(region)=&frame.body[frame.at].kind else{frame.stage=Stage::FallbackLoop;return Ok(None);};
                let (Some(cleanup),Some(boundary))=(&region.finally,region.body.first().map(|statement|statement.span.start)) else{frame.stage=Stage::ExceptionEnd;return Ok(None);};
                let phases=frame.phases.get_or_insert_with(||context.storage_exits.map_completion_snapshots(boundary).into_iter().filter(|state|state.phase==map_completion_payloads::Phase::Handled && state.kind.is_some()).collect());
                let Some(state)=phases.pop_front() else{frame.phases=None;frame.stage=Stage::ExceptionEnd;return Ok(None);};
                let Some(layout)=context.storage_exits.map_completion_layout(boundary,state.phase,state.kind) else{return Ok(None);};
                if layout.is_empty(){return Ok(None);}
                let mut input=frame.results.clone();
                for (place,leaf) in layout {
                    if let Some(root)=input.get(&place.binding).copied() && let Some(output)=self.scoped_array_replace_field(root,&place.fields,*leaf){input.insert(place.binding,output);}
                }
                let mut phase=Box::new(context.clone());phase.storage_exits.reset_map_cleanup_native_plans();
                let mut child=Frame::new(cleanup.clone(),input,frame.local.clone());
                std::mem::swap(context,&mut phase);child.previous=Some(phase);
                frame.waiting=Some((Slot::Phase,Return::Phase(boundary,state.kind)));
                return Ok(Some(child));
            },
            Stage::ExceptionEnd=>{
                let StmtKind::Try(region)=&frame.body[frame.at].kind else{unreachable!()};
                self.plan_map_exception_storage_exit(context,region,&mut frame.results,&frame.local);
                let inputs=self.scoped_map_storage_inputs(&frame.results);
                context.storage_exits.record_map_native_inputs(frame.end,inputs);
                frame.stage=Stage::FallbackLoop;
            },
            Stage::FallbackLoop=>{
                frame.stage=Stage::Finish;
                let statement=&frame.body[frame.at];
                if context.storage_exits.map_iteration_epochs(statement.span).is_empty() && matches!(statement.kind,StmtKind::While{..}|StmtKind::Loop{..}|StmtKind::ForOf{..}){
                    let mut inherited=frame.local.clone();
                    for state in context.storage_exits.map_completion_survivors(statement.span){if state.consistent && !state.released.is_empty(){inherited.remove(&state.place);}}
                    return Ok(Some(frame.child(Slot::Loop,Return::Discard,frame.results.clone(),inherited)));
                }
            },
            Stage::Finish=>{
                self.finish_map_epoch_statement(context,&mut frame.body[frame.at],frame.end,&mut frame.results,&mut frame.local,demand)?;frame.stage=Stage::Next;
            },
            Stage::Next=>{frame.at+=1;frame.stage=Stage::Statement;},
            Stage::Done=>unreachable!("completed Map frame"),
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deeply_nested_active_storage_bodies_preserve_source_and_exit_order() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        // An active completion layout prevents the no-storage fast path. No
        // fabricated callback types are needed for this traversal witness.
        context.storage_exits.record_map_completion_layout(usize::MAX,map_completion_payloads::Phase::Body,None,std::collections::BTreeMap::from([(crate::capture_loan_owners::Place{binding:7,fields:Vec::new()},Type::Int)]));
        assert!(context.storage_exits.has_map_completion_layouts());
        let mut body=Vec::new();
        for start in (0..16_384).rev(){body=vec![Stmt{labels:Vec::new(),control_target:None,kind:StmtKind::Block(body),span:Span::new(start,start+1)}];}
        let demand=map_entry_demand::Demand::checked(&mut body).unwrap();
        let mut results=Results::from([(7,Type::Int)]);
        let inherited=Places::from([crate::capture_loan_owners::Place{binding:7,fields:Vec::new()}]);
        let retained=analyzer.scoped_map_entry_epochs_with_demand(&mut context,&mut body,&mut results,&demand,&inherited).unwrap();
        assert_eq!(retained,inherited);assert_eq!(results,Results::from([(7,Type::Int)]));
        let mut current=body.as_slice();
        for start in 0..16_384 {
            assert_eq!(current.len(),1);assert_eq!(current[0].span,Span::new(start,start+1));
            let StmtKind::Block(inner)=&current[0].kind else{panic!("source body not restored")};current=inner;
        }
        assert!(current.is_empty());
        drop(body);
    }
}
