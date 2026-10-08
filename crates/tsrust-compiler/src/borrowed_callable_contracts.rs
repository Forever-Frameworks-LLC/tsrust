//! Checked borrowed-family metadata keeps environment origins separate.
use crate::analysis::{ArgumentContract,CheckedClosure};
use crate::{Diagnostic,Mode,Span};
use crate::syntax::Type;

pub(crate) struct Family {
    #[cfg(test)]
    pub origins:[usize;2],
    pub parameters:Vec<ArgumentContract>,
    pub result:Type,
    pub invocation:Mode,
}
impl Family {
    pub(crate) fn checked(closures:&[CheckedClosure],origins:[usize;2],span:Span)->Result<Self,Diagnostic> {
        let [left,right]=origins.map(|id|closures.get(id));
        let (Some(left),Some(right))=(left,right) else {
            return Err(Diagnostic::new("borrowed-callable-layout","A borrowed family requires two known concrete environment origins.",span));
        };
        if [left,right].iter().any(|closure|closure.polymorphic || closure.function.template) {
            return Err(Diagnostic::new("borrowed-callable-layout","A borrowed family requires concrete monomorphic invocation bodies.",span));
        }
        if !left.borrowed && !right.borrowed {
            return Err(Diagnostic::new("borrowed-callable-layout","Owned environments use owned callable storage, not a borrowed family.",span));
        }
        if [left,right].iter().any(|closure|!matches!(closure.invocation,Mode::Read|Mode::Mutate|Mode::Consume)) {
            return Err(Diagnostic::new("callback-access","Borrowed dispatch requires readonly, mutable or consuming invocation access.",span));
        }
        let parameters=|closure:&CheckedClosure|closure.parameters.iter().map(|&id|ArgumentContract::from(&closure.function.bindings[id])).collect::<Vec<_>>();
        let parameters=parameters(left);
        let other=right.parameters.iter().map(|&id|ArgumentContract::from(&right.function.bindings[id])).collect::<Vec<_>>();
        let compatible=parameters.len()==other.len() && parameters.iter().zip(&other).all(|(a,b)|
            (a.ty,a.mode,a.omission.is_omittable(),a.rest)==(b.ty,b.mode,b.omission.is_omittable(),b.rest));
        if !compatible || left.function.result!=right.function.result {
            return Err(Diagnostic::new("type","Borrowed callable alternatives require matching argument types, access, cardinality and result types.",span)
                .related(left.span,"First alternative is created here.").related(right.span,"Second alternative is created here."));
        }
        Ok(Self {#[cfg(test)] origins,parameters,result:left.function.result,invocation:if left.invocation==Mode::Consume || right.invocation==Mode::Consume {Mode::Consume}else if left.invocation==Mode::Mutate || right.invocation==Mode::Mutate {Mode::Mutate}else{Mode::Read}})
    }
    #[cfg(test)]
    pub(crate) fn remap(&mut self,ids:&[Option<usize>],span:Span)->Result<(),Diagnostic> {
        let mapped=self.origins.map(|id|ids.get(id).copied().flatten());
        let [Some(left),Some(right)]=mapped else {
            return Err(Diagnostic::new("borrowed-callable-layout","A borrowed family lost a concrete environment during compaction.",span));
        };
        if self.origins[0]!=self.origins[1] && left==right {
            return Err(Diagnostic::new("borrowed-callable-layout","Distinct borrowed environments cannot merge during compaction.",span));
        }
        self.origins=[left,right];
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    fn source()->crate::Analysis {
        crate::check("function main(){const left=[1];const right=[2,3];const first=(value:int):int=>left.length+value;const second=(value:int):int=>right.length+value;console.log(first(1),second(2));}").unwrap()
    }
    #[test]
    fn checked_borrowed_family_retains_contract_and_two_actual_origins() {
        let checked=source();
        let family=Family::checked(&checked.program.closures,[0,1],Span::default()).unwrap();
        assert_eq!(family.origins,[0,1]);
        assert_eq!(family.result,Type::Int);
        assert_eq!(family.invocation,Mode::Read);
        assert_eq!(family.parameters.len(),1);
        assert_eq!(family.parameters[0].ty,Type::Int);
    }
    #[test]
    fn checked_borrowed_family_joins_actual_mutation_access() {
        let checked=crate::check("function main(){let left=0;let right=10;const first=():int=>{left+=1;return left;};const second=():int=>{right+=2;return right;};console.log(first(),second());}").unwrap();
        let family=Family::checked(&checked.program.closures,[0,1],Span::default()).unwrap();
        assert_eq!(family.invocation,Mode::Mutate);
        assert!(checked.program.closures.iter().all(|closure|closure.borrowed));
    }

    #[test]
    fn borrowed_family_rejects_incompatible_arguments_access_and_results() {
        for source in [
            "function main(){const left=[1];const right=[2];const first=(x:int):int=>left.length+x;const second=(x:int,y:int):int=>right.length+x+y;console.log(first(1),second(1,2));}",
            "function main(){const left=[1];const right=[2];const first=():int=>left.length;const second=():boolean=>right.length>0;console.log(first(),second());}",
            "function main(){const left=[1];const right=[2];const first=(x:int[]):int=>left.length+x.length;const second=(x:int[]):int=>{x.push(3);return right.length+x.length;};const a=[1];const b=[2];console.log(first(a),second(b));}",
        ] {
            let checked=crate::check(source).unwrap();
            let error=Family::checked(&checked.program.closures,[0,1],Span::default()).err().expect("incompatible family");
            assert_eq!(error.code,"type");
        }
    }

    #[test]
    fn borrowed_family_preserves_default_cardinality_and_rejects_unchecked_shapes() {
        let mut checked=crate::check("function main(){const left=[1];const right=[2];const first=(x:int=3):int=>left.length+x;const second=(x:int=7):int=>right.length+x;console.log(first(),second());}").unwrap();
        let family=Family::checked(&checked.program.closures,[0,1],Span::default()).unwrap();
        assert!(family.parameters[0].omission.is_omittable());
        checked.program.closures[0].function.template=true;
        assert_eq!(Family::checked(&checked.program.closures,[0,1],Span::default()).err().unwrap().code,"borrowed-callable-layout");
        checked.program.closures[0].function.template=false;
        checked.program.closures[1].polymorphic=true;
        assert!(Family::checked(&checked.program.closures,[0,1],Span::default()).is_err());
        checked.program.closures[1].polymorphic=false;
        checked.program.closures[1].invocation=Mode::Consume;
        assert_eq!(Family::checked(&checked.program.closures,[0,1],Span::default()).unwrap().invocation,Mode::Consume);
        checked.program.closures[1].invocation=Mode::Copy;
        assert_eq!(Family::checked(&checked.program.closures,[0,1],Span::default()).err().unwrap().code,"callback-access");
        assert!(Family::checked(&checked.program.closures,[0,99],Span::default()).is_err());
    }

    #[test]
    fn borrowed_origin_remapping_is_transactional_and_never_merges_owners() {
        let checked=source();
        let mut family=Family::checked(&checked.program.closures,[0,1],Span::default()).unwrap();
        assert!(family.remap(&[Some(2),None],Span::default()).is_err());
        assert_eq!(family.origins,[0,1]);
        assert!(family.remap(&[Some(2),Some(2)],Span::default()).is_err());
        assert_eq!(family.origins,[0,1]);
        family.remap(&[Some(1),Some(0)],Span::default()).unwrap();
        assert_eq!(family.origins,[1,0]);
    }

}
