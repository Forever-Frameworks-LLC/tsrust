pub fn tsrust_std_time_monotonic_now()->f64 {
    static ORIGIN:std::sync::OnceLock<std::time::Instant>=std::sync::OnceLock::new();
    ORIGIN.get_or_init(std::time::Instant::now).elapsed().as_secs_f64()*1000.0
}
