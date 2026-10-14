//! Owned native workers; callable and lifecycle contracts are checked by the compiler.
use crate::native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType};
use crate::Mode;

const fn operation(name: &'static str) -> NativeBinding {
    NativeBinding {
        module: "std:threads", name, returned_borrow: None,
        parameters: &[NativeParameter { name: "value", ty: NativeType::ObservedValue, access: Mode::Consume, optional: false, rest: false }],
        result: NativeType::ObservedValue,
        backend: NativeBackend::Bundled { symbol: "thread_intrinsic" }, features: &[],
    }
}
pub(crate) const CANCEL: NativeBinding = NativeBinding { parameters: &[NativeParameter { name: "worker", ty: NativeType::ObservedValue, access: Mode::Read, optional: false, rest: false }], result: NativeType::Boolean, ..operation("cancel") };
pub(crate) const CANCELLATION_REQUESTED: NativeBinding = NativeBinding { parameters: &[], result: NativeType::Boolean, ..operation("cancellationRequested") };
pub(crate) const CREATE_POOL: NativeBinding = NativeBinding {
    parameters: &[NativeParameter { name: "options", ty: NativeType::ObservedValue, access: Mode::Read, optional: true, rest: false }], ..operation("__poolCreate")
};
const POOL_RECEIVER: NativeParameter = NativeParameter { name: "pool", ty: NativeType::ObservedValue, access: Mode::Read, optional: false, rest: false };
pub(crate) const OWNED_POOL_TASK: NativeBinding = NativeBinding { parameters: &[POOL_RECEIVER, NativeParameter { name: "callback", ty: NativeType::ObservedValue, access: Mode::Consume, optional: false, rest: false }], ..operation("__ownedPoolTask") };
pub(crate) const OWNED_POOL_STATS: NativeBinding = NativeBinding { parameters: &[POOL_RECEIVER], ..operation("__ownedPoolStats") };
pub(crate) const OWNED_POOL_SHUTDOWN: NativeBinding = NativeBinding { parameters: &[POOL_RECEIVER], ..operation("__ownedPoolShutdown") };
pub(crate) const TASK_CANCELLED: NativeBinding = NativeBinding { parameters: &[POOL_RECEIVER], ..operation("__taskCancelled") };
pub(crate) const CHANNEL_CREATE: NativeBinding = operation("__channelCreate");
pub(crate) const CHANNEL_SENDER: NativeBinding = NativeBinding { parameters: &[POOL_RECEIVER], ..operation("__channelSender") };
pub(crate) const CHANNEL_RECEIVE: NativeBinding = NativeBinding { parameters: &[POOL_RECEIVER], ..operation("__channelReceive") };
pub(crate) const CHANNEL_TRY_RECEIVE: NativeBinding = NativeBinding { parameters: &[POOL_RECEIVER], ..operation("__channelTryReceive") };
pub(crate) const CHANNEL_CLOSE: NativeBinding = NativeBinding { parameters: &[POOL_RECEIVER], ..operation("__channelClose") };
pub(crate) const SENDER_SEND: NativeBinding = NativeBinding { parameters: &[POOL_RECEIVER, NativeParameter { name: "value", ty: NativeType::ObservedValue, access: Mode::Consume, optional: false, rest: false }], ..operation("__senderSend") };
pub(crate) const SENDER_TRY_SEND: NativeBinding = NativeBinding { name: "__senderTrySend", ..SENDER_SEND };
pub(crate) const SENDER_CLOSE: NativeBinding = NativeBinding { parameters: &[NativeParameter { access: Mode::Mutate, ..POOL_RECEIVER }], ..operation("__senderClose") };
pub(crate) const SPAWN_SCOPED: NativeBinding = operation("spawnScoped");
pub(crate) const SPAWN: NativeBinding = operation("spawn");
pub(crate) const JOIN: NativeBinding = operation("join");
pub(crate) const DETACH: NativeBinding = operation("detach");
pub(crate) const RUNTIME: &str = include_str!("runtime.rs");

