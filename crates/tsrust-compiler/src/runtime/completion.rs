// Typed source completions are values, so cleanup does not depend on unwinding.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TsrControlTarget {
    Loop(usize),
    Block(usize),
}

// Storage-carrying control exits keep owners alive until finally has run.
enum TsrStorageExit<R> {
    Return(R),
    Break(TsrControlTarget),
    Continue(TsrControlTarget),
}

enum TsrCompletion<T, R, E> {
    Normal(T),
    Return(R),
    Throw(E),
    Break(TsrControlTarget),
    Continue(TsrControlTarget),
}

fn tsr_finally<T, R, E>(
    pending: TsrCompletion<T, R, E>,
    cleanup: impl FnOnce() -> TsrCompletion<(), R, E>,
) -> TsrCompletion<T, R, E> {
    match cleanup() {
        TsrCompletion::Normal(()) => pending,
        TsrCompletion::Return(value) => TsrCompletion::Return(value),
        TsrCompletion::Throw(error) => TsrCompletion::Throw(error),
        TsrCompletion::Break(target) => TsrCompletion::Break(target),
        TsrCompletion::Continue(target) => TsrCompletion::Continue(target),
    }
}

fn tsr_catch<T, R, E, F>(
    pending: TsrCompletion<T, R, E>,
    handler: impl FnOnce(E) -> TsrCompletion<T, R, F>,
) -> TsrCompletion<T, R, F> {
    match pending {
        TsrCompletion::Throw(error) => handler(error),
        TsrCompletion::Normal(value) => TsrCompletion::Normal(value),
        TsrCompletion::Return(value) => TsrCompletion::Return(value),
        TsrCompletion::Break(target) => TsrCompletion::Break(target),
        TsrCompletion::Continue(target) => TsrCompletion::Continue(target),
    }
}

fn tsr_from_result<T, R, E>(result: Result<T, E>) -> TsrCompletion<T, R, E> {
    match result {
        Ok(value) => TsrCompletion::Normal(value),
        Err(error) => TsrCompletion::Throw(error),
    }
}

fn tsr_checked<T, E>(value: Option<T>, failure: impl FnOnce() -> E) -> Result<T, E> {
    value.ok_or_else(failure)
}

// Return through Rust's entry boundary so ordinary owners are disposed first.
fn tsr_entry_result<T,E>(result:Result<T,E>,report:impl FnOnce(&E))->std::process::ExitCode {
    match result {
        Ok(value)=>{drop(value);std::process::ExitCode::SUCCESS}
        Err(error)=>{report(&error);drop(error);std::process::ExitCode::FAILURE}
    }
}
