// Handwritten lowering witness, not compiler output.
#[derive(Clone)]
struct User { name: String }
fn read(user: &User) { println!("{}", user.name); }
fn rename(user: &mut User) { user.name = "Sam".to_owned(); }
fn take(user: User) -> User { user }
// Negative backend control: intentionally omit ownership capture.
fn invalid() -> impl Fn() -> String {
    let user = User { name: "Pat".to_owned() };
    || name(&user)
}
fn name(user: &User) -> String { user.name.trim().to_owned() }
fn main() { let show = invalid(); println!("{}", show()); }
