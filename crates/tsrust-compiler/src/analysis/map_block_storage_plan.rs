//! Checked owner transport after a labelled lexical block finishes.
use super::*;
impl Analyzer {
    pub(super) fn finish_map_block_storage(&mut self,context:&mut Context,statement:&Stmt,results:&mut HashMap<usize,Type>,inner:&HashMap<usize,Type>,retained:&std::collections::BTreeSet<crate::capture_loan_owners::Place>,states:&[storage_exit_availability::MapEntrySurvivors],eligible:&[crate::capture_loan_owners::Place])->Result<(),Diagnostic> {
        let StmtKind::Block(body)=&statement.kind else{unreachable!()};
        let mut epochs=Vec::new();
        for state in states.iter().filter(|state|eligible.contains(&state.place) && retained.contains(&state.place)) {
            let place=&state.place;
            let Some(root)=results.get(&place.binding).copied() else{continue;};
            let Some(Type::Map(map))=self.scoped_array_field_type(root,&place.fields) else{continue;};
            let payload=if state.payloads.is_empty() {
                let Type::Record(record)=self.program.records[map].map_definition.as_ref().unwrap().value else{continue;};
                self.empty_scoped_record_environment(self.scoped_records.declaration(record))?
            }else{
                let mut records=state.payloads.iter().filter_map(|ty|if let Type::Record(record)=ty{Some(*record)}else{None}).collect::<Vec<_>>();
                if records.len()!=state.payloads.len(){continue;}
                records.sort_unstable();
                let mut payload=Type::Record(records[0]);
                for record in records.into_iter().skip(1){let Type::Record(previous)=payload else{unreachable!()};let Some(joined)=self.join_record_types(previous,record,statement.span)? else{return Ok(());};payload=joined;}
                payload
            };
            let output_map=self.scoped_map_value_instance(map,payload);
            let output=self.scoped_array_replace_field(root,&place.fields,Type::Map(output_map)).expect("checked block Map path");
            epochs.push(storage_exit_availability::MapEntryEpoch{place:place.clone(),input:inner[&place.binding],output,empty:state.payloads.is_empty(),before:false});
            results.insert(place.binding,output);
        }
        let mut pending=body.iter().collect::<Vec<_>>();
        while let Some(control)=pending.pop(){match &control.kind {
            StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
            StmtKind::Loop{body,update,tail_reachable,..}=>{pending.extend(body);if *tail_reachable{pending.extend(update);}},
            StmtKind::If{then_body,else_body,..}=>pending.extend(then_body.iter().chain(else_body)),
            // Try controls must finish their completion transport first.
            StmtKind::Break if control.control_target==Some(crate::syntax::ControlTarget::Block(statement.span.start))=>{
                let inputs=context.storage_exits.map_control_inputs(control.span).expect("planned block control input");
                let conversions=epochs.iter().map(|epoch|storage_exit_availability::MapEntryEpoch{input:inputs[&epoch.place.binding],..epoch.clone()}).collect();
                context.storage_exits.record_map_iteration_control(control.span,statement.span.start,false,conversions);
            },
            _=>{},
        }}
        let inputs=results.iter().filter(|(_,ty)|self.contains_scoped_map_storage(**ty)).map(|(binding,ty)|(*binding,*ty)).collect();
        context.storage_exits.record_map_native_inputs(statement.span.end,inputs);
        context.storage_exits.record_map_block_epochs(statement.span.start,epochs);
        Ok(())
    }
}
