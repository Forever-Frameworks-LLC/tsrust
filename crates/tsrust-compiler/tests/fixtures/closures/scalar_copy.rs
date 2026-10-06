// Handwritten lowering witness, not compiler output.
#[derive(Clone)]
struct User { name: String }
fn read(user: &User) { println!("{}", user.name); }
fn rename(user: &mut User) { user.name = "Sam".to_owned(); }
fn take(user: User) -> User { user }
fn make(id: i64) -> impl Fn() -> i64 { move || id }
fn main() {
    let id = 7;
    let get = make(id);
    let flag = true;
    let check = move || flag;
    println!("{id}");
    println!("{}", get());
    println!("{}", get());
    println!("{flag}");
    println!("{}", check());
    println!("{}", check());
}
