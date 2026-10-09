//! Exact break and continue destinations carry independent Map payloads.
use super::*;
use super::map_return_storage::Carrier;
type Layout=std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>;
impl Emitter<'_> {
    pub(super) fn map_completion_carrier<'a>(&self,fun:&'a CheckedFunction,name:String,places:&[String],storage:&str,boundary:usize,phase:crate::analysis::MapCompletionPhase,fallback:&'a Layout)->Option<Carrier<'a>> {
        use crate::completion::CompletionKind;
        let returned=fun.storage_exits.map_completion_layout(boundary,phase,Some(CompletionKind::Return));
        let controls=self.control_targets.snapshot().into_iter().flat_map(|target|[CompletionKind::Break(Some(target)),CompletionKind::Continue(Some(target))]).filter_map(|kind|fun.storage_exits.map_completion_layout(boundary,phase,Some(kind)).map(|layout|(kind,layout))).collect::<Vec<_>>();
        if returned.is_none() && controls.is_empty(){return None;}
        let mut carrier=self.map_return_carrier(fun,name,places,storage,returned.unwrap_or(fallback));
        carrier.native.pop();for (_,layout) in &controls{carrier.native.push(',');carrier.native.push_str(&self.map_error_storage_type(fun,places,layout));}carrier.native.push('>');carrier.controls=controls;Some(carrier)
    }
    pub(super) fn map_carrier_declaration(&self,carrier:&Carrier<'_>)->String {
        let generics=(0..carrier.controls.len()).map(|index|format!(",S{index}=()")).collect::<String>();
        let variants=(0..carrier.controls.len()).map(|index|format!(",Exit{index}(S{index})")).collect::<String>();
        format!("enum {}<R,S,C{generics}>{{Return((R,S)),Common((TsrStorageExit<R>,C)){variants}}}",carrier.name)
    }
    pub(super) fn attach_map_control_storage(&self,fun:&CheckedFunction,body:&[Stmt],places:&[String],carrier:&Carrier<'_>) {
        for (index,(kind,layout)) in carrier.controls.iter().enumerate(){
            let (crate::completion::CompletionKind::Break(Some(target))|crate::completion::CompletionKind::Continue(Some(target)))=kind else{unreachable!("resolved control")};
            self.attach_one_map_control_storage(fun,body,places,layout,*target,kind.is_continue(),format!("{}::Exit{index}",carrier.name));
        }
    }
    pub(super) fn attach_one_map_control_storage(&self,fun:&CheckedFunction,body:&[Stmt],places:&[String],layout:&Layout,target:ControlTarget,continuing:bool,variant:String) {
        let value=|place:&str,start:usize,point:Option<(usize,usize)>| {
            let (binding,fields)=self.map_storage_binding(fun,place).expect("checked return Map storage");
            let target=layout.get(&crate::capture_loan_owners::Place{binding,fields:fields.clone()}).copied().unwrap_or_else(||self.map_storage_leaf_type(fun.bindings[binding].ty,&fields).expect("return Map path"));
            let transferred=point.map_or_else(||fun.storage_exits.transfer_before(start,binding,&fields),|point|fun.storage_exits.transfer_at_expression(point,binding,&fields));
            let input=fun.storage_exits.map_native_input(start,binding).or(Some(fun.bindings[binding].ty)).and_then(|root|self.map_storage_leaf_type(root,&fields));
            self.map_typed_storage_leaf_value(place,transferred,input,target)
        };
        let mut relevant=Vec::new();let mut pending=body.iter().collect::<Vec<_>>();
        while let Some(statement)=pending.pop(){
            if statement.control_target==Some(target) && matches!((&statement.kind,continuing),(StmtKind::Break,false)|(StmtKind::Continue,true)){relevant.push((statement.span.start,None));}
            if let StmtKind::Try(region)=&statement.kind{
                let boundary=region.body.first().map_or(statement.span.start,|statement|statement.span.start);
                let phase=if region.finally.is_some(){crate::analysis::MapCompletionPhase::Cleaned}else{crate::analysis::MapCompletionPhase::Handled};
                let kind=if continuing{crate::completion::CompletionKind::Continue(Some(target))}else{crate::completion::CompletionKind::Break(Some(target))};
                if let Some(source)=fun.storage_exits.map_completion_layout(boundary,phase,Some(kind)){relevant.push((statement.span.start,Some(source)));}
            }
            match &statement.kind{
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
                StmtKind::Loop{body,update,..}=>{pending.extend(body);pending.extend(update);},
                StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
                StmtKind::Try(region)=>pending.extend(region.statements()),_=>{}
            }
        }
        let points=relevant.into_iter().map(|(start,source)|{
            let values=places.iter().map(|place|if let Some(source)=source{
                let (binding,fields)=self.map_storage_binding(fun,place).expect("control Map storage");let key=crate::capture_loan_owners::Place{binding,fields};let target=layout.get(&key).copied().unwrap_or_else(||self.map_storage_leaf_type(fun.bindings[binding].ty,&key.fields).unwrap());
                self.map_typed_storage_leaf_value(place,None,source.get(&key).copied(),target)
            }else{value(place,start,None)}).collect();(start,values)
        }).collect();
        let expressions=std::collections::BTreeMap::new();
        let initial=places.iter().map(|place|{
            let (binding,fields)=self.map_storage_binding(fun,place).expect("control Map storage");let target=layout.get(&crate::capture_loan_owners::Place{binding,fields:fields.clone()}).copied().unwrap_or_else(||self.map_storage_leaf_type(fun.bindings[binding].ty,&fields).unwrap());
            self.map_typed_storage_leaf_value(place,None,Some(target),target)
        }).collect();
        self.exception_regions.attach_control_storage(target,continuing,variant,initial,points,expressions);
    }

    #[inline(never)]
    pub(super) fn map_selected_control_dispatch(&self,fun:&CheckedFunction,kind:crate::completion::CompletionKind,layout:Option<&Layout>,span:crate::Span)->String {
        let (crate::completion::CompletionKind::Break(Some(target))|crate::completion::CompletionKind::Continue(Some(target)))=kind else{unreachable!("resolved control")};
        if self.exception_regions.local_control_target(target){
            let epochs=match target{ControlTarget::Block(label)=>fun.storage_exits.map_block_epochs(label).to_vec(),ControlTarget::Loop(label)|ControlTarget::Tail(label)=>fun.storage_exits.map_iteration_epochs_at_label(label).iter().map(|epoch|crate::analysis::MapEntryEpoch{place:epoch.entry.place.clone(),input:epoch.entry.input,output:epoch.body.output,empty:false,before:false}).collect()};
            if !epochs.is_empty(){
                let mut conversions=String::new();for epoch in &epochs{
                    let input=layout.and_then(|layout|layout.get(&epoch.place)).copied().unwrap_or_else(||self.map_storage_leaf_type(fun.bindings[epoch.place.binding].ty,&epoch.place.fields).expect("control Map path"));
                    let target=self.map_storage_leaf_type(epoch.output,&epoch.place.fields).expect("control Map output");
                    if input!=target{
                        let place=self.map_entry_epoch_place(fun,epoch);let value=self.map_typed_storage_leaf_value(&place,None,Some(input),target);let converted=value.live.as_deref().unwrap_or(&place);
                        let _=write!(conversions,"let mut {place}:{}={converted};",self.native_type(target));
                    }
                }
                let names=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,epoch)).collect::<Vec<_>>().join(",");
                return match target{ControlTarget::Block(label)=>format!("{conversions}break 'map_block_{label} ({names},);"),ControlTarget::Loop(label)|ControlTarget::Tail(label)=>format!("{conversions}break 'map_body_{label} ({},{names},);",!kind.is_continue())};
            }
        }
        let owner=layout.and_then(|layout|self.exception_regions.control_storage_values(target,kind.is_continue(),span).map(|values|self.map_project_outgoing_storage(fun,layout,values)));
        self.exception_regions.dispatch_control(target,kind.is_continue(),span,owner.as_deref())
    }
    #[inline(never)]
    pub(super) fn map_precise_control_cleanups(&self,fun:&CheckedFunction,region:&ExceptionRegion,declarations:&[Stmt],carrier:&Carrier<'_>,cleanup_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,places:&[String],storage:&str,pattern:&str,span:crate::Span)->Vec<String> {
        let boundary=region.body.first().map_or(span.start,|statement|statement.span.start);
        let cleanup=region.finally.as_ref().expect("control cleanup");
        let mut arms=Vec::new();for (_index,(kind,_)) in carrier.controls.iter().enumerate(){
            let plan=fun.storage_exits.map_cleanup_plan(boundary,Some(*kind)).map(|plan|plan.function(fun));let planned=plan.as_ref().unwrap_or(fun);let body=plan.as_ref().map_or(cleanup.as_slice(),|plan|plan.body.as_slice());
            let normal=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,Some(*kind));
            let error=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,Some(crate::completion::CompletionKind::Throw));
            let id=self.completion_next.get();self.completion_next.set(id+1);
            let cleaned=self.map_error_phase_storage_completion_body(planned,body,declarations,id,cleanup_name,contract.clone(),runtime,storage,places,true,true,normal,error);
            let resume=if crate::completion::definitely_abrupt(cleanup,0){"unreachable!(\"cleanup cannot resume control\")".into()}else{self.map_selected_control_dispatch(fun,*kind,normal,span)};
            let abrupt=self.map_layout_carried_cleanup_dispatch(fun,"other",cleanup_name,contract,runtime,places,span,error);
            arms.push(format!("match {cleaned}{{TsrCompletion::Normal(storage)=>{{let {pattern}=storage;{resume}}},other=>{{{abrupt}}}}}"));
        }arms
    }
    pub(super) fn map_select_control_owner(&self,carrier:&Carrier<'_>,pattern:&str,returned:String,common:String,controls:Vec<String>)->String {
        let mut variants=vec![("Return".to_owned(),format!("(pending_return,{pattern})"),returned),("Common".to_owned(),format!("(pending_exit,{pattern})"),common)];
        variants.extend(controls.into_iter().enumerate().map(|(index,body)|(format!("Exit{index}"),pattern.to_owned(),body)));
        let names=(0..variants.len()).map(|index|format!("selected_control{index}")).collect::<Vec<_>>();
        let arms=variants.iter().enumerate().map(|(index,(variant,_,_))|format!("{}::{variant}(owner)=>({})",carrier.name,(0..variants.len()).map(|slot|if slot==index{"Some(owner)"}else{"None"}).collect::<Vec<_>>().join(","))).collect::<Vec<_>>().join(",");
        let mut selected="unreachable!(\"one selected control owner\")".to_owned();
        for (index,(_,pattern,body)) in variants.iter().enumerate().rev(){
            let name=&names[index];let drops=names.iter().enumerate().filter(|(slot,_)|*slot!=index).map(|(_,name)|format!("drop({name});")).collect::<String>();
            selected=format!("if {name}.is_some(){{let {pattern}={name}.unwrap();{drops}{body}}}else{{{selected}}}");
        }
        format!("let ({},)={{let completed=control_owner.unwrap();match completed{{{arms}}}}};{selected}",names.join(","))
    }

 }
