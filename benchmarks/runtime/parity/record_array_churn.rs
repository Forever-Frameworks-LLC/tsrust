// Independent standard Rust array construction, ownership and iteration.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct Wide{name:&'static str,values:Vec<i64>,a:i64,b:i64,c:i64,d:i64}
fn weight(value:&Wide)->i64{value.values[0].checked_add(value.values[1]).unwrap().checked_add(value.a).unwrap().checked_add(value.b).unwrap().checked_add(value.c).unwrap().checked_add(value.d).unwrap()}
fn function0(value0: i64, value1: i64) -> i64 {let mut sum=value1;for i in 0..value0{let width=i.checked_add(value1).unwrap()%17;let mut records=Vec::new();for j in 0..width{records.push(Wide{name:"Pat",values:vec![i,j],a:i.checked_add(value1).unwrap(),b:j,c:value1,d:i});}let mut saved=records.clone();if width>0{records[0].values[0]=value1;let item=saved.remove(usize::try_from(i.checked_add(value1).unwrap()%width).unwrap());sum=sum.checked_add(weight(&item)).unwrap();}for record in &saved{sum=sum.checked_add(weight(record)).unwrap();}for record in &records{sum=sum.checked_add(weight(record)).unwrap();}}sum}
fn main(){}
