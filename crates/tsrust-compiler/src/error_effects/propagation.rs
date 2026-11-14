//! Per-solve body reuse; caller contracts and checked bodies never survive a solve.
use super::*;

pub(super) const FUNCTION:usize=0;
pub(super) const CLOSURE:usize=1;
pub(super) const STORAGE:usize=2;
pub(super) const CALLBACK:usize=3;
pub(super) const METHOD:usize=4;
pub(super) const INVOCATION:usize=5;
pub(super) type Versions=[u64;6];

pub(super) struct Reader<'a> {
    effects:&'a ErrorEffects,
    reads:std::cell::Cell<u8>,
}
impl<'a> Reader<'a> {
    pub(super) fn new(effects:&'a ErrorEffects)->Self {Self{effects,reads:std::cell::Cell::new(0)}}
    // All body contract lookups pass through this reader; ErrorEffects stays Send + Sync.
    pub(super) fn read(&self,category:usize,value:Option<&FailureSet>)->FailureSet {
        self.reads.set(self.reads.get() | (1 << category));
        value.cloned().unwrap_or_else(FailureSet::unknown)
    }
}
impl std::ops::Deref for Reader<'_> {
    type Target=ErrorEffects;
    fn deref(&self)->&ErrorEffects {self.effects}
}

#[derive(Default)]
pub(super) struct Cache {
    dependencies:u8,
    observed:Versions,
    failure:Option<FailureSet>,
    #[cfg(test)] evaluations:usize,
}
impl Cache {
    pub(super) fn effect(&mut self,function:&mut CheckedFunction,effects:&ErrorEffects,versions:&Versions)->FailureSet {
        if self.failure.is_none() || versions.iter().enumerate().any(|(category,value)|self.dependencies & (1 << category)!=0 && *value!=self.observed[category]) {
            let reader=Reader::new(effects);
            self.failure=Some(function_effect_with(function,&reader));
            self.dependencies=reader.reads.get();
            self.observed=*versions;
            #[cfg(test)] {self.evaluations+=1;}
        }
        self.failure.as_ref().unwrap().clone()
    }
}
pub(super) fn grow(versions:&mut Versions,category:usize,changed:bool)->bool {
    if changed {versions[category]+=1;}
    changed
}

#[cfg(test)]
pub(super) fn annotations(function:&CheckedFunction)->(Vec<(crate::Span,bool,bool,bool)>,Vec<(crate::Span,Option<Box<FailureSet>>)>) {
    let expressions=checked_expressions(function).map(|value|(value.span,value.own_may_fail,value.may_fail,value.effects_checked)).collect();
    let mut catches=Vec::new();
    let mut pending=function.body.iter().chain(checked_expressions(function).filter_map(|value|if let ExprKind::Match{body,..}=&value.kind{Some(body)}else{None}).flatten()).collect::<Vec<_>>();
    while let Some(statement)=pending.pop() {
        match &statement.kind {
            StmtKind::Try(region)=>{
                if let Some(handler)=&region.catch {catches.push((statement.span,handler.error_contract.clone()));}
                pending.extend(region.statements());
            }
            StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
            StmtKind::If{then_body,else_body,..}=>pending.extend(then_body.iter().chain(else_body)),
            StmtKind::Loop{body,update,..}=>pending.extend(body.iter().chain(update)),
            _=>{},
        }
    }
    (expressions,catches)
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn body_cache_rechecks_only_changed_contract_categories() {
        let source=crate::format("function leaf():int{return 1;}function main(){leaf();}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let mut program=crate::check(&source).unwrap().program;
        let main=program.functions.iter().position(|function|function.name=="main").unwrap();
        let leaf=program.functions.iter().position(|function|function.name=="leaf").unwrap();
        let mut effects=ErrorEffects{function_errors:vec![FailureSet::default();program.functions.len()],..Default::default()};
        let mut versions=Versions::default();let mut cache=Cache::default();
        assert_eq!(cache.effect(&mut program.functions[main],&effects,&versions),FailureSet::default());
        grow(&mut versions,CLOSURE,true);
        assert_eq!(cache.effect(&mut program.functions[main],&effects,&versions),FailureSet::default());
        assert_eq!(cache.evaluations,1);
        effects.function_errors[leaf]=FailureSet::runtime();grow(&mut versions,FUNCTION,true);
        assert_eq!(cache.effect(&mut program.functions[main],&effects,&versions),FailureSet::runtime());
        assert_eq!(cache.evaluations,2);
    }
}
