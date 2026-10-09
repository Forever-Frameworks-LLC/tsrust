//! Separate Map payload proofs at exception phase and completion boundaries.
use crate::{capture_loan_owners::Place,completion::CompletionKind,syntax::Type};
use std::collections::{BTreeMap,HashMap,HashSet};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Phase { Body, Handled, Cleaned }
#[derive(Clone,Debug)]
pub(crate) struct Snapshot {
    pub(crate) phase:Phase,
    /// None denotes reachable normal completion, including empty storage.
    pub(crate) kind:Option<CompletionKind>,
    pub(crate) payloads:BTreeMap<Place,HashSet<Type>>,
}
#[derive(Clone,Debug,Default)]
pub(crate) struct Snapshots {
    phases:HashMap<usize,Vec<Snapshot>>,
    native_inputs:BTreeMap<usize,BTreeMap<usize,Type>>,
    native_exits:HashMap<usize,BTreeMap<usize,Type>>,
    pub(super) cleanup_loops:HashSet<usize>,
    layouts:HashMap<usize,Vec<(Phase,Option<CompletionKind>,BTreeMap<Place,Type>)>>,
}
impl Snapshots {
    pub(super) fn clear_native(&mut self) {self.native_inputs.clear();self.native_exits.clear();self.cleanup_loops.clear();}
    pub(super) fn has_layouts(&self)->bool {
        self.layouts.values().any(|states|states.iter().any(|(_,_,types)|!types.is_empty()))
    }
    pub(crate) fn record(&mut self,boundary:usize,phase:Phase,kind:Option<CompletionKind>,payloads:BTreeMap<Place,HashSet<Type>>) {
        let states=self.phases.entry(boundary).or_default();
        if let Some(previous)=states.iter_mut().find(|state|state.phase==phase && state.kind==kind) {
            for (place,types) in payloads {previous.payloads.entry(place).or_default().extend(types);}
        } else {states.push(Snapshot{phase,kind,payloads});}
    }
    pub(crate) fn record_native_inputs(&mut self,start:usize,inputs:BTreeMap<usize,Type>) {
        self.native_inputs.insert(start,inputs);
    }
    pub(super) fn record_native_exit(&mut self,boundary:usize,types:BTreeMap<usize,Type>) {self.native_exits.insert(boundary,types);}
    pub(crate) fn native_exit(&self,boundary:usize,binding:usize)->Option<Type> {self.native_exits.get(&boundary)?.get(&binding).copied()}
    pub(crate) fn native_input(&self,start:usize,binding:usize)->Option<Type> {
        self.native_inputs.range(..=start).rev().find_map(|(_,inputs)|inputs.get(&binding).copied())
    }
    pub(super) fn snapshots(&self,boundary:usize)->Vec<Snapshot> {
        self.phases.get(&boundary).cloned().unwrap_or_default()
    }
    pub(super) fn record_layout(&mut self,boundary:usize,phase:Phase,kind:Option<CompletionKind>,types:BTreeMap<Place,Type>) {
        let layouts=self.layouts.entry(boundary).or_default();
        if let Some((_,_,previous))=layouts.iter_mut().find(|(p,k,_)|*p==phase && *k==kind) {*previous=types;}
        else {layouts.push((phase,kind,types));}
    }
    pub(crate) fn layout(&self,boundary:usize,phase:Phase,kind:Option<CompletionKind>)->Option<&BTreeMap<Place,Type>> {
        self.layouts.get(&boundary)?.iter().find(|(p,k,_)|*p==phase && *k==kind).map(|(_,_,types)|types)
    }
    #[cfg(test)]
    pub(crate) fn at(&self,boundary:usize,phase:Phase,kind:Option<CompletionKind>)->Option<&BTreeMap<Place,HashSet<Type>>> {
        self.phases.get(&boundary)?.iter().find(|state|state.phase==phase && state.kind==kind).map(|state|&state.payloads)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finally_cleanup_error_entry_has_independent_native_plan() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{handlers.delete('first');failIf(flag);}finally{for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();}first[0]+=1;}function main():void{try{scenario(false);scenario(true);}catch(error){console.log('caught');}}").unwrap();
        let checked=crate::check(&source).unwrap();let function=checked.program.functions.iter().find(|function|function.name=="scenario").unwrap();
        let region=function.body.iter().find_map(|statement|if let crate::syntax::StmtKind::Try(region)=&statement.kind{Some(region)}else{None}).unwrap();
        let boundary=region.body[0].span.start;let binding=function.bindings.iter().position(|binding|binding.name=="handlers").unwrap();
        let error=function.storage_exits.map_completion_layout(boundary,Phase::Body,Some(CompletionKind::Throw)).unwrap()[&Place{binding,fields:vec![]}];
        assert_ne!(error,function.bindings[binding].ty);
        let plan=function.storage_exits.map_cleanup_plan(boundary,Some(CompletionKind::Throw)).expect("checked error cleanup plan");
        assert_eq!(plan.exits.map_native_input(plan.body[0].span.start,binding),Some(error),"cleanup entry uses the pending error Map");
        let Type::Map(map)=error else{panic!("Map error storage")};let rust=checked.emit_rust();
        let envelope=rust.split("let completion:TsrCompletion<").filter_map(|part|part.split_once("='exception_region").map(|(ty,_)|ty)).find(|ty|ty.contains("FinallyIncoming")).expect("finally body error envelope");
        assert!(envelope.contains(&format!("Map{map}<")),"checked error Map{map} is absent from {envelope}");
    }

    #[test]
    fn borrowed_map_get_matches_the_phase_payload() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[7];const handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>first[0]});try{handlers.clear();handlers.set('new',{run:():int=>second[0]});throw 5;}catch(error){console.log(handlers.size);}first[0]=9;{const current=handlers.get('new');if(current!==undefined)console.log(current.run());}handlers.clear();second[0]=11;}").unwrap();
        let checked=crate::check(&source).unwrap();let function=checked.program.functions.iter().find(|function|function.name=="main").unwrap();let mut body=function.body.clone();
        for statement in &function.body {if let crate::syntax::StmtKind::Block(inner)=&statement.kind {for statement in inner {if let crate::syntax::StmtKind::Let{binding:Some(binding),value,..}=&statement.kind {assert_eq!(function.bindings[*binding].ty,value.ty.unwrap(),"borrowed get binding follows its initializer");}}}}
        let unions=crate::unions::UnionArena::from_snapshot_with_records(checked.program.unions.clone(),&checked.program.records);
        let mut found=false;
        crate::source_walk::body(&mut body,&mut |_|Ok(()),&mut |value| {
            if value.target==Some(crate::syntax::CallTarget::MapMethod(crate::syntax::MapMethod::Get)) && let crate::syntax::ExprKind::Call(callee,_)=&value.kind && let crate::syntax::ExprKind::Field(base,_)=&callee.kind && let Some(Type::Map(map))=base.ty {
                found=true;assert_eq!(unions.payload(value.ty.unwrap().owning()),Some(checked.program.records[map].map_definition.as_ref().unwrap().value));
            }
            Ok(())
        }).unwrap();assert!(found);
    }

    #[test]
    fn body_error_envelope_uses_checked_throw_storage_layout() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{handlers.delete('first');failIf(flag);}catch(error){first[0]+=1;for(const [key,entry] of handlers){console.log(entry.run());}}handlers.clear();}function main():void{scenario(false);scenario(true);}").unwrap();
        let checked=crate::check(&source).unwrap();let function=checked.program.functions.iter().find(|function|function.name=="scenario").unwrap();let region=function.body.iter().find_map(|statement|if let crate::syntax::StmtKind::Try(region)=&statement.kind{Some(region)}else{None}).unwrap();
        let binding=function.bindings.iter().position(|binding|binding.name=="handlers").unwrap();let place=Place{binding,fields:Vec::new()};let boundary=region.body[0].span.start;
        let error=function.storage_exits.map_completion_layout(boundary,Phase::Body,Some(CompletionKind::Throw)).unwrap()[&place];assert_ne!(error,function.bindings[binding].ty);
        assert_eq!(function.storage_exits.map_native_input(region.catch.as_ref().unwrap().body[0].span.start,binding),Some(error),"handler starts with checked error storage");
        let Type::Map(map)=error else{panic!("Map error storage")};let rust=checked.emit_rust();
        let envelope=rust.split("let completion:TsrCompletion<").filter_map(|part|part.split_once("='exception_region").map(|(ty,_)|ty)).find(|ty|ty.contains("StorageIncoming")).expect("body error envelope");
        assert!(envelope.contains(&format!("Map{map}<")),"checked throw storage Map{map} is absent from {envelope}");
    }

    #[test]
    fn finally_body_error_envelope_uses_checked_throw_storage_layout() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{handlers.delete('first');failIf(flag);}catch(error){first[0]+=1;for(const [key,entry] of handlers){console.log(entry.run());}}handlers.clear();}function main():void{scenario(false);scenario(true);}").unwrap();
        let source=source.replace("    handlers.clear();\n}","    finally { console.log(handlers.size); }\n    handlers.clear();\n}");
        let checked=crate::check(&source).unwrap();let function=checked.program.functions.iter().find(|function|function.name=="scenario").unwrap();let region=function.body.iter().find_map(|statement|if let crate::syntax::StmtKind::Try(region)=&statement.kind{Some(region)}else{None}).unwrap();
        let binding=function.bindings.iter().position(|binding|binding.name=="handlers").unwrap();let place=Place{binding,fields:Vec::new()};let boundary=region.body[0].span.start;
        let error=function.storage_exits.map_completion_layout(boundary,Phase::Body,Some(CompletionKind::Throw)).unwrap()[&place];assert_ne!(error,function.bindings[binding].ty);
        assert_eq!(function.storage_exits.map_native_input(region.catch.as_ref().unwrap().body[0].span.start,binding),Some(error),"handler starts with checked error storage");
        let Type::Map(map)=error else{panic!("Map error storage")};let rust=checked.emit_rust();
        let envelope=rust.split("let completion:TsrCompletion<").filter_map(|part|part.split_once("='exception_region").map(|(ty,_)|ty)).find(|ty|ty.contains("StorageIncoming")).expect("body error envelope");
        assert!(envelope.contains(&format!("Map{map}<")),"checked throw storage Map{map} is absent from {envelope}");
    }

    #[test]
    fn cleaned_normal_layout_is_consumed_after_finally() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{failIf(flag);}catch(error){}finally{handlers.delete('first');}first[0]+=1;handlers.clear();}function main():void{scenario(false);}").unwrap();
        let checked=crate::check(&source).unwrap();let function=checked.program.functions.iter().find(|function|function.name=="scenario").unwrap();
        let (position,region)=function.body.iter().enumerate().find_map(|(position,statement)|if let crate::syntax::StmtKind::Try(region)=&statement.kind{Some((position,region))}else{None}).unwrap();
        let binding=function.bindings.iter().position(|binding|binding.name=="handlers").unwrap();let place=Place{binding,fields:Vec::new()};let boundary=region.body[0].span.start;
        let normal=function.storage_exits.map_completion_layout(boundary,Phase::Cleaned,None).unwrap()[&place];
        let before=function.storage_exits.map_completion_layout(boundary,Phase::Handled,None).unwrap()[&place];assert_ne!(normal,before);
        assert_eq!(function.storage_exits.map_native_input(function.body[position+1].span.start,binding),Some(normal));
    }

    #[test]
    fn handled_normal_layout_is_consumed_after_exception_join() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{failIf(flag);handlers.delete('first');}catch(error){handlers.delete('first');}first[0]+=1;handlers.clear();}function main():void{scenario(false);}").unwrap();
        let checked=crate::check(&source).unwrap();let function=checked.program.functions.iter().find(|function|function.name=="scenario").unwrap();
        let (position,region)=function.body.iter().enumerate().find_map(|(position,statement)|if let crate::syntax::StmtKind::Try(region)=&statement.kind{Some((position,region))}else{None}).unwrap();
        let binding=function.bindings.iter().position(|binding|binding.name=="handlers").unwrap();let place=Place{binding,fields:Vec::new()};let boundary=region.body[0].span.start;
        let normal=function.storage_exits.map_completion_layout(boundary,Phase::Handled,None).unwrap()[&place];
        let error=function.storage_exits.map_completion_layout(boundary,Phase::Body,Some(CompletionKind::Throw)).unwrap()[&place];assert_ne!(normal,error);
        assert_eq!(function.storage_exits.map_native_input(function.body[position+1].span.start,binding),Some(normal));
    }

    #[test]
    fn ordinary_no_backedge_break_keeps_live_storage() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});while(true){handlers.delete('first');first[0]+=1;break;}for(const [key,entry] of handlers){console.log(entry.run());}handlers.clear();}").unwrap();
        let checked=crate::check(&source).unwrap();let function=checked.program.functions.iter().find(|function|function.name=="main").unwrap();
        let statement=function.body.iter().find(|statement|matches!(statement.kind,crate::syntax::StmtKind::While{..})).unwrap();
        let epochs=function.storage_exits.map_iteration_epochs(statement.span);assert_eq!(epochs.len(),1);
        assert!(!function.storage_exits.has_map_loop_edge(statement.span,crate::analysis::storage_exit_availability::MapLoopEdge::Back));
        assert!(epochs[0].back.empty);assert!(!epochs[0].break_empty);assert!(!epochs[0].carrier_empty);
    }

    #[test]
    fn ordinary_loop_edges_keep_distinct_origins_and_empty_break() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function scenario(limit:int,stop:boolean):void{const first=[7];const second=[11];const handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>first[0]});let turn=0;while(turn<limit){handlers.delete('old');first[0]+=1;turn++;if(stop)break;handlers.set('new',{run:():int=>second[0]});if(turn<limit)continue;}handlers.clear();first[0]+=10;}function main():void{scenario(0,false);scenario(2,false);scenario(2,true);}").unwrap();
        let checked=crate::check(&source).unwrap();
        let function=checked.program.functions.iter().find(|function|function.name=="scenario").unwrap();
        let loop_statement=function.body.iter().find(|statement|matches!(statement.kind,crate::syntax::StmtKind::While{..})).unwrap();
        let epochs=function.storage_exits.map_iteration_epochs(loop_statement.span);
        assert_eq!(epochs.len(),1);
        let entry=function.storage_exits.map_loop_edge(loop_statement.span,crate::analysis::storage_exit_availability::MapLoopEdge::Entry,&epochs[0].entry.place).unwrap();
        let back=function.storage_exits.map_loop_edge(loop_statement.span,crate::analysis::storage_exit_availability::MapLoopEdge::Back,&epochs[0].entry.place).unwrap();
        assert_ne!(entry,back);
        assert_eq!(entry.len(),1);assert_eq!(back.len(),1);
        assert!(epochs[0].break_empty);assert!(!epochs[0].back.empty);assert!(!epochs[0].carrier_empty);
        let narrowed=source.replace("handlers.set('old', {","const discarded=[13];handlers.set('discard',{run:():int=>discarded[0]});handlers.delete('discard');discarded[0]+=1;handlers.set('old',{");assert_ne!(narrowed,source);
        let narrowed=crate::check(&narrowed).unwrap();
        let function=narrowed.program.functions.iter().find(|function|function.name=="scenario").unwrap();
        let statement=function.body.iter().find(|statement|matches!(statement.kind,crate::syntax::StmtKind::While{..})).unwrap();
        let epoch=&function.storage_exits.map_iteration_epochs(statement.span)[0];
        assert_ne!(epoch.entry.input,epoch.working,"later origins must not be projected into the narrow first-entry family");
        assert_eq!(epoch.entry.output,epoch.working);

    }

    #[test]
    fn labelled_block_join_preserves_normal_and_break_survivors() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function scenario(stop:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});exit:{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');if(stop)break exit;}handlers.clear();}function main():void{scenario(false);scenario(true);}").unwrap();
        for (source,expected) in [(source.clone(),1),(source.replace("handlers.delete('first');", "if(stop)break exit;handlers.delete('first');"),2)] {
            let checked=crate::check(&source).unwrap();
            let function=checked.program.functions.iter().find(|function|function.name=="scenario").unwrap();
            let block=function.body.iter().find(|statement|matches!(statement.kind,crate::syntax::StmtKind::Block(_))&&!statement.labels.is_empty()).unwrap();
            let states=function.storage_exits.map_completion_survivors(block.span);
            assert_eq!(states.len(),1,"labelled block must retain its joined source owner proof");
            assert!(states[0].consistent);
            assert_eq!(states[0].payloads.len(),expected,"an early break must retain the first entry");
            let first=function.bindings.iter().position(|binding|binding.name=="first").unwrap();
            assert!(states[0].released.contains(&first));
        }
    }

    #[test]
    fn nested_loop_break_storage_is_not_an_empty_backedge() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;outer:while(turn<3){try{inner:while(true){try{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;turn++;if(turn<3)continue outer;break inner;}finally{console.log('inner',turn);}}}finally{for(const [key,entry] of handlers){console.log(entry.run());}}}handlers.clear();}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let function=checked.program.functions.iter().find(|function|function.name=="main").unwrap();
        let outer=function.body.iter().find(|statement|matches!(statement.kind,crate::syntax::StmtKind::While{..})).unwrap();
        let crate::syntax::StmtKind::While{body,..}=&outer.kind else{unreachable!()};
        let crate::syntax::StmtKind::Try(region)=&body[0].kind else{unreachable!()};
        let inner=&region.body[0];
        let epochs=function.storage_exits.map_iteration_epochs(inner.span);
        assert_eq!(epochs.len(),1);
        assert!(epochs[0].back.empty,"inner callable controls leave no backedge");
        assert!(!epochs[0].break_empty,"inner break keeps the survivor");
        assert!(!epochs[0].carrier_empty);
        assert!(function.storage_exits.has_map_iteration_controls(outer.span.start),"outer continue must resume through its carried tuple");
        let crate::syntax::StmtKind::While{body,..}=&inner.kind else{unreachable!()};
        let crate::syntax::StmtKind::Try(region)=&body[0].kind else{unreachable!()};
        let binding=epochs[0].entry.place.binding;
        assert_eq!(function.storage_exits.map_native_input(body[0].span.end,binding),Some(function.bindings[binding].ty),"phase exit returns the common physical storage family");
        assert_ne!(function.storage_exits.map_native_exit(region.body[0].span.start,binding),Some(function.bindings[binding].ty),"body deletion remains narrow before transport");
    }

    #[test]
    fn returning_cleanup_loop_plans_storage_without_a_backedge() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function scenario():int{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});while(true){try{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;return first[0];}finally{for(const [key,entry] of handlers){console.log(entry.run());}}}return 0;}function main():void{console.log(scenario());}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let function=checked.program.functions.iter().find(|function|function.name=="scenario").unwrap();
        let statement=function.body.iter().find(|statement|matches!(statement.kind,crate::syntax::StmtKind::While{..})).unwrap();
        assert!(!function.storage_exits.has_map_loop_edge(statement.span,super::super::storage_exit_availability::MapLoopEdge::Back));
        let epochs=function.storage_exits.map_iteration_epochs(statement.span);
        assert_eq!(epochs.len(),1,"callable exit still needs working Map storage");
        assert!(!epochs[0].carrier_empty,"populated entry cannot be discarded");
        let crate::syntax::StmtKind::While{body,..}=&statement.kind else{unreachable!()};
        let crate::syntax::StmtKind::Try(region)=&body[0].kind else{unreachable!()};
        let place=epochs[0].entry.place.clone();
        assert!(function.storage_exits.map_completion_payloads(region.body[0].span.start,Phase::Body,Some(CompletionKind::Return)).unwrap()[&place].len()==1,"return reaches cleanup with the survivor payload");
    }

    #[test]
    fn loop_cleanup_plans_storage_without_bypassing_inner_finally() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;while(turn<3){try{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;turn++;if(turn===2)break;continue;}finally{for(const [key,entry] of handlers){console.log(entry.run());}}}handlers.clear();}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let function=checked.program.functions.iter().find(|function|function.name=="main").unwrap();
        let loop_statement=function.body.iter().find(|statement|matches!(statement.kind,crate::syntax::StmtKind::While{..})).unwrap();
        assert!(!function.storage_exits.map_iteration_epochs(loop_statement.span).is_empty(),"cleanup loop needs carried storage");
        let crate::syntax::StmtKind::While{body,..}=&loop_statement.kind else{unreachable!()};
        let crate::syntax::StmtKind::Try(region)=&body[0].kind else{unreachable!()};
        let mut pending=region.body.iter().collect::<Vec<_>>();let mut count=0;
        while let Some(statement)=pending.pop(){match &statement.kind {
            crate::syntax::StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
            crate::syntax::StmtKind::Break|crate::syntax::StmtKind::Continue=>{count+=1;assert!(function.storage_exits.map_iteration_control(statement.span).is_none(),"source control must pass through finally");},
            _=>{},
        }}
        assert_eq!(count,2);
        assert!(function.storage_exits.has_map_iteration_controls(loop_statement.span.start));
        let override_source=source.replace("} finally {","}finally{if(turn===1)continue;if(turn===2)break;");
        assert_ne!(override_source,source);
        let override_checked=crate::check(&override_source).unwrap();
        let function=override_checked.program.functions.iter().find(|function|function.name=="main").unwrap();
        let statement=function.body.iter().find(|statement|matches!(statement.kind,crate::syntax::StmtKind::While{..})).unwrap();
        let binding=function.bindings.iter().position(|binding|binding.name=="handlers").unwrap();
        let place=Place{binding,fields:Vec::new()};
        assert!(!function.storage_exits.map_iteration_epochs(statement.span).is_empty(),"override storage: entry {:?}, back {:?}, break {:?}",function.storage_exits.map_loop_edge(statement.span,super::super::storage_exit_availability::MapLoopEdge::Entry,&place),function.storage_exits.map_loop_edge(statement.span,super::super::storage_exit_availability::MapLoopEdge::Back,&place),function.storage_exits.map_loop_edge(statement.span,super::super::storage_exit_availability::MapLoopEdge::Break,&place));
    }

    #[test]
    fn native_body_exit_tracks_tail_replacement_conversion() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function main():void{const first=[3];const next=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});try{handlers.set('first',(failIf(false),{run:():int=>next[0]}));}catch(error){}handlers.clear();first[0]=9;next[0]=11;}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let function=checked.program.functions.iter().find(|function|function.name=="main").unwrap();
        let binding=function.bindings.iter().position(|binding|binding.name=="handlers").unwrap();
        let region=function.body.iter().find_map(|statement|if let crate::syntax::StmtKind::Try(region)=&statement.kind{Some(region)}else{None}).unwrap();
        let last=region.body.last().unwrap();
        let conversion=function.storage_exits.map_entry_epochs(last.span).iter().find(|epoch|!epoch.before).expect("replacement conversion");
        assert_ne!(conversion.input,conversion.output);
        assert_eq!(function.storage_exits.map_native_exit(region.body[0].span.start,binding),Some(conversion.output));
    }

    #[test]
    fn phase_layouts_keep_error_payloads_and_empty_cleanup_separate() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{failIf(flag);handlers.delete('first');if(flag)return;}finally{handlers.clear();}}function main():void{scenario(false);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let function=checked.program.functions.iter().find(|function|function.name=="scenario").unwrap();
        let region=function.body.iter().find_map(|statement|if let crate::syntax::StmtKind::Try(region)=&statement.kind{Some(region)}else{None}).unwrap();
        let boundary=region.body[0].span.start;
        let binding=function.bindings.iter().position(|binding|binding.name=="handlers").unwrap();
        let place=Place{binding,fields:Vec::new()};
        let normal=function.storage_exits.map_completion_layout(boundary,Phase::Body,None).expect("normal storage layout")[&place];
        let error=function.storage_exits.map_completion_layout(boundary,Phase::Body,Some(CompletionKind::Throw)).expect("error storage layout")[&place];
        let empty=function.storage_exits.map_completion_layout(boundary,Phase::Cleaned,None).expect("cleanup storage layout")[&place];
        assert_ne!(normal,error);assert_ne!(normal,empty);assert_ne!(error,empty);
        assert_eq!(function.storage_exits.map_completion_layout(boundary,Phase::Body,Some(CompletionKind::Return)).unwrap()[&place],normal);
        assert_eq!(function.storage_exits.map_completion_layout(boundary,Phase::Cleaned,Some(CompletionKind::Return)).unwrap()[&place],empty);
        assert!(function.storage_exits.map_completion_layout(boundary,Phase::Body,Some(CompletionKind::Break(None))).is_none());
        let Type::Map(normal_map)=normal else{panic!("normal Map layout")};
        let payloads=&function.storage_exits.map_completion_payloads(boundary,Phase::Body,None).unwrap()[&place];
        assert_eq!(payloads.len(),1);
        assert_eq!(checked.program.records[normal_map].map_definition.as_ref().unwrap().value,*payloads.iter().next().unwrap());
    }

    #[test]
    fn empty_reachable_states_and_control_targets_remain_distinct() {
        let mut states=Snapshots::default();
        let place=Place{binding:3,fields:vec!["handlers".into()]};
        let payload=|ty|BTreeMap::from([(place.clone(),HashSet::from([ty]))]);
        let first=Some(CompletionKind::Continue(Some(crate::syntax::ControlTarget::Loop(10))));
        let second=Some(CompletionKind::Continue(Some(crate::syntax::ControlTarget::Loop(20))));
        states.record(7,Phase::Body,first,payload(Type::Int));
        states.record(7,Phase::Body,first,payload(Type::String));
        states.record(7,Phase::Body,second,payload(Type::Boolean));
        states.record(7,Phase::Cleaned,None,BTreeMap::new());
        assert_eq!(states.at(7,Phase::Body,first).unwrap()[&place],HashSet::from([Type::Int,Type::String]));
        assert_eq!(states.at(7,Phase::Body,second).unwrap()[&place],HashSet::from([Type::Boolean]));
        assert!(states.at(7,Phase::Cleaned,None).unwrap().is_empty());
        assert!(states.at(7,Phase::Handled,None).is_none());
        assert!(states.at(7,Phase::Cleaned,Some(CompletionKind::Throw)).is_none());
    }
    #[test]
    fn checked_exception_phases_preserve_predeletion_error_payloads() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});try{handlers.set('first',{run:():int=>first[0]});failIf(flag);handlers.delete('first');if(flag)return;}catch(error){failIf(flag);handlers.delete('first');}finally{console.log(handlers.size);}handlers.clear();}function main():void{scenario(false);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let function=checked.program.functions.iter().find(|function|function.name=="scenario").unwrap();
        let region=function.body.iter().find_map(|statement|if let crate::syntax::StmtKind::Try(region)=&statement.kind{Some(region)}else{None}).unwrap();
        let boundary=region.body[0].span.start;
        let normal=function.storage_exits.map_completion_payloads(boundary,Phase::Body,None).unwrap();
        let thrown=function.storage_exits.map_completion_payloads(boundary,Phase::Body,Some(CompletionKind::Throw)).unwrap();
        let place=normal.keys().find(|place|place.fields.is_empty()).unwrap();
        assert_eq!(normal[place].len(),1);
        assert!(thrown[place].is_superset(&normal[place]));
        assert!(thrown[place].len()>normal[place].len());
        let mut handler=region.catch.as_ref().unwrap().body.clone();
        let handler_contract=crate::error_effects::annotate_region_body(&mut handler,&checked.program.error_effects);
        // The explicit handler failure precedes deletion and retains the first entry.
        assert!(handler_contract.explicit.contains(&Type::Int));
        let handler_error=function.storage_exits.map_completion_payloads(boundary,Phase::Handled,Some(CompletionKind::Throw)).unwrap();
        assert_eq!(handler_error[place],thrown[place]);
        assert_eq!(function.storage_exits.map_completion_payloads(boundary,Phase::Cleaned,Some(CompletionKind::Throw)).unwrap()[place],handler_error[place]);
        assert_eq!(function.storage_exits.map_completion_payloads(boundary,Phase::Cleaned,None).unwrap()[place],normal[place]);
        assert_eq!(function.storage_exits.map_completion_payloads(boundary,Phase::Cleaned,Some(CompletionKind::Return)).unwrap()[place],normal[place]);
        let cleared=source.replace("console.log(handlers.size);", "handlers.clear();");
        assert_ne!(cleared,source);
        let cleared=crate::format(&cleared).unwrap();
        assert_eq!(crate::format(&cleared).unwrap(),cleared);
        let cleared=crate::check(&cleared).unwrap();
        let function=cleared.program.functions.iter().find(|function|function.name=="scenario").unwrap();
        let region=function.body.iter().find_map(|statement|if let crate::syntax::StmtKind::Try(region)=&statement.kind{Some(region)}else{None}).unwrap();
        let boundary=region.body[0].span.start;
        assert!(function.storage_exits.map_completion_payloads(boundary,Phase::Cleaned,None).unwrap().is_empty());
        assert!(function.storage_exits.map_completion_payloads(boundary,Phase::Cleaned,Some(CompletionKind::Return)).unwrap().is_empty());
    }

    #[test]
    fn checked_native_points_follow_deletion_and_refill_families() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const third=[7];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});handlers.delete('first');first[0]=9;handlers.set('third',{run:():int=>third[0]});handlers.clear();third[0]=11;}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let function=checked.program.functions.iter().find(|function|function.name=="main").unwrap();
        let binding=function.bindings.iter().position(|binding|binding.name=="handlers").unwrap();
        let mutation=function.body.iter().find(|statement|matches!(&statement.kind,crate::syntax::StmtKind::Assign{target,..} if matches!(&target.kind,crate::syntax::ExprKind::Index(base,_) if matches!(&base.kind,crate::syntax::ExprKind::Name(name) if name=="first")))).unwrap();
        let narrowed=function.storage_exits.map_native_input(mutation.span.start,binding).unwrap();
        let insertion=function.body.iter().rev().find_map(|statement|if let crate::syntax::StmtKind::Expr(expression)=&statement.kind && expression.target==Some(crate::syntax::CallTarget::MapMethod(crate::syntax::MapMethod::Set)){Some((statement,expression))}else{None}).unwrap();
        let widened=function.storage_exits.map_native_input(insertion.0.span.start,binding).unwrap();
        assert_ne!(narrowed,widened);
        let crate::syntax::ExprKind::Call(callee,_)=&insertion.1.kind else{unreachable!()};
        let crate::syntax::ExprKind::Field(receiver,_)=&callee.kind else{unreachable!()};
        assert_eq!(receiver.ty,Some(widened));
        let Type::Map(narrowed)=narrowed else{panic!("narrowed Map")};
        let Type::Map(widened)=widened else{panic!("widened Map")};
        assert_ne!(checked.program.records[narrowed].map_definition.as_ref().unwrap().value,checked.program.records[widened].map_definition.as_ref().unwrap().value);
    }

    #[test]
    fn post_test_condition_native_point_uses_carried_storage() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});do{handlers.delete('first');}while(handlers.size>1);first[0]=9;handlers.clear();}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let function=checked.program.functions.iter().find(|function|function.name=="main").unwrap();
        let binding=function.bindings.iter().position(|binding|binding.name=="handlers").unwrap();
        let condition=function.body.iter().find_map(|statement|if let crate::syntax::StmtKind::Loop{condition:Some(condition),post_test:true,..}=&statement.kind{Some(condition)}else{None}).unwrap();
        let crate::syntax::ExprKind::Binary{left,..}=&condition.kind else{panic!("condition comparison")};
        let crate::syntax::ExprKind::Field(receiver,_)=&left.kind else{panic!("Map size")};
        assert_eq!(function.storage_exits.map_native_input(condition.span.start,binding),receiver.ty);
    }

}
