use rayon::prelude::*;

/// Sums squared values using the selected worker pool's Rayon scheduler.
/// The input buffer moves into Rust and is released when calculation ends.
pub fn sum_squares(values: Vec<i64>) -> i64 {
    values.into_par_iter().map(|value| value * value).sum()
}

/// Waits for this worker's cancellation request.
pub fn listen(task: &TaskContext) {
    while !task.cancellation_requested() { std::thread::yield_now(); }
}
