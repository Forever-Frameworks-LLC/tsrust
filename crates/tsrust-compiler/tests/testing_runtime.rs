// The generated runtime contains optional entrypoints not used by every fixture.
#[allow(dead_code, reason="Exercise the bundled runner with a small concrete program context.")]
mod runtime {
    include!("../src/stdlib/futures/runtime.rs");
    use std::{cell::Cell,rc::Rc,sync::{Arc,atomic::{AtomicUsize,Ordering}}};

    struct String(&'static str);
    impl AsRef<str> for String { fn as_ref(&self)->&str { self.0 } }
    #[derive(Default)]
    struct State {
        events:Vec<&'static str>,
        job:Option<tsrust_native_futures::Job<(),()>>,
        send:Option<tokio::sync::oneshot::Sender<()>>,
        drops:Arc<AtomicUsize>,
        listener:Option<tokio::net::TcpListener>,
    }
    struct TsrTestContext<'p> { state:&'p mut State }
    trait ProgramCallback<A,R> {
        fn invoke_program(&mut self,args:A,program:&mut TsrTestContext<'_>,registry:&mut TsrTestRegistry)->R;
    }
    fn tsr_console_test_begin(_: &str) {}
    fn tsr_console_test_end(_: bool) {}
    include!("../src/stdlib/testing/runtime/mod.rs");
    include!("../src/stdlib/testing/runtime/registration.rs");
    include!("../src/stdlib/testing/runtime/hooks.rs");
    include!("../src/stdlib/testing/runtime/async.rs");

    mod futures_controls {
        use super::tsrust_native_futures::{self,*};
        include!("../src/stdlib/futures/runtime_tests.rs");
    }

    struct DropCount(Arc<AtomicUsize>);
    impl Drop for DropCount { fn drop(&mut self) { self.0.fetch_add(1,Ordering::SeqCst); } }
    struct Action { name:&'static str, calls:Rc<Cell<usize>> }
    impl TsrTestAsyncCallback for Action {
        fn invoke_async<'a,'p:'a>(&'a mut self,program:&'a mut TsrTestContext<'p>,registry:&'a mut TsrTestRegistry)
            ->std::pin::Pin<Box<dyn std::future::Future<Output=Result<(),std::string::String>>+'a>> {
            Box::pin(async move {
                let body=async move {
                tokio::task::yield_now().await;
                assert!(TSR_TEST_RUNNING.with(Cell::get),"running guard must survive suspension");
                self.calls.set(self.calls.get()+1);
                program.state.events.push(self.name);
                match self.name {
                    "setup" => {
                        let (send,receive)=tokio::sync::oneshot::channel();
                        let (ready,started)=tokio::sync::oneshot::channel();
                        let guard=DropCount(program.state.drops.clone());
                        program.state.job=Some(tsrust_native_futures::start(async move {
                            let _guard=guard;
                            let _=ready.send(());
                            let _=receive.await;
                            Ok(())
                        }));
                        program.state.send=Some(send);
                        started.await.unwrap();
                    },
                    "use setup" => {
                        assert_eq!(program.state.drops.load(Ordering::SeqCst),0,"setup job must survive between callbacks");
                        program.state.send.take().unwrap().send(()).unwrap();
                        assert!(program.state.job.take().unwrap().join().await.is_ok());
                        assert_eq!(program.state.drops.load(Ordering::SeqCst),1);
                    },
                    "guard" => {
                        assert!(tsrust_testing_run_tests_result(registry,program).unwrap_err().contains("inside a test"));
                        assert!(tsrust_testing_register_test_result(registry,&String("late"),Box::new(|_,_|Ok(()))).is_err());
                        assert!(tsrust_testing_register_hook_result(registry,0,Box::new(|_,_|Ok(()))).is_err());
                    },
                    "error" => return Err("awaited failure".into()),
                    "panic" => panic!("awaited panic"),
                    "network setup" => {
                        program.state.listener=Some(tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap());
                    },
                    "network" => {
                        let listener=program.state.listener.as_ref().unwrap();
                        let address=listener.local_addr().unwrap();
                        let client=tokio::spawn(async move {
                            let stream=tokio::net::TcpStream::connect(address).await.unwrap();
                            let mut byte=[0];
                            loop {
                                stream.readable().await.unwrap();
                                match stream.try_read(&mut byte) {
                                    Ok(1)=>break,
                                    Err(error) if error.kind()==std::io::ErrorKind::WouldBlock=>{},
                                    value=>panic!("network read: {value:?}"),
                                }
                            }
                            assert_eq!(byte,[42]);
                        });
                        let (stream,_)=listener.accept().await.unwrap();
                        loop {
                            stream.writable().await.unwrap();
                            match stream.try_write(&[42]) {
                                Ok(1)=>break,
                                Err(error) if error.kind()==std::io::ErrorKind::WouldBlock=>{},
                                value=>panic!("network write: {value:?}"),
                            }
                        }
                        client.await.unwrap();
                    },
                    "network teardown" => { program.state.listener.take(); },
                    "orphan" => {
                        let guard=DropCount(program.state.drops.clone());
                        let (ready,started)=tokio::sync::oneshot::channel();
                        program.state.job=Some(tsrust_native_futures::start(async move {
                            let _guard=guard;let _=ready.send(());
                            std::future::pending::<()>().await;Ok(())
                        }));
                        started.await.unwrap();
                        return Err("failure with a live child".into());
                    },
                    _ => {},
                }
                Ok(())
                };
                tokio::time::timeout(std::time::Duration::from_secs(10),body).await.unwrap_or_else(|_|Err("async runner fixture timed out".into()))
            })
        }
    }
    fn callback(name:&'static str,calls:&Rc<Cell<usize>>)->TsrTestCallback {
        tsrust_testing_async_callback(Action{name,calls:calls.clone()})
    }
    fn hook(registry:&mut TsrTestRegistry,kind:i64,name:&'static str,calls:&Rc<Cell<usize>>) {
        tsrust_testing_register_hook_result(registry,kind,callback(name,calls)).unwrap();
    }
    fn test(registry:&mut TsrTestRegistry,name:&'static str,calls:&Rc<Cell<usize>>) {
        tsrust_testing_register_test_result(registry,&String(name),callback(name,calls)).unwrap();
    }

