//! Ownership paths for retained capture loans, independent of checker dispatch.
use std::collections::{BTreeMap, BTreeSet};
use crate::Span;
#[path="map_entry_payloads.rs"]
mod map_entry_payloads;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Place {
    pub(crate) binding: usize,
    pub(crate) fields: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Owners {
    map_payloads: map_entry_payloads::Payloads,
    entries: BTreeMap<Place, BTreeSet<usize>>,
    pending: BTreeMap<(Place, usize, usize), BTreeSet<usize>>,
    tracked: BTreeSet<usize>,
    payload: BTreeSet<usize>,
}

impl Owners {
    pub(crate) fn attach_map_payload(&mut self, place: Place, ty: crate::syntax::Type) {
        self.map_payloads.attach(place,ty);
    }
    pub(crate) fn map_payloads(&self, place: &Place) -> std::collections::HashSet<crate::syntax::Type> {
        self.map_payloads.under(place)
    }
    pub(crate) fn tracked(&self) -> &BTreeSet<usize> { &self.tracked }

    pub(crate) fn replace_payload(&mut self, loans: impl IntoIterator<Item = usize>) {
        self.payload.clear();
        for loan in loans {
            self.payload.insert(loan);
        }
    }

    pub(crate) fn attach(&mut self, place: Place, loans: impl IntoIterator<Item = usize>) {
        let mut loans = loans.into_iter().peekable();
        if loans.peek().is_some() {
            let entry = self.entries.entry(place).or_default();
            for loan in loans {
                self.tracked.insert(loan);
                entry.insert(loan);
            }
        }
    }

    /// Merge a payload into a newly constructed destination without discarding
    /// alternatives already attached there. Borrowed aliases keep source owners.
    pub(crate) fn extend_from(&mut self, source: &Place, destination: &Place, consume: bool) {
        if source == destination {
            return;
        }
        self.map_payloads.extend_from(source,destination,consume);
        let selected = self
            .entries
            .iter()
            .filter(|(place, _)| {
                place.binding == source.binding && place.fields.starts_with(&source.fields)
            })
            .map(|(place, loans)| (place.clone(), loans.clone()))
            .collect::<Vec<_>>();
        for (place, loans) in selected {
            if consume {
                self.entries.remove(&place);
            }
            let mut fields = destination.fields.clone();
            fields.extend_from_slice(&place.fields[source.fields.len()..]);
            self.attach(
                Place {
                    binding: destination.binding,
                    fields,
                },
                loans,
            );
        }
    }

    pub(crate) fn begin_consumption(&mut self, source: &Place, site: Span) {
        let loans = self.loans_under(source);
        if !loans.is_empty() {
            self.pending.entry((source.clone(), site.start, site.end)).or_default().extend(loans);
        }
    }

    pub(crate) fn resolve_consumption(&mut self, source: &Place, site: Span, loans: &BTreeSet<usize>) {
        let key = (source.clone(), site.start, site.end);
        if let Some(pending) = self.pending.get_mut(&key) {
            pending.retain(|loan| !loans.contains(loan));
            if pending.is_empty() {
                self.pending.remove(&key);
            }
        }
    }

    pub(crate) fn payload(&self,source:&Place)->Vec<(Vec<String>,BTreeSet<usize>)>{
        self.entries.iter().filter(|(place,_)|place.binding==source.binding && place.fields.starts_with(&source.fields))
            .map(|(place,loans)|(place.fields[source.fields.len()..].to_vec(),loans.clone())).collect()
    }

    /// Discard this owner without removing loans held by other owners or payloads.
    pub(crate) fn discard(&mut self,source:&Place){
        self.map_payloads.discard(source);
        self.entries.retain(|place,_|place.binding!=source.binding || !place.fields.starts_with(&source.fields));
    }

    pub(crate) fn loans_under(&self, source: &Place) -> BTreeSet<usize> {
        self.entries.iter().filter(|(place, _)| place.binding == source.binding
            && place.fields.starts_with(&source.fields)).flat_map(|(_, loans)| loans.iter().copied()).collect()
    }

