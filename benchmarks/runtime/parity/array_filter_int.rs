// Independent standard Rust iterators with retained callback order and state.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn visit_predicate(trace:&mut i64,visits:&mut i64,index:usize,i:i64,len:usize){*trace=trace.checked_mul(131).unwrap().checked_add(i64::try_from(index).unwrap()).unwrap().checked_add(*visits).unwrap().checked_add(i).unwrap().checked_add(i64::try_from(len).unwrap()).unwrap()%1000000007;*visits=visits.checked_add(1).unwrap();}
fn visit_transform(trace:&mut i64,visits:&mut i64,i:i64,seed:i64){*trace=trace.checked_mul(131).unwrap().checked_add(*visits).unwrap().checked_add(i).unwrap().checked_add(seed).unwrap()%1000000007;*visits=visits.checked_add(1).unwrap();}
fn function0(value0: i64, value1: i64) -> Array<NumericArray<i64>> {let values=[value1,value1.checked_add(1).unwrap(),2,3,value1.checked_add(4).unwrap(),5,6,7];let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(1).unwrap()).unwrap());let mut trace=value1;let mut visits=0_i64;for i in 0..value0{let selected=values.iter().enumerate().filter_map(|(index,value)|{visit_transform(&mut trace,&mut visits,i,value1);if value.checked_add(i).unwrap().checked_add(i64::try_from(index).unwrap()).unwrap()%3!=0{Some(value.clone())}else{None}}).collect::<Vec<_>>();output.push(NumericArray(selected));}output.push(NumericArray(vec![trace,visits]));Array(output)}
fn main(){}
