//! Checked survivor types define local native storage lifetime boundaries.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn scoped_map_entry_epochs(&mut self, context:&mut Context, body:&mut [Stmt], mut results:HashMap<usize,Type>) -> Result<(),Diagnostic> {
        if !context.storage_exits.has_map_entry_survivors() && !context.storage_exits.has_map_completion_layouts() {return Ok(());}
        let demand=map_entry_demand::Demand::checked(body)?;
        self.scoped_map_entry_epochs_with_demand(context,body,&mut results,&demand,&std::collections::BTreeSet::new()).map(|_|())
    }
    pub(super) fn begin_map_epoch_body(&self,context:&Context,body:&[Stmt],results:&HashMap<usize,Type>,local:&mut std::collections::BTreeSet<crate::capture_loan_owners::Place>)->bool {
        if !context.storage_exits.has_map_entry_survivors() && !context.storage_exits.has_map_completion_layouts() {return false;}
        for item in body.iter() {
            if let StmtKind::Let{binding:Some(binding),value,..}=&item.kind {
                let mut pending=vec![(value,Vec::new())];
                while let Some((value,path))=pending.pop() {
                    match &value.kind {
                        ExprKind::Unary{op:UnaryOp::RecordCarrier,value,..}=>pending.push((value,path)),
                        ExprKind::Record(fields)=>for (name,value) in fields {let mut nested=path.clone();nested.push(name.text.clone());pending.push((value,nested));},
                        ExprKind::MapConstructor(arguments,_) | ExprKind::NativeConstructor(arguments,_) if arguments.is_empty() && matches!(value.ty,Some(Type::Map(_)))=>{local.insert(crate::capture_loan_owners::Place{binding:*binding,fields:path});},
                        _=>{},
                    }
                }
            }
        }
        if local.is_empty() && !context.storage_exits.has_map_completion_layouts() && !results.keys().any(|binding|context.storage_exits.map_entry_epoch_root(*binding)) {
            return false;
        }
        true
    }
    pub(super) fn prepare_map_epoch_statement(&mut self,context:&mut Context,statement:&mut Stmt,results:&mut HashMap<usize,Type>,local:&mut std::collections::BTreeSet<crate::capture_loan_owners::Place>)->Result<usize,Diagnostic> {
            let mut writes=Vec::new();
            let mut statement_end=statement.span.end;
            crate::source_walk::body(std::slice::from_mut(statement),&mut |_|Ok(()),&mut |value| {
                statement_end=statement_end.max(value.span.end);
                if let ExprKind::Call(_,arguments)=&value.kind {
                    for argument in arguments {if argument.ty.is_some_and(|ty|self.contains_scoped_map_storage(ty)) && let Some(binding)=Self::root_binding(argument){local.retain(|place|place.binding!=binding);}}
                }
                if value.target==Some(CallTarget::MapMethod(MapMethod::Set))
                    && let ExprKind::Call(callee,_) =&value.kind
                    && let ExprKind::Field(base,_) =&callee.kind
                    && let Some((_,fields))=Self::direct_fields(base)
                    && let Some(binding)=Self::root_binding(base)
                    && local.contains(&crate::capture_loan_owners::Place{binding,fields:fields.clone()})
                    && let Some(Type::Record(incoming))=context.storage_exits.map_entry_write(value.span)
                {writes.push((crate::capture_loan_owners::Place{binding,fields},incoming,value.span));}
                Ok(())
            })?;
            self.plan_map_iteration_entry(context,statement,results,local)?;
            // Conditional and loop storage include incoming payloads before entry,
            // retaining the untaken-arm or zero-iteration survivor.
            if !matches!(statement.kind,StmtKind::Try(_)) {
                for (place,incoming,span) in writes {
                    if context.storage_exits.map_iteration_epochs(statement.span).iter().any(|epoch|epoch.entry.place==place){continue;}
                    let Some(root)=results.get(&place.binding).copied() else {continue;};
                    let Some(Type::Map(input))=self.scoped_array_field_type(root,&place.fields) else {continue;};
                    let Type::Record(previous)=self.program.records[input].map_definition.as_ref().unwrap().value else {continue;};
                    let empty=self.absent_record_environment(previous);
                    let payload=if empty || previous==incoming {Type::Record(incoming)} else {
                        let Some(joined)=self.join_record_types(previous,incoming,span)? else {continue;};joined
                    };
                    if payload==Type::Record(previous) {continue;}
                    let output=self.scoped_map_value_instance(input,payload);
                    if input!=output {
                        let output=self.scoped_array_replace_field(root,&place.fields,Type::Map(output)).expect("checked Map field");
                        // Several writes share one pre-statement conversion. Its
                        // source must remain the original storage representation.
                        let prior=context.storage_exits.map_entry_epochs(statement.span).iter().find(|epoch|epoch.before && epoch.place==place);
                        let (input,empty)=prior.map(|epoch|(epoch.input,epoch.empty)).unwrap_or((root,empty));
                        context.storage_exits.record_map_entry_epoch(statement.span,storage_exit_availability::MapEntryEpoch{place:place.clone(),input,output,empty,before:true});
                        results.insert(place.binding,output);
                    }
                }
            }
            if let StmtKind::Let{binding:Some(binding),value,..}=&statement.kind
                && let Some(ty)=self.planned_scoped_map_result(value,results)
            {results.insert(*binding,ty);context.bindings[*binding].ty=ty;}
            let native_inputs=self.scoped_map_storage_inputs(results);
            context.storage_exits.record_map_native_inputs(statement.span.start,native_inputs);
            self.empty_map_iteration_carriers(context,std::slice::from_ref(statement),results)?;
            let field_initializer=matches!(&statement.kind,StmtKind::Let{binding:Some(binding),..} if local.iter().any(|place|place.binding==*binding && !place.fields.is_empty()));
            if !field_initializer {self.retarget_scoped_map_body(context,std::slice::from_mut(statement),results,&HashMap::new())?;}
        Ok(statement_end)
    }
    pub(super) fn finish_map_epoch_statement(&mut self,context:&mut Context,statement:&mut Stmt,statement_end:usize,results:&mut HashMap<usize,Type>,local:&mut std::collections::BTreeSet<crate::capture_loan_owners::Place>,demand:&map_entry_demand::Demand)->Result<(),Diagnostic> {
            let mut snapshots=Vec::new();
            crate::source_walk::body(std::slice::from_mut(statement),&mut |_|Ok(()),&mut |value| {
                // A callee may write entries not represented by local set sites.
                // Its storage effects need an interprocedural proof before a
                // local snapshot can authorize environment elimination.
                if let ExprKind::Call(_,arguments)=&value.kind {
                    for argument in arguments {
                        if argument.ty.is_some_and(|ty|self.contains_scoped_map_storage(ty)) && let Some(binding)=Self::root_binding(argument) {local.retain(|place|place.binding!=binding);}
                    }
                }
                if matches!(value.target,Some(CallTarget::MapMethod(MapMethod::Delete|MapMethod::Set)))
                    && let Some(state)=context.storage_exits.map_entry_survivors(value.span)
                    && state.consistent && local.contains(&state.place)
                {snapshots.push((value.span,state.clone()));}
                Ok(())
            })?;
            // Nested controls have distinct reaching states and need their own
            // return transport; do not apply their deletion to the parent scope.
            if matches!(statement.kind,StmtKind::If{..}|StmtKind::While{..}|StmtKind::Loop{..}|StmtKind::ForOf{..}) {
                snapshots.clear();
                for state in context.storage_exits.map_completion_survivors(statement.span) {
                    if state.consistent && local.contains(&state.place) {snapshots.push((statement.span,state.clone()));}
                }
            } else if matches!(statement.kind,StmtKind::Try(_)) {return Ok(());}
            snapshots.sort_by_key(|(span,_)|span.start);
            for (span,state) in snapshots {
                if !demand.after(statement_end,&state.released) {continue;}
                let Some(root)=results.get(&state.place.binding).copied() else {continue;};
                let Some(Type::Map(input))=self.scoped_array_field_type(root,&state.place.fields) else {continue;};
                if state.payloads.is_empty() {
                    let Type::Record(record)=self.program.records[input].map_definition.as_ref().unwrap().value else {continue;};
                    let declaration=self.scoped_records.declaration(record);
                    let payload=self.empty_scoped_record_environment(declaration)?;
                    let output=self.scoped_map_value_instance(input,payload);
                    if input!=output {
                        context.storage_exits.record_map_entry_epoch(statement.span,storage_exit_availability::MapEntryEpoch{place:state.place.clone(),input:root,output:self.scoped_array_replace_field(root,&state.place.fields,Type::Map(output)).expect("checked Map field"),empty:true,before:false});
                        results.insert(state.place.binding,self.scoped_array_replace_field(root,&state.place.fields,Type::Map(output)).expect("checked Map field"));
                    }
                    continue;
                }
                let mut records=state.payloads.iter().filter_map(|ty|match ty {Type::Record(id)=>Some(*id),_=>None}).collect::<Vec<_>>();
                if records.len()!=state.payloads.len() {continue;}
                records.sort_unstable();
                let mut common=Some(records[0]);
                for record in records.into_iter().skip(1) {
                    let payload=common.unwrap();
                    if payload!=record {
                        let Some(Type::Record(joined))=self.join_record_types(payload,record,span)? else {common=None;break;};
                        common=Some(joined);
                    }
                }
                let Some(payload)=common else {continue;};
                let output=self.scoped_map_value_instance(input,Type::Record(payload));
                if input==output {continue;}
                context.storage_exits.record_map_entry_epoch(statement.span,storage_exit_availability::MapEntryEpoch{place:state.place.clone(),input:root,output:self.scoped_array_replace_field(root,&state.place.fields,Type::Map(output)).expect("checked Map field"),empty:false,before:false});
                results.insert(state.place.binding,self.scoped_array_replace_field(root,&state.place.fields,Type::Map(output)).expect("checked Map field"));
            }
            let inputs=self.scoped_map_storage_inputs(results);
            context.storage_exits.record_map_native_inputs(statement_end,inputs);
        Ok(())
    }
    pub(super) fn absent_record_environment(&self, record:usize) -> bool {
        let mut pending=vec![Type::Record(record)];
        while let Some(ty)=pending.pop() {
            match ty {
                Type::Record(id)=>pending.extend(self.program.records[id].fields.iter().filter_map(|field|field.ty)),
                Type::Callback(id) if self.absent_callable_type(Type::Callback(id))=>return true,
                _=>{},
            }
        }
        false
    }
}
