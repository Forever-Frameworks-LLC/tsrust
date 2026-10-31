//! Canonical checked type identities; runtime values use ordinary Rust enums.
use crate::diagnostic::{Diagnostic, Span};
use crate::syntax::{Type, UnionId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct UnionType {
    /// A nominal recursive slot is never flattened into its own definition.
    pub recursive: bool,
    /// Synthetic declaration used by checked annotations to close the type knot.
    pub recursive_declaration: Option<usize>,
    pub members: Vec<Type>,
}

#[derive(Default)]
struct ArenaState {
    types: Vec<UnionType>,
    identities: HashMap<Vec<Type>, UnionId>,
    named_types: HashMap<usize, Type>,
}

#[derive(Default)]
pub(crate) struct UnionArena(RefCell<ArenaState>);

impl UnionArena {
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize { self.0.borrow().types.len() }
    pub(crate) fn from_snapshot(types:Vec<UnionType>)->Self {
        let identities=types.iter().enumerate().filter(|(_,union)|!union.recursive).map(|(index,union)| {
            (union.members.clone(),UnionId{index:u32::try_from(index).expect("existing union index"),copy:union.members.iter().all(|member|member.is_copy()),scalar_members:union.members.iter().all(|member|member.is_scalar())})
        }).collect();
        Self(RefCell::new(ArenaState{types,identities,named_types:HashMap::new()}))
    }
    pub(crate) fn from_snapshot_with_records(types: Vec<UnionType>, records: &[crate::syntax::Record]) -> Self {
        let arena = Self::from_snapshot(types);
        arena.register_named_types(records);
        arena
    }
    pub(crate) fn register_named_types(&self, records: &[crate::syntax::Record]) {
        let mut state = self.0.borrow_mut();
        state.named_types.clear();
        for (id, record) in records.iter().enumerate() {
            if record.map_definition.is_some() { state.named_types.insert(id, Type::Map(id)); }
            if record.set_definition.is_some() { state.named_types.insert(id, Type::Set(id)); }
            if let Some(definition)=&record.literal_definition {state.named_types.insert(id,Type::Literal(id,definition.kind()));}
            if let Some(definition) = &record.enum_definition {
                state.named_types.insert(id, Type::Enum(id, definition.kind));
            }
        }
    }
    pub(crate) fn register_literal_type(&self,id:usize,kind:crate::literal_types::LiteralKind){self.0.borrow_mut().named_types.insert(id,Type::Literal(id,kind));}
    pub(crate) fn register_map_type(&self, id: usize) { self.0.borrow_mut().named_types.insert(id, Type::Map(id)); }
    pub(crate) fn register_set_type(&self, id: usize) { self.0.borrow_mut().named_types.insert(id, Type::Set(id)); }
    pub(crate) fn named_type(&self, id: usize) -> Type {
        let arena=self.0.borrow();
        arena.named_types.get(&id).copied().or_else(||arena.types.iter().enumerate().find_map(|(index,union)|
            (union.recursive_declaration==Some(id)).then_some(Type::Union(UnionId{index:index as u32,copy:false, scalar_members:false}))))
            .unwrap_or(Type::Record(id))
    }
    pub(crate) fn is_recursive(&self,id:UnionId)->bool {
        self.0.borrow().types[id.index()].recursive
    }

    pub(crate) fn recursive_declaration(&self,id:UnionId)->Option<usize> {
        self.0.borrow().types[id.index()].recursive_declaration
    }
    pub(crate) fn name_recursive(&self,id:UnionId,declaration:usize,span:Span)->Result<(),Diagnostic> {
        let mut arena=self.0.borrow_mut();
        if arena.types.iter().any(|union|union.recursive_declaration==Some(declaration)) {
            return Err(Diagnostic::new("schema-recursion","Recursive declaration already names a type.",span));
        }
        let slot=arena.types.get_mut(id.index()).ok_or_else(||Diagnostic::new("schema-recursion","Unknown recursive type slot.",span))?;
        if !slot.recursive || slot.recursive_declaration.is_some() {
            return Err(Diagnostic::new("schema-recursion","Recursive type names resolve exactly once.",span));
        }
        slot.recursive_declaration=Some(declaration);
        Ok(())
    }

    /// Reserve one stable, owning identity before inferring its definition.
    /// This slot is compiler metadata; no runtime allocation or handle is made.
    pub(crate) fn reserve_recursive(&self,span:Span)->Result<Type,Diagnostic> {
        let mut arena=self.0.borrow_mut();
        let index=u32::try_from(arena.types.len()).map_err(|_|Diagnostic::new("unsupported","Program union type limit exceeded.",span))?;
        arena.types.push(UnionType{recursive:true,recursive_declaration:None,members:Vec::new()});
        Ok(Type::Union(UnionId{index,copy:false,scalar_members:false}))
    }

    /// Resolve a reserved slot once. Structural productivity and native layout
    /// validation remain separate checks; neither is implied by binding a type.
    pub(crate) fn bind_recursive(&self,id:UnionId,members:Vec<Type>,span:Span)->Result<(),Diagnostic> {
        let mut arena=self.0.borrow_mut();
        let slot=arena.types.get_mut(id.index()).ok_or_else(||Diagnostic::new("schema-recursion","Unknown recursive type slot.",span))?;
        if !slot.recursive || !slot.members.is_empty() {
            return Err(Diagnostic::new("schema-recursion","Recursive type definitions resolve exactly once.",span));
        }
        if members.is_empty() || members.iter().any(|member|matches!(member,Type::Void)) {
            return Err(Diagnostic::new("schema-recursion","A recursive type requires a value definition.",span));
        }
        if members.contains(&Type::Union(id)) {
            return Err(Diagnostic::new("schema-recursion","A recursive type cannot directly contain itself.",span));
        }
        slot.members=members;
        Ok(())
    }

    pub fn members(&self, id: UnionId) -> Vec<Type> {
        self.0.borrow().types[id.index()].members.clone()
    }

    pub(crate) fn matches_members(&self, snapshot: &[UnionType]) -> bool {
        let state=self.0.borrow();
        state.types.len()==snapshot.len() && state.types.iter().zip(snapshot).all(|(current,old)|current.members==old.members)
    }

    pub fn snapshot(&self) -> Vec<UnionType> {
        self.0.borrow().types.clone()
    }

    pub(crate) fn remap_closure_origins(&self, origins: &[Option<usize>]) {
        let mut arena = self.0.borrow_mut();
        for union in &mut arena.types {
            for member in &mut union.members {
                if let Type::Closure(origin) = member
                    && let Some(Some(mapped)) = origins.get(*origin)
                {
                    *origin = *mapped;
                }
            }
        }
        // Union IDs remain stable in checked trees. Removed discovery origins
        // in unused symbolic types do not become concrete checked facts.
        arena.identities = arena
            .types
            .iter()
            .enumerate()
            .filter(|(_, union)| !union.recursive)
            .map(|(index, union)| {
                (
                    union.members.clone(),
                    UnionId {
                        index: u32::try_from(index).expect("existing union index"),
                        copy: union.members.iter().all(|member| member.is_copy()),
                        scalar_members: union.members.iter().all(|member|member.is_scalar()),
                    },
                )
            })
            .collect();
    }

    pub fn intern(
        &self,
        types: impl IntoIterator<Item = Type>,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        let mut members = Vec::new();
        let mut seen = HashSet::new();
        for value in types {
            let alternatives = match value {
                Type::Union(id) if !self.is_recursive(id) => self.members(id),
                member => vec![member],
            };
            for member in alternatives {
                if seen.insert(member) {
                    members.push(member);
                }
            }
        }
        if members.is_empty() {
            return Err(Diagnostic::new(
                "type",
                "A union must contain at least one value type.",
                span,
            ));
        }
        if members.iter().any(|ty| matches!(ty, Type::Void)) {
            return Err(Diagnostic::new(
                "unsupported",
                "Void unions are not supported.",
                span,
            ));
        }
        let payloads = members.iter().filter(|ty| !matches!(ty, Type::Null | Type::Undefined)).count();
        if payloads > 1 && members.iter().any(|ty| matches!(ty, Type::Closure(_) | Type::Callable(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_))) {
            return Err(Diagnostic::new("unsupported", "Callable unions support one callable payload with null, undefined, or both.", span));
        }
        // Canonical data alternatives precede absent tags, independent of spelling.
        members.sort_by_key(|ty| match ty {
            Type::Null => (1, String::new()),
            Type::Undefined => (2, String::new()),
            _ => (0, format!("{ty:?}")),
        });
        if members.len() == 1 {
            return Ok(members[0]);
        }
        let mut arena = self.0.borrow_mut();
        if let Some(&id) = arena.identities.get(&members) {
            return Ok(Type::Union(id));
        }
        let id = UnionId {
            index: u32::try_from(arena.types.len()).map_err(|_| {
                Diagnostic::new("unsupported", "Program union type limit exceeded.", span)
            })?,
            copy: members.iter().all(|ty| ty.is_copy()),
            scalar_members: members.iter().all(|ty|ty.is_scalar()),
        };
        arena.types.push(UnionType {
            recursive: false,
            recursive_declaration: None,
            members: members.clone(),
        });
        arena.identities.insert(members, id);
        Ok(Type::Union(id))
    }

    pub fn accepts(&self, expected: Type, actual: Type) -> bool {
        if expected.accepts_scalar_value(actual) {
            return true;
        }
        let Type::Union(id) = expected else {
            return false;
        };
        let arena = self.0.borrow();
        let members = &arena.types[id.index()].members;
        match actual {
            Type::Union(other) => arena.types[other.index()]
                .members
                .iter()
                .all(|ty| members.iter().any(|member| member.accepts_scalar_value(*ty))),
            other => members.iter().any(|member| member.accepts_scalar_value(other)),
        }
    }

    pub(crate) fn present(&self, ty: Type, span: Span) -> Result<Option<Type>, Diagnostic> {
        let members = ty.union().map(|id| self.members(id)).unwrap_or_else(|| vec![ty]);
        let present = members.into_iter().filter(|member| !matches!(member, Type::Null | Type::Undefined)).collect::<Vec<_>>();
        if present.is_empty() { Ok(None) } else { self.intern(present, span).map(Some) }
    }

    pub fn payload(&self, ty: Type) -> Option<Type> {
        let Type::Union(id) = ty else {
            return None;
        };
        let arena = self.0.borrow();
        let mut payloads = arena.types[id.index()].members.iter().copied().filter(|ty| !matches!(ty, Type::Null | Type::Undefined));
        let first = payloads.next();
        if payloads.next().is_none() { first } else { None }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::ArrayElement;

    fn intern(arena: &UnionArena, members: &[Type]) -> Type {
        arena
            .intern(members.iter().copied(), Span::new(0, 1))
            .unwrap()
    }

    #[test]
    fn recursive_array_slots_retain_identity_through_normalization_and_snapshots() {
        let arena=UnionArena::default();
        let root=arena.reserve_recursive(Span::default()).unwrap();
        let Type::Union(id)=root else {unreachable!()};
        let array=Type::Array(ArrayElement::from_value_type(root).unwrap());
        arena.bind_recursive(id,vec![array],Span::default()).unwrap();
        assert_eq!(arena.members(id),vec![array]);
        assert!(!root.is_copy());
        assert_eq!(intern(&arena,&[root]),root);
        let optional=intern(&arena,&[root,Type::Null]);
        let Type::Union(optional_id)=optional else {unreachable!()};
        assert!(arena.members(optional_id).contains(&root));
        assert!(!arena.members(optional_id).contains(&array));
        let resumed=UnionArena::from_snapshot(arena.snapshot());
        assert!(resumed.is_recursive(id));
        assert_eq!(intern(&resumed,&[root,Type::Null]),optional);
        resumed.remap_closure_origins(&[]);
        assert_eq!(intern(&resumed,&[root]),root);
    }

    #[test]
    fn recursive_type_binding_is_single_assignment_and_rejects_direct_self() {
        let arena=UnionArena::default();
        let root=arena.reserve_recursive(Span::default()).unwrap();
        let Type::Union(id)=root else {unreachable!()};
        assert!(arena.bind_recursive(id,vec![root],Span::default()).is_err());
        assert!(arena.bind_recursive(id,Vec::new(),Span::default()).is_err());
        let array=Type::Array(ArrayElement::from_value_type(root).unwrap());
        arena.bind_recursive(id,vec![Type::Int,array],Span::default()).unwrap();
        assert!(arena.bind_recursive(id,vec![Type::String],Span::default()).is_err());
        assert_eq!(arena.members(id),vec![Type::Int,array]);
    }

    #[test]
    fn canonical_nullable_identity_and_subset_widening() {
        let arena = UnionArena::default();
        let narrow = intern(&arena, &[Type::Null, Type::Int, Type::Int]);
        let wide = intern(&arena, &[Type::Undefined, narrow]);
        assert_eq!(
            wide,
            intern(&arena, &[Type::Null, Type::Undefined, Type::Int])
        );
        assert_eq!(narrow, intern(&arena, &[Type::Int, Type::Null]));
        assert_eq!(arena.snapshot().len(), 2);
        assert_eq!(intern(&arena, &[Type::Int, Type::Int]), Type::Int);
        assert!(wide.is_copy());
        assert!(arena.accepts(wide, narrow));
        assert!(arena.accepts(wide, Type::Int));
        assert!(!arena.accepts(narrow, wide));
        assert!(!arena.accepts(wide, Type::Number));
    }

    #[test]
    fn absent_alternatives_and_owned_payloads_keep_distinct_contracts() {
        let arena = UnionArena::default();
        let absent = intern(&arena, &[Type::Undefined, Type::Null]);
        assert!(absent.is_copy());
        assert_eq!(arena.payload(absent), None);
        assert!(arena.accepts(absent, Type::Null));
        assert!(arena.accepts(absent, Type::Undefined));
        assert!(!arena.accepts(Type::Null, absent));
        for payload in [
            Type::String,
            Type::Array(ArrayElement::Int(0)),
            Type::Record(0),
        ] {
            let union = intern(&arena, &[payload, Type::Undefined]);
            assert!(!union.is_copy());
            assert_eq!(arena.payload(union), Some(payload));
        }
        assert_ne!(
            intern(&arena, &[Type::Record(0), Type::Null]),
            intern(&arena, &[Type::Record(1), Type::Null])
        );
    }

    #[test]
    fn invalid_domains_do_not_create_arena_entries() {
        let arena = UnionArena::default();
        for members in [
            vec![],
            vec![Type::Void, Type::Undefined],
            vec![Type::Closure(0), Type::Callback(0), Type::Null],
            vec![Type::Callable(0), Type::StoredCallable(0), Type::Undefined],
            vec![Type::BorrowedCallable(0), Type::Int, Type::Null],
            vec![Type::BorrowedCallable(0), Type::BorrowedCallable(1), Type::Null],
        ] {
            assert!(arena.intern(members, Span::new(0, 1)).is_err());
        }
        assert!(arena.snapshot().is_empty());
        // Several data alternatives form one valid tagged union.
        assert!(matches!(
            arena.intern([Type::Int, Type::String, Type::Null], Span::new(0, 1)),
            Ok(Type::Union(_))
        ));
        assert_eq!(arena.snapshot().len(), 1);
    }

    #[test]
    fn nullable_callable_payloads_keep_identity_and_never_become_copy() {
        let arena = UnionArena::default();
        for payload in [
            Type::Closure(0),
            Type::Callable(0),
            Type::Callback(0),
            Type::StoredCallable(0),
            Type::BorrowedCallable(0),
        ] {
            let narrow = intern(&arena, &[payload, Type::Null]);
            let wide = intern(&arena, &[Type::Undefined, narrow]);
            assert_eq!(
                wide,
                intern(&arena, &[Type::Null, payload, Type::Undefined])
            );
            assert_eq!(arena.payload(narrow), Some(payload));
            assert_eq!(arena.payload(wide), Some(payload));
            assert!(!narrow.is_copy());
            assert!(!wide.is_copy());
            assert!(arena.accepts(wide, narrow));
            assert!(arena.accepts(wide, payload));
            assert!(!arena.accepts(narrow, wide));
        }
        assert_eq!(arena.snapshot().len(), 10);
    }

    #[test]
    fn closure_compaction_preserves_union_ids_and_unresolved_symbolic_members() {
        let arena = UnionArena::default();
        let first = intern(&arena, &[Type::Closure(5), Type::Null]);
        let second = intern(&arena, &[Type::Closure(7), Type::Undefined]);
        let symbolic = intern(&arena, &[Type::Closure(2), Type::Null]);
        let mut origins = vec![None; 8];
        origins[5] = Some(1);
        origins[7] = Some(0);
        arena.remap_closure_origins(&origins);
        assert_eq!(arena.payload(first), Some(Type::Closure(1)));
        assert_eq!(arena.payload(second), Some(Type::Closure(0)));
        assert_eq!(arena.payload(symbolic), Some(Type::Closure(2)));
        assert_eq!(first, intern(&arena, &[Type::Closure(1), Type::Null]));
        assert_eq!(second, intern(&arena, &[Type::Closure(0), Type::Undefined]));
        assert!(!first.is_copy());
        assert!(!second.is_copy());
        assert_eq!(arena.snapshot().len(), 3);
    }
}

#[cfg(test)]
mod resumed_arena_tests {
    use super::*;
    #[test]
    fn resumed_arena_preserves_existing_ids_and_appends_catch_unions() {
        let original=UnionArena::default();
        let span=Span::new(0,0);
        let first=original.intern([Type::String,Type::Int],span).unwrap();
        let second=original.intern([Type::Int,Type::Undefined],span).unwrap();
        let resumed=UnionArena::from_snapshot(original.snapshot());
        assert_eq!(resumed.snapshot().len(),2);
        assert_eq!(resumed.intern([Type::Int,Type::String],span).unwrap(),first);
        assert_eq!(resumed.intern([Type::Undefined,Type::Int],span).unwrap(),second);
        let added=resumed.intern([first,Type::Record(0)],span).unwrap();
        let Type::Union(id)=added else {panic!("union");};
        assert_eq!(id.index(),2);
        assert_eq!(resumed.snapshot().len(),3);
    }
}

#[cfg(test)]
mod scalar_array_storage_tests {
    use super::*;
    use crate::syntax::ArrayElement;
    #[test]
    fn scalar_union_storage_metadata_survives_snapshots_and_keeps_recursive_indirection() {
        let arena=UnionArena::default();
        let span=Span::default();
        let scalar=arena.intern([Type::Int,Type::String,Type::Null],span).unwrap();
        let id=scalar.union().unwrap();
        let array=Type::Array(ArrayElement::Union(id,0));
        assert!(crate::storage::inline_array(array));
        assert!(!scalar.is_copy());
        assert!(!crate::storage::inline_array(Type::Array(ArrayElement::Union(id,1))));
        assert!(crate::storage::array_storage_bound(ArrayElement::Union(id,0))>std::mem::size_of::<Vec<()>>());
        let resumed=UnionArena::from_snapshot(arena.snapshot());
        assert_eq!(resumed.intern([Type::Null,Type::Int,Type::String],span).unwrap(),scalar);
        for payload in [Type::Record(0),Type::Array(ArrayElement::Int(0)),Type::Closure(0)] {
            let union=resumed.intern([payload,Type::Null],span).unwrap().union().unwrap();
            assert!(!crate::storage::inline_array(Type::Array(ArrayElement::Union(union,0))));
        }
        let recursive=resumed.reserve_recursive(span).unwrap().union().unwrap();
        resumed.bind_recursive(recursive,vec![Type::Int,Type::Null],span).unwrap();
        assert!(!crate::storage::inline_array(Type::Array(ArrayElement::Union(recursive,0))));
    }
}
