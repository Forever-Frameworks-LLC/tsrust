// Independent standard Rust arrays, Vec ownership and checked index arithmetic.
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
struct Trace{predicates:i64,left:i64,right:i64}
fn predicate(trace:&mut Trace,flag:bool)->bool{trace.predicates=trace.predicates.checked_add(1).unwrap();flag}
fn branch(trace:&mut Trace,at:i64,size:i64,left:bool)->Vec<bool>{if left{trace.left=trace.left.checked_add(1).unwrap();}else{trace.right=trace.right.checked_add(1).unwrap();}let mut values=Vec::new();for j in 0..size{values.push(at.checked_add(j).unwrap().checked_add(if left{0}else{1}).unwrap()%3==0);}values}
fn function0(value0: i64, value1: i64) -> Array<NumericArray<i64>> {let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(1).unwrap()).unwrap());let mut trace=Trace{predicates:0,left:0,right:0};for i in 0..value0{let size=i.checked_mul(7).unwrap().checked_add(value1.checked_mul(2).unwrap()).unwrap()%4;let at=i.checked_mul(13).unwrap().checked_add(value1.checked_mul(2).unwrap()).unwrap()%8;let flag=predicate(&mut trace,i.checked_add(value1).unwrap()%2==0);let selected=if flag{branch(&mut trace,at,size,true)}else{branch(&mut trace,at,size,false)};output.push(NumericArray(selected.into_iter().map(i64::from).collect()));}output.push(NumericArray(vec![trace.predicates,trace.left,trace.right]));Array(output)}
fn main(){}
