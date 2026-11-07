//! Finite specializations retain recursive edges within their concrete component.
use super::Registry;
use crate::{Diagnostic, Span, syntax::{ArrayElement, Record, Type}, unions::UnionArena};
use std::collections::{HashMap, HashSet};

impl Registry {
    pub(crate) fn recursive_parameter(&self, ty: Type) -> Type {
        let Type::Record(base) = ty else { return ty; };
        if self.declaration(base) != base { return ty; }
        let mut instances = self.recursive_parameters.get(&base).into_iter().flatten().copied();
        let Some(instance) = instances.next() else { return ty; };
        if instances.all(|other| other == instance) { Type::Record(instance) } else { ty }
    }

    /// Empty recursive storage has no runtime instance. Intern its complete
    /// component field plan separately from ordinary root-only specialization.
    pub(crate) fn materialize_empty_component(&mut self,mut plans:Vec<(usize,Vec<Type>)>,records:&mut Vec<Record>,unions:&UnionArena)->Result<HashMap<usize,usize>,Diagnostic>{
        plans.sort_by_key(|(id,_)|*id);
        assert!(plans.windows(2).all(|pair|pair[0].0!=pair[1].0),"one empty mode per recursive component member");
        let signature=plans.iter().flat_map(|(id,types)|std::iter::once(Type::Record(*id)).chain(types.iter().copied())).collect::<Vec<_>>();
        for(id,_)in &plans{assert!(signature.len()>records[*id].fields.len(),"component signatures cannot alias ordinary field keys");}
        let key=(plans[0].0,signature);
        // The sorted component is reserved contiguously in the append-only
        // record arena. One signature key suffices for every entry member.
        if let Some(&first)=self.recursive_instances.get(&key){return Ok(plans.iter().enumerate().map(|(offset,(id,_))|(*id,first+offset)).collect());}
        let first=records.len();let remap=plans.iter().enumerate().map(|(offset,(id,_))|(*id,first+offset)).collect::<HashMap<_,_>>();
        let mut instances=Vec::new();
        for(id,types)in &plans{
            let mut record=records[*id].clone();
            if let Some(map)=&mut record.map_definition{assert_eq!(types.len(),2);map.key=remap_type(types[0],&remap,unions)?;map.value=remap_type(types[1],&remap,unions)?;map.entry=None;}
            else{assert_eq!(types.len(),record.fields.len());for(field,ty)in record.fields.iter_mut().zip(types){field.ty=Some(remap_type(*ty,&remap,unions)?);}}
            if let Some(application)=&mut record.application{if let Some(map)=&record.map_definition{application.resolved_arguments=vec![map.key,map.value];}else{for ty in &mut application.resolved_arguments{*ty=remap_type(*ty,&remap,unions)?;}}}
            instances.push(record);
        }
        records.extend(instances);
        for(id,_)in plans{let instance=remap[&id];self.bases.insert(instance,self.declaration(id));}
        self.recursive_instances.insert(key,first);
        // Absence-only layouts are not live parameter candidates. Root-only
        // matching must not confuse them with a differently planned component.
        Ok(remap)
    }

