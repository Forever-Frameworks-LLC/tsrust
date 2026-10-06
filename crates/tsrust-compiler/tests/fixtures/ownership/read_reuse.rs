#[derive(Clone)]
struct User { name: String }

fn log(user: &User) { println!("{}", user.name); }
fn main() {
    let user = User { name: "Pat".to_owned() };
    log(&user);
    log(&user);
}
