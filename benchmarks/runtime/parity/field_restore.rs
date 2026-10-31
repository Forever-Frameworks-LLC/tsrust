// Independent standard Rust record ownership, recursion and retained arrays.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn sub(a: i64, b: i64) -> i64 {
    a.checked_sub(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn size(n: i64) -> usize {
    usize::try_from(n).unwrap()
}
struct Details {
    name: String,
    scores: Vec<i64>,
}
struct User {
    details: Details,
    score: i64,
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = value1;
    for i in 0..value0 {
        let mut user = User {
            details: Details {
                name: "Pat".into(),
                scores: vec![i, value1],
            },
            score: add(i, 1),
        };
        let scores = user.details.scores;
        let _name = user.details.name;
        sum = add(sum, user.score);
        user.details.scores = vec![add(i, 2)];
        user.details.name = "Sam".into();
        for score in scores {
            sum = add(sum, score);
        }
        sum = add(sum, user.details.scores[0]);
        let restored = user;
        sum = add(sum, restored.score);
    }
    sum
}
fn main() {}
