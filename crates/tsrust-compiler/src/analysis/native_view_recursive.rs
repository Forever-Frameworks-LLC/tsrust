//! Canonical loan paths stop at the first recursive record boundary.
use super::*;
use super::native_view_loans::{Loan, Segment};
use crate::native_return_borrows::BorrowAccess;

pub(super) struct Shapes {
    pub(super) recursive: crate::recursive_view_shapes::Index,
    native_types: Vec<Option<crate::native_signatures::NativeTypeId>>,
    incomplete: Vec<bool>,
    scoped_families: Vec<(usize,crate::native_view_callable_storage::Family)>,
    fields: Vec<Vec<(String, Type)>>,
    readonly_view: Option<usize>,
    map_values: Vec<Option<Type>>,
    native_elements: Vec<Option<Type>>,
    unions: Vec<crate::unions::UnionType>,
}
impl Shapes {
    fn new(analyzer: &Analyzer) -> Self {
        Self {
            native_types:analyzer.program.records.iter().map(|record|record.native_type).collect(),
            incomplete:analyzer.program.records.iter().map(|record|record.fields.iter().any(|field|field.ty.is_none())).collect(),
            scoped_families:analyzer.callables.storage.scoped_native_families().map(|(id,family)|(id,family.clone())).collect(),
            native_elements: analyzer.program.records.iter().map(|record| record.native_element()).collect(),
            map_values: analyzer.program.records.iter().map(|record| record.map_definition.as_ref().map(|map| map.value)).collect(),
            readonly_view: analyzer.program.records.iter().position(|record| record.native_type == Some(crate::native_signatures::NativeTypeId::ByteView)),
            recursive: crate::recursive_view_shapes::Index::with_storage(&analyzer.program.records, &|id| analyzer.unions.members(id), &analyzer.callables.storage),
            fields: analyzer.program.records.iter().map(|record| record.fields.iter()
                .filter_map(|field| field.ty.map(|ty| (field.name.text.clone(), ty))).collect()).collect(),
            unions: analyzer.unions.snapshot(),
        }
    }

