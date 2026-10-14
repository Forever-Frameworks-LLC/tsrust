//! Completion expressions share the statement scheduler and retain lexical scopes.
use super::*;
use statement_work::Work;
use std::rc::Rc;
impl Emitter<'_> {
    pub(super) fn schedule_completion_body<'a,'s:'a>(&'s self,output:&mut String,fun:&'a CheckedFunction,body:&'a [Stmt],id:usize,error_name:String,contract:crate::error_effects::FailureSet,runtime:Option<Type>,initialize_parameters:bool,pending:&mut Vec<Work<'a,'s>>) {
        self.completion_next.set(self.completion_next.get().max(id+1));
        let locals=exception_contract_emit::local_control_targets(body);
        let region=self.exception_regions.enter_typed(id,locals,error_name.clone(),contract,runtime);
        if self.return_passes_through.take(){self.exception_regions.mark_return_passes_through();}
        if let Some(route)=self.inline_return.borrow_mut().take(){self.exception_regions.set_inline_return(route);}
        if let Some(route)=self.inline_abrupt.borrow_mut().take(){self.exception_regions.set_inline_abrupt(route);}
        let mut prefix_code=String::new();
        let prefix=if initialize_parameters {
            self.parameter_initialization(&mut prefix_code,fun);
            for slot in self.finally_moves.parameters(fun){if !self.restored_slots.contains(fun,slot){let _=writeln!(prefix_code,"let mut value{slot}=Some(value{slot});");}}
            body.iter().take_while(|stmt|fun.bindings.iter().any(|binding|binding.input_ty.is_some() && binding.name.starts_with("pattern#") && binding.declaration_span.is_some_and(|span|stmt.span.start>=span.start && stmt.span.end<=span.end))).count()
        }else{0};
        // Final rendering occurs while the typed region is still installed.
        pending.push(Work::Region(region));
        pending.push(Work::Resume(Box::new(move |output,_,values|{
            let statements=values.pop().expect("completed exception body");
            let expression=exception_emit::completion_region(id,&statements);
            let _=write!(output,"{{ let completion:TsrCompletion<(),{},{}>={expression}; completion }}",self.native_type(fun.result),error_name);
        })));
        pending.push(Work::Captured);
        pending.push(Work::Body(&body[prefix..],Rc::new(body[prefix..].iter().collect()),1,None));
        pending.push(Work::Text(prefix_code));
        pending.push(Work::Capture);
        let _=output;
    }

    /// The handler starts only after the body's region and storage guards unwind.
    pub(super) fn schedule_completion_catch<'a,'s:'a>(&'s self,output:&mut String,fun:&'a CheckedFunction,body:&'a [Stmt],handler:&'a CatchClause,ids:(usize,usize),incoming_name:String,outgoing_name:String,outgoing:crate::error_effects::FailureSet,runtime:Option<Type>,direct_returns:bool,inline:Option<exception_contract_emit::InlineExit>,inline_abrupt:Option<exception_contract_emit::InlineExit>,work:&mut Vec<Work<'a,'s>>) {
        let incoming=handler.error_contract.as_deref().expect("checked catch contract");
        *self.inline_return.borrow_mut()=inline.clone();
        self.return_passes_through.set(direct_returns);
        let body_name=incoming_name.clone();
        work.push(Work::Resume(Box::new(move |output,work,values| {
            let pending=values.pop().expect("completed catch body");
            *self.inline_return.borrow_mut()=inline;
            *self.inline_abrupt.borrow_mut()=inline_abrupt;
            self.return_passes_through.set(direct_returns);
            work.push(Work::Resume(Box::new(move |output,_,values| {
                let handled=values.pop().expect("completed catch handler");
                let setup=if let Some(binding)=handler.binding {
                    let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
                    let value=exception_contract_emit::checked_catch_value("caught_error",&incoming_name,incoming,&unions,runtime,fun.bindings[binding].declaration_span.unwrap_or_default()).expect("checked catch payload");
                    format!("let {}value{binding}={value};",if fun.bindings[binding].mutable {"mut "}else{""})
                }else{"std::mem::drop(caught_error);".into()};
                output.push_str(&exception_emit::catch_region(&pending,"caught_error",&format!("{{ {setup} {handled} }}")));
            })));
            work.push(Work::Captured);
            self.schedule_completion_body(output,fun,&handler.body,ids.1,outgoing_name,outgoing,runtime,false,work);
            work.push(Work::Capture);
        })));
        work.push(Work::Captured);
        self.schedule_completion_body(output,fun,body,ids.0,body_name,incoming.clone(),runtime,false,work);
        work.push(Work::Capture);
    }

    pub(super) fn schedule_completion_handlers<'a,'s:'a>(&'s self,output:&mut String,fun:&'a CheckedFunction,region:&'a ExceptionRegion,ids:(usize,usize,usize),names:(String,String),outgoing:crate::error_effects::FailureSet,runtime:Option<Type>,include_cleanup:bool,work:&mut Vec<Work<'a,'s>>) {
        let cleanup_body=if include_cleanup {region.finally.as_deref()} else {None};
        if let Some(cleanup)=cleanup_body && crate::completion::definitely_abrupt(cleanup) {
            let mut pending_region=region.clone();pending_region.finally=None;
            let mut statements=vec![Stmt{labels:vec![],control_target:None,kind:StmtKind::Try(Box::new(pending_region)),span:crate::Span::default()}];
            let pending_contract=crate::error_effects::annotate_region_body(&mut statements,&self.program.error_effects);
            let (declaration,pending_name)=self.completion_error_layout(&format!("PendingError{}",ids.0),&pending_contract,runtime,crate::Span::default());
            let guarded=self.cleanup_replaces_exits.replace(true);
            let incoming_name=names.0.clone();
            work.push(Work::Resume(Box::new(move |output,work,values| {
                let pending=values.pop().expect("completed pending region");
                self.cleanup_replaces_exits.set(guarded);
                work.push(Work::Resume(Box::new(move |output,_,values| {
                    let cleanup=values.pop().expect("completed replacing cleanup");
                    let _=write!(output,"{{ {declaration} let pending_completion={pending}; let cleanup_completion={cleanup}; std::mem::drop(pending_completion); cleanup_completion }}");
                })));
                work.push(Work::Captured);
                self.schedule_completion_body(output,fun,cleanup,ids.2,names.1,outgoing,runtime,false,work);
                work.push(Work::Capture);
            })));
            work.push(Work::Captured);
            // Defer setup as well as emission; no recursive scheduler entry.
            work.push(Work::Resume(Box::new(move |output,work,_| {
                self.schedule_completion_handlers(output,fun,region,ids,(incoming_name,pending_name),pending_contract,runtime,false,work);
            })));
            work.push(Work::Capture);
            return;
        }
        if let Some(body)=cleanup_body {
            let error_name=names.1.clone();let contract=outgoing.clone();
            work.push(Work::Resume(Box::new(move |output,work,values| {
                let cleanup=values.pop().expect("completed cleanup region");
                self.schedule_completion_pending(output,fun,region,ids,names,outgoing,runtime,cleanup_body,Some(cleanup),work);
            })));
            work.push(Work::Captured);
            self.schedule_completion_body(output,fun,body,ids.2,error_name,contract,runtime,false,work);
            work.push(Work::Capture);
        }else{
            self.schedule_completion_pending(output,fun,region,ids,names,outgoing,runtime,None,None,work);
        }
    }

    fn schedule_completion_pending<'a,'s:'a>(&'s self,output:&mut String,fun:&'a CheckedFunction,region:&'a ExceptionRegion,ids:(usize,usize,usize),names:(String,String),outgoing:crate::error_effects::FailureSet,runtime:Option<Type>,cleanup_body:Option<&'a [Stmt]>,cleanup:Option<String>,work:&mut Vec<Work<'a,'s>>) {
        // Keep partial-move state visible to Rust across cleanup. Storage-carrying
        // frames retain their dedicated transport and cannot use this direct route.
        let partial_return=exception_contract_emit::has_partial_return(fun,&region.body)
            || region.catch.as_ref().is_some_and(|handler|exception_contract_emit::has_partial_return(fun,&handler.body))
            || cleanup_body.is_some_and(|body|exception_contract_emit::has_partial_return(fun,body));
        let terminal_body=crate::completion::definitely_abrupt(&region.body)
            && region.catch.as_ref().is_none_or(|handler|crate::completion::definitely_abrupt(&handler.body))
            && exception_contract_emit::escaping_controls(&region.body).is_empty()
            && region.catch.as_ref().is_none_or(|handler|exception_contract_emit::escaping_controls(&handler.body).is_empty())
            && cleanup_body.is_none_or(|body|exception_contract_emit::escaping_controls(body).is_empty());
        let inline_exit=||{
            // Retain the lexical body boundary whenever initialized root slots
            // suffice. Its locals must drop before cleanup executes.
            if cleanup.is_some() && (!partial_return || terminal_body){return None;}
            if self.cleanup_replaces_exits.get() || !self.exception_regions.plain_return_destination(){return None;}
            let binding=format!("finally_exit{}",ids.0);
            let pending=binding.clone();
            let pending=cleanup.as_ref().map_or_else(||pending.clone(),|cleanup|exception_emit::finally_region(&pending,cleanup));
            let (outer_name,outer_contract,_)=self.exception_regions.current_transport().expect("enclosing cleanup transport");
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            let widened=exception_contract_emit::checked_completion_conversion(&pending,&names.1,&outer_name,&outgoing,&outer_contract,&unions,runtime,crate::Span::default()).expect("checked cleanup errors");
            let returned=self.exception_regions.return_value("value",crate::Span::default());
            let thrown=self.exception_regions.native_throw("error");
            let controls=cleanup_body.map_or_else(String::new,|body|exception_contract_emit::escaping_controls(body).into_iter().map(|(target,continuing,span)|{
                let label=match target {ControlTarget::Loop(id)|ControlTarget::Tail(id)=>format!("TsrControlTarget::Loop({id})"),ControlTarget::Block(id)=>format!("TsrControlTarget::Block({id})")};
                let kind=if continuing{"Continue"}else{"Break"};let route=self.exception_regions.control_at(target,continuing,span);
                format!("TsrCompletion::{kind}({label})=>{{{route}}},")
            }).collect::<String>());
            let abrupt=self.exception_regions.inline_abrupt("completion").map_or_else(
                ||format!("TsrCompletion::Return(value)=>{{{returned}}},TsrCompletion::Throw(error)=>{{{thrown}}}"),
                |route|format!("completion @ (TsrCompletion::Return(_)|TsrCompletion::Throw(_))=>{{{route}}}"));
            Some(exception_contract_emit::InlineExit{binding,ty:format!("TsrCompletion<(),{},{}>",self.native_type(fun.result),&names.1),code:format!("match {widened} {{{abrupt},{controls}_=>unreachable!(\"checked abrupt cleanup exit\")}}")})
        };
        let inline_abrupt=inline_exit();
        let inline=if cleanup.is_some(){inline_abrupt.clone()}else{None};
        let direct_returns=cleanup_body.is_none() && !self.cleanup_replaces_exits.get();

        work.push(Work::Resume(Box::new(move |output,_,values| {
            let pending=values.pop().expect("completed handled region");
            if let Some(cleanup)=cleanup {output.push_str(&exception_emit::finally_region(&pending,&cleanup));}else{output.push_str(&pending);}
        })));
        work.push(Work::Captured);
        if let Some(handler)=&region.catch {
            self.schedule_completion_catch(output,fun,&region.body,handler,(ids.0,ids.1),names.0,names.1,outgoing,runtime,direct_returns,inline,inline_abrupt,work);
        }else{
            *self.inline_return.borrow_mut()=inline;
            *self.inline_abrupt.borrow_mut()=inline_abrupt;
            self.return_passes_through.set(direct_returns);
            self.schedule_completion_body(output,fun,&region.body,ids.0,names.1,outgoing,runtime,false,work);
        }
        work.push(Work::Capture);
    }

    pub(super) fn schedule_nested_completion<'a,'s:'a>(&'s self,output:&mut String,fun:&'a CheckedFunction,statement:&'a Stmt,region:&'a ExceptionRegion,work:&mut Vec<Work<'a,'s>>) {
        let (outer_name,outer_contract,runtime)=self.exception_regions.current_transport().expect("checked enclosing completion transport");
        let first=self.completion_next.get();self.completion_next.set(first+3);
        let mut discovery=vec![statement.clone()];
        let outgoing=crate::error_effects::annotate_region_body(&mut discovery,&self.program.error_effects);
        let (out_decl,out_name)=self.completion_error_layout(&format!("RegionError{first}"),&outgoing,runtime,statement.span);
        let (in_decl,in_name)=if let Some(handler)=&region.catch {
            self.completion_error_layout(&format!("RegionIncoming{first}"),handler.error_contract.as_deref().expect("checked catch contract"),runtime,statement.span)
        }else{(String::new(),out_name.clone())};
        work.push(Work::Text(" }".into()));
        if self.schedule_field_control_transport(fun,statement,region,first,&in_name,&out_name,&outgoing,runtime,work).is_some()
            || self.schedule_field_completion_transport(fun,statement,region,first,&in_name,&out_name,&outgoing,runtime,work).is_some() {
            let _=write!(output,"{{ {in_decl} {out_decl} ");
            return;
        }
        work.pop(); // Neither field plan scheduled work; remove its wrapper.
        let names=(in_name.clone(),out_name.clone());let contract=outgoing.clone();
        work.push(Work::Resume(Box::new(move |output,_,values| {
            let pending=values.pop().expect("completed nested region");
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let widened=exception_contract_emit::checked_completion_conversion(&pending,&out_name,&outer_name,&outgoing,&outer_contract,&unions,runtime,statement.span).expect("checked enclosing error contract");
        let dispatch=self.exception_regions.dispatch(&widened,&self.control_targets.snapshot(),statement.span).expect("checked completion destination");
        let _=write!(output,"{{ {in_decl} {out_decl} {dispatch} }}");
        })));
        work.push(Work::Captured);
        self.schedule_completion_handlers(output,fun,region,(first,first+1,first+2),names,contract,runtime,true,work);
        work.push(Work::Capture);
    }
}
