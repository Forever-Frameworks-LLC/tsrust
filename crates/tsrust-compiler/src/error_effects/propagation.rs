//! Per-solve body reuse; caller contracts and checked bodies never survive a solve.
use super::*;

/// One contract entry a body reads. Bodies re-evaluate only when an entry they
/// actually read has grown, rather than when any entry of its category changed.
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub(super) enum Key {Function(usize),Closure(usize),Storage(usize),Callback(usize),Method(usize,usize),Invocation(usize)}

/// Per-entry growth counters for one solve; absent entries are at version 0.
#[derive(Default)]
pub(super) struct Versions(crate::fast_hash::FastMap<Key,u64>);
impl Versions {
    fn get(&self,key:Key)->u64 {self.0.get(&key).copied().unwrap_or(0)}
    pub(super) fn bump(&mut self,key:Key) {*self.0.entry(key).or_insert(0)+=1;}
}

pub(super) struct Reader<'a> {
    effects:&'a ErrorEffects,
    reads:std::cell::RefCell<Vec<Key>>,
}
impl<'a> Reader<'a> {
    pub(super) fn new(effects:&'a ErrorEffects)->Self {Self{effects,reads:Default::default()}}
    // All body contract lookups pass through this reader; ErrorEffects stays Send + Sync.
    pub(super) fn read(&self,key:Key,value:Option<&FailureSet>)->FailureSet {
        self.reads.borrow_mut().push(key);
        value.cloned().unwrap_or_else(FailureSet::unknown)
    }
}
impl std::ops::Deref for Reader<'_> {
    type Target=ErrorEffects;
    fn deref(&self)->&ErrorEffects {self.effects}
}

#[derive(Default)]
pub(super) struct Cache {
    observed:Vec<(Key,u64)>,
    failure:Option<FailureSet>,
    #[cfg(test)] evaluations:usize,
    traced:usize,
}
impl Cache {
    pub(super) fn effect(&mut self,function:&mut CheckedFunction,effects:&ErrorEffects,versions:&Versions)->FailureSet {
        if self.failure.is_none() || self.observed.iter().any(|(key,version)|versions.get(*key)!=*version) {
            let reader=Reader::new(effects);
            self.failure=Some(function_effect_with(function,&reader));
            let mut keys=reader.reads.into_inner();keys.sort_unstable_by_key(|key|format_key(*key));keys.dedup();
            self.observed=keys.into_iter().map(|key|(key,versions.get(key))).collect();
            #[cfg(test)] {self.evaluations+=1;}
            self.traced+=1;
        }
        self.failure.as_ref().unwrap().clone()
    }
}
fn format_key(key:Key)->(u8,usize,usize) {
    match key {Key::Function(id)=>(0,id,0),Key::Closure(id)=>(1,id,0),Key::Storage(id)=>(2,id,0),Key::Callback(id)=>(3,id,0),Key::Method(id,method)=>(4,id,method),Key::Invocation(id)=>(5,id,0)}
}
impl Cache {pub(super) fn evaluations_traced(&self)->usize {self.traced}}
/// Record growth of one entry; returns `changed` for fixpoint bookkeeping.
pub(super) fn grow(versions:&mut Versions,key:Key,changed:bool)->bool {
    if changed {versions.bump(key);}
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
    #[test] fn body_cache_rechecks_only_changed_read_entries() {
        let source=crate::format("function leaf():int{return 1;}function main(){leaf();}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let mut program=crate::check(&source).unwrap().program;
        let main=program.functions.iter().position(|function|function.name=="main").unwrap();
        let leaf=program.functions.iter().position(|function|function.name=="leaf").unwrap();
        let mut effects=ErrorEffects{function_errors:vec![FailureSet::default();program.functions.len()],..Default::default()};
        let mut versions=Versions::default();let mut cache=Cache::default();
        assert_eq!(cache.effect(&mut program.functions[main],&effects,&versions),FailureSet::default());
        grow(&mut versions,Key::Closure(0),true);
        grow(&mut versions,Key::Function(main),true);
        assert_eq!(cache.effect(&mut program.functions[main],&effects,&versions),FailureSet::default());
        assert_eq!(cache.evaluations,1);
        effects.function_errors[leaf]=FailureSet::runtime();grow(&mut versions,Key::Function(leaf),true);
        assert_eq!(cache.effect(&mut program.functions[main],&effects,&versions),FailureSet::runtime());
        assert_eq!(cache.evaluations,2);
    }
}
