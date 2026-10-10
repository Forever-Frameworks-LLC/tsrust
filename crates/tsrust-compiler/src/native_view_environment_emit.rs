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
        let mut names = Vec::new();
        let mut pending = vec![(Type::Closure(origin), path.to_vec(), guards.to_vec())];
        while let Some((ty, path, guards)) = pending.pop() {
            if !self.native_views.contains(ty) && !matches!(ty, Type::Closure(_)) {
                continue;
            }
            match ty {
                Type::Record(id) if native_view_emit::access(self.program, id).is_some() => {
                    names.push(plan.slot(input, &path, &guards).to_owned())
                }
                Type::Closure(id) => {
                    for capture in self.program.closures[id].captures.iter().rev() {
                        let mut child = path.clone();
                        child.push(Segment::Field(format!("capture{}", capture.inner)));
                        pending.push((
                            self.native_captures
                                .ty(&self.program.closures[id].function, capture.inner),
                            child,
                            guards.clone(),
                        ));
                    }
                }
                Type::Record(id) => {
                    for field in self.program.records[id].fields.iter().rev() {
                        let mut child = path.clone();
                        child.push(Segment::Field(field.name.text.clone()));
                        pending.push((field.ty.expect("checked field"), child, guards.clone()));
                    }
                }
                Type::Array(element) => {
                    let mut child = path;
                    child.push(Segment::Element);
                    pending.push((element.ty(), child, guards));
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
                        pending.push((member, path.clone(), child));
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
