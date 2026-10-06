// Handwritten lowering witness, not compiler output.
#[derive(Clone)]
struct User { name: String }
fn read(user: &User) { println!("{}", user.name); }
fn rename(user: &mut User) { user.name = "Sam".to_owned(); }
fn take(user: User) -> User { user }
fn make(mut user: User) -> impl FnMut() -> String {
    move || { rename(&mut user); name(&user) }
}
fn name(user: &User) -> String { user.name.trim().to_owned() }
fn main() {
    let mut change = make(User { name: "Pat".to_owned() });
    println!("{}", change());
    println!("{}", change());
}
