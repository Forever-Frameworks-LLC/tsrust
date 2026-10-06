#[derive(Clone)]
struct User { name: String }

fn rename_and_log(first: &mut User, second: &User) {
    first.name = "Sam".to_owned();
    println!("{}", second.name);
}
fn main() {
    let mut user = User { name: "Pat".to_owned() };
    rename_and_log(&mut user, &user);
}