    // A mutual component must retain every supplied concrete member's fields,
    // not just its entry record. Normalize links only after collecting those
    // plans; otherwise an inhabited callback environment becomes its declaration.
    fn materialize_recursive_member_plans(&mut self,base:usize,types:Vec<Type>,component:&[usize],records:&mut Vec<Record>,unions:&UnionArena,absent:&dyn Fn(usize)->bool)->Result<usize,Diagnostic>{
        let mut plans=HashMap::from([(base,types.clone())]);let mut aliases=HashMap::new();
        let mut pending=std::collections::VecDeque::from(types);let mut seen=HashSet::new();
        while let Some(ty)=pending.pop_front(){
            if !seen.insert(ty){continue;}
            match ty {
                Type::Record(source)=>{
                    let declaration=self.declaration(source);
                    if !component.contains(&declaration) || plans.contains_key(&source){continue;}
                    // An absence-only owned callback has a compiler-created
                    // Callback placeholder where the declaration owns storage.
                    // The supplied entry plan dominates that ghost backedge.
                    let ghost=declaration==base && records[source].fields.iter().zip(&records[base].fields).any(|(actual,declared)|{
                        let actual=actual.ty.expect("checked component field");let declared=declared.ty.expect("checked component field");
                        matches!(actual,Type::Callback(id) if absent(id)) && matches!(unions.payload(declared).unwrap_or(declared),Type::StoredCallable(_))
                    });
                    if ghost{aliases.insert(source,base);continue;}
                    // Distinct inhabited instances remain distinct nodes even
                    // when they share a declaration. Never first-win their fields.
                    let fields=records[source].fields.iter().map(|field|field.ty.expect("checked component field")).collect::<Vec<_>>();
                    pending.extend(fields.iter().copied());plans.insert(source,fields);
                }
                Type::Array(element)=>pending.push_back(element.with_depth(0).ty()),
                Type::Union(id)|Type::BorrowedUnion(id)=>pending.extend(unions.members(id)),
                _=>{},
            }
        }
        let fields=plans.into_iter().map(|(id,types)|Ok((id,types.into_iter().map(|ty|remap_type(ty,&aliases,unions)).collect::<Result<Vec<_>,Diagnostic>>()?))).collect::<Result<Vec<_>,Diagnostic>>()?;
        // Reuse the existing full-component cache/reservation/remapping path.
        // A root-only key would alias different inhabited sibling environments.
        let candidates=self.recursive_parameters.get(&base).into_iter().flatten().copied().collect::<Vec<_>>();
        for candidate in candidates{if self.recursive_member_plans_match(base,candidate,&fields,records,unions){return Ok(candidate);}}
        let remap=self.materialize_empty_component(fields,records,unions)?;
        for(&original,&instance)in &remap{
            // Required absent callbacks make this member uninhabited. Keep its
            // layout for links, but never select it for a constructor or result.
            // An optional absent callback does not make its record uninhabited.
            if records[instance].fields.iter().any(|field|!field.optional && matches!(field.ty,Some(Type::Callback(id)) if absent(id))){continue;}
            let declaration=self.declaration(original);self.recursive_parameters.entry(declaration).or_default().insert(instance);
        }
        Ok(remap[&base])
    }

    fn recursive_member_plans_match(&self,base:usize,target:usize,fields:&[(usize,Vec<Type>)],records:&[Record],unions:&UnionArena)->bool{
        let plans=fields.iter().map(|(id,types)|(*id,types.as_slice())).collect::<HashMap<_,_>>();
        let mut pending=vec![(Type::Record(base),Type::Record(target))];let mut seen=HashSet::new();
        while let Some((source,target))=pending.pop(){
            if !seen.insert((source,target)){continue;}
            if source==target && !matches!(source,Type::Record(id) if plans.contains_key(&id)){continue;}
            match(source,target){
                (Type::Record(a),Type::Record(b)) if self.declaration(a)==self.declaration(b)=>{
                    if records[a].application.as_ref().map(|application|&application.resolved_arguments)!=records[b].application.as_ref().map(|application|&application.resolved_arguments){return false;}
                    let source=plans.get(&a).map(|types|types.to_vec()).unwrap_or_else(||records[a].fields.iter().map(|field|field.ty.unwrap()).collect());
                    if source.len()!=records[b].fields.len(){return false;}
                    pending.extend(source.into_iter().zip(records[b].fields.iter().map(|field|field.ty.unwrap())));
                }
                (Type::Array(a),Type::Array(b)) if a.depth()==b.depth()=>pending.push((a.with_depth(0).ty(),b.with_depth(0).ty())),
                (Type::Union(a),Type::Union(b))|(Type::BorrowedUnion(a),Type::BorrowedUnion(b))=>{
                    let source=unions.members(a);let target=unions.members(b);if source.len()!=target.len(){return false;}
                    let mut used=HashSet::new();
                    for member in source{
                        let selected=target.iter().position(|&value|value==member).or_else(||{
                            let Type::Record(id)=member else{return None;};
                            let mut candidates=target.iter().enumerate().filter(|(_,value)|matches!(value,Type::Record(other) if self.declaration(*other)==self.declaration(id)));
                            let (index,_)=candidates.next()?;candidates.next().is_none().then_some(index)
                        });
                        let Some(index)=selected else{return false;};if !used.insert(index){return false;}pending.push((member,target[index]));
                    }
                }
                _=>return false,
            }
        }true
    }

