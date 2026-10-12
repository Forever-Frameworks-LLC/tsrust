impl<C: ReadCallback<A, R>, A, R> ReadCallback<A, R> for TsrSharedReadCallback<C> {
    fn invoke_read(&self, args: A) -> R { self.0.invoke_read(args) }
    fn invoke_read_with_count(&self, args: A, count: usize) -> R {
        self.0.invoke_read_with_count(args, count)
    }
}
impl<C: ReadCallback<A, R>, A, R> Callback<A, R> for TsrSharedReadCallback<C> {
    fn invoke(&mut self, args: A) -> R { self.0.invoke_read(args) }
    fn invoke_with_count(&mut self, args: A, count: usize) -> R {
        self.0.invoke_read_with_count(args, count)
    }
}
