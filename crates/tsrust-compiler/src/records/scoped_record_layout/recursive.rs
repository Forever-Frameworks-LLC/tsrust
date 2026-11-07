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

    pub(crate) fn materialize_recursive(
        &mut self,
        base: usize,
        types: Vec<Type>,
        records: &mut Vec<Record>,
        unions: &UnionArena,
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
            self.recursive_parameters.entry(declaration).or_default().insert(instance);
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