    pub(crate) fn materialize_recursive(
        &mut self,
        base: usize,
        types: Vec<Type>,
        records: &mut Vec<Record>,
        unions: &UnionArena,
        absent: &dyn Fn(usize)->bool,
    ) -> Result<usize, Diagnostic> {
        let base = self.declaration(base);
        if let Some(&id) = self.recursive_instances.get(&(base, types.clone())) { return Ok(id); }
        let mut edges = vec![Vec::new(); records.len()];
        let mut reverse = vec![Vec::new(); records.len()];
        for (id, record) in records.iter().enumerate() {
            let mut pending = record.fields.iter().filter_map(|field| field.ty).collect::<Vec<_>>();
            let mut seen = HashSet::new();
            while let Some(ty) = pending.pop() {
                if !seen.insert(ty) { continue; }
                match ty {
                    Type::Record(child) => { edges[id].push(child); reverse[child].push(id); }
                    Type::Array(element) => pending.push(element.with_depth(0).ty()),
                    Type::Union(union) | Type::BorrowedUnion(union) => pending.extend(unions.members(union)),
                    _ => {}
                }
            }
        }
        let reach = |graph: &[Vec<usize>]| {
            let mut seen = HashSet::new();
            let mut pending = vec![base];
            while let Some(id) = pending.pop() {
                if seen.insert(id) { pending.extend(graph[id].iter().copied()); }
            }
            seen
        };
        let forward = reach(&edges);
        let back = reach(&reverse);
        let mut component = forward.intersection(&back).copied().collect::<Vec<_>>();
        component.sort_unstable();
        if component.len() == 1 && !edges[base].contains(&base) {
            return Ok(self.materialize(base, types, records));
        }
        if component.len()>1 {
            return self.materialize_recursive_member_plans(base,types,&component,records,unions,absent);
        }
        // A literal checked against a concrete parameter already names the
        // component's concrete links. Normalize those links before lookup so
        // it reuses the same finite layout as a nominally annotated literal.
        let declarations = self.bases.iter().filter_map(|(&instance, &declaration)|
            component.contains(&declaration).then_some((instance, declaration))).collect();
        let types = types.into_iter().map(|ty| remap_type(ty, &declarations, unions)).collect::<Result<Vec<_>, _>>()?;
        let key = (base, types);
        if let Some(&id) = self.recursive_instances.get(&key) { return Ok(id); }
        // Another member may have materialized this component already. Reuse
        // its matching record instead of cloning a second, symbolic component.
        let mut matching = Vec::new();
        for &instance in self.recursive_parameters.get(&base).into_iter().flatten() {
            let fields = records[instance].fields.iter().map(|field|
                remap_type(field.ty.expect("concrete field"), &declarations, unions)).collect::<Result<Vec<_>, _>>()?;
            if fields == key.1 { matching.push(instance); }
        }
        if let [instance] = matching.as_slice() {
            self.recursive_instances.insert(key, *instance);
            return Ok(*instance);
        }
        // Reserve identities before rewriting any edge. No recursively expanded
        // record or callback environment is needed, even for mutually recursive types.
        let first = records.len();
        let remap = component.iter().enumerate().map(|(offset, &id)| (id, first + offset)).collect::<HashMap<_, _>>();
        let mut instances = Vec::new();
        for &id in &component {
            let mut record = records[id].clone();
            if id == base {
                assert_eq!(record.fields.len(), key.1.len(), "checked concrete field count");
                for (field, &ty) in record.fields.iter_mut().zip(&key.1) { field.ty = Some(ty); }
            }
            for field in &mut record.fields {
                if let Some(ty) = field.ty { field.ty = Some(remap_type(ty, &remap, unions)?); }
            }
            instances.push(record);
        }
        records.extend(instances);
        for (&original, &instance) in &remap {
            let declaration = self.declaration(original);
            self.bases.insert(instance, declaration);
            if !records[instance].fields.iter().any(|field|!field.optional && matches!(field.ty,Some(Type::Callback(id)) if absent(id))) {
                self.recursive_parameters.entry(declaration).or_default().insert(instance);
            }
        }
        let instance = remap[&base];
        self.recursive_instances.insert(key, instance);
        Ok(instance)
    }
}

