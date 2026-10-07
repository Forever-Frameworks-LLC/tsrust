//! Scoped conditional families preserve concrete capture lifetimes.
use super::*;
impl Analyzer {
    pub(super) fn common_borrowed_conditional_family(&mut self,left:Type,right:Type,span:Span)->Result<Option<usize>,Diagnostic> {
        let origins=|ty|match ty{Type::Closure(id)=>Some(vec![id]),Type::BorrowedCallable(id)=>self.callables.storage.origins(id).map(<[usize]>::to_vec),_=>None};
        let(Some(mut ids),Some(other))=(origins(left),origins(right)) else{return Ok(None);};
        ids.extend(other);ids.sort_unstable();ids.dedup();
        let Some(&a)=ids.iter().find(|&&id|self.closures[id].borrowed) else{return Ok(None);};
        if ids.len()<2{return Ok(None);}
        let b=*ids.iter().find(|&&id|id!=a).unwrap();
        let mut family=crate::borrowed_callable_contracts::Family::checked(&self.closures,[a,b],span)?;
        for &id in &ids{if id==a{continue;}let checked=crate::borrowed_callable_contracts::Family::checked(&self.closures,[a,id],span)?;family.invocation=family.invocation.max(checked.invocation);}
        let kind=match family.invocation {Mode::Read=>CallbackKind::Read,Mode::Consume=>CallbackKind::Once,_=>CallbackKind::Reusable};
        let minimum=family.parameters.iter().rposition(|parameter|!parameter.rest && !parameter.omission.is_omittable()).map_or(0,|index|index+1);
        let rest=family.parameters.last().is_some_and(|parameter|parameter.rest);
        let structural=self.callables.shapes.intern_rest(family.parameters.iter().map(|p|(p.ty.into(),p.mode)).collect(),family.result.into(),Some(kind),minimum,rest);
        let declaration=self.callables.callbacks.len();
        self.callables.callbacks.push(CallbackContract {structural,parameter_names:self.closures[a].parameters.iter().map(|&id|self.closures[a].function.bindings[id].name.clone()).collect(),params:family.parameters,result:family.result,kind});
        Ok(Some(self.callables.storage.intern_borrowed(structural,declaration,ids)))
    }
}
