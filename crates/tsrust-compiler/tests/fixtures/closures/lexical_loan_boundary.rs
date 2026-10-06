// Handwritten lowering witness, not compiler output.
#[derive(Clone)]
struct User { name: String }
fn read(user: &User) { println!("{}", user.name); }
fn rename(user: &mut User) { user.name = "Sam".to_owned(); }
fn take(user: User) -> User { user }
fn main() {
    let mut user = User { name: "Pat".to_owned() };
    let show = || read(&user);
    show();
    rename(&mut user);
    read(&user);
}
