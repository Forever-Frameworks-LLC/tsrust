//! Coverage of a closed variant family, independent of payload representation.

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CoverageError {
    UnknownVariant,
    Unreachable,
}

pub(crate) struct Coverage {
    covered: Vec<bool>,
    remaining: usize,
}

impl Coverage {
    pub(crate) fn new(variant_count: usize) -> Self {
        Self { covered: vec![false; variant_count], remaining: variant_count }
    }

    /// `None` is an unconditional wildcard; variant indices come from the checked
    /// type's declaration order, never from source spelling or runtime payloads.
    pub(crate) fn add(&mut self, variant: Option<usize>) -> Result<(), CoverageError> {
        if let Some(index) = variant {
            let slot = self.covered.get_mut(index).ok_or(CoverageError::UnknownVariant)?;
            if *slot { return Err(CoverageError::Unreachable); }
            *slot = true;
            self.remaining -= 1;
        } else {
            if self.remaining == 0 { return Err(CoverageError::Unreachable); }
            self.covered.fill(true);
            self.remaining = 0;
        }
        Ok(())
    }

    pub(crate) fn missing(&self) -> Vec<usize> {
        self.covered.iter().enumerate().filter_map(|(index, covered)| (!covered).then_some(index)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_requires_each_variant_and_reports_missing_in_declaration_order() {
        let mut coverage = Coverage::new(3);
        assert_eq!(coverage.missing(), vec![0, 1, 2]);
        assert_eq!(coverage.add(Some(1)), Ok(()));
        assert_eq!(coverage.missing(), vec![0, 2]);
        assert_eq!(coverage.add(Some(0)), Ok(()));
        assert_eq!(coverage.add(Some(2)), Ok(()));
        assert!(coverage.missing().is_empty());
    }

    #[test]
    fn duplicate_variants_and_arms_after_complete_coverage_are_unreachable() {
        let mut coverage = Coverage::new(2);
        assert_eq!(coverage.add(Some(0)), Ok(()));
        assert_eq!(coverage.add(Some(0)), Err(CoverageError::Unreachable));
        assert_eq!(coverage.missing(), vec![1]);
        assert_eq!(coverage.add(Some(1)), Ok(()));
        assert_eq!(coverage.add(None), Err(CoverageError::Unreachable));
    }

    #[test]
    fn wildcard_covers_only_remaining_variants_and_rejects_following_arms() {
        let mut coverage = Coverage::new(2);
        assert_eq!(coverage.add(Some(0)), Ok(()));
        assert_eq!(coverage.add(None), Ok(()));
        assert!(coverage.missing().is_empty());
        assert_eq!(coverage.add(Some(1)), Err(CoverageError::Unreachable));
        assert_eq!(coverage.add(None), Err(CoverageError::Unreachable));
    }

    #[test]
    fn invalid_variant_does_not_change_coverage_or_panic() {
        let mut coverage = Coverage::new(2);
        assert_eq!(coverage.add(Some(2)), Err(CoverageError::UnknownVariant));
        assert_eq!(coverage.add(Some(usize::MAX)), Err(CoverageError::UnknownVariant));
        assert_eq!(coverage.missing(), vec![0, 1]);
    }
}
