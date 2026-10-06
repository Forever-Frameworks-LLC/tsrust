fn tsrust_testing_style(text: &str, code: &str) -> std::string::String {
    let mode = std::env::var("TSRUST_COLOR").unwrap_or_else(|_| "auto".to_owned());
    let color = mode == "always" || (mode == "auto" && std::io::IsTerminal::is_terminal(&std::io::stdout()) && std::env::var_os("NO_COLOR").is_none() && std::env::var("TERM").as_deref() != Ok("dumb"));
    if color { format!("\x1b[{code}m{text}\x1b[0m") } else { text.to_owned() }
}
// Bundled synchronous registry and failure boundaries, included in generated programs.
enum TsrTestEntry {
    Test { name: std::string::String, callback: Box<dyn FnMut() -> Result<(), std::string::String>> },
    Suite(usize),
}
struct TsrTestSuite {
    name: std::string::String, displayed: bool, entries: Vec<TsrTestEntry>,
    hooks: [Vec<Box<dyn FnMut() -> Result<(), std::string::String>>>; 4],
}
impl TsrTestSuite {
    fn new(name: std::string::String, displayed: bool) -> Self {
        Self { name, displayed, entries: Vec::new(), hooks: std::array::from_fn(|_| Vec::new()) }
    }
}
struct TsrTestRegistry { suites: Vec<TsrTestSuite>, current: Vec<usize> }
impl Default for TsrTestRegistry {
    fn default() -> Self { Self { suites: vec![TsrTestSuite::new(std::string::String::new(), false)], current: vec![0] } }
}
#[derive(Default)]
struct TsrTestCounts { passed: i64, failed: i64, hook_failures: i64 }
thread_local! {
    static TSR_TESTS: std::cell::RefCell<TsrTestRegistry> = std::cell::RefCell::new(TsrTestRegistry::default());
    static TSR_TEST_RUNNING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
// Prepared registration callables restore their entry scope on every return.
struct TsrSuiteScopeGuard { depth: usize }
impl TsrSuiteScopeGuard {
    fn new() -> Self { Self { depth: TSR_TESTS.with(|registry|registry.borrow().current.len()) } }
}
impl Drop for TsrSuiteScopeGuard {
    fn drop(&mut self) {
        TSR_TESTS.with(|registry|registry.borrow_mut().current.truncate(self.depth));
    }
}
fn tsrust_testing_open_scope(name: &String, displayed: bool) {
    assert!(!TSR_TEST_RUNNING.with(|running| running.get()), "describe() cannot register suites while a test is running");
    TSR_TESTS.with(|registry| {
        let mut registry = registry.borrow_mut();
        let parent = *registry.current.last().unwrap();
        let id = registry.suites.len();
        registry.suites.push(TsrTestSuite::new(name.as_ref().to_owned(), displayed));
        registry.suites[parent].entries.push(TsrTestEntry::Suite(id));
        registry.current.push(id);
    });
}
fn tsrust_testing_begin_suite(name: &String) { tsrust_testing_open_scope(name, true); }
fn tsrust_testing_begin_suite_result(name:&String)->Result<(),std::string::String> {
    if TSR_TEST_RUNNING.with(|running|running.get()) {return Err("describe() cannot register suites while a test is running".to_owned());}
    tsrust_testing_open_scope(name,true);
    Ok(())
}
fn tsrust_testing_end_suite_result()->Result<(),std::string::String> {
    if TSR_TESTS.with(|registry|registry.borrow().current.len()<=1) {return Err("No suite registration is active".to_owned());}
    tsrust_testing_end_suite();
    Ok(())
}
fn tsrust_testing_begin_file(name: &String) { tsrust_testing_open_scope(name, false); }
fn tsrust_testing_begin_file_result(name:&String)->Result<(),std::string::String> {
    if TSR_TEST_RUNNING.with(|running|running.get()) {return Err("Test files cannot register while a test is running".to_owned());}
    tsrust_testing_open_scope(name,false);
    Ok(())
}
fn tsrust_testing_end_suite() {
    TSR_TESTS.with(|registry| { let mut registry = registry.borrow_mut(); assert!(registry.current.len() > 1, "No suite registration is active"); registry.current.pop(); });
}
fn tsrust_testing_invoke(callback: &mut dyn FnMut() -> Result<(), std::string::String>) -> Option<std::string::String> {
    TSR_TEST_RUNNING.with(|running| running.set(true));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback));
    TSR_TEST_RUNNING.with(|running| running.set(false));
    match result {
        Ok(result) => result.err(),
        Err(error) => Some(error.downcast_ref::<&str>().copied()
        .or_else(|| error.downcast_ref::<std::string::String>().map(|value| value.as_str()))
        .unwrap_or("native test failed").to_owned()),
    }
}
fn tsrust_testing_hooks(suite: &mut TsrTestSuite, kind: usize, reverse: bool, stop_on_failure: bool) -> Vec<std::string::String> {
    let mut errors = Vec::new();
    let length = suite.hooks[kind].len();
    for position in 0..length {
        let index = if reverse { length - position - 1 } else { position };
        if let Some(error) = tsrust_testing_invoke(&mut *suite.hooks[kind][index]) {
            errors.push(error);
            if stop_on_failure { break; }
        }
    }
    errors
}
fn tsrust_testing_has_tests(registry: &TsrTestRegistry, id: usize) -> bool {
    registry.suites[id].entries.iter().any(|entry| match entry {
        TsrTestEntry::Test { .. } => true,
        TsrTestEntry::Suite(child) => tsrust_testing_has_tests(registry, *child),
    })
}
fn tsrust_testing_run_suite(registry: &mut TsrTestRegistry, id: usize, path: &mut Vec<std::string::String>, ancestors: &mut Vec<usize>, blocked: Option<&str>, counts: &mut TsrTestCounts) {
    if !tsrust_testing_has_tests(registry, id) { return; }
    let displayed = registry.suites[id].displayed;
    if displayed { path.push(registry.suites[id].name.clone()); }
    ancestors.push(id);
    let label = if path.is_empty() { if registry.suites[id].name.is_empty() { "root".to_owned() } else { registry.suites[id].name.clone() } } else { path.join(" > ") };
    let setup_errors = if blocked.is_none() { tsrust_testing_hooks(&mut registry.suites[id], 0, false, true) } else { Vec::new() };
    for error in &setup_errors { counts.hook_failures += 1; println!("{} {label} beforeAll: {error}", tsrust_testing_style("HOOK FAIL", "1;31")); }
    let setup_error = setup_errors.first().map(|error| format!("beforeAll: {error}"));
    let blocked = blocked.or(setup_error.as_deref());
    let entries = std::mem::take(&mut registry.suites[id].entries);
    for entry in entries {
        match entry {
            TsrTestEntry::Suite(child) => tsrust_testing_run_suite(registry, child, path, ancestors, blocked, counts),
            TsrTestEntry::Test { name, mut callback } => {
                path.push(name); let name = path.join(" > "); path.pop();
                let mut errors = Vec::new();
                if let Some(error) = blocked { errors.push(error.to_owned()); }
                else {
                    for &scope in ancestors.iter() {
                        let setup = tsrust_testing_hooks(&mut registry.suites[scope], 2, false, true);
                        if !setup.is_empty() { errors.extend(setup.into_iter().map(|error| format!("beforeEach: {error}"))); break; }
                    }
                    if errors.is_empty() { if let Some(error) = tsrust_testing_invoke(&mut *callback) { errors.push(error); } }
                    for &scope in ancestors.iter().rev() {
                        errors.extend(tsrust_testing_hooks(&mut registry.suites[scope], 3, true, false).into_iter().map(|error| format!("afterEach: {error}")));
                    }
                }
                if errors.is_empty() { counts.passed += 1; println!("{} {name}", tsrust_testing_style("PASS", "1;32")); }
                else { counts.failed += 1; println!("{} {name}: {}", tsrust_testing_style("FAIL", "1;31"), errors.join("; ")); }
            }
        }
    }
    // Teardown belongs to scopes whose setup was attempted, including failed setup.
    if setup_error.is_some() || blocked.is_none() {
        for error in tsrust_testing_hooks(&mut registry.suites[id], 1, true, false) { counts.hook_failures += 1; println!("{} {label} afterAll: {error}", tsrust_testing_style("HOOK FAIL", "1;31")); }
    }
    ancestors.pop();
    if displayed { path.pop(); }
}
fn tsrust_testing_run_tests() -> i64 {
    tsrust_testing_run_tests_result().unwrap_or_else(|error| panic!("{error}"))
}
fn tsrust_testing_run_tests_result() -> Result<i64, std::string::String> {
    if TSR_TEST_RUNNING.with(|running| running.get()) {
        return Err("runTests() cannot run inside a test".to_owned());
    }
    if TSR_TESTS.with(|registry|registry.borrow().current.len()!=1) {
        return Err("runTests() cannot execute while a suite is registering".to_owned());
    }
    static HOOK: std::sync::Once = std::sync::Once::new();
    HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |information| {
            if !TSR_TEST_RUNNING.with(|running| running.get()) { previous(information); }
        }));
    });
    let mut registry = TSR_TESTS.with(|registry| {
        let mut registry = registry.borrow_mut();
        std::mem::take(&mut *registry)
    });
    let mut counts = TsrTestCounts::default();
    tsrust_testing_run_suite(&mut registry, 0, &mut Vec::new(), &mut Vec::new(), None, &mut counts);
    print!("{} passed, {} failed, 0 skipped", tsrust_testing_style(&counts.passed.to_string(), "1;32"), tsrust_testing_style(&counts.failed.to_string(), if counts.failed == 0 { "1" } else { "1;31" }));
    if counts.hook_failures != 0 { print!(", {} hook failures", counts.hook_failures); }
    println!();
    Ok(counts.failed + counts.hook_failures)
}
fn tsrust_testing_finish_tests() { if tsrust_testing_run_tests() != 0 { std::process::exit(1); } }

fn tsrust_testing_finish_tests_result()->Result<(),std::string::String>{
    let failed=tsrust_testing_run_tests_result()?;
    if failed==0 {Ok(())} else {Err(format!("test run failed: {failed} failure(s)"))}
}
