// Independent Rust monotonic clock with one process-lifetime epoch.
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
static EPOCH: std::sync::LazyLock<std::time::Instant> =
    std::sync::LazyLock::new(std::time::Instant::now);
fn now() -> f64 {
    EPOCH.elapsed().as_secs_f64() * 1000.0
}
fn function0(value0: i64, value1: i64) -> NumericArray<bool> {
    let _ = value1;
    let mut values = Vec::new();
    let mut previous = now();
    for _ in 0..value0 {
        let current = now();
        values.push(current.is_finite() && current >= previous && current >= 0.0);
        previous = current;
    }
    NumericArray(values)
}
fn main() {}
