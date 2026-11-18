// Runtime faults (overflow, bounds, failed assertions) unwind to the nearest
// catch point instead of travelling in every caller's Result. The fault's
// details wait in a thread-local; the panic payload is only a marker, so any
// data can be carried and nothing about the details has to be `Send`.
struct TsrFault;
struct TsrFaultInfo {
    name: String,
    code: String,
    message: String,
    source: String,
    start: i64,
    end: i64,
}
thread_local! {
    static TSR_FAULT: std::cell::RefCell<Option<TsrFaultInfo>> = const { std::cell::RefCell::new(None) };
    // Catch points active on this thread; a fault raised with none is reported by the hook.
    static TSR_FAULT_CATCHERS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
fn tsr_fault_text(fault: &TsrFaultInfo) -> std::string::String {
    std::format!("Uncaught {}: {} ({}:{}..{})\n", &*fault.name, &*fault.message, &*fault.source, fault.start, fault.end)
}
fn tsr_fault_write(text: &str) {
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(text.as_bytes());
}
/// Installed once, before the first fault. Faults are silent while a catch point
/// will receive them; one raised with no catch point on its thread is reported
/// here, so a missed boundary cannot lose it. Every other panic is a defect and
/// goes to the previous hook unchanged.
fn tsr_install_fault_hook() {
    static INSTALLED: std::sync::Once = std::sync::Once::new();
    INSTALLED.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if info.payload().is::<TsrFault>() {
                if TSR_FAULT_CATCHERS.with(|catchers| catchers.get()) == 0 {
                    TSR_FAULT.with(|slot| if let Some(fault) = slot.borrow().as_ref() { tsr_fault_write(&tsr_fault_text(fault)); });
                }
                return;
            }
            previous(info)
        }));
    });
}
#[cold]
#[inline(never)]
fn tsr_raise_fault_parts(name: String, code: String, message: String, source: String, start: i64, end: i64) -> ! {
    tsr_install_fault_hook();
    TSR_FAULT.with(|slot| *slot.borrow_mut() = Some(TsrFaultInfo { name, code, message, source, start, end }));
    std::panic::panic_any(TsrFault)
}
#[cold]
#[inline(never)]
fn tsr_raise_fault(name: &'static str, code: &'static str, message: &'static str, source: &'static str, start: i64, end: i64) -> ! {
    tsr_raise_fault_parts(String::Borrowed(name), String::Borrowed(code), String::Borrowed(message), String::Borrowed(source), start, end)
}
/// Run `body`; a fault it raises comes back as its details. Other panics continue.
fn tsr_catch_faults<R>(body: impl FnOnce() -> R) -> Result<R, TsrFaultInfo> {
    struct Active;
    impl Drop for Active { fn drop(&mut self) { TSR_FAULT_CATCHERS.with(|catchers| catchers.set(catchers.get() - 1)); } }
    TSR_FAULT_CATCHERS.with(|catchers| catchers.set(catchers.get() + 1));
    let active = Active;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
    drop(active);
    match result {
        Ok(value) => Ok(value),
        Err(payload) if payload.is::<TsrFault>() => Err(TSR_FAULT.with(|slot| slot.borrow_mut().take()).expect("a raised fault stores its details")),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}
/// Owners are dropped while unwinding, before the report is written.
fn tsr_fault_exit(fault: &TsrFaultInfo) -> std::process::ExitCode {
    tsr_fault_write(&tsr_fault_text(fault));
    std::process::ExitCode::FAILURE
}
#[allow(dead_code)]
fn tsr_fault_entry(body: fn()) -> std::process::ExitCode {
    match tsr_catch_faults(body) { Ok(()) => std::process::ExitCode::SUCCESS, Err(fault) => tsr_fault_exit(&fault) }
}
