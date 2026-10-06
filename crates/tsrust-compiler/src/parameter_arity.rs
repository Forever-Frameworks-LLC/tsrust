//! Argument bounds shared by checked source callable contracts.

/// Defaults before required parameters still occupy an argument position.
/// Only the suffix after the last required position may be omitted.
pub(crate) fn minimum_arguments<P>(
    parameters: &[P],
    mut is_omittable: impl FnMut(&P) -> bool,
) -> usize {
    parameters
        .iter()
        .rposition(|parameter| !is_omittable(parameter))
        .map_or(0, |position| position + 1)
}
