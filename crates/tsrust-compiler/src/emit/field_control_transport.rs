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
    pub(super) fn field_control_transport(&self,fun:&CheckedFunction,statement:&Stmt,region:&ExceptionRegion,first:usize,incoming_name:&str,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>)->Option<String> {
        let cleanup=region.finally.as_ref()?;
        let inherited_errors=self.exception_regions.field_error_fields();
        let catch=region.catch.as_ref().filter(|handler|handler.error_contract.as_ref().is_some_and(|errors|errors.runtime || errors.opaque || !errors.explicit.is_empty()));
        let mut destinations=exception_contract_emit::escaping_controls(&region.body);
        if let Some(handler)=catch {destinations.extend(exception_contract_emit::escaping_controls(&handler.body));}
        destinations.extend(exception_contract_emit::escaping_controls(cleanup));
        destinations.dedup_by_key(|(target,continuing,_)|(*target,*continuing));
        if destinations.is_empty(){return None;}
        let mut body_places=partial_places(fun,&region.body);
        if let Some(handler)=catch {body_places.extend(partial_places(fun,&handler.body));}
        let mut places=body_places.clone();places.extend(partial_places(fun,cleanup));
        if places.is_empty(){return None;}
        if let Some(fields)=&inherited_errors {
            for (name,_) in fields {
                let place=self.field_native_place(fun,name).expect("checked inherited field owner");
                body_places.push(place.clone());places.push(place);
            }
        }
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
        let caught_fields=if catch.is_some(){self.control_completion_fields(fun,boundary,crate::analysis::MapCompletionPhase::Body,Some(CompletionKind::Throw),&body_places)?}else{vec![]};
        let mut return_candidates=inherited_errors.clone().unwrap_or_default();
        for field in &caught_fields{if !return_candidates.iter().any(|(name,_)|name==&field.0){return_candidates.push(field.clone());}}
        let return_fields=[Handled,Cleaned].map(|phase|return_candidates.iter().filter(|(name,_)|{
            fun.storage_exits.completion_field_available(boundary,phase,Some(CompletionKind::Return),&self.field_native_place(fun,name).expect("checked native field owner"))==Some(true)
        }).cloned().collect::<Fields>());
        let (return_pattern,return_restore)=restoration(&values(&return_fields[0]),&format!("control_return_input{first}"));
        let (returned_pattern,returned_restore)=restoration(&values(&return_fields[1]),&format!("control_return_output{first}"));
        let normal_input=self.control_completion_fields(fun,boundary,Handled,None,&body_places)?;
        let normal_output=self.control_completion_fields(fun,boundary,Cleaned,None,&places)?;
        let body_normal=fun.storage_exits.has_completion_owners(boundary,Handled,None);
        let cleaned_normal=fun.storage_exits.has_completion_owners(boundary,Cleaned,None);
        let name=format!("FieldControl{first}");
        let generics=(0..controls.len()).map(|index|format!(",C{index}")).collect::<String>();
        let variants=(0..controls.len()).map(|index|format!(",Exit{index}(C{index})")).collect::<String>();
        let declaration=format!("enum {name}<R,F{generics}>{{Return((R,F)),Common((TsrStorageExit<R>,())){variants}}}");
        self.completion_next.set(self.completion_next.get().max(first+3+controls.len()));
        let pending=if let Some(handler)=catch {
            let incoming=handler.error_contract.as_deref().expect("checked field control catch");
            let normal=self.control_completion_fields(fun,boundary,crate::analysis::MapCompletionPhase::Body,None,&body_places)?;
            let has_normal=fun.storage_exits.has_completion_owners(boundary,crate::analysis::MapCompletionPhase::Body,None);
            let pending=self.field_control_body(fun,&region.body,first,incoming_name,incoming,runtime,&name,&controls,&return_fields,false,&normal,has_normal,Some(&caught_fields));
            let (normal_pattern,normal_restore)=restoration(&values(&normal),&format!("caught_control_normal{first}"));
            let (error_pattern,error_restore)=restoration(&values(&caught_fields),&format!("caught_control_error{first}"));
            let normal=if has_normal{format!("{{{normal_restore}TsrCompletion::Normal({})}}",tuple(&values(&normal_input)))}else{"unreachable!(\"no normal caught control body\")".into()};
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
            let setup=if let Some(binding)=handler.binding{
                let value=exception_contract_emit::checked_catch_value("caught_error",incoming_name,incoming,&unions,runtime,statement.span).expect("checked control catch value");
                format!("let {}value{binding}={value};",if fun.bindings[binding].mutable{"mut "}else{""})
            }else{"drop(caught_error);".into()};
            let id=self.completion_next.get();self.completion_next.set(id+1);
            let handled=self.field_control_body(fun,&handler.body,id,error_name,contract,runtime,&name,&controls,&return_fields,false,&normal_input,body_normal,None);
            format!("match {pending}{{TsrCompletion::Normal({normal_pattern})=>{normal},TsrCompletion::Throw((caught_error,{error_pattern}))=>{{{error_restore}{setup}{handled}}},TsrCompletion::Return(value)=>TsrCompletion::Return(value),_=>unreachable!(\"checked caught field control\")}}")
        }else{self.field_control_body(fun,&region.body,first,error_name,contract,runtime,&name,&controls,&return_fields,false,&normal_input,body_normal,None)};
        let (input_pattern,input_restore)=restoration(&values(&normal_input),&format!("control_normal_input{first}"));
        let (output_pattern,output_restore)=restoration(&values(&normal_output),&format!("control_normal_output{first}"));
        let normal_cleanup=if body_normal{self.field_control_body(fun,cleanup,first+1,error_name,contract,runtime,&name,&controls,&return_fields,true,&normal_output,cleaned_normal,None)}else{"unreachable!(\"no normal body completion\")".into()};
        let returned_cleanup=self.field_control_body(fun,cleanup,first+2,error_name,contract,runtime,&name,&controls,&return_fields,true,&return_fields[1],!crate::completion::definitely_abrupt(cleanup),None);
        let returned=format!("{{{return_restore}match {returned_cleanup}{{TsrCompletion::Normal(fields)=>TsrCompletion::Return({name}::Return((pending_return,fields))),TsrCompletion::Return(value)=>TsrCompletion::Return(value),TsrCompletion::Throw(error)=>TsrCompletion::Throw(error),_=>unreachable!(\"checked carried return cleanup\")}}}}");
        let (error_pattern,thrown)=if let Some(fields)=&inherited_errors {
            let (pattern,restore)=restoration(&values(fields),&format!("pending_control_error{first}"));
            let id=self.completion_next.get();self.completion_next.set(id+1);
            let survives=fun.storage_exits.has_completion_owners(boundary,Cleaned,Some(CompletionKind::Throw));
            let cleanup=self.field_control_body(fun,cleanup,id,error_name,contract,runtime,&name,&controls,&return_fields,true,fields,survives,None);
            (format!("(pending_error,{pattern})"),format!("{{{restore}match {cleanup}{{TsrCompletion::Normal(fields)=>TsrCompletion::Throw((pending_error,fields)),TsrCompletion::Return(value)=>TsrCompletion::Return(value),TsrCompletion::Throw(error)=>TsrCompletion::Throw(error),_=>unreachable!(\"checked carried control error\")}}}}"))
        }else{("pending_error".into(),exception_emit::finally_region("TsrCompletion::Throw(pending_error)",&returned_cleanup))};
        let mut arms=String::new();let mut dispatch=String::new();
        for (index,control) in controls.iter().enumerate() {
            let (pattern,restore)=restoration(&values(&control.input),&format!("control_input{first}_{index}"));
            let (out_pattern,out_restore)=restoration(&values(&control.output),&format!("control_output{first}_{index}"));
            let kind=if control.continuing{CompletionKind::Continue(Some(control.target))}else{CompletionKind::Break(Some(control.target))};
            let survives=fun.storage_exits.has_completion_owners(boundary,Cleaned,Some(kind));
            let completed=self.field_control_body(fun,cleanup,first+3+index,error_name,contract,runtime,&name,&controls,&return_fields,true,&control.output,survives,None);
            let _=write!(arms,"TsrCompletion::Return({name}::Exit{index}({pattern}))=>{{{restore}match {completed}{{TsrCompletion::Normal(fields)=>TsrCompletion::Return({name}::Exit{index}(fields)),TsrCompletion::Return(value)=>TsrCompletion::Return(value),TsrCompletion::Throw(error)=>TsrCompletion::Throw(error),_=>unreachable!(\"carried cleanup control\")}}}},");
            let route=self.exception_regions.control_at(control.target,control.continuing,control.span);
            let _=write!(dispatch,"TsrCompletion::Return({name}::Exit{index}({out_pattern}))=>{{{out_restore}{route}}},");
        }
        let normal=if body_normal{format!("{{{input_restore}{normal_cleanup}}}")}else{normal_cleanup};
        let joined=format!("match {pending}{{TsrCompletion::Normal({input_pattern})=>{normal},TsrCompletion::Return({name}::Return((pending_return,{return_pattern})))=>{returned},TsrCompletion::Throw({error_pattern})=>{thrown},{arms}_=>unreachable!(\"checked field control\")}}");
        let (outer_name,outer_contract,_)=self.exception_regions.current_transport().expect("outer field control");
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
        if let Some(errors)=&inherited_errors {
            let (error_pattern,error_restore)=restoration(&values(errors),&format!("outgoing_control_error{first}"));
            let pending=format!("TsrCompletion::<(),{},{}>::Throw(error)",self.native_type(fun.result),error_name);
            let widened=exception_contract_emit::checked_completion_conversion(&pending,error_name,&outer_name,contract,&outer_contract,&unions,runtime,statement.span).expect("checked control field error forwarding");
            let returned=self.exception_regions.return_value("value",statement.span);let thrown=self.exception_regions.native_throw("error");
            let normal=if cleaned_normal{output_restore}else{"unreachable!(\"no normal cleanup completion\")".into()};
            return Some(format!("{{{declaration}match {joined}{{TsrCompletion::Normal({output_pattern})=>{{{normal}}},TsrCompletion::Return({name}::Return((value,{returned_pattern})))=>{{{returned_restore}{returned}}},TsrCompletion::Throw((error,{error_pattern}))=>{{{error_restore}match {widened}{{TsrCompletion::Throw(error)=>{{{thrown}}},_=>unreachable!(\"selected control field error\")}}}},{dispatch}_=>unreachable!(\"checked field control dispatch\")}}}}"));
        }
        let widened=exception_contract_emit::checked_completion_conversion(&joined,error_name,&outer_name,contract,&outer_contract,&unions,runtime,statement.span).expect("checked field control errors");
        let returned=self.exception_regions.return_value("value",statement.span);let thrown=self.exception_regions.native_throw("error");
        let normal=if cleaned_normal{output_restore}else{"unreachable!(\"no normal cleanup completion\")".into()};
        Some(format!("{{{declaration}match {widened}{{TsrCompletion::Normal({output_pattern})=>{{{normal}}},TsrCompletion::Return({name}::Return((value,{returned_pattern})))=>{{{returned_restore}{returned}}},TsrCompletion::Throw(error)=>{{{thrown}}},{dispatch}_=>unreachable!(\"checked field control dispatch\")}}}}"))
    }
    pub(super) fn control_completion_fields(&self,fun:&CheckedFunction,boundary:usize,phase:crate::analysis::MapCompletionPhase,kind:Option<CompletionKind>,places:&[Place])->Option<Fields> {
        let mut fields=vec![];
        for place in places {
            let binding=&fun.bindings[place.binding];
            if binding.capture_reference || binding.borrowed_place.is_some() || binding.mutable_place
                || binding.parameter && binding.mode!=Mode::Consume
                || self.restored_capture(fun,place.binding){return None;}
            if let Some(layout)=self.owned_record_layout(fun,place.binding){
                for leaf in &layout.leaves {
                    let mut ty=binding.ty;let mut names=Vec::new();
                    for index in &leaf.path {
                        let Type::Record(record)=ty else{return None;};
                        let field=self.program.records[record].fields.get(*index)?;
                        names.push(field.name.text.clone());ty=field.ty?;
                    }
                    if !names.starts_with(&place.fields){continue;}
                    let projected=Place{binding:place.binding,fields:names};
                    if fun.storage_exits.completion_field_available(boundary,phase,kind,&projected)==Some(true){fields.push((leaf.native_name(place.binding),leaf.ty));}
                }
                continue;
            }
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
    fn field_control_body(&self,fun:&CheckedFunction,body:&[Stmt],id:usize,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,name:&str,controls:&[ControlFields],returns:&[Fields;2],cleaned:bool,normal:&Fields,has_normal:bool,error_fields:Option<&Fields>)->String {
        let inherited_errors=error_fields.cloned().or_else(||self.exception_regions.field_error_fields());
        let _region=self.exception_regions.enter_typed(id,exception_contract_emit::local_control_targets(body),error_name.into(),contract.clone(),runtime);
        self.exception_regions.attach_storage(vec![],Default::default(),Default::default(),inherited_errors.is_some(),body.first().map_or(0,|statement|statement.span.start));
        let error_type=if let Some(errors)=&inherited_errors {
            let storage=errors.iter().map(|(place,ty)|native_storage_availability::Value{place:place.clone(),empty:String::new(),live:None,transferred:None,target:Some(*ty)}).collect();
            self.exception_regions.attach_error_storage(storage,Default::default(),Default::default());
            self.exception_regions.mark_field_error_storage(errors);
            format!("({error_name},{})",tuple(&errors.iter().map(|(_,ty)|self.native_type(*ty)).collect::<Vec<_>>()))
        }else{error_name.to_owned()};
        self.exception_regions.carry_storage_on_control_exit();
        let returned=if cleaned{&returns[1]}else{&returns[0]};
        let storage=returned.iter().map(|(place,ty)|native_storage_availability::Value{place:place.clone(),empty:String::new(),live:None,transferred:None,target:Some(*ty)}).collect();
        self.exception_regions.attach_return_storage(name.into(),storage,Default::default(),Default::default());
        let mut arguments=vec![self.native_type(fun.result),tuple(&returned.iter().map(|(_,ty)|self.native_type(*ty)).collect::<Vec<_>>())];
        for (index,control) in controls.iter().enumerate() {
            let fields=if cleaned{&control.output}else{&control.input};
            arguments.push(tuple(&fields.iter().map(|(_,ty)|self.native_type(*ty)).collect::<Vec<_>>()));
            let storage=fields.iter().map(|(place,_)|native_storage_availability::Value{place:place.clone(),empty:String::new(),live:None,transferred:None,target:None}).collect();
            self.exception_regions.attach_control_storage(control.target,control.continuing,format!("{name}::Exit{index}"),storage,Default::default(),Default::default());
        }
        let normal_type=tuple(&normal.iter().map(|(_,ty)|self.native_type(*ty)).collect::<Vec<_>>());
        let normal_value=if has_normal{format!("TsrCompletion::Normal({})",tuple(&values(normal)))}else{"unreachable!(\"checked abrupt field body\")".into()};
        let mut statements=String::new();self.block(&mut statements,fun,body,1);
        format!("{{let completion:TsrCompletion<{normal_type},{name}<{}>,{error_type}>='exception_region{id}:{{{statements}{normal_value}}};completion}}",arguments.join(","))
    }
}
