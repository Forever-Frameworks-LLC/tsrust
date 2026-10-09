//! Materialize distinct Map payload families for each exception completion.
use super::*;
impl Analyzer {
    #[inline(never)]
    pub(super) fn plan_map_completion_layouts(&mut self,context:&mut Context,boundary:usize)->Result<(),Diagnostic> {
        let snapshots=context.storage_exits.map_completion_snapshots(boundary);
        let places=snapshots.iter().flat_map(|state|state.payloads.keys().cloned()).collect::<std::collections::BTreeSet<_>>();
        for state in snapshots {
            let mut types=std::collections::BTreeMap::new();
            for place in &places {
                let Some(binding)=context.bindings.get(place.binding) else{continue;};
                let Some(Type::Map(map))=self.scoped_array_field_type(binding.ty,&place.fields) else{continue;};
                let Some(definition)=self.program.records[map].map_definition.as_ref() else{continue;};
                let Type::Record(original)=definition.value else{continue;};
                let payloads=state.payloads.get(place);
                let payload=if payloads.is_none_or(HashSet::is_empty) {
                    self.empty_scoped_record_environment(self.scoped_records.declaration(original))?
                }else {
                    let payloads=payloads.unwrap();
                    let mut records=payloads.iter().filter_map(|ty|if let Type::Record(id)=ty{Some(*id)}else{None}).collect::<Vec<_>>();
                    if records.len()!=payloads.len(){continue;}
                    records.sort_unstable();
                    let mut joined=Some(Type::Record(records[0]));
                    for record in records.into_iter().skip(1) {
                        let Type::Record(previous)=joined.unwrap() else{unreachable!()};
                        joined=self.join_record_types(previous,record,crate::Span::new(boundary,boundary))?;
                        if joined.is_none(){break;}
                    }
                    let Some(payload)=joined else{continue;};payload
                };
                let output=self.scoped_map_value_instance(map,payload);
                types.insert(place.clone(),Type::Map(output));
            }
            context.storage_exits.record_map_completion_layout(boundary,state.phase,state.kind,types);
        }
        Ok(())
    }
}
