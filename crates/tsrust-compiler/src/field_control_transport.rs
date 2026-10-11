//! Carry destination-specific fields through cleanup without extending local drop scopes.
use super::*;
use super::field_completion_transport::{normalize,partial_places,restoration,tuple};
use crate::capture_loan_owners::Place;
use crate::completion::CompletionKind;
use crate::analysis::MapCompletionPhase::{Handled,Cleaned};
type Fields=Vec<(String,Type)>;
struct ControlFields {target:ControlTarget,continuing:bool,span:crate::Span,input:Fields,output:Fields}
fn values(fields:&Fields)->Vec<String>{fields.iter().map(|(name,_)|name.clone()).collect()}
impl Emitter<'_> {
    pub(super) fn field_control_transport(&self,fun:&CheckedFunction,statement:&Stmt,region:&ExceptionRegion,first:usize,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>)->Option<String> {
        let cleanup=region.finally.as_ref()?;
        if region.catch.as_ref().is_some_and(|handler|handler.error_contract.as_ref().is_some_and(|errors|errors.runtime || errors.opaque || !errors.explicit.is_empty())){return None;}
        let mut destinations=exception_contract_emit::escaping_controls(&region.body);
        destinations.extend(exception_contract_emit::escaping_controls(cleanup));
        destinations.dedup_by_key(|(target,continuing,_)|(*target,*continuing));
        if destinations.is_empty(){return None;}
        let mut body_places=partial_places(fun,&region.body);
        let mut places=body_places.clone();places.extend(partial_places(fun,cleanup));
        if places.is_empty(){return None;}
        normalize(fun,statement.span.start,&mut body_places);normalize(fun,statement.span.start,&mut places);
        let boundary=region.body.first()?.span.start;
        let mut controls=vec![];
        for (target,continuing,span) in destinations {
            if controls.iter().any(|control:&ControlFields|control.target==target && control.continuing==continuing){continue;}
            let kind=if continuing{CompletionKind::Continue(Some(target))}else{CompletionKind::Break(Some(target))};
            let input=self.control_completion_fields(fun,boundary,Handled,Some(kind),&body_places)?;
            let output=self.control_completion_fields(fun,boundary,Cleaned,Some(kind),&places)?;
            controls.push(ControlFields{target,continuing,span,input,output});
        }
        let normal_input=self.control_completion_fields(fun,boundary,Handled,None,&body_places)?;
        let normal_output=self.control_completion_fields(fun,boundary,Cleaned,None,&places)?;
        let body_normal=fun.storage_exits.has_completion_owners(boundary,Handled,None);
        let cleaned_normal=fun.storage_exits.has_completion_owners(boundary,Cleaned,None);
        let name=format!("FieldControl{first}");
        let generics=(0..controls.len()).map(|index|format!(",C{index}")).collect::<String>();
        let variants=(0..controls.len()).map(|index|format!(",Exit{index}(C{index})")).collect::<String>();
        let declaration=format!("enum {name}<R{generics}>{{Return((R,())),Common((TsrStorageExit<R>,())){variants}}}");
        self.completion_next.set(self.completion_next.get().max(first+3+controls.len()));
        let pending=self.field_control_body(fun,&region.body,first,error_name,contract,runtime,&name,&controls,false,&normal_input,body_normal);
        let (input_pattern,input_restore)=restoration(&values(&normal_input),&format!("control_normal_input{first}"));
        let (output_pattern,output_restore)=restoration(&values(&normal_output),&format!("control_normal_output{first}"));
        let normal_cleanup=if body_normal{self.field_control_body(fun,cleanup,first+1,error_name,contract,runtime,&name,&controls,true,&normal_output,cleaned_normal)}else{"unreachable!(\"no normal body completion\")".into()};
        let returned_cleanup=self.field_control_body(fun,cleanup,first+2,error_name,contract,runtime,&name,&controls,true,&vec![],true);
        let returned=exception_emit::finally_region(&format!("TsrCompletion::Return({name}::Return((pending_return,())))"),&returned_cleanup);
        let thrown=exception_emit::finally_region("TsrCompletion::Throw(pending_error)",&returned_cleanup);
        let mut arms=String::new();let mut dispatch=String::new();
        for (index,control) in controls.iter().enumerate() {
            let (pattern,restore)=restoration(&values(&control.input),&format!("control_input{first}_{index}"));
            let (out_pattern,out_restore)=restoration(&values(&control.output),&format!("control_output{first}_{index}"));
            let kind=if control.continuing{CompletionKind::Continue(Some(control.target))}else{CompletionKind::Break(Some(control.target))};
            let survives=fun.storage_exits.has_completion_owners(boundary,Cleaned,Some(kind));
            let completed=self.field_control_body(fun,cleanup,first+3+index,error_name,contract,runtime,&name,&controls,true,&control.output,survives);
            let _=write!(arms,"TsrCompletion::Return({name}::Exit{index}({pattern}))=>{{{restore}match {completed}{{TsrCompletion::Normal(fields)=>TsrCompletion::Return({name}::Exit{index}(fields)),TsrCompletion::Return(value)=>TsrCompletion::Return(value),TsrCompletion::Throw(error)=>TsrCompletion::Throw(error),_=>unreachable!(\"carried cleanup control\")}}}},");
            let route=self.exception_regions.control_at(control.target,control.continuing,control.span);
            let _=write!(dispatch,"TsrCompletion::Return({name}::Exit{index}({out_pattern}))=>{{{out_restore}{route}}},");
        }
        let normal=if body_normal{format!("{{{input_restore}{normal_cleanup}}}")}else{normal_cleanup};
        let joined=format!("match {pending}{{TsrCompletion::Normal({input_pattern})=>{normal},TsrCompletion::Return({name}::Return((pending_return,())))=>{returned},TsrCompletion::Throw(pending_error)=>{thrown},{arms}_=>unreachable!(\"checked field control\")}}");
        let (outer_name,outer_contract,_)=self.exception_regions.current_transport().expect("outer field control");
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
        let widened=exception_contract_emit::checked_completion_conversion(&joined,error_name,&outer_name,contract,&outer_contract,&unions,runtime,statement.span).expect("checked field control errors");
        let returned=self.exception_regions.return_value("value",statement.span);let thrown=self.exception_regions.native_throw("error");
        let normal=if cleaned_normal{output_restore}else{"unreachable!(\"no normal cleanup completion\")".into()};
        Some(format!("{{{declaration}match {widened}{{TsrCompletion::Normal({output_pattern})=>{{{normal}}},TsrCompletion::Return({name}::Return((value,())))=>{{{returned}}},TsrCompletion::Throw(error)=>{{{thrown}}},{dispatch}_=>unreachable!(\"checked field control dispatch\")}}}}"))
    }
    fn control_completion_fields(&self,fun:&CheckedFunction,boundary:usize,phase:crate::analysis::MapCompletionPhase,kind:Option<CompletionKind>,places:&[Place])->Option<Fields> {
        let mut fields=vec![];
        for place in places {
            let binding=&fun.bindings[place.binding];
            if binding.capture_reference || binding.borrowed_place.is_some() || binding.mutable_place
                || binding.parameter && (place.binding>=fun.parameter_count || binding.mode!=Mode::Consume)
                || self.restored_capture(fun,place.binding) || self.owned_record_layout(fun,place.binding).is_some(){return None;}
            if !fun.storage_exits.completion_field_available(boundary,phase,kind,place).unwrap_or(false){continue;}
            let mut ty=binding.ty;let mut native=format!("value{}",place.binding);
            for field in &place.fields {
                let Type::Record(record)=ty else{return None;};
                let index=self.program.records[record].fields.iter().position(|candidate|candidate.name.text==*field)?;
                ty=self.program.records[record].fields[index].ty?;let _=write!(native,".field{index}");
            }
            fields.push((native,ty));
        }Some(fields)
    }
    fn field_control_body(&self,fun:&CheckedFunction,body:&[Stmt],id:usize,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,name:&str,controls:&[ControlFields],cleaned:bool,normal:&Fields,has_normal:bool)->String {
        let _region=self.exception_regions.enter_typed(id,exception_contract_emit::local_control_targets(body),error_name.into(),contract.clone(),runtime);
        self.exception_regions.attach_storage(vec![],Default::default(),Default::default(),false,body.first().map_or(0,|statement|statement.span.start));
        self.exception_regions.carry_storage_on_control_exit();
        self.exception_regions.attach_return_storage(name.into(),vec![],Default::default(),Default::default());
        let mut arguments=vec![self.native_type(fun.result)];
        for (index,control) in controls.iter().enumerate() {
            let fields=if cleaned{&control.output}else{&control.input};
            arguments.push(tuple(&fields.iter().map(|(_,ty)|self.native_type(*ty)).collect::<Vec<_>>()));
            let storage=fields.iter().map(|(place,_)|native_storage_availability::Value{place:place.clone(),empty:String::new(),live:None,transferred:None,target:None}).collect();
            self.exception_regions.attach_control_storage(control.target,control.continuing,format!("{name}::Exit{index}"),storage,Default::default(),Default::default());
        }
        let normal_type=tuple(&normal.iter().map(|(_,ty)|self.native_type(*ty)).collect::<Vec<_>>());
        let normal_value=if has_normal{format!("TsrCompletion::Normal({})",tuple(&values(normal)))}else{"unreachable!(\"checked abrupt field body\")".into()};
        let mut statements=String::new();self.block(&mut statements,fun,body,1);
        format!("{{let completion:TsrCompletion<{normal_type},{name}<{}>,{error_name}>='exception_region{id}:{{{statements}{normal_value}}};completion}}",arguments.join(","))
    }
}
