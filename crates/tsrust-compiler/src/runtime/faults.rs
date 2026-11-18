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
    line: i64,
    column: i64,
    // Captured only when RUST_BACKTRACE is set; printed by the uncaught report.
    backtrace: std::backtrace::Backtrace,
}
thread_local! {
    static TSR_FAULT: std::cell::RefCell<Option<TsrFaultInfo>> = const { std::cell::RefCell::new(None) };
    // Catch points active on this thread; a fault raised with none is reported by the hook.
    static TSR_FAULT_CATCHERS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
/// `Uncaught RangeError: message` then `    at file:line:column`.
/// A fault without a source position (a runtime helper's limit) has no `at` line.
fn tsr_fault_text(fault: &TsrFaultInfo) -> std::string::String {
    if fault.line == 0 { return std::format!("Uncaught {}: {}\n", &*fault.name, &*fault.message); }
    std::format!("Uncaught {}: {}\n    at {}:{}:{}\n", &*fault.name, &*fault.message, &*fault.source, fault.line, fault.column)
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
            let payload = info.payload();
            let text = payload.downcast_ref::<std::string::String>().map(|text| text.as_str()).or_else(|| payload.downcast_ref::<&'static str>().copied());
            let fault = tsr_fault_pending(payload.downcast_ref()) || text.is_some_and(|text| text.contains('\u{1f}'));
            if fault && TSR_FAULT_CATCHERS.with(|catchers| catchers.get()) > 0 { return; }
            previous(info)
        }));
    });
}
/// Whether a Tsrust catch point is active on this thread.
#[allow(dead_code)]
fn tsr_fault_catching() -> bool {
    TSR_FAULT_CATCHERS.with(|catchers| catchers.get()) > 0
}
#[cold]
#[inline(never)]
fn tsr_raise_fault_parts(name: TsrFaultText, code: TsrFaultText, message: TsrFaultText, source: TsrFaultText, start: i64, end: i64, line: i64, column: i64) -> ! {
    let text: std::string::String = std::string::String::from(&*message);
    // With no catch point on this thread (Rust code calling generated
    // functions), nothing would take the record: panic with the text alone.
    if TSR_FAULT_CATCHERS.with(|catchers| catchers.get()) == 0 { std::panic::panic_any(text) }
    TSR_FAULT.with(|slot| *slot.borrow_mut() = Some(TsrFaultInfo { name, code, message, source, start, end, line, column, backtrace: std::backtrace::Backtrace::capture() }));
    std::panic::panic_any(text)
}
#[cold]
#[inline(never)]
fn tsr_raise_fault(name: &'static str, code: &'static str, message: &'static str, source: &'static str, start: i64, end: i64, line: i64, column: i64) -> ! {
    tsr_raise_fault_parts(TsrFaultText::Borrowed(name), TsrFaultText::Borrowed(code), TsrFaultText::Borrowed(message), TsrFaultText::Borrowed(source), start, end, line, column)
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
        Err(payload) => {
            let text = payload.downcast_ref::<std::string::String>().map(|text| text.as_str()).or_else(|| payload.downcast_ref::<&'static str>().copied());
            match tsr_capacity_fault(text) { Some(fault) => {drop(payload); Err(fault)}, None => std::panic::resume_unwind(payload) }
        }
    }
}
/// Owners are dropped while unwinding, before the report is written. The
/// process ends here with status 1, whoever called the entry.
fn tsr_fault_exit(fault: &TsrFaultInfo) -> std::process::ExitCode {
    tsr_fault_write(&tsr_fault_text(fault));
    if fault.backtrace.status() == std::backtrace::BacktraceStatus::Captured {
        tsr_fault_write(&std::format!("native backtrace:\n{}\n", fault.backtrace));
    }
    std::process::exit(1)
}
#[allow(dead_code)]
fn tsr_fault_entry(body: fn()) -> std::process::ExitCode {
    tsr_install_fault_hook();
    match tsr_catch_faults(body) { Ok(()) => std::process::ExitCode::SUCCESS, Err(fault) => tsr_fault_exit(&fault) }
}
/// A length or allocation limit is a `RangeError` with code `capacity`.
/// Runtime helpers keep their ordinary checks (`expect(failure)`); with
/// unwinding faults the emitter appends the limit's location to `failure`,
/// separated by U+001F, and a panic whose text carries it is that fault.
fn tsr_capacity_fault(text: Option<&str>) -> Option<TsrFaultInfo> {
    let text = text.filter(|text| text.contains('\u{1f}'))?;
    let owned = |text: &str| TsrFaultText::Owned(std::string::String::from(text));
    let mut parts = text.split('\u{1f}');
    let message = owned(parts.next().unwrap_or(text));
    let source = owned(parts.next().unwrap_or(""));
    // `expect` on a `Result` appends `: <error>` after the last field.
    let mut number = || parts.next().and_then(|part| part.split(|c: char| !c.is_ascii_digit()).next()?.parse::<i64>().ok()).unwrap_or(0);
    let (start, end, line, column) = (number(), number(), number(), number());
    Some(TsrFaultInfo { name: TsrFaultText::Borrowed("RangeError"), code: TsrFaultText::Borrowed("capacity"), message, source, start, end, line, column, backtrace: std::backtrace::Backtrace::capture() })
}
/// A boundary that turns every panic into a failure (the test runner): faults
/// become the text value mode panics with; other panics stay as they are.
#[allow(dead_code)]
fn tsr_catch_faults_as_panics<R>(body: impl FnOnce() -> R) -> std::thread::Result<R> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tsr_catch_faults(body))) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(fault)) => {
            // The text matches value mode: assertions read `AssertionError:
            // message at file:line:column` (library sites without a location);
            // other faults and runner errors read as their message.
            let text = if &*fault.name != "AssertionError" {
                std::string::String::from(&*fault.message)
            } else if fault.line == 0 || fault.source.starts_with("std:") {
                std::format!("AssertionError: {}", &*fault.message)
            } else {
                // Sources inside the working directory are reported relative to it.
                let path = std::path::Path::new(&*fault.source);
                let relative = std::env::current_dir().ok().into_iter()
                    .flat_map(|directory| [directory.clone(), directory.canonicalize().unwrap_or(directory)])
                    .find_map(|directory| path.strip_prefix(&directory).ok().map(|relative| relative.display().to_string()))
                    .unwrap_or_else(|| std::string::String::from(&*fault.source));
                std::format!("AssertionError: {} at {}:{}:{}", &*fault.message, relative, fault.line, fault.column)
            };
            Err(std::boxed::Box::new(text))
        }
        Err(panic) => Err(panic),
    }
}
