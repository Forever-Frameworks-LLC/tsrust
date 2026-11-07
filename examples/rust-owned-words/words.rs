/// Split POSIX shell words. Unclosed quotes or a trailing escape return None.
/// Empty input returns Some with an empty vector. No shell command is executed.
pub fn split(value: &str) -> Option<Vec<String>> {
    shlex::split(value)
}
