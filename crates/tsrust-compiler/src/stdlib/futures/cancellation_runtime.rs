mod tsrust_native_cancellation {
    pub struct CancellationToken(tokio_util::sync::CancellationToken);
    impl CancellationToken {
        pub fn new()->Self{Self(tokio_util::sync::CancellationToken::new())}
        pub fn share(&self)->Self{Self(self.0.clone())}
        pub fn child(&self)->Self{Self(self.0.child_token())}
        pub fn cancel(&self){self.0.cancel();}
        pub fn is_cancelled(&self)->bool{self.0.is_cancelled()}
        pub async fn cancelled(&self){self.0.cancelled().await;}
    }
}
