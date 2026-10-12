// Callback adapters are linked only when a hook registration binding is used.
fn tsrust_testing_register_hook_result(kind:i64,callback:Box<dyn FnMut()->Result<(),std::string::String>>)->Result<(),std::string::String> {
    if TSR_TEST_RUNNING.with(|running|running.get()) {return Err("Hooks cannot register while tests or hooks are running".to_owned());}
    if !(0..4).contains(&kind) {return Err("Unknown hook kind".to_owned());}
    TSR_TESTS.with(|registry|{let mut registry=registry.borrow_mut();let suite=*registry.current.last().expect("registry root scope");registry.suites[suite].hooks[kind as usize].push(callback);});
    Ok(())
}
fn tsrust_testing_register_hook<C: Callback<(), ()> + 'static>(kind: i64, mut callback: C) {
    assert!(!TSR_TEST_RUNNING.with(|running| running.get()), "Hooks cannot register while tests or hooks are running");
    assert!((0..4).contains(&kind), "Unknown hook kind");
    TSR_TESTS.with(|registry| {
        let mut registry = registry.borrow_mut();
        let suite = *registry.current.last().unwrap();
        registry.suites[suite].hooks[kind as usize].push(Box::new(move || { callback.invoke(()); Ok(()) }));
    });
}
