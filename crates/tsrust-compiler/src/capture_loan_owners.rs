//! Ownership paths for retained capture loans, independent of checker dispatch.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Place {
    pub(crate) binding: usize,
    pub(crate) fields: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Owners {
    entries: BTreeMap<Place, BTreeSet<usize>>,
}

impl Owners {
    pub(crate) fn attach(&mut self, place: Place, loans: impl IntoIterator<Item = usize>) {
        self.entries.entry(place).or_default().extend(loans);
    }

    pub(crate) fn live(&self) -> BTreeSet<usize> {
        self.entries.values().flatten().copied().collect()
    }

    pub(crate) fn transfer(&mut self, source: &Place, destination: &Place) -> BTreeSet<usize> {
        if source == destination {
            return BTreeSet::new();
        }
        assert!(
            source.binding != destination.binding
                || (!source.fields.starts_with(&destination.fields)
                    && !destination.fields.starts_with(&source.fields)),
            "overlapping ownership transfers require a checked temporary"
        );
        let previous = self.live();
        let moved = self
            .entries
            .keys()
            .filter(|place| {
                place.binding == source.binding && place.fields.starts_with(&source.fields)
            })
            .cloned()
            .collect::<Vec<_>>();
        let moved = moved
            .into_iter()
            .map(|place| {
                let loans = self.entries.remove(&place).expect("selected owner");
                let mut fields = destination.fields.clone();
                fields.extend_from_slice(&place.fields[source.fields.len()..]);
                (
                    Place {
                        binding: destination.binding,
                        fields,
                    },
                    loans,
                )
            })
            .collect::<Vec<_>>();
        self.entries.retain(|place, _| {
            place.binding != destination.binding || !place.fields.starts_with(&destination.fields)
        });
        for (place, loans) in moved {
            self.attach(place, loans);
        }
        previous.difference(&self.live()).copied().collect()
    }

    pub(crate) fn leave_scope(&mut self, first_binding: usize) -> BTreeSet<usize> {
        let previous = self.live();
        self.entries
            .retain(|place, _| place.binding < first_binding);
        previous.difference(&self.live()).copied().collect()
    }

    pub(crate) fn reaching(left: Option<Self>, right: Option<Self>) -> Self {
        match (left, right) {
            (Some(mut left), Some(right)) => {
                left.join(&right);
                left
            }
            (Some(state), None) | (None, Some(state)) => state,
            (None, None) => Self::default(),
        }
    }

    pub(crate) fn join(&mut self, other: &Self) {
        for (place, loans) in &other.entries {
            self.attach(place.clone(), loans.iter().copied());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn place(binding: usize, fields: &[&str]) -> Place {
        Place {
            binding,
            fields: fields.iter().map(|name| (*name).to_owned()).collect(),
        }
    }
    #[test]
    fn extracted_field_disposal_releases_only_its_independent_loan() {
        let mut owners = Owners::default();
        owners.attach(place(1, &["left"]), [10]);
        owners.attach(place(1, &["right"]), [11]);
        assert!(
            owners
                .transfer(&place(1, &["left"]), &place(2, &[]))
                .is_empty()
        );
        assert_eq!(owners.leave_scope(2), BTreeSet::from([10]));
        assert_eq!(owners.live(), BTreeSet::from([11]));
    }
    #[test]
    fn nested_transfer_to_outer_storage_survives_scope_exit() {
        let mut owners = Owners::default();
        owners.attach(place(2, &["inner", "run"]), [10]);
        owners.transfer(&place(2, &[]), &place(1, &["saved"]));
        assert!(owners.leave_scope(2).is_empty());
        assert_eq!(
            owners.entries.get(&place(1, &["saved", "inner", "run"])),
            Some(&BTreeSet::from([10]))
        );
        assert_eq!(owners.leave_scope(1), BTreeSet::from([10]));
    }
    #[test]
    fn replacement_preserves_shared_loans_and_releases_only_dead_payloads() {
        let mut owners = Owners::default();
        owners.attach(place(1, &["run"]), [10, 11]);
        owners.attach(place(2, &[]), [12]);
        owners.attach(place(0, &[]), [11]);
        assert_eq!(
            owners.transfer(&place(2, &[]), &place(1, &["run"])),
            BTreeSet::from([10])
        );
        assert_eq!(owners.live(), BTreeSet::from([11, 12]));
    }
    #[test]
    fn branch_join_keeps_every_reaching_live_owner() {
        let mut left = Owners::default();
        left.attach(place(1, &["left"]), [10]);
        let mut right = Owners::default();
        right.attach(place(2, &["right"]), [10, 11]);
        left.join(&right);
        assert_eq!(left.leave_scope(2), BTreeSet::from([11]));
        assert_eq!(left.live(), BTreeSet::from([10]));
        assert!(
            left.transfer(&place(1, &["left"]), &place(1, &["left"]))
                .is_empty()
        );
    }
    #[test]
    fn abrupt_arms_do_not_contribute_to_normal_continuation() {
        let mut left = Owners::default();
        left.attach(place(0, &[]), [10]);
        let mut right = Owners::default();
        right.attach(place(1, &[]), [11]);
        assert_eq!(
            Owners::reaching(Some(left.clone()), Some(right.clone())).live(),
            BTreeSet::from([10, 11])
        );
        assert_eq!(
            Owners::reaching(Some(left.clone()), None).live(),
            BTreeSet::from([10])
        );
        assert_eq!(
            Owners::reaching(None, Some(right)).live(),
            BTreeSet::from([11])
        );
        assert!(Owners::reaching(None, None).live().is_empty());
    }
}
