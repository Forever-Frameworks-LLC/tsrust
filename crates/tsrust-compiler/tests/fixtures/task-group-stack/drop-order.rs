#![allow(dead_code)]
include!(env!("TSRUST_STACK_RUNTIME"));
use std::{cell::Cell,future::Future,pin::Pin,rc::Rc,sync::Mutex,task::Poll};
static EVENTS:Mutex<Vec<i32>>=Mutex::new(Vec::new());
struct ProbeValue{index:i32,value:i64,scope:usize,check_scope:bool}
impl Drop for ProbeValue{fn drop(&mut self){if self.check_scope{assert_eq!(tsrust_native_futures::probe_scope(),self.scope,"result dropped outside the task-local group");}EVENTS.lock().unwrap().push(100+self.index);}}
struct ProbeFuture<'a>{index:i32,value:&'a Rc<Cell<i64>>,ready:bool,fail:bool,scope:Cell<usize>,completed:Cell<bool>,check_result_scope:bool,_pin:std::marker::PhantomPinned}
impl Future for ProbeFuture<'_>{
    type Output=Result<ProbeValue,tsrust_native_futures::Failure<()>>;
    fn poll(self:Pin<&mut Self>,_:&mut std::task::Context<'_>)->Poll<Self::Output>{
        let this=self.as_ref().get_ref();assert!(!this.completed.get(),"completed future was polled again");this.scope.set(tsrust_native_futures::probe_scope());
        if this.fail{this.completed.set(true);Poll::Ready(Err(tsrust_native_futures::Failure::Worker(())))}
        else if this.ready{this.completed.set(true);Poll::Ready(Ok(ProbeValue{index:this.index,value:this.value.get(),scope:this.scope.get(),check_scope:this.check_result_scope}))}
        else{Poll::Pending}
    }
}
impl Drop for ProbeFuture<'_>{fn drop(&mut self){if self.scope.get()!=0{assert_eq!(tsrust_native_futures::probe_scope(),self.scope.get(),"future dropped outside the task-local group");}EVENTS.lock().unwrap().push(self.index+1);}}
include!(env!("TSRUST_STACK_GROUP"));
fn main(){
    tsrust_native_futures::block_on(async {
        let values=[Rc::new(Cell::new(7)),Rc::new(Cell::new(8)),Rc::new(Cell::new(9)),Rc::new(Cell::new(10))];
        let make=|index:usize,ready,fail,check_result_scope|ProbeFuture{index:index as i32,value:&values[index],ready,fail,scope:Cell::new(0),completed:Cell::new(false),check_result_scope,_pin:std::marker::PhantomPinned};
        // An immediately completed input drops before later polling. On error,
        // stored successful output drops before pending inputs in source order.
        let result=probe((make(0,true,false,true),make(1,false,false,true),make(2,false,true,true),make(3,false,false,true))).await;
        assert!(matches!(result,Err(tsrust_native_futures::Failure::Worker(()))));assert_eq!(*EVENTS.lock().unwrap(),[1,100,2,3,4]);EVENTS.lock().unwrap().clear();
        // Cancelling a pending group has the same output-first/index-order drop.
        let mut pending=Box::pin(probe((make(0,true,false,true),make(1,false,false,true),make(2,false,false,true),make(3,false,false,true))));
        std::future::poll_fn(|cx|{assert!(pending.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;assert_eq!(*EVENTS.lock().unwrap(),[1]);drop(pending);
        assert_eq!(*EVENTS.lock().unwrap(),[1,100,2,3,4]);EVENTS.lock().unwrap().clear();
        // Unpolled inputs retain tuple/source drop order; no pin or scope is entered.
        drop(probe((make(0,false,false,false),make(1,false,false,false),make(2,false,false,false),make(3,false,false,false))));
        assert_eq!(*EVENTS.lock().unwrap(),[1,2,3,4]);EVENTS.lock().unwrap().clear();
        let result=probe((make(0,true,false,false),make(1,true,false,false),make(2,true,false,false),make(3,true,false,false))).await.ok().expect("successful group");
        assert_eq!(result.iter().map(|value|value.value).collect::<Vec<_>>(),[7,8,9,10]);assert_eq!(*EVENTS.lock().unwrap(),[1,2,3,4]);drop(result);
        assert_eq!(*EVENTS.lock().unwrap(),[1,2,3,4,100,101,102,103]);
    });
}
