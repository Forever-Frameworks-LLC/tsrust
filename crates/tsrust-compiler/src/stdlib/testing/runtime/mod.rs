fn tsrust_testing_pretty() -> bool { std::env::var("TSRUST_TEST_REPORTER").as_deref() == Ok("pretty") }
fn tsrust_testing_style(text: &str, code: &str) -> std::string::String {
    let mode = std::env::var("TSRUST_COLOR").unwrap_or_else(|_| "auto".to_owned());
    let color = mode == "always" || (mode == "auto" && std::io::IsTerminal::is_terminal(&std::io::stdout()) && std::env::var_os("NO_COLOR").is_none() && std::env::var("TERM").as_deref() != Ok("dumb"));
    if color { format!("\x1b[{code}m{text}\x1b[0m") } else { text.to_owned() }
}
type TsrTestCallback = Box<dyn for<'p> FnMut(&mut TsrTestContext<'p>, &mut TsrTestRegistry) -> Result<(), std::string::String>>;
// Bundled registry and failure boundaries, included in generated programs.
enum TsrTestEntry {
    Test { name: std::string::String, callback: TsrTestCallback },
    Suite(usize),
}
struct TsrTestSuite {
    name: std::string::String, displayed: bool, entries: Vec<TsrTestEntry>,
    hooks: [Vec<TsrTestCallback>; 4],
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
struct TsrTestCounts { passed: i64, failed: i64, hook_failures: i64, files_passed: i64, files_failed: i64, failures: Vec<(std::string::String,std::string::String)> }
thread_local! {
    static TSR_TEST_RUNNING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
// Restore suite registration depth on typed returns and native unwinding.
fn tsrust_testing_scope<R>(registry: &mut TsrTestRegistry, body: impl FnOnce(&mut TsrTestRegistry) -> R) -> R {
    let depth = registry.current.len();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(registry)));
    registry.current.truncate(depth);
    match result { Ok(value) => value, Err(error) => std::panic::resume_unwind(error) }
}
fn tsrust_testing_open_scope(registry: &mut TsrTestRegistry, name: &String, displayed: bool) {
    assert!(!TSR_TEST_RUNNING.with(|running| running.get()), "describe() cannot register suites while a test is running");
    {
        let parent = *registry.current.last().unwrap();
        let id = registry.suites.len();
        registry.suites.push(TsrTestSuite::new(name.as_ref().to_owned(), displayed));
        registry.suites[parent].entries.push(TsrTestEntry::Suite(id));
        registry.current.push(id);
    }
}
fn tsrust_testing_begin_suite(registry: &mut TsrTestRegistry, name: &String) { tsrust_testing_open_scope(registry, name, true); }
fn tsrust_testing_begin_suite_result(registry: &mut TsrTestRegistry, name:&String)->Result<(),std::string::String> {
    if TSR_TEST_RUNNING.with(|running|running.get()) {return Err("describe() cannot register suites while a test is running".to_owned());}
    tsrust_testing_open_scope(registry,name,true);
    Ok(())
}
fn tsrust_testing_end_suite_result(registry: &mut TsrTestRegistry)->Result<(),std::string::String> {
    if registry.current.len()<=1 {return Err("No suite registration is active".to_owned());}
    tsrust_testing_end_suite(registry);
    Ok(())
}
fn tsrust_testing_begin_file(registry: &mut TsrTestRegistry, name: &String) { tsrust_testing_open_scope(registry, name, false); }
fn tsrust_testing_begin_file_result(registry: &mut TsrTestRegistry, name:&String)->Result<(),std::string::String> {
    if TSR_TEST_RUNNING.with(|running|running.get()) {return Err("Test files cannot register while a test is running".to_owned());}
    tsrust_testing_open_scope(registry,name,false);
    Ok(())
}
fn tsrust_testing_end_suite(registry: &mut TsrTestRegistry) { assert!(registry.current.len() > 1, "No suite registration is active"); registry.current.pop(); }
fn tsrust_testing_invoke(callback: &mut TsrTestCallback, program: &mut TsrTestContext<'_>, registry: &mut TsrTestRegistry) -> Option<std::string::String> {
    TSR_TEST_RUNNING.with(|running| running.set(true));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(program, registry)));
    TSR_TEST_RUNNING.with(|running| running.set(false));
    match result {
        Ok(result) => result.err(),
        Err(error) => Some(error.downcast_ref::<&str>().copied()
        .or_else(|| error.downcast_ref::<std::string::String>().map(|value| value.as_str()))
        .unwrap_or("native test failed").to_owned()),
    }
}
fn tsrust_testing_hooks(suite: &mut TsrTestSuite, kind: usize, reverse: bool, stop_on_failure: bool, program: &mut TsrTestContext<'_>, registry: &mut TsrTestRegistry) -> Vec<std::string::String> {
    let mut errors = Vec::new();
    let length = suite.hooks[kind].len();
    for position in 0..length {
        let index = if reverse { length - position - 1 } else { position };
        if let Some(error) = tsrust_testing_invoke(&mut suite.hooks[kind][index], program, registry) {
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
fn tsrust_testing_run_suite(registry: &mut TsrTestRegistry, id: usize, path: &mut Vec<std::string::String>, ancestors: &mut Vec<usize>, blocked: Option<&str>, counts: &mut TsrTestCounts, program: &mut TsrTestContext<'_>, active: &mut TsrTestRegistry) {
    if !tsrust_testing_has_tests(registry, id) { return; }
    let started = std::time::Instant::now();
    let previous_failures = counts.failed + counts.hook_failures;
    let file = id != 0 && !registry.suites[id].displayed;
    if file && tsrust_testing_pretty() {
        let name = &registry.suites[id].name;
        let cwd = std::env::current_dir().unwrap_or_default();
        let relative = std::path::Path::new(name).strip_prefix(&cwd).unwrap_or(std::path::Path::new(name));
        println!("\n {}", tsrust_testing_style(&relative.display().to_string(), "1"));
    }
    let displayed = registry.suites[id].displayed;
    if displayed { path.push(registry.suites[id].name.clone()); }
    ancestors.push(id);
    let label = if path.is_empty() { if registry.suites[id].name.is_empty() { "root".to_owned() } else { registry.suites[id].name.clone() } } else { path.join(" > ") };
    let label = if file { let cwd=std::env::current_dir().unwrap_or_default(); let path=std::path::Path::new(&label); path.strip_prefix(cwd).unwrap_or(path).display().to_string() } else { label };
    tsr_console_test_begin(&format!("{label} > beforeAll"));
    let setup_errors = if blocked.is_none() { tsrust_testing_hooks(&mut registry.suites[id], 0, false, true, program, active) } else { Vec::new() };
    tsr_console_test_end(true);
    for error in &setup_errors { counts.hook_failures += 1; println!("{} {label} beforeAll: {error}", tsrust_testing_style("HOOK FAIL", "1;31")); }
    let setup_error = setup_errors.first().map(|error| format!("beforeAll: {error}"));
    let blocked = blocked.or(setup_error.as_deref());
    let entries = std::mem::take(&mut registry.suites[id].entries);
    for entry in entries {
        match entry {
            TsrTestEntry::Suite(child) => tsrust_testing_run_suite(registry, child, path, ancestors, blocked, counts, program, active),
            TsrTestEntry::Test { name, mut callback } => {
                let test_started = std::time::Instant::now();
                path.push(name); let name = path.join(" > "); path.pop();
                tsr_console_test_begin(&name);
                let mut errors = Vec::new();
                if let Some(error) = blocked { errors.push(error.to_owned()); }
                else {
                    for &scope in ancestors.iter() {
                        let setup = tsrust_testing_hooks(&mut registry.suites[scope], 2, false, true, program, active);
                        if !setup.is_empty() { errors.extend(setup.into_iter().map(|error| format!("beforeEach: {error}"))); break; }
                    }
                    if errors.is_empty() { if let Some(error) = tsrust_testing_invoke(&mut callback, program, active) { errors.push(error); } }
                    for &scope in ancestors.iter().rev() {
                        errors.extend(tsrust_testing_hooks(&mut registry.suites[scope], 3, true, false, program, active).into_iter().map(|error| format!("afterEach: {error}")));
                    }
                }
                if errors.is_empty() {
                    counts.passed += 1;
                    if tsrust_testing_pretty() { println!("   {} {name} {}", tsrust_testing_style("✓", "32"), tsrust_testing_style(&format!("({:.2} ms)",test_started.elapsed().as_secs_f64()*1000.0),"2")); }
                    else { println!("{} {name}", tsrust_testing_style("PASS", "1;32")); }
                } else {
                    counts.failed += 1;
                    if tsrust_testing_pretty() {
                        println!("   {} {name}",tsrust_testing_style("×","31"));
                        let file_name = ancestors.iter().find_map(|&scope| (scope != 0 && !registry.suites[scope].displayed).then(|| registry.suites[scope].name.clone())).unwrap_or_default();
                        let cwd=std::env::current_dir().unwrap_or_default();
                        let file_name=std::path::Path::new(&file_name).strip_prefix(&cwd).unwrap_or(std::path::Path::new(&file_name)).display();
                        counts.failures.push((format!("{file_name} > {name}"),errors.join("\n")));
                    } else { println!("{} {name}: {}", tsrust_testing_style("FAIL", "1;31"), errors.join("; ")); }
                }
                tsr_console_test_end(false);
            }
        }
    }
    // Teardown belongs to scopes whose setup was attempted, including failed setup.
    if setup_error.is_some() || blocked.is_none() {
        tsr_console_test_begin(&format!("{label} > afterAll"));
        for error in tsrust_testing_hooks(&mut registry.suites[id], 1, true, false, program, active) { counts.hook_failures += 1; println!("{} {label} afterAll: {error}", tsrust_testing_style("HOOK FAIL", "1;31")); }
        tsr_console_test_end(true);
    }
    if file {
        if counts.failed + counts.hook_failures > previous_failures { counts.files_failed += 1; } else { counts.files_passed += 1; }
        if tsrust_testing_pretty() { println!("   {}",tsrust_testing_style(&format!("{:.2} ms",started.elapsed().as_secs_f64()*1000.0),"2")); }
    }
    ancestors.pop();
    if displayed { path.pop(); }
}
fn tsrust_testing_run_tests(active: &mut TsrTestRegistry, program: &mut TsrTestContext<'_>) -> i64 {
    tsrust_testing_run_tests_result(active, program).unwrap_or_else(|error| panic!("{error}"))
}
fn tsrust_testing_run_tests_result(active: &mut TsrTestRegistry, program: &mut TsrTestContext<'_>) -> Result<i64, std::string::String> {
    if TSR_TEST_RUNNING.with(|running| running.get()) {
        return Err("runTests() cannot run inside a test".to_owned());
    }
    if active.current.len()!=1 {
        return Err("runTests() cannot execute while a suite is registering".to_owned());
    }
    static HOOK: std::sync::Once = std::sync::Once::new();
    HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |information| {
            if !TSR_TEST_RUNNING.with(|running| running.get()) { previous(information); }
        }));
    });
    let mut registry = std::mem::take(active);
    let started = std::time::Instant::now();
    let mut counts = TsrTestCounts::default();
    tsrust_testing_run_in_scope(||tsrust_testing_run_suite(&mut registry, 0, &mut Vec::new(), &mut Vec::new(), None, &mut counts, program, active));
    if tsrust_testing_pretty() {
        if !counts.failures.is_empty() {
            println!("\n {}",tsrust_testing_style("Failed Tests","1;31"));
            for (index,(name,error)) in counts.failures.iter().enumerate() {
                println!("\n {}",tsrust_testing_style(&format!("{}. {name}",index+1),"1;31"));
                for line in error.lines() { println!("   {}",tsrust_testing_style(line,if line.starts_with("Expected:") {"32"} else if line.starts_with("Received:") {"31"} else {"0"})); }
            }
        }
        println!("\n Test Files  {} passed | {} failed ({})",counts.files_passed,counts.files_failed,counts.files_passed+counts.files_failed);
        println!("      Tests  {} passed | {} failed ({})",tsrust_testing_style(&counts.passed.to_string(),"32"),tsrust_testing_style(&counts.failed.to_string(),if counts.failed>0 {"31"}else{"2"}),counts.passed+counts.failed);
        if counts.hook_failures>0 { println!("      Hooks  {} failed",counts.hook_failures); }
        println!("   Duration  {:.2} ms (tests)\n",started.elapsed().as_secs_f64()*1000.0);
        return Ok(counts.failed + counts.hook_failures);
    }
    print!("{} passed, {} failed, 0 skipped", tsrust_testing_style(&counts.passed.to_string(), "1;32"), tsrust_testing_style(&counts.failed.to_string(), if counts.failed == 0 { "1" } else { "1;31" }));
    if counts.hook_failures != 0 { print!(", {} hook failures", counts.hook_failures); }
    println!();
    Ok(counts.failed + counts.hook_failures)
}
fn tsrust_testing_finish_tests(active: &mut TsrTestRegistry, program: &mut TsrTestContext<'_>) { if tsrust_testing_run_tests(active, program) != 0 { std::process::exit(1); } }

fn tsrust_testing_finish_tests_result(active: &mut TsrTestRegistry, program: &mut TsrTestContext<'_>)->Result<(),std::string::String>{
    let failed=tsrust_testing_run_tests_result(active, program)?;
    if failed==0 {Ok(())} else {Err(format!("test run failed: {failed} failure(s)"))}
}
