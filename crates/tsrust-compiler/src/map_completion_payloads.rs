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
    #[cfg(test)]
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
