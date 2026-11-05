mod tsrust_native_signals {
    #[derive(Debug)] pub struct Error{pub code:&'static str,pub message:String}
    pub struct Listener{
        #[cfg(unix)] interrupt:tokio::signal::unix::Signal,
        #[cfg(unix)] terminate:tokio::signal::unix::Signal,
        #[cfg(windows)] interrupt:tokio::signal::windows::CtrlC,
        #[cfg(windows)] control_break:tokio::signal::windows::CtrlBreak,
    }
    pub fn listen()->Result<Listener,Error>{
        #[cfg(unix)] {
            if tokio::runtime::Handle::try_current().is_err(){return Err(Error{code:"runtime",message:"Signal registration requires an async runtime".into()});}
            use tokio::signal::unix::{signal,SignalKind};
            let failure=|_|Error{code:"signal",message:"Cannot register shutdown signals".into()};
            Ok(Listener{interrupt:signal(SignalKind::interrupt()).map_err(failure)?,terminate:signal(SignalKind::terminate()).map_err(failure)?})
        }
        #[cfg(windows)] {
            if tokio::runtime::Handle::try_current().is_err(){return Err(Error{code:"runtime",message:"Signal registration requires an async runtime".into()});}
            use tokio::signal::windows::{ctrl_c,ctrl_break};
            let failure=|_|Error{code:"signal",message:"Cannot register shutdown signals".into()};
            Ok(Listener{interrupt:ctrl_c().map_err(failure)?,control_break:ctrl_break().map_err(failure)?})
        }
        #[cfg(not(any(unix,windows)))] {Err(Error{code:"unsupported",message:"Shutdown signals require a Unix or Windows target".into()})}
    }
    impl Listener{
        pub async fn receive(&mut self)->Result<String,Error>{
            #[cfg(unix)] {std::future::poll_fn(|cx|{
                use std::task::Poll;
                for (signal,name) in [(&mut self.interrupt,"SIGINT"),(&mut self.terminate,"SIGTERM")]{
                    if let Poll::Ready(value)=signal.poll_recv(cx){return Poll::Ready(value.map(|_|name.to_owned()).ok_or_else(||Error{code:"closed",message:"Signal listener closed".into()}));}
                }Poll::Pending
            }).await}
            #[cfg(windows)] {std::future::poll_fn(|cx|{
                use std::task::Poll;
                if let Poll::Ready(value)=self.interrupt.poll_recv(cx){return Poll::Ready(value.map(|_|"SIGINT".to_owned()).ok_or_else(||Error{code:"closed",message:"Signal listener closed".into()}));}
                if let Poll::Ready(value)=self.control_break.poll_recv(cx){return Poll::Ready(value.map(|_|"SIGBREAK".to_owned()).ok_or_else(||Error{code:"closed",message:"Signal listener closed".into()}));}
                Poll::Pending
            }).await}
            #[cfg(not(any(unix,windows)))] {Err(Error{code:"unsupported",message:"Shutdown signals require a Unix or Windows target".into()})}
        }
    }
}
