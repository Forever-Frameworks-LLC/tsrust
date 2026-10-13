//! Admission policy for nested syntax.
//!
//! The parser keeps every suspended grammar position on the heap, so it does
//! not need a nesting bound for its own safety. Later consumers of the syntax
//! tree still recurse. `CONSUMERS` keeps the documented source limits for
//! them at one boundary. `UNBOUNDED` parses the same grammar without limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NestingPolicy {
    limit: usize,
}

impl NestingPolicy {
    /// Documented source limits that protect recursive tree consumers.
    pub(crate) const CONSUMERS: Self = Self { limit: 128 };
    /// No nesting limit. Resource use grows with the input only.
    #[allow(dead_code)]
    pub(crate) const UNBOUNDED: Self = Self { limit: usize::MAX };

    pub(crate) fn bounded(self) -> bool {
        self.limit != usize::MAX
    }
    /// The bound applies to a depth that would open one more level.
    pub(crate) fn reached(self, depth: usize) -> bool {
        depth >= self.limit
    }
    /// The bound applies to a completed nesting measure.
    pub(crate) fn exceeded(self, depth: usize) -> bool {
        depth > self.limit
    }
}
