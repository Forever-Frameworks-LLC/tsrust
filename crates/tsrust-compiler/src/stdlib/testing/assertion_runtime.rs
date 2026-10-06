
fn tsrust_testing_check_assertion(condition: bool, message: &String) {
    if !condition { panic!("AssertionError: {}", message.as_ref()); }
}
