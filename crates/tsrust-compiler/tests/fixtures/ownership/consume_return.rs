#[derive(Clone)]
struct User { name: String }

fn take(user: User) -> User { user }
fn main() {
    let user = User { name: "Pat".to_owned() };
    let saved = take(user);
    println!("{}", saved.name);
}
