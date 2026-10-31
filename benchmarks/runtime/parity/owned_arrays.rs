// Independent standard Rust array construction, ownership and iteration.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct Entry{name:&'static str,score:i64}
fn function0(value0: i64, value1: i64) -> i64 {let mut sum=value1;for i in 0..value0{let mut entries=vec![Entry{name:" Pat ",score:i},Entry{name:"Sam",score:value1}];entries.push(Entry{name:" Jo ",score:i.checked_add(value1).unwrap()});entries[1].score=i.checked_add(1).unwrap();for entry in &entries{sum=sum.checked_add(entry.score).unwrap().checked_add(1).unwrap();}let saved=entries[1].clone();sum=sum.checked_add(saved.score).unwrap();let mut names=vec![" A "];names.push(" B ");sum=sum.checked_add(i64::try_from(names.len()).unwrap()).unwrap();}sum}
fn main(){}
