// Independent standard Rust iterators with retained callback order and state.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn visit_predicate(trace:&mut i64,visits:&mut i64,index:usize,i:i64,len:usize){*trace=trace.checked_mul(131).unwrap().checked_add(i64::try_from(index).unwrap()).unwrap().checked_add(*visits).unwrap().checked_add(i).unwrap().checked_add(i64::try_from(len).unwrap()).unwrap()%1000000007;*visits=visits.checked_add(1).unwrap();}
fn visit_transform(trace:&mut i64,visits:&mut i64,i:i64,seed:i64){*trace=trace.checked_mul(131).unwrap().checked_add(*visits).unwrap().checked_add(i).unwrap().checked_add(seed).unwrap()%1000000007;*visits=visits.checked_add(1).unwrap();}
fn function0(value0: i64, value1: i64) -> Vec<Vec<f64>> {let values=[value1 as f64+0.25,-0.0,2.25,value1 as f64+1.5,0.0,4.5,5.25,-2.5];let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(1).unwrap()).unwrap());let mut trace=value1;let mut visits=0_i64;for i in 0..value0{let selected=values.iter().enumerate().map(|(index,value)|{visit_transform(&mut trace,&mut visits,i,value1);*value*2.0}).collect::<Vec<_>>();output.push(selected);}output.push(vec![trace as f64,visits as f64]);output}
fn main(){}