    #[test]
    fn async_callbacks_borrow_non_send_state_and_share_setup_runtime() {
        let calls=Rc::new(Cell::new(0));let mut state=State::default();let mut registry=TsrTestRegistry::default();
        hook(&mut registry,0,"setup",&calls);hook(&mut registry,1,"teardown",&calls);
        hook(&mut registry,2,"before",&calls);hook(&mut registry,3,"after",&calls);
        test(&mut registry,"use setup",&calls);test(&mut registry,"guard",&calls);
        assert_eq!(tsrust_testing_run_tests_result(&mut registry,&mut TsrTestContext{state:&mut state}),Ok(0));
        assert_eq!(state.events,["setup","before","use setup","after","before","guard","after","teardown"]);
        assert_eq!(calls.get(),8);
        assert!(!TSR_TEST_RUNNING.with(Cell::get));
    }

    #[test]
    fn awaited_errors_panics_and_hook_failures_keep_teardown_and_next_tests() {
        let calls=Rc::new(Cell::new(0));let mut state=State::default();let mut registry=TsrTestRegistry::default();
        hook(&mut registry,3,"after",&calls);hook(&mut registry,1,"teardown",&calls);
        test(&mut registry,"error",&calls);test(&mut registry,"panic",&calls);test(&mut registry,"last",&calls);
        assert_eq!(tsrust_testing_run_tests_result(&mut registry,&mut TsrTestContext{state:&mut state}),Ok(2));
        assert_eq!(state.events,["error","after","panic","after","last","after","teardown"]);
        assert!(!TSR_TEST_RUNNING.with(Cell::get));
        state.events.clear();hook(&mut registry,0,"error",&calls);hook(&mut registry,1,"teardown",&calls);
        test(&mut registry,"blocked",&calls);
        assert_eq!(tsrust_testing_run_tests_result(&mut registry,&mut TsrTestContext{state:&mut state}),Ok(2));
        assert_eq!(state.events,["error","teardown"]);
    }

    #[test]
    fn failed_callback_children_are_drained_before_runner_returns() {
        let calls=Rc::new(Cell::new(0));let mut state=State::default();let mut registry=TsrTestRegistry::default();
        test(&mut registry,"orphan",&calls);
        assert_eq!(tsrust_testing_run_tests_result(&mut registry,&mut TsrTestContext{state:&mut state}),Ok(1));
        assert_eq!(state.drops.load(Ordering::SeqCst),1);
    }

    #[test]
    fn runner_reuses_an_existing_application_scope() {
        tsrust_native_futures::block_on(async {
            let outside_drops=Arc::new(AtomicUsize::new(0));let guard=DropCount(outside_drops.clone());
            let (ready,started)=tokio::sync::oneshot::channel();
            let outside=tsrust_native_futures::start(async move {
                let _guard=guard;let _=ready.send(());
                std::future::pending::<()>().await;Ok::<(),tsrust_native_futures::Failure<()>>(())
            });
            started.await.unwrap();
            let calls=Rc::new(Cell::new(0));let mut state=State::default();let mut registry=TsrTestRegistry::default();
            test(&mut registry,"guard",&calls);test(&mut registry,"orphan",&calls);
            assert_eq!(tsrust_testing_run_tests_result(&mut registry,&mut TsrTestContext{state:&mut state}),Ok(1));
            assert_eq!(calls.get(),2);assert_eq!(state.drops.load(Ordering::SeqCst),1);
            assert_eq!(outside_drops.load(Ordering::SeqCst),0,"test cleanup must not cancel the application's other jobs");
            outside.cancel();let _=outside.join().await;
            assert_eq!(outside_drops.load(Ordering::SeqCst),1);
        });
    }

    #[test]
    fn async_network_setup_remains_live_for_test_and_teardown() {
        let calls=Rc::new(Cell::new(0));let mut state=State::default();let mut registry=TsrTestRegistry::default();
        hook(&mut registry,0,"network setup",&calls);hook(&mut registry,1,"network teardown",&calls);
        test(&mut registry,"network",&calls);
        assert_eq!(tsrust_testing_run_tests_result(&mut registry,&mut TsrTestContext{state:&mut state}),Ok(0));
        assert_eq!(state.events,["network setup","network","network teardown"]);
        assert!(state.listener.is_none());
    }

    #[test]
    fn failing_async_teardown_does_not_skip_remaining_hooks() {
        let calls=Rc::new(Cell::new(0));let mut state=State::default();let mut registry=TsrTestRegistry::default();
        hook(&mut registry,3,"after",&calls);hook(&mut registry,3,"error",&calls);
        hook(&mut registry,1,"teardown",&calls);hook(&mut registry,1,"panic",&calls);
        test(&mut registry,"first",&calls);test(&mut registry,"last",&calls);
        assert_eq!(tsrust_testing_run_tests_result(&mut registry,&mut TsrTestContext{state:&mut state}),Ok(3));
        assert_eq!(state.events,["first","error","after","last","error","after","panic","teardown"]);
        assert!(!TSR_TEST_RUNNING.with(Cell::get));
    }
}