fn remap_type(ty: Type, records: &HashMap<usize, usize>, unions: &UnionArena) -> Result<Type, Diagnostic> {
    Ok(match ty {
        Type::Record(id) => Type::Record(records.get(&id).copied().unwrap_or(id)),
        Type::Map(id) => Type::Map(records.get(&id).copied().unwrap_or(id)),
        Type::Array(element) => {
            let leaf = remap_type(element.with_depth(0).ty(), records, unions)?;
            Type::Array(ArrayElement::from_value_type(leaf).expect("preserved array leaf").with_depth(element.depth()))
        }
        Type::Union(id) | Type::BorrowedUnion(id) => {
            let members = unions.members(id).into_iter().map(|member| remap_type(member, records, unions)).collect::<Result<Vec<_>, _>>()?;
            let mapped = unions.intern(members, Span::default())?;
            if matches!(ty, Type::BorrowedUnion(_)) { Type::BorrowedUnion(mapped.union().expect("preserved union alternatives")) } else { mapped }
        }
        _ => ty,
    })
}

#[cfg(test)]
mod member_plan_tests {
    use super::*;
    #[test]
    fn mutual_specialization_preserves_inhabited_siblings_and_root_precedence(){
        let unions=UnionArena::default();let span=Span::default();
        let first_link=unions.intern([Type::Record(0),Type::Undefined],span).unwrap();
        let second_link=unions.intern([Type::Record(1),Type::Undefined],span).unwrap();
        let mut records=vec![crate::cloneability::tests::record(&[Type::StoredCallable(0),second_link]),crate::cloneability::tests::record(&[Type::StoredCallable(0),first_link])];
        let mut registry=Registry::default();
        let ghost=registry.materialize(0,vec![Type::Callback(1),second_link],&mut records);
        let ghost_link=unions.intern([Type::Record(ghost),Type::Undefined],span).unwrap();
        let sibling=registry.materialize(1,vec![Type::StoredCallable(149),ghost_link],&mut records);
        let sibling_link=unions.intern([Type::Record(sibling),Type::Undefined],span).unwrap();
        let fields=vec![Type::StoredCallable(148),sibling_link];
        let first=registry.materialize_recursive(0,fields.clone(),&mut records,&unions,&|id|id==1).unwrap();
        let Type::Record(second)=unions.payload(records[first].fields[1].ty.unwrap()).unwrap()else{panic!("second member")};
        assert_eq!(records[second].fields[0].ty,Some(Type::StoredCallable(149)));
        let Type::Record(back)=unions.payload(records[second].fields[1].ty.unwrap()).unwrap()else{panic!("first backedge")};
        assert_eq!(back,first);assert_eq!(records[back].fields[0].ty,Some(Type::StoredCallable(148)));
        assert_eq!(registry.materialize_recursive(0,fields,&mut records,&unions,&|id|id==1).unwrap(),first);
        let concrete_fields=records[first].fields.iter().map(|field|field.ty.unwrap()).collect();
        let count=records.len();assert_eq!(registry.materialize_recursive(first,concrete_fields,&mut records,&unions,&|id|id==1).unwrap(),first);assert_eq!(records.len(),count,"equivalent concrete component is a fixed point");
        let other=registry.materialize(1,vec![Type::StoredCallable(150),ghost_link],&mut records);
        let other_link=unions.intern([Type::Record(other),Type::Undefined],span).unwrap();
        let distinct=registry.materialize_recursive(0,vec![Type::StoredCallable(148),other_link],&mut records,&unions,&|id|id==1).unwrap();assert_ne!(distinct,first,"full component key retains sibling environments");
    }
    #[test]
    fn optional_absent_callback_member_keeps_both_inhabited_sibling_environments(){
        let unions=UnionArena::default();let span=Span::default();
        let optional=|ty|unions.intern([ty,Type::Undefined],span).unwrap();
        let first_link=optional(Type::Record(0));let second_link=optional(Type::Record(1));
        let mut records=vec![crate::cloneability::tests::record(&[optional(Type::StoredCallable(0)),second_link]),crate::cloneability::tests::record(&[Type::StoredCallable(0),first_link])];
        let mut registry=Registry::default();
        let older=registry.materialize(1,vec![Type::StoredCallable(149),first_link],&mut records);
        let ghost=registry.materialize(0,vec![optional(Type::Callback(1)),optional(Type::Record(older))],&mut records);
        let newer=registry.materialize(1,vec![Type::StoredCallable(150),optional(Type::Record(ghost))],&mut records);
        let root=registry.materialize_recursive(0,vec![optional(Type::StoredCallable(148)),optional(Type::Record(newer))],&mut records,&unions,&|id|id==1).unwrap();
        let mut pending=vec![Type::Record(root)];let mut seen=HashSet::new();let mut families=HashSet::new();let mut retained_optional_ghost=false;
        while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}match ty{
            Type::Record(id)=>pending.extend(records[id].fields.iter().map(|field|field.ty.unwrap())),
            Type::Union(id)=>pending.extend(unions.members(id)),
            Type::StoredCallable(id)=>{families.insert(id);},
            Type::Callback(1)=>retained_optional_ghost=true,
            _=>{},
        }}
        assert!(families.contains(&148));assert!(families.contains(&149));assert!(families.contains(&150));assert!(retained_optional_ghost,"optional ghost is an inhabited container, not an absent required record");
    }
    #[test]
    fn required_absent_sibling_is_not_a_live_constructor_or_result_candidate(){
        let unions=UnionArena::default();let span=Span::default();
        let first_link=unions.intern([Type::Record(0),Type::Undefined],span).unwrap();
        let second_link=unions.intern([Type::Record(1),Type::Undefined],span).unwrap();
        let mut records=vec![crate::cloneability::tests::record(&[Type::StoredCallable(0),second_link]),crate::cloneability::tests::record(&[Type::StoredCallable(0),first_link])];
        let mut registry=Registry::default();
        let ghost=registry.materialize(1,vec![Type::Callback(72),first_link],&mut records);
        let ghost_link=unions.intern([Type::Record(ghost),Type::Undefined],span).unwrap();
        let first=registry.materialize_recursive(0,vec![Type::StoredCallable(148),ghost_link],&mut records,&unions,&|id|id==72).unwrap();
        let Type::Record(second)=unions.payload(records[first].fields[1].ty.unwrap()).unwrap()else{panic!("empty sibling layout retained")};
        assert_eq!(records[second].fields[0].ty,Some(Type::Callback(72)));
        assert_eq!(registry.recursive_parameter(Type::Record(0)),Type::Record(first));
        assert_eq!(registry.recursive_parameter(Type::Record(1)),Type::Record(1),"later Second constructors/results must not select Callback72");
        let live=registry.materialize_recursive(1,vec![Type::StoredCallable(149),first_link],&mut records,&unions,&|id|id==72).unwrap();
        assert_eq!(records[live].fields[0].ty,Some(Type::StoredCallable(149)));
        assert_eq!(registry.recursive_parameter(Type::Record(1)),Type::Record(live));
    }
}
