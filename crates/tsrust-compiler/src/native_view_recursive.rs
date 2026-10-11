//! Canonical loan paths stop at the first recursive record boundary.
use super::*;
use super::native_view_loans::{Loan, Segment};
use crate::native_return_borrows::BorrowAccess;

pub(super) struct Shapes {
    recursive: crate::recursive_view_shapes::Index,
    fields: Vec<Vec<(String, Type)>>,
    unions: Vec<crate::unions::UnionType>,
}
impl Shapes {
    fn new(analyzer: &Analyzer) -> Self {
        Self {
            recursive: crate::recursive_view_shapes::Index::new(&analyzer.program.records, &|id| analyzer.unions.members(id)),
            fields: analyzer.program.records.iter().map(|record| record.fields.iter()
                .filter_map(|field| field.ty.map(|ty| (field.name.text.clone(), ty))).collect()).collect(),
            unions: analyzer.unions.snapshot(),
        }
    }

    fn normalize(&self, root: Type, loan: &mut Loan) {
        let mut pending = vec![(root, 0)];
        let mut seen = HashSet::new();
        let mut boundary: Option<(usize, BorrowAccess)> = None;
        while let Some((ty, depth)) = pending.pop() {
            if !seen.insert((ty, depth)) { continue; }
            if let Type::Record(id) = ty {
                if let Some(access) = self.recursive.access(id) {
                    match &mut boundary {
                        Some((position, previous)) if *position == depth => {
                            if access == BorrowAccess::Exclusive { *previous = access; }
                        }
                        Some((position, _)) if *position < depth => {}
                        _ => boundary = Some((depth, access)),
                    }
                    continue;
                }
                if let Some(Segment::Field(name)) = loan.path.get(depth)
                    && let Some(fields) = self.fields.get(id)
                    && let Some((_, child)) = fields.iter().find(|(field, _)| field == name) {
                    pending.push((*child, depth + 1));
                }
            } else {
                match ty {
                    Type::Array(element) if loan.path.get(depth) == Some(&Segment::Element) => pending.push((element.ty(), depth + 1)),
                    Type::Union(id) | Type::BorrowedUnion(id) => {
                        if let Some(union) = self.unions.get(id.index()) {
                            for &member in &union.members {
                                if loan.guards.iter().any(|guard| guard.path == loan.path[..depth] && member != Type::Record(guard.success)) { continue; }
                                pending.push((member, depth));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Some((depth, access)) = boundary {
            loan.path.truncate(depth);
            loan.guards.retain(|guard| guard.path.len() <= depth);
            loan.whole = true;
            if access == BorrowAccess::Exclusive { loan.access = access; }
        }
    }
}

impl Analyzer {
    #[inline(never)]
    pub(super) fn refresh_native_recursive_shapes(&self, context: &mut Context) {
        if !self.native_borrow_types { return; }
        if context.native_recursive_shapes.as_ref().is_none_or(|shapes|
            shapes.fields.len() != self.program.records.len() || shapes.unions.len() != self.unions.len()) {
            context.native_recursive_shapes = Some(std::rc::Rc::new(Shapes::new(self)));
        }
    }
}

pub(super) fn normalize(context: &Context, ty: Type, loans: &mut [Loan]) {
    if let Some(shapes) = &context.native_recursive_shapes {
        for loan in loans { shapes.normalize(ty, loan); }
    }
}
