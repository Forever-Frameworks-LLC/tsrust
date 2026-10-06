// Handwritten lowering witness, not compiler output.
#[derive(Clone)]
struct User { name: String }
fn read(user: &User) { println!("{}", user.name); }
fn rename(user: &mut User) { user.name = "Sam".to_owned(); }
fn take(user: User) -> User { user }
fn make(user: User) -> impl FnOnce() -> User { move || take(user) }
fn main() {
    let get = make(User { name: "Pat".to_owned() });
    let saved = get();
    read(&saved);
}
