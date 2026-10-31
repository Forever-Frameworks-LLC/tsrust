//! Admission policy for nested syntax.
//!
//! The parser keeps every suspended grammar position on the heap, and the
//! checker and emitter walk the tree on work stacks, so source nesting needs
//! no bound. The policy remains as the single place to reintroduce one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NestingPolicy {
    limit: usize,
}

impl NestingPolicy {
    /// Public entry points. Every consumer of the tree walks it without
    /// recursion, so source nesting is limited only by memory.
    pub(crate) const CONSUMERS: Self = Self { limit: usize::MAX };
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
