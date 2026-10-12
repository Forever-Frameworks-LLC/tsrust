//! Checked capture paths identify retained inputs inside concrete environments.
use super::*;

pub(super) fn mode(closures: &[CheckedClosure], mut ty: Type, path: &[usize]) -> Option<Mode> {
    let mut result = None;
    for &inner in path {
        let Type::Closure(origin) = ty else {
            return None;
        };
        let closure = closures.get(origin)?;
        let capture = closure
            .captures
            .iter()
            .find(|capture| capture.inner == inner)?;
        result = Some(capture.storage);
        ty = closure.function.bindings.get(inner)?.ty;
    }
    result
}

/// Unknown or cyclic paths retain their receiver; they never become direct inputs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Projection {
    #[default]
    Direct,
    Captures(Vec<usize>),
    Opaque,
}
impl Projection {
    pub(crate) fn is_direct(&self) -> bool {
        matches!(self, Self::Direct)
    }
    pub(crate) fn captures(&self) -> &[usize] {
        match self {
            Self::Captures(path) => path,
            _ => &[],
        }
    }
    pub(super) fn prepend(&mut self, inner: usize) {
        match self {
            Self::Direct => *self = Self::Captures(vec![inner]),
            Self::Captures(path) => path.insert(0, inner),
            Self::Opaque => {}
        }
    }
    pub(super) fn validate(&mut self, closures: &[CheckedClosure], ty: Type) {
        if let Self::Captures(path) = self
            && (path.len() > closures.len() || mode(closures, ty, path).is_none())
        {
            *self = Self::Opaque;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_environment_projection_stays_opaque_under_recursive_composition() {
        let mut projection = Projection::default();
        projection.prepend(7);
        assert_eq!(projection.captures(), &[7]);
        projection.validate(&[], Type::Int);
        assert_eq!(projection, Projection::Opaque);
        for _ in 0..256 {
            projection.prepend(9);
        }
        assert_eq!(projection, Projection::Opaque);
        assert!(!projection.is_direct());
    }
}
