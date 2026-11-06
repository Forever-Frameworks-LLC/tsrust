//! Returned environments name capture payload lifetimes from typed source paths.
use super::native_view_lifetime_emit::Lifetimes;
use super::*;
use crate::analysis::native_view_loans::{Segment, VariantGuard};

impl Emitter<'_> {
    pub(super) fn native_view_environment_type(
        &self,
        origin: usize,
        input: Option<usize>,
        plan: &Lifetimes,
        path: &[Segment],
        guards: &[VariantGuard],
    ) -> String {
        if self.native_captures.lifetimes[origin] == 0 {
            return self.native_type(Type::Closure(origin));
        }
        enum Slot { Borrow(Vec<Segment>,Vec<VariantGuard>), Value(Type, Vec<Segment>, Vec<VariantGuard>) }
        let mut names = Vec::new();
        let mut pending = vec![Slot::Value(Type::Closure(origin), path.to_vec(), guards.to_vec())];
        while let Some(slot) = pending.pop() {
            let (ty,path,guards)=match slot {
                Slot::Borrow(path,guards)=>{names.push(plan.slot(input,&path,&guards).to_owned());continue;},
                Slot::Value(ty,path,guards)=>(ty,path,guards),
            };
            if !self.native_views.contains(ty) && !matches!(ty, Type::Closure(_)) {
                continue;
            }
            match ty {
                Type::StoredCallable(family) => {
                    // The family names one lifetime per slot, in alternative order.
                    let Some(layout) = self.program.stored_contracts.scoped_native_family(family) else { continue; };
                    for alternative in &layout.alternatives {
                        for slot in &alternative.slots {
                            let mut child = path.clone();
                            child.push(Segment::CallableAlternative(alternative.origin));
                            child.push(Segment::Field(format!("capture{}", slot.capture)));
                            let prefix = child.clone();
                            child.extend(slot.path.iter().cloned());
                            let mut slot_guards = guards.clone();
                            slot_guards.extend(slot.guards.iter().map(|guard| {
                                let mut guard = guard.clone();
                                let mut full = prefix.clone();
                                full.extend(guard.path);
                                guard.path = full;
                                guard
                            }));
                            names.push(plan.slot(input, &child, &slot_guards).to_owned());
                        }
                    }
                }
                Type::Record(id) if native_view_emit::access(self.program, id).is_some() || self.native_views.recursive(id) => {
                    names.push(plan.slot(input, &path, &guards).to_owned())
                }
                Type::Closure(id) => {
                    for capture in self.program.closures[id].captures.iter().rev() {
                        let mut child = path.clone();
                        child.push(Segment::Field(format!("capture{}", capture.inner)));
                        pending.push(Slot::Value(
                            self.native_captures
                                .ty(&self.program.closures[id].function, capture.inner),
                            child,
                            guards.clone(),
                        ));
                        // Outer loans precede carried view lifetimes in the physical capture layout.
                        if matches!(capture.storage, Mode::Read | Mode::Mutate) {
                            let mut borrow=path.clone();borrow.push(Segment::CaptureBorrow(capture.inner));
                            pending.push(Slot::Borrow(borrow,guards.clone()));
                        }
                    }
                }
                Type::Record(id) if self.program.records[id].native_element().is_some() => {
                    let mut child=path;child.push(Segment::Element);
                    pending.push(Slot::Value(self.program.records[id].native_element().unwrap(),child,guards));
                }
                Type::Record(id) => {
                    for field in self.program.records[id].fields.iter().rev() {
                        let mut child = path.clone();
                        child.push(Segment::Field(field.name.text.clone()));
                        pending.push(Slot::Value(field.ty.expect("checked field"), child, guards.clone()));
                    }
                }
                Type::Map(id) => {
                    let mut child=path;child.push(Segment::Element);
                    pending.push(Slot::Value(self.program.records[id].map_definition.as_ref().unwrap().value,child,guards));
                }
                Type::Array(element) => {
                    let mut child = path;
                    child.push(Segment::Element);
                    pending.push(Slot::Value(element.ty(), child, guards));
                }
                Type::Union(id) | Type::BorrowedUnion(id) => {
                    for &member in self.program.unions[id.index()].members.iter().rev() {
                        let mut child = guards.clone();
                        if let Type::Record(success) = member {
                            child.push(VariantGuard {
                                path: path.clone(),
                                success,
                            });
                        }
                        pending.push(Slot::Value(member, path.clone(), child));
                    }
                }
                _ => (),
            }
        }
        if names.len() != self.native_captures.lifetimes[origin] {
            return self.native_type(Type::Closure(origin));
        }
        self.named_record_environment_type(Type::Closure(origin), &names)
    }
}
