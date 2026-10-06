#[derive(Clone)]
struct User { name: String }

fn rename(user: &mut User) { user.name = "Sam".to_owned(); }
fn main() {
    let user = User { name: "Pat".to_owned() };
    let mut backup = user.clone();
    rename(&mut backup);
    println!("{}", user.name);
    println!("{}", backup.name);
}
