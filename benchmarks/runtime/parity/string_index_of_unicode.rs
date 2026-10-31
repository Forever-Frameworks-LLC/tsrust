// Independent standard Rust find with matched clamped UTF-8 byte positions.
type String=std::borrow::Cow<'static,str>;
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn index_of(text:&str,needle:&str,position:Option<i64>)->i64{let start=position.map(|x|usize::try_from(x.max(0)).unwrap().min(text.len())).unwrap_or(0);let suffix=text.get(start..).expect("UTF-8 boundary required");suffix.find(needle).map(|at|i64::try_from(start.checked_add(at).unwrap()).unwrap()).unwrap_or(-1)}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let needles=["","A","é","😀","é","absent\u{0000}","é😀"];let positions:[i64;14]=[-7, 0, 1, 3, 7, 8, 9, 11, 15, 16, 18, 22, 23, 999];let mut sum=value1;
    for i in 0..value0{let position=positions[usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%14).unwrap()];let needle=needles[usize::try_from(i.checked_mul(17).unwrap().checked_add(value1).unwrap()%7).unwrap()];let provided=if i.checked_add(value1).unwrap()%2==0{None}else{Some(position)};let code=index_of(value2.as_ref(),needle,provided).checked_add(1).unwrap();sum=sum.checked_mul(131).unwrap().checked_add(code).unwrap()%1000000007;}sum
}
fn main(){}
