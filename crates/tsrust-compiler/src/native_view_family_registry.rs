//! Prepared native layouts have identities separate from admitted source storage.
use crate::native_view_callable_storage::Family;
use crate::{Diagnostic,Span};
use std::collections::{HashMap,HashSet};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct PreparedId(usize);

#[derive(Clone,Default)]
pub(crate) struct Registry {
    layouts:Vec<Family>,
    bindings:HashMap<usize,PreparedId>,
}

impl Registry {
    pub(crate) fn get(&self,binding:usize)->Option<&Family> {
        self.bindings.get(&binding).map(|id|&self.layouts[id.0])
    }

    pub(crate) fn prepare(&mut self,binding:usize,layout:Family) {
        let id=match self.layouts.iter().position(|existing|existing==&layout) {
            Some(id)=>PreparedId(id),
            None=>{let id=PreparedId(self.layouts.len());self.layouts.push(layout);id},
        };
        self.bindings.insert(binding,id);
    }

    pub(crate) fn remapped(&self,ids:&[Option<usize>])->Result<Self,Diagnostic> {
        let mut seen=HashSet::new();
        if ids.iter().flatten().any(|id|!seen.insert(*id)) {
            return Err(Diagnostic::new("callable-layout","Native callable layout compaction requires distinct retained origins.",Span::default()));
        }
        let mut next=self.clone();
        for layout in &mut next.layouts {
            for alternative in &mut layout.alternatives {
                alternative.origin=ids.get(alternative.origin).copied().flatten().ok_or_else(||Diagnostic::new(
                    "callable-layout","A prepared native callable layout lost its environment origin.",Span::default()))?;
                for slot in &mut alternative.slots {
                    crate::native_view_origin_remap::path(&mut slot.path,ids)?;
                    crate::native_view_origin_remap::guards(&mut slot.guards,ids)?;
                }
            }
            layout.alternatives.sort_by_key(|alternative|alternative.origin);
        }
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_view_callable_storage::{Alternative,Slot};
    use crate::analysis::native_view_loans::Segment;
    use crate::native_return_borrows::BorrowAccess;

    fn layout()->Family {
        let mut contracts=crate::callable_contracts::Contracts::default();
        let signature=contracts.intern([],crate::syntax::Type::Int,Some(crate::syntax::CallbackKind::OwnedOnce));
        Family {signature,invocation:crate::Mode::Consume,alternatives:vec![
            Alternative{origin:0,slots:vec![Slot{capture:4,path:vec![Segment::Field("first".into())],guards:vec![],access:BorrowAccess::ReadOnly}]},
            Alternative{origin:1,slots:vec![Slot{capture:9,path:vec![Segment::Field("second".into())],guards:vec![],access:BorrowAccess::Exclusive}]},
        ]}
    }

    #[test]
    fn native_view_prepared_registry_keeps_slot_identity_and_canonical_layouts() {
        let mut registry=Registry::default();
        registry.prepare(3,layout());registry.prepare(7,layout());
        assert_eq!(registry.bindings[&3],registry.bindings[&7]);
        assert_eq!(registry.layouts.len(),1);
        let mut changed=layout();changed.alternatives[0].slots[0].access=BorrowAccess::Exclusive;
        registry.prepare(8,changed);
        assert_ne!(registry.bindings[&3],registry.bindings[&8]);
        assert_eq!(registry.layouts.len(),2);
        let mapped=registry.remapped(&[Some(1),Some(0)]).unwrap();
        let alternatives=&mapped.get(3).unwrap().alternatives;
        assert_eq!(alternatives[0].origin,0);
        assert_eq!(alternatives[0].slots[0].capture,9);
        assert_eq!(alternatives[0].slots[0].access,BorrowAccess::Exclusive);
        assert_eq!(alternatives[1].origin,1);
        assert_eq!(alternatives[1].slots[0].capture,4);
        assert_eq!(registry.get(3).unwrap().alternatives[0].slots[0].capture,4);
    }

    #[test]
    fn native_view_prepared_registry_rejects_missing_and_aliased_origin_maps() {
        let mut registry=Registry::default();registry.prepare(3,layout());
        for mapping in [&[Some(0),None][..],&[Some(0),Some(0)][..]] {
            assert!(registry.remapped(mapping).is_err());
            assert_eq!(registry.get(3),Some(&layout()));
        }
    }
}
