#[derive(Clone)]
struct User { name: String }

fn log_both(first: &User, second: &User) {
    println!("{}", first.name);
    println!("{}", second.name);
}
fn main() {
    let user = User { name: "Pat".to_owned() };
    log_both(&user, &user);
}
