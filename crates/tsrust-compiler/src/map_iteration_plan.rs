//! Carry proven surviving Map environments across normally completing iterations.
use super::*;
impl Analyzer {
    #[inline(never)]
    pub(super) fn plan_map_iteration_entry(&mut self,context:&mut Context,statement:&Stmt,results:&mut HashMap<usize,Type>,local:&std::collections::BTreeSet<crate::capture_loan_owners::Place>)->Result<(),Diagnostic> {
        let (body,post_test)=match &statement.kind {
            StmtKind::While{body,..}=>(body,false),
            StmtKind::Loop{condition:Some(_),body,update,post_test,tail_reachable:true,..} if ordinary_body(update)=>(body,*post_test),
            StmtKind::ForOf{body,..}=>(body,false),
            _=>return Ok(()),
        };
        if !statement.labels.is_empty() || !ordinary_body(body){return Ok(());}
        let mut epochs=Vec::new();
        for place in local {
            if !context.storage_exits.map_completion_survivors(statement.span).iter().any(|state|state.consistent && state.place==*place && !state.released.is_empty()){continue;}
            let entry=context.storage_exits.map_loop_edge(statement.span,storage_exit_availability::MapLoopEdge::Entry,place);
            let back=context.storage_exits.map_loop_edge(statement.span,storage_exit_availability::MapLoopEdge::Back,place);
            if !context.storage_exits.has_map_loop_edge(statement.span,storage_exit_availability::MapLoopEdge::Back){continue;}
            if !post_test && entry!=back {continue;}
            let empty=back.is_none_or(HashSet::is_empty);
            let Some(input)=results.get(&place.binding).copied() else{continue;};
            let Some(Type::Map(map))=self.scoped_array_field_type(input,&place.fields) else{continue;};
            let payload=if empty {
                let Type::Record(record)=self.program.records[map].map_definition.as_ref().unwrap().value else{continue;};
                self.empty_scoped_record_environment(self.scoped_records.declaration(record))?
            }else{
                let back=back.unwrap();
                let mut records=back.iter().filter_map(|ty|if let Type::Record(record)=ty{Some(*record)}else{None}).collect::<Vec<_>>();
                if records.len()!=back.len(){continue;}
                records.sort_unstable();
                let mut payload=Type::Record(records[0]);
                for record in records.into_iter().skip(1){let Type::Record(previous)=payload else{unreachable!()};let Some(joined)=self.join_record_types(previous,record,statement.span)? else{return Ok(());};payload=joined;}
                payload
            };
            let output_map=self.scoped_map_value_instance(map,payload);
            let output=self.scoped_array_replace_field(input,&place.fields,Type::Map(output_map)).expect("checked Map path");
            if input==output{continue;}
            let entry=storage_exit_availability::MapEntryEpoch{place:place.clone(),input,output,empty,before:true};
            if !post_test {context.storage_exits.record_map_entry_epoch(statement.span,entry.clone());}
            results.insert(place.binding,output);
            let back=storage_exit_availability::MapEntryEpoch{place:place.clone(),input:output,output,empty,before:false};
            epochs.push(storage_exit_availability::MapIterationEpoch{entry,back});
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
        if let StmtKind::Loop{update,..}=&mut statement.kind {
            self.scoped_map_entry_epochs_with_demand(context,update,&mut inner,demand,&local)?;
        }
        for epoch in &mut epochs{epoch.back.input=*inner.get(&epoch.entry.place.binding).expect("carried Map root");}
        context.storage_exits.record_map_iteration_epochs(statement.span,epochs);
        Ok(())
    }
}
fn ordinary_body(body:&[Stmt])->bool {
    let mut pending=body.iter().collect::<Vec<_>>();
    while let Some(statement)=pending.pop(){
        if !statement.labels.is_empty(){return false;}
        match &statement.kind {
            StmtKind::Block(body)=>pending.extend(body),
            StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
            StmtKind::Let{..}|StmtKind::Expr(_)|StmtKind::Assign{..}=>{},
            _=>return false,
        }
    }
    true
}
