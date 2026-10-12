//! Native owners can remain initialized after their source names become unusable.
use std::collections::BTreeMap;
use crate::Span;

#[derive(Clone)]
pub(super) struct Value {
    pub place: String,
    pub empty: String,
    /// Conversion moves the live owner; unavailable source bindings use empty.
    pub live: Option<String>,
    pub transferred: Option<Span>,
    pub target: Option<crate::syntax::Type>,
}

#[derive(Clone, Default)]
pub(super) struct State(BTreeMap<String, usize>);
impl State {
    pub fn renew(&mut self, places: &[String], boundary: usize) {
        for place in places { self.0.insert(place.clone(), boundary); }
    }
    pub fn available(&self, place: &str, transferred: Option<Span>) -> bool {
        transferred.is_none_or(|span| self.0.get(place).is_some_and(|boundary|span.end<=*boundary))
    }
    pub fn render(&self, values: &[Value]) -> String {
        if values.is_empty(){return "()".into();}
        format!("({},)", values.iter().map(|value| if self.available(&value.place,value.transferred) {value.live.as_deref().unwrap_or(value.place.as_str())} else {value.empty.as_str()}).collect::<Vec<_>>().join(","))
    }
}

/// Emission state follows callable and branch boundaries even without exceptions.
#[derive(Default)]
pub(super) struct Stack(std::cell::RefCell<State>);
pub(crate) struct Scope<'a> { stack: &'a Stack, previous: Option<State> }
impl Stack {
    pub fn scope(&self) -> Scope<'_> {
        Scope { stack: self, previous: Some(self.0.borrow().clone()) }
    }
    pub fn callable_boundary(&self) -> Scope<'_> {
        Scope { stack: self, previous: Some(std::mem::take(&mut *self.0.borrow_mut())) }
    }
    pub fn renew(&self, places: &[String], boundary: usize) { self.0.borrow_mut().renew(places,boundary); }
    pub fn available(&self, place: &str, transferred: Option<Span>) -> bool { self.0.borrow().available(place,transferred) }
    pub fn render(&self, values: &[Value]) -> String { self.0.borrow().render(values) }
}
impl Drop for Scope<'_> {
    fn drop(&mut self) { *self.stack.0.borrow_mut()=self.previous.take().expect("native availability scope"); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projected_storage_never_revives_a_transferred_source_owner() {
        let mut state=State::default();
        assert_eq!(state.render(&[]),"()");
        let value=Value{place:"owner".into(),empty:"Empty::new()".into(),live:Some("move_survivors(owner)".into()),transferred:Some(Span::new(10,20)),target:None};
        assert_eq!(state.render(std::slice::from_ref(&value)),"(Empty::new(),)");
        state.renew(&["owner".into()],19);
        assert_eq!(state.render(std::slice::from_ref(&value)),"(Empty::new(),)");
        state.renew(&["owner".into()],20);
        assert_eq!(state.render(std::slice::from_ref(&value)),"(move_survivors(owner),)");
        let mut ordinary=value;ordinary.live=None;
        assert_eq!(state.render(&[ordinary]),"(owner,)");
    }
}
