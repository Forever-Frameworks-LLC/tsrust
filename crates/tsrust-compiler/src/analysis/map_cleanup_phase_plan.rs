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
        CheckedFunction{storage_exits:*self.exits.clone(),native_returns:original.native_returns.clone(), native_mutations:original.native_mutations.clone(),template:original.template,specialization:original.specialization,name:original.name.clone(),bindings,parameter_count:original.parameter_count,result:original.result,body:self.body.clone()}
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
