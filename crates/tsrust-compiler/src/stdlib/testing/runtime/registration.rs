fn tsrust_testing_register_test<C: ProgramCallback<(), ()> + 'static>(registry: &mut TsrTestRegistry, name: &String, mut callback: C) {
    assert!(!TSR_TEST_RUNNING.with(|running| running.get()), "test() cannot register tests while a test is running");
    {
        let suite = *registry.current.last().unwrap();
        registry.suites[suite].entries.push(TsrTestEntry::Test { name: name.as_ref().to_owned(), callback: Box::new(move |program, registry| { callback.invoke_program((), program, registry); Ok(()) }) });
    }
}

// Prepared exception programs supply a callback whose typed errors are already
// rendered as runner diagnostics. Guard failure returns before registry mutation.
fn tsrust_testing_register_test_result(
    registry: &mut TsrTestRegistry,
    name: &String,
    callback: TsrTestCallback,
) -> Result<(), std::string::String> {
    if TSR_TEST_RUNNING.with(|running| running.get()) {
        return Err("test() cannot register tests while a test is running".to_owned());
    }
    {
        let suite = *registry.current.last().expect("registry root scope");
        registry.suites[suite].entries.push(TsrTestEntry::Test {
            name: name.as_ref().to_owned(), callback,
        });
    }
    Ok(())
}
