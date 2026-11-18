// Runtime faults (overflow, bounds, failed assertions) unwind to the nearest
// catch point instead of travelling in every caller's Result. The fault's
// details wait in a thread-local; the panic payload is the message text, as
// with an `expect` panic. A pending detail record is what marks a panic as a
// fault, so foreign panics with text payloads are never mistaken for one.
type TsrFaultText = std::borrow::Cow<'static, str>;
struct TsrFaultInfo {
    name: TsrFaultText,
    code: TsrFaultText,
    message: TsrFaultText,
    source: TsrFaultText,
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
/// Whether panic text is the pending fault's own message (payload types stay inferred).
fn tsr_fault_pending(text: Option<&std::string::String>) -> bool {
    text.is_some_and(|text| TSR_FAULT.with(|slot| slot.borrow().as_ref().is_some_and(|fault| *text == *fault.message)))
}
/// Installed once by the program's entry (Rust code calling generated functions
/// directly keeps its own hook). A fault is silent while a Tsrust
/// catch point will receive it. Raised with none active on its thread (a missed
/// boundary, or Rust code calling generated functions), it goes to the previous
/// hook like any other panic, so it stays visible. Other panics are defects and
/// go to the previous hook unchanged.
fn tsr_install_fault_hook() {
    static INSTALLED: std::sync::Once = std::sync::Once::new();
    INSTALLED.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if tsr_fault_pending(info.payload().downcast_ref()) && TSR_FAULT_CATCHERS.with(|catchers| catchers.get()) > 0 { return; }
            previous(info)
        }));
    });
}
#[cold]
#[inline(never)]
fn tsr_raise_fault_parts(name: TsrFaultText, code: TsrFaultText, message: TsrFaultText, source: TsrFaultText, start: i64, end: i64) -> ! {
    let text: std::string::String = std::string::String::from(&*message);
    TSR_FAULT.with(|slot| *slot.borrow_mut() = Some(TsrFaultInfo { name, code, message, source, start, end }));
    std::panic::panic_any(text)
}
#[cold]
#[inline(never)]
fn tsr_raise_fault(name: &'static str, code: &'static str, message: &'static str, source: &'static str, start: i64, end: i64) -> ! {
    tsr_raise_fault_parts(TsrFaultText::Borrowed(name), TsrFaultText::Borrowed(code), TsrFaultText::Borrowed(message), TsrFaultText::Borrowed(source), start, end)
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
        Err(payload) if tsr_fault_pending(payload.downcast_ref()) => {drop(payload); Err(TSR_FAULT.with(|slot| slot.borrow_mut().take()).expect("a raised fault stores its details"))},
        Err(payload) => std::panic::resume_unwind(payload),
    }
}
/// Owners are dropped while unwinding, before the report is written. The
/// process ends here with status 1, whoever called the entry.
fn tsr_fault_exit(fault: &TsrFaultInfo) -> std::process::ExitCode {
    tsr_fault_write(&tsr_fault_text(fault));
    std::process::exit(1)
}
#[allow(dead_code)]
fn tsr_fault_entry(body: fn()) -> std::process::ExitCode {
    tsr_install_fault_hook();
    match tsr_catch_faults(body) { Ok(()) => std::process::ExitCode::SUCCESS, Err(fault) => tsr_fault_exit(&fault) }
}
