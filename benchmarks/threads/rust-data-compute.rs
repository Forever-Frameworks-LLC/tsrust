pub struct Job {pub seed:i64}
pub enum Outcome {Complete {value:i64},Skipped}
/// Run the recurrence job, consuming its input and returning an owned outcome.
pub fn calculate(job:Job)->Outcome {
    let mut value=job.seed;
    for _ in 0..250000 {value=value*48271%2147483647;}
    Outcome::Complete{value}
}
