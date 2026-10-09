//! Scoped conditional families preserve concrete capture lifetimes.
use super::*;
impl Analyzer {
    pub(super) fn common_borrowed_conditional_family(&mut self,left:Type,right:Type,span:Span)->Result<Option<usize>,Diagnostic> {
        self.callback_family(left,right,span,false)
    }
    /// The family of a callback field of a record. Unlike a bare callable, it
    /// also joins environments that own every capture.
    pub(super) fn record_field_callback_family(&mut self,left:Type,right:Type,span:Span)->Result<Option<usize>,Diagnostic> {
        self.callback_family(left,right,span,true)
    }
    fn callback_family(&mut self,left:Type,right:Type,span:Span,owned:bool)->Result<Option<usize>,Diagnostic> {
        let origins=|ty|match ty{Type::Closure(id)=>Some(vec![id]),Type::BorrowedCallable(id)=>self.callables.storage.origins(id).map(<[usize]>::to_vec),_=>None};
        let(Some(mut ids),Some(other))=(origins(left),origins(right)) else{return Ok(None);};
        ids.extend(other);ids.sort_unstable();ids.dedup();
        if ids.len()<2{return Ok(None);}
        let a=match ids.iter().copied().find(|&id|self.closures[id].borrowed) {
            Some(a)=>a,
            None if owned && ids.iter().all(|&id|!self.closures[id].polymorphic && !self.closures[id].function.template && !self.deferred_generic_values.contains_key(&id))=>ids[0],
            None=>return Ok(None),
        };
        let b=*ids.iter().find(|&&id|id!=a).unwrap();
        let checked=|closures:&[CheckedClosure],origins:[usize;2]|if owned{crate::borrowed_callable_contracts::Family::checked_for_record_field(closures,origins,span)}else{crate::borrowed_callable_contracts::Family::checked(closures,origins,span)};
        let mut family=checked(&self.closures,[a,b])?;
        for &id in &ids{if id==a{continue;}let checked=checked(&self.closures,[a,id])?;family.invocation=family.invocation.max(checked.invocation);}
        let kind=match family.invocation {Mode::Read=>CallbackKind::Read,Mode::Consume=>CallbackKind::Once,_=>CallbackKind::Reusable};
        let minimum=family.parameters.iter().rposition(|parameter|!parameter.rest && !parameter.omission.is_omittable()).map_or(0,|index|index+1);
        let rest=family.parameters.last().is_some_and(|parameter|parameter.rest);
        let structural=self.callables.shapes.intern_rest(family.parameters.iter().map(|p|(p.ty.into(),p.mode)).collect(),family.result.into(),Some(kind),minimum,rest);
        let declaration=self.callables.callbacks.len();
        self.callables.callbacks.push(CallbackContract {structural,parameter_names:self.closures[a].parameters.iter().map(|&id|self.closures[a].function.bindings[id].name.clone()).collect(),params:family.parameters,result:family.result,kind});
        Ok(Some(self.callables.storage.intern_borrowed(structural,declaration,ids)))
    }
}
