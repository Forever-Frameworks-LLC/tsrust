// Handwritten lowering witness, not compiler output.
#[derive(Clone)]
struct User { name: String }
fn read(user: &User) { println!("{}", user.name); }
fn rename(user: &mut User) { user.name = "Sam".to_owned(); }
fn take(user: User) -> User { user }
fn maybe(user: User, flag: bool) {
    if flag {
        let get = move || take(user);
        drop(get);
    }
    read(&user);
}
fn main() { maybe(User { name: "Pat".to_owned() }, false); }
