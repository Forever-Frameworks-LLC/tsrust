#[derive(Clone)]
struct User { name: String }

fn normalize(user: &User) -> String {
    user.name.trim().to_owned().to_lowercase()
}
fn main() {
    let user = User { name: " Pat ".to_owned() };
    println!("{}", normalize(&user));
    println!("{}", user.name);
}