    fn matches(&self, analyzer:&Analyzer)->bool {
        self.fields.len()==analyzer.program.records.len()
            && analyzer.unions.matches_members(&self.unions)
            && self.scoped_families.len()==analyzer.callables.storage.scoped_native_families().count()
            && self.scoped_families.iter().all(|(id,family)|analyzer.callables.storage.scoped_native_family(*id)==Some(family))
            && analyzer.program.records.iter().enumerate().all(|(id,record)|
                self.native_types[id]==record.native_type
                && self.native_elements[id]==record.native_element()
                && self.map_values[id]==record.map_definition.as_ref().map(|map|map.value)
                && self.incomplete[id]==record.fields.iter().any(|field|field.ty.is_none())
                && self.fields[id].iter().map(|(name,ty)|(name.as_str(),*ty))
                    .eq(record.fields.iter().filter_map(|field|field.ty.map(|ty|(field.name.text.as_str(),ty)))))
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
                if loan.path.get(depth) == Some(&Segment::Element)
                    && let Some(Some(element)) = self.native_elements.get(id) {
                    pending.push((*element, depth + 1));
                }
                if let Some(Segment::Field(name)) = loan.path.get(depth)
                    && let Some(fields) = self.fields.get(id)
                    && let Some((_, child)) = fields.iter().find(|(field, _)| field == name) {
                    pending.push((*child, depth + 1));
                }
            } else {
                match ty {
                    Type::Map(id) if loan.path.get(depth) == Some(&Segment::Element) => { if let Some(Some(value))=self.map_values.get(id) { pending.push((*value, depth+1)); } },
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
    pub(super) fn native_recursive_shapes(&self)->std::rc::Rc<Shapes> {
        let mut cache=self.native_recursive_shapes.borrow_mut();
        if cache.as_ref().is_none_or(|shapes|!shapes.matches(self)) {
            *cache=Some(std::rc::Rc::new(Shapes::new(self)));
        }
        std::rc::Rc::clone(cache.as_ref().unwrap())
    }

    #[inline(never)]
    pub(super) fn refresh_native_recursive_shapes(&self, context: &mut Context) {
        if !self.native_borrow_types { return; }
        context.native_recursive_shapes=Some(self.native_recursive_shapes());
    }
}

pub(super) fn normalize(context: &Context, ty: Type, loans: &mut [Loan]) {
    if let Some(shapes) = &context.native_recursive_shapes {
        for loan in loans { shapes.normalize(ty, loan); }
    }
}

pub(super) fn readonly_source(context: &Context, ty: Type) -> bool {
    matches!(ty, Type::Record(id) if context.native_recursive_shapes.as_ref().is_some_and(|shapes| shapes.readonly_view == Some(id)))
}

#[cfg(test)] mod latency_cache_tests {
    use super::*;
    fn analyzer()->Analyzer {
        let source="type Window={};type Link={next:int;view:Window;};function main(){}";
        let formatted=crate::formatting::format(source).unwrap();assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
        Analyzer::new(parsed).unwrap()
    }
    #[test] fn latency_cache_shares_shapes_across_functions_but_checks_in_place_record_changes() {
        let mut analyzer=analyzer();let original=analyzer.native_recursive_shapes();
        for _ in 0..256 {let mut context=Context::new("independent".into());analyzer.refresh_native_recursive_shapes(&mut context);assert!(std::rc::Rc::ptr_eq(&original,context.native_recursive_shapes.as_ref().unwrap()));}
        analyzer.program.records[1].fields[0].ty=Some(Type::Array(crate::syntax::ArrayElement::Record(1,0)));
        let recursive=analyzer.native_recursive_shapes();assert!(!std::rc::Rc::ptr_eq(&original,&recursive));assert_eq!(recursive.recursive.access(1),Some(BorrowAccess::ReadOnly));
        analyzer.program.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteViewMut);
        let exclusive=analyzer.native_recursive_shapes();assert!(!std::rc::Rc::ptr_eq(&recursive,&exclusive));assert_eq!(exclusive.recursive.access(1),Some(BorrowAccess::Exclusive));
        analyzer.program.records[1].fields[1].ty=None;assert_eq!(analyzer.native_recursive_shapes().recursive.access(1),None);
        assert_eq!(recursive.recursive.access(1),Some(BorrowAccess::ReadOnly),"older contexts retain their own immutable shape");
    }
    #[test] fn latency_cache_notices_recursive_union_definition_without_a_new_union_id() {
        let mut analyzer=analyzer();let span=Span::default();let ty=analyzer.unions.reserve_recursive(span).unwrap();let id=ty.union().unwrap();
        analyzer.program.records[1].fields[0].ty=Some(ty);let before=analyzer.native_recursive_shapes();let count=analyzer.unions.len();
        analyzer.unions.bind_recursive(id,vec![Type::Record(1),Type::Undefined],span).unwrap();
        let after=analyzer.native_recursive_shapes();assert_eq!(analyzer.unions.len(),count);assert!(!std::rc::Rc::ptr_eq(&before,&after));assert_eq!(after.recursive.access(1),Some(BorrowAccess::ReadOnly));
    }
    #[test] fn latency_cache_notices_new_alternatives_in_an_existing_scoped_family() {
        let source="type Window={};type Holder={next:int;action:int;};function first(view:Window){return ()=>view;}function second(view:Window){return ()=>view;}function main(){}";
        let mut parsed=crate::syntax::parse(source).unwrap();parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
        let mut analyzer=Analyzer::new(parsed).unwrap();
        let factories=analyzer.program.functions.iter().enumerate().filter_map(|(id,function)|matches!(function.name.text.as_str(),"first"|"second").then_some(id)).collect::<Vec<_>>();
        for factory in factories {analyzer.function(factory).unwrap();}
        let result=analyzer.closures[0].function.result;
        let structural=analyzer.callables.shapes.intern([],result,Some(CallbackKind::OwnedOnce));
        let callback=CallbackContract{structural,parameter_names:vec![],params:vec![],result,kind:CallbackKind::OwnedOnce};
        let mut families=Vec::new();
        for origin in [0,1] {let Type::StoredCallable(id)=analyzer.callables.storage.intern_scoped_native(&analyzer.callables.shapes,&callback,0,[origin],&analyzer.closures,Span::default()).unwrap().unwrap()else{unreachable!()};families.push(id);}
        let ty=analyzer.callables.storage.intern_recursive_scoped_native(&analyzer.callables.shapes,std::slice::from_ref(&callback),families[0],1,1,&analyzer.closures,Span::default()).unwrap();
        analyzer.program.records[1].fields[0].ty=Some(Type::Array(crate::syntax::ArrayElement::Record(1,0)));analyzer.program.records[1].fields[1].ty=Some(ty);
        let before=analyzer.native_recursive_shapes();
        assert_eq!(analyzer.callables.storage.intern_recursive_scoped_native(&analyzer.callables.shapes,std::slice::from_ref(&callback),families[1],1,1,&analyzer.closures,Span::default()).unwrap(),ty);
        let after=analyzer.native_recursive_shapes();assert!(!std::rc::Rc::ptr_eq(&before,&after));assert_eq!(before.scoped_families.len(),after.scoped_families.len());
        let Type::StoredCallable(id)=ty else{unreachable!()};assert_eq!(before.scoped_families.iter().find(|(family,_)|*family==id).unwrap().1.alternatives.len(),1);assert_eq!(after.scoped_families.iter().find(|(family,_)|*family==id).unwrap().1.alternatives.len(),2);
    }

}
