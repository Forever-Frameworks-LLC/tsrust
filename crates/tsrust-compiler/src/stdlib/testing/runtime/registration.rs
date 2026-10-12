fn tsrust_testing_register_test<C: Callback<(), ()> + 'static>(name: &String, mut callback: C) {
    assert!(!TSR_TEST_RUNNING.with(|running| running.get()), "test() cannot register tests while a test is running");
    TSR_TESTS.with(|registry| {
        let mut registry = registry.borrow_mut();
        let suite = *registry.current.last().unwrap();
        registry.suites[suite].entries.push(TsrTestEntry::Test { name: name.as_ref().to_owned(), callback: Box::new(move || { callback.invoke(()); Ok(()) }) });
    });
}

// Prepared exception programs supply a callback whose typed errors are already
// rendered as runner diagnostics. Guard failure returns before registry mutation.
fn tsrust_testing_register_test_result(
    name: &String,
    callback: Box<dyn FnMut() -> Result<(), std::string::String>>,
) -> Result<(), std::string::String> {
    if TSR_TEST_RUNNING.with(|running| running.get()) {
        return Err("test() cannot register tests while a test is running".to_owned());
    }
    TSR_TESTS.with(|registry| {
        let mut registry = registry.borrow_mut();
        let suite = *registry.current.last().expect("registry root scope");
        registry.suites[suite].entries.push(TsrTestEntry::Test {
            name: name.as_ref().to_owned(), callback,
        });
    });
    Ok(())
}