    pub(crate) fn live(&self) -> BTreeSet<usize> {
        self.entries.values().chain(self.pending.values()).flatten().copied().chain(self.payload.iter().copied()).collect()
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
        self.map_payloads.transfer(source,destination);
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
        self.map_payloads.leave_scope(first_binding);
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
        self.map_payloads.join(&other.map_payloads);
        self.tracked.extend(other.tracked.iter().copied());
        self.payload.extend(other.payload.iter().copied());
        for (place, loans) in &other.entries {
            self.attach(place.clone(), loans.iter().copied());
        }
        for (site, loans) in &other.pending {
            self.pending.entry(site.clone()).or_default().extend(loans.iter().copied());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn entry_payload_types_follow_moves_and_uncertain_deletion() {
        use crate::syntax::Type;
        let mut left=Owners::default();
        let root=place(1,&["handlers"]);
        let first=place(1,&["handlers","\0map-entry:string:first"]);
        let second=place(1,&["handlers","\0map-entry:string:second"]);
        left.attach(first.clone(),[10]);left.attach(second.clone(),[10]);
        left.attach_map_payload(first.clone(),Type::Record(17));
        left.attach_map_payload(second.clone(),Type::Record(23));
        let mut right=left.clone();right.discard(&first);
        left.join(&right);
        assert_eq!(left.map_payloads(&root),std::collections::HashSet::from([Type::Record(17),Type::Record(23)]));
        left.transfer(&root,&place(2,&[]));
        assert!(left.map_payloads(&root).is_empty());
        left.discard(&place(2,&["\0map-entry:string:first"]));
        assert_eq!(left.map_payloads(&place(2,&[])),std::collections::HashSet::from([Type::Record(23)]));
        assert_eq!(left.live(),BTreeSet::from([10]));
        assert_eq!(left.leave_scope(2),BTreeSet::from([10]));
        assert!(left.map_payloads(&place(2,&[])).is_empty());
    }
    fn place(binding: usize, fields: &[&str]) -> Place {
        Place {
            binding,
            fields: fields.iter().map(|name| (*name).to_owned()).collect(),
        }
    }
    #[test]
    fn disposed_and_branch_only_loans_remain_known_without_remaining_live() {
        let mut owners = Owners::default();
        owners.attach(place(1, &[]), [10]);
        let mut branch = owners.clone();
        branch.attach(place(2, &[]), [11]);
        branch.leave_scope(1);
        assert!(branch.live().is_empty());
        assert_eq!(branch.tracked(), &BTreeSet::from([10, 11]));
        owners.join(&branch);
        assert_eq!(owners.live(), BTreeSet::from([10]));
        assert_eq!(owners.tracked(), &BTreeSet::from([10, 11]));
        owners.leave_scope(1);
        assert_eq!(owners.tracked(), &BTreeSet::from([10, 11]));
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
    #[test]
    fn unresolved_consumption_survives_original_binding_disposal() {
        let mut owners = Owners::default();
        let source = place(1, &["run"]);
        owners.attach(source.clone(), [10]);
        owners.begin_consumption(&source, Span::new(10, 11));
        assert!(owners.leave_scope(1).is_empty());
        assert_eq!(owners.live(), BTreeSet::from([10]));
    }

    #[test]
    fn resolving_a_move_does_not_clear_other_consumption_sites() {
        let mut owners = Owners::default();
        let source = place(1, &["run"]);
        owners.attach(source.clone(), [10]);
        owners.begin_consumption(&source, Span::new(10, 11));
        owners.begin_consumption(&source, Span::new(20, 21));
        owners.resolve_consumption(&source, Span::new(20, 21), &BTreeSet::from([10]));
        assert!(owners.leave_scope(1).is_empty());
        assert_eq!(owners.live(), BTreeSet::from([10]));
        owners.resolve_consumption(&source, Span::new(10, 11), &BTreeSet::from([10]));
        assert!(owners.live().is_empty());
    }

    #[test]
    fn branch_join_preserves_unresolved_retention_after_known_move() {
        let mut left = Owners::default();
        let source = place(1, &["run"]);
        left.attach(source.clone(), [10]);
        left.begin_consumption(&source, Span::new(10, 11));
        let mut right = left.clone();
        right.resolve_consumption(&source, Span::new(10, 11), &BTreeSet::from([10]));
        left.join(&right);
        assert!(left.leave_scope(1).is_empty());
        assert_eq!(left.live(), BTreeSet::from([10]));
    }

}