pub(crate) fn prepare(program: &mut crate::syntax::Program) -> Result<(), crate::Diagnostic> {
    let mut used = false;
    let pool_name = crate::native_type_resolution::linked_name(crate::native_signatures::NativeTypeId::ThreadPool);
    for function in &mut program.functions {
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            use crate::syntax::{Expr, ExprKind};
            let construction = matches!(&expression.kind, ExprKind::NativeConstructor(_, Some(annotation)) if matches!(&annotation.kind, crate::type_syntax::ValueAnnotationKind::Named(name) if name.text == pool_name));
            if construction {
                let original = std::mem::replace(&mut expression.kind, ExprKind::Integer("0".into()));
                let arguments = match original { ExprKind::NativeConstructor(arguments, _) | ExprKind::Call(_, arguments) => arguments, _ => unreachable!() };
                let binding = "__poolCreate";
                let call = Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(format!("@native:std:threads#{binding}")), expression.span)), arguments), expression.span);
                expression.kind = ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@native:std:result#unwrap".into()), expression.span)), vec![call]);
                used = true;
            }
            used |= matches!(&expression.kind, crate::syntax::ExprKind::Name(name) if name.starts_with("@native:std:threads#"));
            Ok(())
        })?;
    }
    let channels = program.records.iter().any(|record| matches!(record.native_type, Some(crate::native_signatures::NativeTypeId::Channel | crate::native_signatures::NativeTypeId::Sender)));
    if channels {
        for identity in [crate::native_signatures::NativeTypeId::Channel, crate::native_signatures::NativeTypeId::Sender, crate::native_signatures::NativeTypeId::SendError, crate::native_signatures::NativeTypeId::ChannelError] {
            crate::native_type_resolution::intern(&mut program.records, identity, crate::Span::default())?;
        }
    }
    if used {
        for identity in [crate::native_signatures::NativeTypeId::Thread, crate::native_signatures::NativeTypeId::ScopedThread, crate::native_signatures::NativeTypeId::ThreadError, crate::native_signatures::NativeTypeId::SpawnError] {
            crate::native_type_resolution::intern(&mut program.records, identity, crate::Span::default())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    include!("runtime.rs");
    use tsrust_native_threads::*;

    #[test]
    fn result_and_error_move_without_cloning_and_panics_are_distinct() {
        let value = vec![3, 5, 8]; let pointer = value.as_ptr() as usize;
        let caller = std::thread::current().id();
        let worker = spawn_dedicated(move || { assert_ne!(std::thread::current().id(),caller); Ok::<_,String>(value) }).unwrap();
        let returned = match join(worker) { Ok(value) => value, Err(_) => panic!("worker failed") };
        assert_eq!(returned.as_ptr() as usize,pointer);
        let error = String::from("owned failure"); let pointer = error.as_ptr() as usize;
        let worker = spawn_dedicated(move || Err::<(),_>(error)).unwrap();
        match join(worker) { Err(JoinError::Worker(error)) => assert_eq!(error.as_ptr() as usize,pointer), _ => panic!("lost worker error") }
        let worker = spawn_dedicated(|| -> Result<(),()> { panic!("backend failure") }).unwrap();
        assert!(matches!(join(worker),Err(JoinError::Panic)));
    }

    #[test]
    fn scoped_drop_waits_for_owned_completion_and_overlap_is_real() {
        use std::sync::{Arc, Barrier, atomic::{AtomicUsize, Ordering}};
        let barrier = Arc::new(Barrier::new(2));
        let count = Arc::new(AtomicUsize::new(0));
        let left_barrier = barrier.clone(); let left_count = count.clone();
        let first = spawn_dedicated(move || { left_barrier.wait(); left_count.fetch_add(1, Ordering::SeqCst); Ok::<_, ()>(()) }).unwrap();
        let right_count = count.clone();
        let second = spawn_dedicated(move || { barrier.wait(); right_count.fetch_add(1, Ordering::SeqCst); Err::<(), _>("worker failure") }).unwrap();
        drop(first); drop(second);
        assert_eq!(count.load(Ordering::SeqCst), 2);
        let worker = spawn_dedicated(|| Ok::<_, ()>(42)).unwrap();
        assert!(matches!(join(worker), Ok(42)));
    }

    #[test]
    fn detach_does_not_wait_and_worker_disposes_its_owned_state() {
        use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
        struct Payload(Arc<AtomicUsize>);
        impl Drop for Payload { fn drop(&mut self) { self.0.fetch_add(1,Ordering::SeqCst); } }
        let count=Arc::new(AtomicUsize::new(0));
        let payload=Payload(count.clone());
        let (release,wait)=std::sync::mpsc::channel();
        let (done,finished)=std::sync::mpsc::channel();
        let worker=spawn_dedicated(move || { wait.recv().unwrap(); drop(payload); done.send(()).unwrap(); Ok::<_,()>(()) }).unwrap();
        detach(worker);
        assert_eq!(count.load(Ordering::SeqCst),0);
        release.send(()).unwrap();
        finished.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert_eq!(count.load(Ordering::SeqCst),1);
    }
}
