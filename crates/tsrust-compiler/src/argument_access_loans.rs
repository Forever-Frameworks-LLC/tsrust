//! Temporary argument loans retain proven static record projections.
use std::collections::{HashMap, hash_map::Entry};
use crate::{analysis::Mode, syntax::{Expr, ExprKind, Type}, Span};

/// Existing root loans remain conservative unless an argument proves a static path.
#[derive(Clone, Default)]
pub(crate) struct Loans {
    roots: HashMap<usize, Vec<(Mode, Span)>>,
    fields: HashMap<(usize, Mode, Span), Vec<String>>,
}
impl std::ops::Deref for Loans {
    type Target = HashMap<usize, Vec<(Mode, Span)>>;
    fn deref(&self) -> &Self::Target { &self.roots }
}
impl Loans {
    pub(crate) fn new() -> Self { Self::default() }
    pub(crate) fn entry(&mut self, root: usize) -> Entry<'_, usize, Vec<(Mode, Span)>> {
        // Broad insertions must never inherit an earlier narrow identity.
        self.fields.retain(|(binding, _, _), _| *binding != root);
        self.roots.entry(root)
    }
    pub(crate) fn argument(&mut self, root: usize, mode: Mode, arg: &Expr) {
        let key = (root, mode, arg.span);
        let previous = self.roots.get(&root).is_some_and(|items| items.contains(&(mode, arg.span)));
        let mut path = Vec::new();
        let mut cursor = arg;
        while let ExprKind::Field(base, name) = &cursor.kind {
            if !matches!(base.ty, Some(Type::Record(_))) { path.clear(); break; }
            path.push(name.clone());
            cursor = base;
        }
        let proven = !path.is_empty() && matches!(cursor.kind, ExprKind::Name(_)) && cursor.binding == Some(root);
        if proven {
            path.reverse();
            if !previous { self.fields.insert(key, path); }
            else if self.fields.get(&key) != Some(&path) { self.fields.remove(&key); }
        } else { self.fields.remove(&key); }
        self.roots.entry(root).or_default().push((mode, arg.span));
    }
    pub(crate) fn overlaps(&self, root: usize, mode: Mode, span: Span, path: Option<&[String]>) -> bool {
        match (self.fields.get(&(root, mode, span)), path) {
            (Some(earlier), Some(current)) => earlier.starts_with(current) || current.starts_with(earlier),
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn broad_insertions_discard_previous_field_proofs() {
        let mut loans=super::Loans::new();
        let span=crate::Span {start:1,end:2};
        loans.roots.insert(0,vec![(crate::analysis::Mode::Mutate,span)]);
        loans.fields.insert((0,crate::analysis::Mode::Mutate,span),vec!["left".to_owned()]);
        let right=vec!["right".to_owned()];
        assert!(!loans.overlaps(0,crate::analysis::Mode::Mutate,span,Some(&right)));
        loans.entry(0).or_default().push((crate::analysis::Mode::Read,span));
        assert!(loans.overlaps(0,crate::analysis::Mode::Mutate,span,Some(&right)));
    }

    #[test]
    fn overlapping_mutable_record_arguments_remain_rejected() {
        for arguments in ["pair.left,pair.left","pair,pair.left"] {
            let source=if arguments=="pair,pair.left" {
                "type Leaf={value:int;};type Pair={left:Leaf;right:Leaf;};function take(all:Mutable<Pair>,part:Mutable<Leaf>):void{}function main(){const pair:Pair={left:{value:1},right:{value:2}};take(pair,pair.left);}".to_owned()
            }else{format!("type Leaf={{value:int;}};type Pair={{left:Leaf;right:Leaf;}};function take(a:Mutable<Leaf>,b:Mutable<Leaf>):void{{}}function main(){{const pair:Pair={{left:{{value:1}},right:{{value:2}}}};take({arguments});}}")};
            let source=crate::format(&source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
            let error=crate::check(&source).err().expect("overlapping arguments must fail");assert_eq!(error.code,"conflicting-access","{error:?}");
        }
    }
    #[test]
    fn static_field_loans_keep_opaque_and_ancestor_conflicts() {
        let sources = [
            "type Leaf={value:int;};type Pair={left:Leaf;right:Leaf;};function take(part:Mutable<Leaf>,all:Mutable<Pair>):void{}function main(){const pair:Pair={left:{value:1},right:{value:2}};take(pair.left,pair);}",
            "type Leaf={value:int;};type Branch={leaf:Leaf;};function take(all:Mutable<Branch>,part:Mutable<Leaf>):void{}function main(){const branch:Branch={leaf:{value:1}};take(branch,branch.leaf);}",
            "type Leaf={value:int;};function take(a:Mutable<Leaf>,b:Mutable<Leaf>):void{}function main(){const items:Leaf[]=[{value:1},{value:2}];take(items[0],items[1]);}",
            "type Leaf={value:int;};type Pair={left:Leaf;right:Leaf;};function take(a:Mutable<Leaf>,b:ReadOnly<Leaf>):void{}function main(){const pair:Pair={left:{value:1},right:{value:2}};take(pair.left,pair.left);}",
        ];
        for source in sources {
            let source=crate::format(source).unwrap();
            assert_eq!(crate::format(&source).unwrap(),source);
            let error=crate::check(&source).err().expect("overlapping or opaque access must fail");
            assert_eq!(error.code,"conflicting-access","{error:?}");
        }
    }
    #[test]
    fn hidden_capture_arguments_remain_conservative() {
        let source=crate::format("type Leaf={value:int;};type Pair={left:Leaf;right:Leaf;};function main(){const pair:Pair={left:{value:1},right:{value:2}};function touch(part:Mutable<Leaf>):void{console.log(pair.right.value);}touch(pair.left);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let error=crate::check(&source).err().expect("hidden whole-root capture must conflict");
        assert_eq!(error.code,"conflicting-access","{error:?}");
    }
    #[test]
    fn mutable_scalar_contracts_remain_rejected() {
        let source=crate::format("type Pair={left:int;right:int;};function take(a:Mutable<int>,b:Mutable<int>):void{}function main(){const pair:Pair={left:1,right:2};take(pair.left,pair.left);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let error=crate::check(&source).err().expect("mutable scalar arguments must not copy away their loan");
        assert_eq!(error.code,"callback-access","{error:?}");
    }
}
