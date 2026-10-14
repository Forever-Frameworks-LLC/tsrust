//! Borrow source declarations; scope transport must not clone descendant trees.
use super::Stmt;
#[derive(Clone, Copy)]
pub(super) enum Declarations<'a, 'v> {
    Body(&'a [Stmt]),
    Visible(&'v [&'a Stmt]),
}
impl<'a, 'v> Declarations<'a, 'v> {
    pub(super) fn iter(self) -> impl DoubleEndedIterator<Item = &'a Stmt> {
        let (body, visible) = match self { Self::Body(body) => (body, &[][..]), Self::Visible(visible) => (&[][..], visible) };
        body.iter().chain(visible.iter().copied())
    }
    pub(super) fn prefix(self, count: usize) -> Self {
        match self { Self::Body(body) => Self::Body(&body[..count]), Self::Visible(visible) => Self::Visible(&visible[..count]) }
    }
}
