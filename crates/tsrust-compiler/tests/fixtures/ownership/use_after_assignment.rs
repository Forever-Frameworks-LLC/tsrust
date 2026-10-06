#[derive(Clone)]
struct User { name: String }

fn main() {
    let user = User { name: "Pat".to_owned() };
    let backup = user;
    println!("{}", backup.name);
    println!("{}", user.name);
}
