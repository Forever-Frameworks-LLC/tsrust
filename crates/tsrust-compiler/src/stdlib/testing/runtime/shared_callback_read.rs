impl<C: ReadCallback<A, R>, A, R> ReadCallback<A, R> for TsrSharedReadCallback<C> {
    fn invoke_read(&self, args: A) -> R { self.0.invoke_read(args) }
}
impl<C: ReadCallback<A, R>, A, R> Callback<A, R> for TsrSharedReadCallback<C> {
    fn invoke(&mut self, args: A) -> R { self.0.invoke_read(args) }
}
