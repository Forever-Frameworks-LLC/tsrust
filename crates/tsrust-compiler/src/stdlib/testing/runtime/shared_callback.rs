// Shared ownership of an immutable callback environment. Cloning this handle
// never clones its captures; the last handle releases the original environment.
struct TsrSharedReadCallback<C>(std::rc::Rc<C>);
impl<C> TsrSharedReadCallback<C> {
    fn new(callback: C) -> Self { Self(std::rc::Rc::new(callback)) }
}
impl<C> Clone for TsrSharedReadCallback<C> {
    fn clone(&self) -> Self { Self(std::rc::Rc::clone(&self.0)) }
}
