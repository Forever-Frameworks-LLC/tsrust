//! Dense array lengths follow ownership paths independently of capture loans.
use super::Place;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape { Dense(usize), Unknown }

#[derive(Clone, Debug, Default)]
pub(super) struct Shapes(BTreeMap<Place, Shape>);

fn contains(root: &Place, place: &Place) -> bool {
    root.binding == place.binding && place.fields.starts_with(&root.fields)
}

impl Shapes {
    pub(super) fn record(&mut self, place: Place, length: usize) {
        self.0.entry(place).and_modify(|shape| {
            if *shape != Shape::Dense(length) { *shape = Shape::Unknown; }
        }).or_insert(Shape::Dense(length));
    }
    pub(super) fn length(&self, place: &Place) -> Option<usize> {
        match self.0.get(place) { Some(Shape::Dense(length)) => Some(*length), _ => None }
    }
    pub(super) fn set_length(&mut self, place: Place, length: usize) {
        self.0.insert(place, Shape::Dense(length));
    }
    pub(super) fn invalidate(&mut self, root: &Place) {
        for (place, shape) in &mut self.0 {
            if contains(root, place) { *shape = Shape::Unknown; }
        }
    }
    pub(super) fn discard(&mut self, root: &Place) {
        self.0.retain(|place, _| !contains(root, place));
    }
    pub(super) fn extend_from(&mut self, source: &Place, destination: &Place, consume: bool) {
        if source == destination { return; }
        let selected = self.0.iter().filter(|(place, _)| contains(source, place))
            .map(|(place, shape)| (place.clone(), *shape)).collect::<Vec<_>>();
        for (place, shape) in selected {
            if consume { self.0.remove(&place); }
            let mut fields = destination.fields.clone();
            fields.extend_from_slice(&place.fields[source.fields.len()..]);
            let place = Place { binding: destination.binding, fields };
            self.0.entry(place).and_modify(|previous| {
                if *previous != shape { *previous = Shape::Unknown; }
            }).or_insert(shape);
        }
    }
    pub(super) fn transfer(&mut self, source: &Place, destination: &Place) {
        if source == destination { return; }
        self.discard(destination);
        self.extend_from(source, destination, true);
    }
    pub(super) fn leave_scope(&mut self, first: usize) {
        self.0.retain(|place, _| place.binding < first);
    }
    pub(super) fn join(&mut self, other: &Self) {
        for (place, shape) in &mut self.0 {
            if other.0.get(place) != Some(shape) { *shape = Shape::Unknown; }
        }
        for place in other.0.keys() {
            self.0.entry(place.clone()).or_insert(Shape::Unknown);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root(binding: usize) -> Place { Place { binding, fields: vec![] } }
    #[test]
    fn moves_preserve_lengths_and_discard_destination_history() {
        let mut shapes = Shapes::default();
        shapes.record(root(1), 2);
        shapes.record(root(2), 5);
        shapes.transfer(&root(1), &root(2));
        assert_eq!(shapes.length(&root(1)), None);
        assert_eq!(shapes.length(&root(2)), Some(2));
        shapes.leave_scope(2);
        assert_eq!(shapes.length(&root(2)), None);
    }
    #[test]
    fn uncertainty_is_sticky_until_replacement() {
        let mut shapes = Shapes::default();
        shapes.record(root(1), 2);
        shapes.record(root(1), 3);
        shapes.record(root(1), 2);
        assert_eq!(shapes.length(&root(1)), None);
        shapes.discard(&root(1));
        shapes.record(root(1), 2);
        assert_eq!(shapes.length(&root(1)), Some(2));
        shapes.invalidate(&root(1));
        assert_eq!(shapes.length(&root(1)), None);
    }
    #[test]
    fn branch_only_or_different_lengths_cannot_select_a_slot() {
        let mut left = Shapes::default();
        left.record(root(1), 2);
        let mut right = left.clone();
        right.record(root(2), 3);
        left.join(&right);
        assert_eq!(left.length(&root(1)), Some(2));
        assert_eq!(left.length(&root(2)), None);
        right.discard(&root(1));
        right.record(root(1), 1);
        left.join(&right);
        assert_eq!(left.length(&root(1)), None);
    }
}
