mod tsrust_native_signals {
    #[derive(Debug)] pub struct Error{pub code:&'static str,pub message:String}
    pub struct Listener{
        #[cfg(unix)] interrupt:tokio::signal::unix::Signal,
        #[cfg(unix)] terminate:tokio::signal::unix::Signal,
    }
    pub fn listen()->Result<Listener,Error>{
        #[cfg(unix)] {
            if tokio::runtime::Handle::try_current().is_err(){return Err(Error{code:"runtime",message:"Signal registration requires an async runtime".into()});}
            use tokio::signal::unix::{signal,SignalKind};
            let failure=|_|Error{code:"signal",message:"Cannot register shutdown signals".into()};
            Ok(Listener{interrupt:signal(SignalKind::interrupt()).map_err(failure)?,terminate:signal(SignalKind::terminate()).map_err(failure)?})
        }
        #[cfg(not(unix))] {Err(Error{code:"unsupported",message:"Shutdown signals require a Unix target".into()})}
    }
    impl Listener{
        pub async fn receive(&mut self)->Result<String,Error>{
            #[cfg(unix)] {std::future::poll_fn(|cx|{
                use std::task::Poll;
                for (signal,name) in [(&mut self.interrupt,"SIGINT"),(&mut self.terminate,"SIGTERM")]{
                    if let Poll::Ready(value)=signal.poll_recv(cx){return Poll::Ready(value.map(|_|name.to_owned()).ok_or_else(||Error{code:"closed",message:"Signal listener closed".into()}));}
                }Poll::Pending
            }).await}
            #[cfg(not(unix))] {Err(Error{code:"unsupported",message:"Shutdown signals require a Unix target".into()})}
        }
    }
}
