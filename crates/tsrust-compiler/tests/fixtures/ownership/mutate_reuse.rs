#[derive(Clone)]
struct User { name: String }

fn rename(user: &mut User) { user.name = "Sam".to_owned(); }
fn log(user: &User) { println!("{}", user.name); }
fn main() {
    let mut user = User { name: "Pat".to_owned() };
    rename(&mut user);
    log(&user);
}
