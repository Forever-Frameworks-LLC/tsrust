//! Pending completions own independent cleanup native storage plans.
use super::*;
#[derive(Clone,Debug)]
pub(crate) struct Plan {
    pub(crate) body:Vec<Stmt>,
    pub(crate) binding_types:Vec<(usize,Type)>,
    pub(crate) exits:Box<storage_exit_availability::Exits>,
}
#[derive(Clone,Debug,Default)]
pub(crate) struct Plans(std::collections::BTreeMap<usize,Vec<(Option<crate::completion::CompletionKind>,Plan)>>);
impl Plan {
    pub(crate) fn function(&self,original:&CheckedFunction)->CheckedFunction {
        let mut bindings=original.bindings.clone();
        for &(binding,ty) in &self.binding_types{bindings[binding].ty=ty;}
        CheckedFunction{storage_exits:*self.exits.clone(),template:original.template,specialization:original.specialization,name:original.name.clone(),bindings,parameter_count:original.parameter_count,result:original.result,body:self.body.clone()}
    }
}
impl Plans {
    pub(super) fn insert(&mut self,boundary:usize,kind:Option<crate::completion::CompletionKind>,plan:Plan) {
        let plans=self.0.entry(boundary).or_default();
        if let Some((_,previous))=plans.iter_mut().find(|(pending,_)|*pending==kind){*previous=plan;}else{plans.push((kind,plan));}
    }

    pub(crate) fn get(&self,boundary:usize,kind:Option<crate::completion::CompletionKind>)->Option<&Plan> {
        self.0.get(&boundary)?.iter().find(|(pending,_)|*pending==kind).map(|(_,plan)|plan)
    }
}

impl Analyzer {
    #[inline(never)]
    pub(super) fn plan_map_cleanup_phases(&mut self,context:&mut Context,region:&ExceptionRegion,results:&HashMap<usize,Type>,demand:&map_entry_demand::Demand,local:&std::collections::BTreeSet<crate::capture_loan_owners::Place>)->Result<(),Diagnostic> {
        let (Some(cleanup),Some(boundary))=(&region.finally,region.body.first().map(|statement|statement.span.start)) else{return Ok(());};
        let snapshots=context.storage_exits.map_completion_snapshots(boundary);
        for state in snapshots.into_iter().filter(|state|state.phase==map_completion_payloads::Phase::Handled && state.kind.is_some()) {
            let Some(layout)=context.storage_exits.map_completion_layout(boundary,state.phase,state.kind).cloned() else{continue;};
            if layout.is_empty(){continue;}
            let mut phase=context.clone();phase.storage_exits.reset_map_cleanup_native_plans();
            let mut input=results.clone();
            for (place,leaf) in layout {
                if let Some(root)=input.get(&place.binding).copied() && let Some(output)=self.scoped_array_replace_field(root,&place.fields,leaf){input.insert(place.binding,output);}
            }
            let mut body=cleanup.clone();
            self.scoped_map_entry_epochs_with_demand(&mut phase,&mut body,&mut input,demand,local)?;
            let binding_types=phase.bindings.iter().zip(&context.bindings).enumerate().filter_map(|(binding,(phase,original))|(phase.ty!=original.ty).then_some((binding,phase.ty))).collect();
            context.storage_exits.record_map_cleanup_plan(boundary,state.kind,Plan{body,binding_types,exits:Box::new(phase.storage_exits)});
        }
        Ok(())
    }
}
