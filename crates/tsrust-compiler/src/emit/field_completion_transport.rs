//! Keep partial field ownership visible without executing cleanup inside body scopes.
use super::*;
use statement_work::Work;
use std::rc::Rc;
use crate::capture_loan_owners::Place;

pub(super) struct FieldPlan {pub(super) input_fields:Vec<String>,pub(super) output_fields:Vec<String>}
struct FieldPhase<'a>{body:&'a [Stmt],id:Option<usize>,error_name:String,contract:crate::error_effects::FailureSet,kind:FieldPhaseKind}
enum FieldPhaseKind{Caught{normal:Vec<(String,Type)>,errors:Vec<(String,Type)>,has_normal:bool},Return(String,Vec<(String,Type)>),Normal(String)}
impl Emitter<'_> {
    pub(super) fn schedule_field_completion_transport<'a,'s:'a>(&'s self,fun:&'a CheckedFunction,statement:&'a Stmt,region:&'a ExceptionRegion,first:usize,incoming_name:&str,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,work:&mut Vec<Work<'a,'s>>)->Option<()> {
        let cleanup=region.finally.as_ref()?;
        let inherited_errors=self.exception_regions.field_error_fields();
        let catch=region.catch.as_ref().filter(|handler|handler.error_contract.as_ref().is_some_and(|errors|errors.runtime || errors.opaque || !errors.explicit.is_empty()));
        let FieldPlan{mut input_fields,mut output_fields}=self.completion_field_plan_inner(fun,statement,region,true)?;
        let boundary=region.body.first().or_else(||cleanup.first())?.span.start;
        if let Some(fields)=&inherited_errors {
            for (name,_) in fields {
                let place=self.field_native_place(fun,name).expect("checked inherited field owner");
                for (phase,names) in [(crate::analysis::MapCompletionPhase::Handled,&mut input_fields),(crate::analysis::MapCompletionPhase::Cleaned,&mut output_fields)] {
                    if fun.storage_exits.completion_field_available(boundary,phase,None,&place)==Some(true) && !names.contains(name){names.push(name.clone());}
                }
            }
        }
        let return_fields=inherited_errors.as_ref().map_or_else(Vec::new,|fields|fields.iter().filter(|(name,_)|{
            fun.storage_exits.completion_field_available(boundary,crate::analysis::MapCompletionPhase::Handled,Some(crate::completion::CompletionKind::Return),&self.field_native_place(fun,name).expect("checked native field owner"))==Some(true)
        }).cloned().collect::<Vec<_>>());
        let return_names=return_fields.iter().map(|(name,_)|name.clone()).collect::<Vec<_>>();
        let (return_pattern,return_restore)=restoration(&return_names,&format!("pending_return_field{first}"));
        let returned_pattern=if return_fields.is_empty(){"pending_return".into()}else{format!("(pending_return,{return_pattern})")};
        let body_return=if return_fields.is_empty(){"value".into()}else{format!("(value,{})",tuple(&return_names))};
        let values=tuple(&input_fields);
        let (pattern,restore)=restoration(&input_fields,&format!("input_field{first}"));
        let output_values=tuple(&output_fields);
        let (output_pattern,output_restore)=restoration(&output_fields,&format!("normal_field{first}"));
        let has_normal=fun.storage_exits.has_completion_owners(boundary,crate::analysis::MapCompletionPhase::Handled,None);
        let output_restore=if fun.storage_exits.has_completion_owners(boundary,crate::analysis::MapCompletionPhase::Cleaned,None){output_restore}else{"unreachable!(\"no normal field cleanup completion\")".into()};

        let mut phases=Vec::new();
        let catch_render=if let Some(handler)=catch {
            let incoming=handler.error_contract.as_deref().expect("checked catch contract");
            let mut places=partial_places(fun,&region.body);normalize(fun,statement.span.start,&mut places);
            use crate::analysis::MapCompletionPhase::Body;
            let normal=self.control_completion_fields(fun,boundary,Body,None,&places)?;
            let errors=self.control_completion_fields(fun,boundary,Body,Some(crate::completion::CompletionKind::Throw),&places)?;
            let names=|fields:&Vec<(String,Type)>|fields.iter().map(|(name,_)|name.clone()).collect::<Vec<_>>();
            let (body_pattern,body_restore)=restoration(&names(&normal),&format!("body_field{first}"));
            let (error_pattern,error_restore)=restoration(&names(&errors),&format!("caught_field{first}"));
            let has_normal=fun.storage_exits.has_completion_owners(boundary,Body,None);
            let normal_result=if has_normal{format!("{{{body_restore}TsrCompletion::Normal({values})}}") }else{"unreachable!(\"no normal throwing body completion\")".into()};
            let setup=if let Some(binding)=handler.binding {
                let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
                let value=exception_contract_emit::checked_catch_value("caught_error",incoming_name,incoming,&unions,runtime,statement.span).expect("checked field catch value");
                format!("let {}value{binding}={value};",if fun.bindings[binding].mutable{"mut "}else{""})
            }else{"drop(caught_error);".into()};
            phases.push(FieldPhase{body:&region.body,id:Some(first),error_name:incoming_name.into(),contract:incoming.clone(),kind:FieldPhaseKind::Caught{normal,errors,has_normal}});
            phases.push(FieldPhase{body:&handler.body,id:Some(first+1),error_name:error_name.into(),contract:contract.clone(),kind:FieldPhaseKind::Return(values.clone(),return_fields.clone())});
            Some((body_pattern,normal_result,error_pattern,error_restore,setup))
        }else{
            phases.push(FieldPhase{body:&region.body,id:Some(first),error_name:error_name.into(),contract:contract.clone(),kind:FieldPhaseKind::Return(values.clone(),return_fields)});
            None
        };
        if has_normal{phases.push(FieldPhase{body:cleanup,id:Some(first+2),error_name:error_name.into(),contract:contract.clone(),kind:FieldPhaseKind::Normal(output_values)});}
        phases.push(FieldPhase{body:cleanup,id:Some(first+3),error_name:error_name.into(),contract:contract.clone(),kind:FieldPhaseKind::Normal("()".into())});
        let (error_pattern,error_restore,error_values)=inherited_errors.as_ref().map_or_else(||("pending_error".to_owned(),String::new(),String::new()),|fields|{
            let names=fields.iter().map(|(name,_)|name.clone()).collect::<Vec<_>>();
            let (pattern,restore)=restoration(&names,&format!("pending_error_field{first}"));
            (format!("(pending_error,{pattern})"),restore,tuple(&names))
        });
        if inherited_errors.is_some(){phases.push(FieldPhase{body:cleanup,id:None,error_name:error_name.into(),contract:contract.clone(),kind:FieldPhaseKind::Normal(error_values)});}
        self.completion_next.set(self.completion_next.get().max(first+4));
        let count=phases.len();let error_name=error_name.to_owned();let contract=contract.clone();
        work.push(Work::Resume(Box::new(move |output,_,results| {
            let mut results=results.split_off(results.len()-count).into_iter();
            let body=results.next().expect("field body");
            let pending=if let Some((body_pattern,normal_result,error_pattern,error_restore,setup))=catch_render {
                let handled=results.next().expect("field handler");
                format!("match {body}{{TsrCompletion::Normal({body_pattern})=>{normal_result},TsrCompletion::Throw((caught_error,{error_pattern}))=>{{{error_restore}{setup}{handled}}},TsrCompletion::Return(value)=>TsrCompletion::Return({body_return}),_=>unreachable!(\"checked field catch control\")}}")
            }else{body};
            let normal_cleanup=if has_normal{results.next().expect("normal field cleanup")}else{"unreachable!(\"no normal handled field completion\")".into()};
            let returned_cleanup=results.next().expect("return field cleanup");
            let returned=exception_emit::finally_region("TsrCompletion::Return(pending_return)",&returned_cleanup);
            let thrown=if inherited_errors.is_some(){
                let cleanup=results.next().expect("error field cleanup");
                format!("{{{error_restore}match {cleanup}{{TsrCompletion::Normal(fields)=>TsrCompletion::Throw((pending_error,fields)),TsrCompletion::Return(value)=>TsrCompletion::Return(value),TsrCompletion::Throw(error)=>TsrCompletion::Throw(error),_=>unreachable!(\"checked error cleanup\")}}}}")
            }else{exception_emit::finally_region("TsrCompletion::Throw(pending_error)",&returned_cleanup)};
            let error_name=error_name.as_str();let contract=&contract;
        let joined=format!("match {pending}{{TsrCompletion::Normal({pattern})=>{{{restore}{normal_cleanup}}},TsrCompletion::Return({returned_pattern})=>{{{return_restore}{returned}}},TsrCompletion::Throw({error_pattern})=>{thrown},_=>unreachable!(\"checked field completion control\")}}");
        let (outer_name,outer_contract,_)=self.exception_regions.current_transport().expect("outer field completion");
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
        if let Some(errors)=&inherited_errors {
            let names=errors.iter().map(|(name,_)|name.clone()).collect::<Vec<_>>();
            let (error_pattern,error_restore)=restoration(&names,&format!("outgoing_error_field{first}"));
            let pending=format!("TsrCompletion::<(),{},{}>::Throw(error)",self.native_type(fun.result),error_name);
            let widened=exception_contract_emit::checked_completion_conversion(&pending,error_name,&outer_name,contract,&outer_contract,&unions,runtime,statement.span).expect("checked field error forwarding");
            let returned=self.exception_regions.return_value("value",statement.span);
            let thrown=self.exception_regions.native_throw("error");
            output.push_str(&format!("match {joined}{{TsrCompletion::Normal({output_pattern})=>{{{output_restore}}},TsrCompletion::Return(value)=>{{{returned}}},TsrCompletion::Throw((error,{error_pattern}))=>{{{error_restore}match {widened}{{TsrCompletion::Throw(error)=>{{{thrown}}},_=>unreachable!(\"selected field error\")}}}},_=>unreachable!(\"checked field completion\")}}"));return;
        }
        let widened=exception_contract_emit::checked_completion_conversion(&joined,error_name,&outer_name,contract,&outer_contract,&unions,runtime,statement.span).expect("checked field error conversion");
        let returned=self.exception_regions.return_value("value",statement.span);
        let thrown=self.exception_regions.native_throw("error");
        output.push_str(&format!("match {widened}{{TsrCompletion::Normal({output_pattern})=>{{{output_restore}}},TsrCompletion::Return(value)=>{{{returned}}},TsrCompletion::Throw(error)=>{{{thrown}}},_=>unreachable!(\"checked field cleanup control\")}}"))

        })));
        for phase in phases.into_iter().rev(){
            work.push(Work::Captured);
            work.push(Work::Resume(Box::new(move |_,work,_| {
                let id=phase.id.unwrap_or_else(||{let id=self.completion_next.get();self.completion_next.set(id+1);id});
                match phase.kind {
                    FieldPhaseKind::Caught{normal,errors,has_normal}=>self.schedule_field_error_body(fun,phase.body,id,&phase.error_name,&phase.contract,runtime,&normal,&errors,has_normal,work),
                    FieldPhaseKind::Return(normal,returns)=>self.schedule_field_return_body(fun,phase.body,id,&phase.error_name,&phase.contract,runtime,&normal,&returns,work),
                    FieldPhaseKind::Normal(normal)=>self.schedule_field_body(fun,phase.body,id,&phase.error_name,&phase.contract,runtime,&normal,work),
                }
            })));
            work.push(Work::Capture);
        }
        Some(())
    }
    pub(super) fn field_native_place(&self,fun:&CheckedFunction,native:&str)->Option<Place> {
        if native.contains("_field") {
            let (binding,fields)=self.map_storage_binding(fun,native)?;
            return Some(Place{binding,fields});
        }
        let mut parts=native.split('.');
        let binding=parts.next()?.strip_prefix("value")?.parse::<usize>().ok()?;
        let mut ty=fun.bindings.get(binding)?.ty;let mut fields=vec![];
        for part in parts {
            let index=part.strip_prefix("field")?.parse::<usize>().ok()?;
            let Type::Record(record)=ty else{return None;};
            let field=self.program.records.get(record)?.fields.get(index)?;
            fields.push(field.name.text.clone());ty=field.ty?;
        }Some(Place{binding,fields})
    }
    pub(super) fn completion_field_plan(&self,fun:&CheckedFunction,statement:&Stmt,region:&ExceptionRegion)->Option<FieldPlan> {
        self.completion_field_plan_inner(fun,statement,region,false)
    }
    pub(super) fn completion_field_plan_inner(&self,fun:&CheckedFunction,statement:&Stmt,region:&ExceptionRegion,allow_catch_transfers:bool)->Option<FieldPlan> {
        let cleanup=region.finally.as_ref()?;
        let catch=region.catch.as_ref().filter(|handler|handler.error_contract.as_ref().is_some_and(|errors|errors.runtime || errors.opaque || !errors.explicit.is_empty()));
        // Control exits need their own surviving-field payloads.
        if !exception_contract_emit::escaping_controls(&region.body).is_empty()
            || catch.is_some_and(|handler|!exception_contract_emit::escaping_controls(&handler.body).is_empty())
            || !exception_contract_emit::escaping_controls(cleanup).is_empty() {return None;}
        let mut body_places=partial_places(fun,&region.body);
        if let Some(handler)=catch {body_places.extend(partial_places(fun,&handler.body));}
        let mut places=body_places.clone();places.extend(partial_places(fun,cleanup));
        if places.is_empty(){return None;}
        // With no field transfer in the throwing body, original storage remains
        // initialized at catch entry. Body transfers need precise error payloads.
        if !allow_catch_transfers && catch.is_some() && let Some(start)=region.body.first() {
            let end=crate::error_effects::body_expressions(&region.body).iter().map(|value|value.span.end).chain(region.body.iter().map(|statement|statement.span.end)).max().unwrap_or(start.span.end);
            if fun.storage_exits.expression_points().filter(|point|point.0>=start.span.start && point.1<=end).any(|point|
                body_places.iter().any(|place|fun.storage_exits.transfer_at_expression(point,place.binding,&place.fields).is_some())) {return None;}
        }
        if crate::completion::definitely_abrupt(cleanup) || crate::completion::definitely_abrupt(&region.body) && catch.is_none_or(|handler|crate::completion::definitely_abrupt(&handler.body)) {
            return Some(FieldPlan{input_fields:vec![],output_fields:vec![]});
        }
        normalize(fun,statement.span.start,&mut body_places);
        normalize(fun,statement.span.start,&mut places);
        let boundary=region.body.first().or_else(||cleanup.first())?.span.start;
        use crate::analysis::MapCompletionPhase::{Handled,Cleaned};
        let mut input_fields=vec![];let mut output_fields=vec![];
        for (selection,phase,fields) in [(&body_places,Handled,&mut input_fields),(&places,Cleaned,&mut output_fields)] {
            fields.extend(self.control_completion_fields(fun,boundary,phase,None,selection)?.into_iter().map(|(name,_)|name));
        }
        Some(FieldPlan{input_fields,output_fields})
    }
    fn schedule_field_error_body<'a,'s:'a>(&'s self,fun:&'a CheckedFunction,body:&'a [Stmt],id:usize,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,normal:&[(String,Type)],errors:&[(String,Type)],has_normal:bool,work:&mut Vec<Work<'a,'s>>) {
        let normal_type=tuple(&normal.iter().map(|(_,ty)|self.native_type(*ty)).collect::<Vec<_>>());
        let normal_value=if has_normal{format!("TsrCompletion::Normal({})",tuple(&normal.iter().map(|(name,_)|name.clone()).collect::<Vec<_>>()))}else{"unreachable!(\"checked abrupt throwing body\")".into()};
        self.schedule_field_carried_body(fun,body,id,error_name,contract,runtime,&normal_type,&normal_value,errors,&[],work);
    }
    fn schedule_field_carried_body<'a,'s:'a>(&'s self,fun:&'a CheckedFunction,body:&'a [Stmt],id:usize,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,normal_type:&str,normal_value:&str,errors:&[(String,Type)],returns:&[(String,Type)],work:&mut Vec<Work<'a,'s>>) {
        self.completion_next.set(self.completion_next.get().max(id+1));
        let scope=self.exception_regions.enter_typed(id,exception_contract_emit::local_control_targets(body),error_name.into(),contract.clone(),runtime);
        let storage=errors.iter().map(|(place,ty)|native_storage_availability::Value{place:place.clone(),empty:String::new(),live:None,transferred:None,target:Some(*ty)}).collect();
        self.exception_regions.attach_storage(storage,Default::default(),Default::default(),true,body.first().map_or(0,|statement|statement.span.start));
        self.exception_regions.mark_field_error_storage(errors);
        let mut result=self.native_type(fun.result);
        if !returns.is_empty(){
            let storage=returns.iter().map(|(place,ty)|native_storage_availability::Value{place:place.clone(),empty:String::new(),live:None,transferred:None,target:Some(*ty)}).collect();
            self.exception_regions.attach_field_return_storage(storage);
            result=format!("({result},{})",tuple(&returns.iter().map(|(_,ty)|self.native_type(*ty)).collect::<Vec<_>>()));
        }
        let error_type=tuple(&errors.iter().map(|(_,ty)|self.native_type(*ty)).collect::<Vec<_>>());
        work.push(Work::Region(scope));
        work.push(Work::Text(format!("{normal_value}}};completion}}")));
        work.push(Work::Body(body,Rc::new(body.iter().collect()),1,None));
        work.push(Work::Text(format!("{{let completion:TsrCompletion<{normal_type},{result},({error_name},{error_type})>='exception_region{id}:{{")));
    }
    fn schedule_field_return_body<'a,'s:'a>(&'s self,fun:&'a CheckedFunction,body:&'a [Stmt],id:usize,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,normal:&str,returns:&[(String,Type)],work:&mut Vec<Work<'a,'s>>) {
        if returns.is_empty(){self.schedule_field_body(fun,body,id,error_name,contract,runtime,normal,work);return;}
        let errors=self.exception_regions.field_error_fields().expect("return companions accompany error fields");
        self.schedule_field_carried_body(fun,body,id,error_name,contract,runtime,"_",&format!("TsrCompletion::Normal({normal})"),&errors,returns,work);
    }
    fn schedule_field_body<'a,'s:'a>(&'s self,fun:&'a CheckedFunction,body:&'a [Stmt],id:usize,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,normal:&str,work:&mut Vec<Work<'a,'s>>) {
        if let Some(errors)=self.exception_regions.field_error_fields(){
            self.schedule_field_carried_body(fun,body,id,error_name,contract,runtime,"_",&format!("TsrCompletion::Normal({normal})"),&errors,&[],work);return;
        }
        self.completion_next.set(self.completion_next.get().max(id+1));
        let scope=self.exception_regions.enter_typed(id,exception_contract_emit::local_control_targets(body),error_name.into(),contract.clone(),runtime);
        work.push(Work::Region(scope));
        work.push(Work::Text(format!("TsrCompletion::Normal({normal})}};completion}}")));
        work.push(Work::Body(body,Rc::new(body.iter().collect()),1,None));
        work.push(Work::Text(format!("{{let completion:TsrCompletion<_,{},{}>='exception_region{id}:{{",self.native_type(fun.result),error_name)));
    }

}
pub(super) fn normalize(fun:&CheckedFunction,start:usize,places:&mut Vec<Place>) {
    places.retain(|place|fun.bindings[place.binding].declaration_span.is_none_or(|span|span.start<start));
    places.sort();places.dedup();
    let all=places.clone();places.retain(|place|!all.iter().any(|parent|parent.binding==place.binding && parent.fields.len()<place.fields.len() && place.fields.starts_with(&parent.fields)));
}
pub(super) fn restoration(fields:&[String],prefix:&str)->(String,String) {
    let names=(0..fields.len()).map(|index|format!("{prefix}_{index}")).collect::<Vec<_>>();
    (tuple(&names),fields.iter().zip(&names).map(|(field,value)|format!("{field}={value};")).collect())
}

pub(super) fn tuple(values:&[String])->String {if values.is_empty(){"()".into()}else{format!("({},)",values.join(","))}}
pub(super) fn partial_places(fun:&CheckedFunction,body:&[Stmt])->Vec<Place> {
    let mut places=vec![];let mut pending=body.iter().collect::<Vec<_>>();
    while let Some(statement)=pending.pop(){match &statement.kind {
        StmtKind::Try(region)=>pending.extend(region.statements()),
        StmtKind::Return(Some(value))=>places.extend(fun.storage_exits.partial_places_within((statement.span.start,statement.span.end),value.span)),
        StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
        StmtKind::Loop{body,update,..}=>{pending.extend(body);pending.extend(update);},
        StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
        _=>{}
    }}places
}
