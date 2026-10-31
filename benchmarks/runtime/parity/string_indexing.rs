// Independent Rust checked ASCII indexing via Result, with owned retained rows.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn at(text:&str,index:i64)->Result<&str,()>{let index=usize::try_from(index).map_err(|_|())?;let end=index.checked_add(1).ok_or(())?;text.get(index..end).ok_or(())}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let letters="abcdefghij";let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let position=i.checked_add(value1).unwrap()%10;let first=at(letters,position).unwrap();let pair=format!("{}{}",at(letters,position).unwrap(),at(letters,9_i64.checked_sub(position).unwrap()).unwrap());let later=at(letters,position.checked_add(5).unwrap()).unwrap_or("out");output.push(Array(vec![String::Borrowed(first),String::Owned(pair),String::Borrowed(later),String::Owned(position.to_string()),String::Owned(value2.to_string())]));}output
}
fn main(){}
