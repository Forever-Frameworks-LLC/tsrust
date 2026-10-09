//! Carry proven surviving Map environments across checked iteration edges.
use super::*;
impl Analyzer {
    #[inline(never)]
    pub(super) fn plan_map_iteration_entry(&mut self,context:&mut Context,statement:&Stmt,results:&mut HashMap<usize,Type>,local:&std::collections::BTreeSet<crate::capture_loan_owners::Place>)->Result<(),Diagnostic> {
        let (body,post_test)=match &statement.kind {
            StmtKind::While{body,..}=>(body,false),
            StmtKind::Loop{condition:Some(_),body,update,post_test,tail_reachable} if !tail_reachable || ordinary_body(update,statement.span.start)=>(body,*post_test),
            StmtKind::ForOf{body,..}=>(body,false),
            _=>return Ok(()),
        };
        if !statement.labels.is_empty() || !ordinary_body(body,statement.span.start){return Ok(());}
        let cleanup_body=body.iter().any(|statement|matches!(statement.kind,StmtKind::Try(_)));
        let mut epochs=Vec::new();
        for place in local {
            if !context.storage_exits.map_completion_survivors(statement.span).iter().any(|state|state.consistent && state.place==*place && !state.released.is_empty()){continue;}
            let entry=context.storage_exits.map_loop_edge(statement.span,storage_exit_availability::MapLoopEdge::Entry,place);
            let back=context.storage_exits.map_loop_edge(statement.span,storage_exit_availability::MapLoopEdge::Back,place);
            // A callable exit can leave no backedge at all. Its cleanup still
            // needs the working storage family for entry deletion and disposal.
            if !context.storage_exits.has_map_loop_edge(statement.span,storage_exit_availability::MapLoopEdge::Back) && !cleanup_body {continue;}
            if !post_test && entry!=back && !cleanup_body {continue;}
            let broken=context.storage_exits.map_loop_edge(statement.span,storage_exit_availability::MapLoopEdge::Break,place);
            if context.storage_exits.has_map_loop_edge(statement.span,storage_exit_availability::MapLoopEdge::Break)
                && broken!=back && !cleanup_body {continue;}
            let entry_empty=entry.is_none_or(HashSet::is_empty);
            let back_empty=back.is_none_or(HashSet::is_empty);
            // A cleanup override can turn an error into a loop control. Carry
            // every reaching edge family without discarding the first entry.
            let mut carried=back.cloned().unwrap_or_default();
            if cleanup_body {
                if let Some(entry)=entry {carried.extend(entry.iter().copied());}
                if let Some(broken)=broken {carried.extend(broken.iter().copied());}
            }
            let empty=carried.is_empty();
            let Some(input)=results.get(&place.binding).copied() else{continue;};
            let Some(Type::Map(map))=self.scoped_array_field_type(input,&place.fields) else{continue;};
            let payload=if empty {
                let Type::Record(record)=self.program.records[map].map_definition.as_ref().unwrap().value else{continue;};
                self.empty_scoped_record_environment(self.scoped_records.declaration(record))?
            }else{
                let mut records=carried.iter().filter_map(|ty|if let Type::Record(record)=ty{Some(*record)}else{None}).collect::<Vec<_>>();
                if records.len()!=carried.len(){continue;}
                records.sort_unstable();
                let mut payload=Type::Record(records[0]);
                for record in records.into_iter().skip(1){let Type::Record(previous)=payload else{unreachable!()};let Some(joined)=self.join_record_types(previous,record,statement.span)? else{return Ok(());};payload=joined;}
                payload
            };
            let output_map=self.scoped_map_value_instance(map,payload);
            let output=self.scoped_array_replace_field(input,&place.fields,Type::Map(output_map)).expect("checked Map path");
            if input==output && !cleanup_body {continue;}
            let entry=storage_exit_availability::MapEntryEpoch{place:place.clone(),input,output,empty:entry_empty,before:true};
            if !post_test && input!=output {context.storage_exits.record_map_entry_epoch(statement.span,entry.clone());}
            results.insert(place.binding,output);
            let back=storage_exit_availability::MapEntryEpoch{place:place.clone(),input:output,output,empty:back_empty,before:false};
            epochs.push(storage_exit_availability::MapIterationEpoch{entry,body:back.clone(),back,carrier_empty:empty});
        }
        if !epochs.is_empty(){context.storage_exits.record_map_iteration_epochs(statement.span,epochs);}
        Ok(())
    }
    #[inline(never)]
    pub(super) fn plan_map_iteration_body(&mut self,context:&mut Context,statement:&mut Stmt,results:&HashMap<usize,Type>,demand:&map_entry_demand::Demand)->Result<(),Diagnostic> {
        let mut epochs=context.storage_exits.map_iteration_epochs(statement.span).to_vec();
        if epochs.is_empty(){return Ok(());}
        let body=match &mut statement.kind {StmtKind::While{body,..}|StmtKind::Loop{body,..}|StmtKind::ForOf{body,..}=>body,_=>return Ok(())};
        let local=epochs.iter().map(|epoch|epoch.entry.place.clone()).collect();
        let mut inner=results.clone();
        // Both first and repeated bodies use the original working family.
        // Only the backedge storage excludes released environments.
        for epoch in &epochs {
            let root=inner[&epoch.entry.place.binding];
            let working=self.scoped_array_field_type(epoch.entry.input,&epoch.entry.place.fields).expect("entry Map path");
            inner.insert(epoch.entry.place.binding,self.scoped_array_replace_field(root,&epoch.entry.place.fields,working).expect("working Map path"));
        }
        self.scoped_map_entry_epochs_with_demand(context,body,&mut inner,demand,&local)?;
        self.plan_map_iteration_body_family(context,body,&mut inner,&mut epochs)?;
        if let StmtKind::Loop{update,tail_reachable:true,..}=&mut statement.kind {
            self.scoped_map_entry_epochs_with_demand(context,update,&mut inner,demand,&local)?;
        }
        self.plan_map_iteration_controls(context,statement,&epochs)?;
        for epoch in &mut epochs{epoch.back.input=*inner.get(&epoch.entry.place.binding).expect("carried Map root");}
        if let StmtKind::Loop{condition:Some(condition),post_test:true,tail_reachable:true,..}=&statement.kind {
            // The condition runs after backedge conversion, not in the last
            // body statement's working storage representation.
            let inputs=results.iter().filter(|(_,ty)|self.contains_scoped_map_storage(**ty)).map(|(binding,ty)|(*binding,*ty)).collect();
            context.storage_exits.record_map_native_inputs(condition.span.start,inputs);
        }
        context.storage_exits.record_map_iteration_epochs(statement.span,epochs);
        Ok(())
    }
    #[inline(never)]
    fn plan_map_iteration_body_family(&mut self,context:&mut Context,body:&[Stmt],inner:&mut HashMap<usize,Type>,epochs:&mut [storage_exit_availability::MapIterationEpoch])->Result<(),Diagnostic> {
        let mut controls=Vec::new();let mut pending=body.iter().collect::<Vec<_>>();
        while let Some(statement)=pending.pop(){match &statement.kind {
            StmtKind::Block(body)=>pending.extend(body),
            StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
            StmtKind::Break|StmtKind::Continue=>controls.push(statement.span),
            _=>{},
        }}
        for epoch in epochs {
            let place=&epoch.entry.place;
            epoch.body.input=inner[&place.binding];
            let mut output=epoch.body.input;
            for span in &controls {
                let inputs=context.storage_exits.map_control_inputs(*span).expect("planned control input");
                let incoming=self.scoped_array_field_type(inputs[&place.binding],&place.fields).expect("control Map path");
                let previous=self.scoped_array_field_type(output,&place.fields).expect("body Map path");
                if previous!=incoming {
                    let joined=self.join_scoped_map_types(previous,incoming,*span)?.expect("compatible iteration Map families");
                    output=self.scoped_array_replace_field(output,&place.fields,joined).expect("joined body Map path");
                }
            }
            epoch.body.output=output;
            epoch.body.empty=self.map_iteration_empty_root(epoch.body.input,&place.fields);
            inner.insert(place.binding,output);
        }
        Ok(())
    }
    fn map_iteration_empty_root(&self,root:Type,fields:&[String])->bool {
        let Some(Type::Map(map))=self.scoped_array_field_type(root,fields) else{return false;};
        let Type::Record(record)=self.program.records[map].map_definition.as_ref().unwrap().value else{return false;};
        self.absent_record_environment(record)
    }
    fn plan_map_iteration_controls(&mut self,context:&mut Context,statement:&Stmt,epochs:&[storage_exit_availability::MapIterationEpoch])->Result<(),Diagnostic> {
        let body=match &statement.kind{StmtKind::While{body,..}|StmtKind::Loop{body,..}|StmtKind::ForOf{body,..}=>body,_=>return Ok(())};
        let mut pending=body.iter().collect::<Vec<_>>();
        while let Some(control)=pending.pop(){
            match &control.kind {
                StmtKind::Block(body)=>pending.extend(body),
                StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
                StmtKind::Try(region)=>{
                    let mut nested=region.body.iter().chain(region.catch.iter().flat_map(|handler|&handler.body)).chain(region.finally.iter().flatten()).collect::<Vec<_>>();
                    while let Some(item)=nested.pop(){match &item.kind {
                        StmtKind::Block(body)=>nested.extend(body),
                        StmtKind::If{then_body,else_body,..}=>{nested.extend(then_body);nested.extend(else_body);},
                        StmtKind::Try(region)=>{nested.extend(&region.body);if let Some(handler)=&region.catch{nested.extend(&handler.body);}nested.extend(region.finally.iter().flatten());},
                        StmtKind::Break|StmtKind::Continue if matches!(item.control_target,Some(crate::syntax::ControlTarget::Loop(id)|crate::syntax::ControlTarget::Tail(id)) if id==statement.span.start)=>context.storage_exits.record_map_cleanup_loop(statement.span.start),
                        _=>{},
                    }}
                },
                StmtKind::Break|StmtKind::Continue=>{
                    let inputs=context.storage_exits.map_control_inputs(control.span).expect("planned control input");
                    let conversions=epochs.iter().map(|epoch|storage_exit_availability::MapEntryEpoch{place:epoch.entry.place.clone(),input:inputs[&epoch.entry.place.binding],output:epoch.body.output,empty:self.map_iteration_empty_root(inputs[&epoch.entry.place.binding],&epoch.entry.place.fields),before:false}).collect();
                    context.storage_exits.record_map_iteration_control(control.span,statement.span.start,matches!(control.kind,StmtKind::Continue),conversions);
                },
                _=>{},
            }
        }
        Ok(())
    }

}
fn ordinary_body(body:&[Stmt],label:usize)->bool {
    let mut pending=body.iter().collect::<Vec<_>>();
    while let Some(statement)=pending.pop(){
        if !statement.labels.is_empty(){return false;}
        match &statement.kind {
            StmtKind::Block(body)=>pending.extend(body),
            StmtKind::Try(region)=>{
                pending.extend(&region.body);
                if let Some(handler)=&region.catch{pending.extend(&handler.body);}
                for cleanup in region.finally.iter().flatten() {
                    if let StmtKind::ForOf{body,..}=&cleanup.kind {pending.extend(body);}
                    else {pending.push(cleanup);}
                }
            },
            StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
            // Callable exits are transported by the completion region. They
            // cannot resume the loop before its owning cleanup has run.
            StmtKind::Let{..}|StmtKind::Expr(_)|StmtKind::Assign{..}|StmtKind::Return(_)|StmtKind::Throw(_)=>{},
            StmtKind::Break|StmtKind::Continue if matches!(statement.control_target,Some(crate::syntax::ControlTarget::Loop(id)|crate::syntax::ControlTarget::Tail(id)) if id==label)=>{},
            _=>return false,
        }
    }
    true
}
