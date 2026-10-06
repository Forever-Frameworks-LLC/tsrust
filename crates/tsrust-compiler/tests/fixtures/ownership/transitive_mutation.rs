#[derive(Clone)]
struct User { name: String }

fn rename(user: &mut User) { user.name = "Sam".to_owned(); }
fn normalize(user: &mut User) { rename(user); }
fn main() {
    let mut user = User { name: "Pat".to_owned() };
    normalize(&mut user);
    println!("{}", user.name);
}
