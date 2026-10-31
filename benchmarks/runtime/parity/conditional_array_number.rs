// Independent standard Rust arrays, Vec ownership and checked index arithmetic.
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
struct Trace{predicates:i64,left:i64,right:i64}
fn predicate(trace:&mut Trace,flag:bool)->bool{trace.predicates=trace.predicates.checked_add(1).unwrap();flag}
fn branch(trace:&mut Trace,at:i64,size:i64,left:bool)->Vec<f64>{if left{trace.left=trace.left.checked_add(1).unwrap();}else{trace.right=trace.right.checked_add(1).unwrap();}let pool=[0.0,-0.0,0.5,-0.5,f64::INFINITY,f64::NEG_INFINITY,f64::NAN,4.0];let mut values=Vec::new();for j in 0..size{values.push(pool[usize::try_from(at.checked_add(j).unwrap().checked_add(if left{0}else{3}).unwrap()%8).unwrap()]);}values}
fn function0(value0: i64, value1: i64) -> Vec<Vec<f64>> {let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(1).unwrap()).unwrap());let mut trace=Trace{predicates:0,left:0,right:0};for i in 0..value0{let size=i.checked_mul(7).unwrap().checked_add(value1.checked_mul(2).unwrap()).unwrap()%4;let at=i.checked_mul(13).unwrap().checked_add(value1.checked_mul(2).unwrap()).unwrap()%8;let flag=predicate(&mut trace,i.checked_add(value1).unwrap()%3==0);let selected=if flag{branch(&mut trace,at,size,true)}else{branch(&mut trace,at,size,false)};output.push(selected);}output.push(vec![trace.predicates as f64,trace.left as f64,trace.right as f64]);output}
fn main(){}
