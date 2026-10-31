// Independent standard Rust array construction, ownership and iteration.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Profile{score:i64}
struct Entry{profile:Profile,name:&'static str}
struct Catalog{users:Vec<Entry>,rows:Vec<Vec<i64>>}
fn append(catalog:&mut Catalog,value:i64){catalog.users.push(Entry{profile:Profile{score:value.checked_add(2).unwrap()},name:"Sam"});catalog.rows[0].push(value);}
fn function0(value0: i64, value1: i64) -> i64 {let mut sum=value1;for i in 0..value0{let mut catalog=Catalog{users:vec![Entry{profile:Profile{score:i},name:"Pat"}],rows:vec![vec![value1]]};append(&mut catalog,i);for user in &catalog.users{sum=sum.checked_add(user.profile.score).unwrap();}for row in &catalog.rows{for score in row{sum=sum.checked_add(*score).unwrap();}}catalog.users[0].profile=Profile{score:i.checked_add(1).unwrap()};sum=sum.checked_add(catalog.users[0].profile.score).unwrap();}sum}
fn main(){}
