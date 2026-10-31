// Independent standard Rust UTF-8 byte-position search with boundary checks.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn search(text:&str,needle:&str,position:Option<i64>)->bool{
    let at=position.map(|position|usize::try_from(position.max(0)).unwrap().min(text.len())).unwrap_or(text.len());
    text.get(..at).expect("UTF-8 boundary required").ends_with(needle)
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let needles=["","A","alpha","Z","missing\u{0000}","Aalpha","a"];let positions:[i64;14]=[-7, 0, 1, 3, 7, 8, 9, 11, 15, 16, 18, 22, 23, 999];let mut sum=value1;
    for i in 0..value0{
        let position=positions[usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%14).unwrap()];
        let needle=needles[usize::try_from(i.checked_mul(17).unwrap().checked_add(value1).unwrap()%7).unwrap()];
        let provided=if i.checked_add(value1).unwrap()%2==0{None}else{Some(position)};
        let code=i64::from(search(value2.as_ref(),needle,provided));sum=sum.checked_mul(131).unwrap().checked_add(code).unwrap()%1000000007;
    }sum
}
fn main(){}
