/// Runs one recurrence job, consuming and returning only copied integer values.
pub fn calculate(seed:i64)->i64 {
    let mut value=seed;
    for _ in 0..250000 {value=value*48271%2147483647;}
    value
}
