//! Point-sensitive payload alternatives move with the owning storage path.
use super::Place;
use crate::syntax::Type;
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Payloads(BTreeMap<Place, HashSet<Type>>);

impl Payloads {
    pub(super) fn attach(&mut self, place: Place, ty: Type) {
        self.0.entry(place).or_default().insert(ty);
    }
    pub(super) fn under(&self, root: &Place) -> HashSet<Type> {
        self.0
            .iter()
            .filter(|(place, _)| {
                contains(root, place)
                    && (place.fields == root.fields
                        || (place.fields.len() == root.fields.len() + 1
                            && place
                                .fields
                                .last()
                                .is_some_and(|field| field.starts_with("\0map-entry:"))))
            })
            .flat_map(|(_, types)| types.iter().copied())
            .collect()
    }
    pub(super) fn snapshot(&self)->BTreeMap<Place,HashSet<Type>> {
        let mut roots=BTreeMap::<Place,HashSet<Type>>::new();
        for (place,types) in &self.0 {
            let mut root=place.clone();
            if root.fields.last().is_some_and(|field|field.starts_with("\0map-entry:")){root.fields.pop();}
            roots.entry(root).or_default().extend(types.iter().copied());
        }
        roots
    }
    pub(super) fn discard(&mut self, root: &Place) {
        self.0.retain(|place, _| !contains(root, place));
    }
    pub(super) fn extend_from(&mut self, source: &Place, destination: &Place, consume: bool) {
        if source == destination {
            return;
        }
        let selected = self
            .0
            .iter()
            .filter(|(place, _)| contains(source, place))
            .map(|(place, types)| (place.clone(), types.clone()))
            .collect::<Vec<_>>();
        for (place, types) in selected {
            if consume {
                self.0.remove(&place);
            }
            let mut fields = destination.fields.clone();
            fields.extend_from_slice(&place.fields[source.fields.len()..]);
            self.0
                .entry(Place {
                    binding: destination.binding,
                    fields,
                })
                .or_default()
                .extend(types);
        }
    }
    pub(super) fn transfer(&mut self, source: &Place, destination: &Place) {
        if source == destination {
            return;
        }
        let mut moved = Self::default();
        moved.0 = self
            .0
            .iter()
            .filter(|(place, _)| contains(source, place))
            .map(|(place, types)| (place.clone(), types.clone()))
            .collect();
        moved.extend_from(source, destination, true);
        self.discard(source);
        self.discard(destination);
        self.join(&moved);
    }
    pub(super) fn leave_scope(&mut self, first_binding: usize) {
        self.0.retain(|place, _| place.binding < first_binding);
    }
    pub(super) fn join(&mut self, other: &Self) {
        for (place, types) in &other.0 {
            self.0
                .entry(place.clone())
                .or_default()
                .extend(types.iter().copied());
        }
    }
}

fn contains(root: &Place, place: &Place) -> bool {
    place.binding == root.binding && place.fields.starts_with(&root.fields)
}
