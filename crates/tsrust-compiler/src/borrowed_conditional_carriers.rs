//! Scoped conditional families preserve concrete capture lifetimes.
use super::*;
impl Analyzer {
    pub(super) fn common_borrowed_conditional_family(&mut self,left:Type,right:Type,span:Span)->Result<Option<usize>,Diagnostic> {
        let (Type::Closure(a),Type::Closure(b))=(left,right) else {return Ok(None);};
        if !self.closures[a].borrowed && !self.closures[b].borrowed {return Ok(None);}
        let family=crate::borrowed_callable_contracts::Family::checked(&self.closures,[a,b],span)?;
        let kind=match family.invocation {Mode::Read=>CallbackKind::Read,Mode::Consume=>CallbackKind::Once,_=>CallbackKind::Reusable};
        let minimum=family.parameters.iter().rposition(|parameter|!parameter.rest && !parameter.omission.is_omittable()).map_or(0,|index|index+1);
        let rest=family.parameters.last().is_some_and(|parameter|parameter.rest);
        let structural=self.callables.shapes.intern_rest(family.parameters.iter().map(|p|(p.ty.into(),p.mode)).collect(),family.result.into(),Some(kind),minimum,rest);
        let declaration=self.callables.callbacks.len();
        self.callables.callbacks.push(CallbackContract {structural,parameter_names:self.closures[a].parameters.iter().map(|&id|self.closures[a].function.bindings[id].name.clone()).collect(),params:family.parameters,result:family.result,kind});
        Ok(Some(self.callables.storage.intern_borrowed(structural,declaration,family.origins)))
    }
}
