// Handwritten lowering witness, not compiler output.
#[derive(Clone)]
struct User { name: String }
fn read(user: &User) { println!("{}", user.name); }
fn rename(user: &mut User) { user.name = "Sam".to_owned(); }
fn take(user: User) -> User { user }
fn make(user: &User) -> impl Fn() -> String + use<> {
    let saved = user.clone();
    move || name(&saved)
}
fn name(user: &User) -> String { user.name.trim().to_owned() }
fn main() {
    let mut user = User { name: "Pat".to_owned() };
    let show = make(&user);
    rename(&mut user);
    println!("{}", show());
    read(&user);
}
