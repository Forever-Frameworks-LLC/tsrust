// Same invocation ABI as the compiler's concrete owned callback environments.
trait Callback<A,R>{fn invoke(&mut self,args:A)->R;}
enum Step<T>{Done,Value(T)}
struct Pull<C,T>{callback:C,item:std::marker::PhantomData<fn()->T>}
impl<C,T> Iterator for Pull<C,T> where C:Callback<(),Outcome<Step<T>>>{type Item=Outcome<T>;fn next(&mut self)->Option<Self::Item>{match self.callback.invoke(()){Ok(Step::Done)=>None,Ok(Step::Value(value))=>Some(Ok(value)),Err(error)=>Some(Err(error))}}}
struct ArrayPull<T>{values:std::vec::IntoIter<T>}
impl<T> Callback<(),Outcome<Step<T>>> for ArrayPull<T>{fn invoke(&mut self,_:())->Outcome<Step<T>>{Ok(self.values.next().map_or(Step::Done,Step::Value))}}
fn pull_source<T>(values:Vec<T>)->Pull<ArrayPull<T>,T>{Pull{callback:ArrayPull{values:values.into_iter()},item:std::marker::PhantomData}}
struct MapPull<I,F>{source:I,callback:F}
impl<I,F,T,U> Callback<(),Outcome<Step<U>>> for MapPull<I,F> where I:Iterator<Item=Outcome<T>>,F:FnMut(T)->Outcome<U>{fn invoke(&mut self,_:())->Outcome<Step<U>>{match self.source.next(){None=>Ok(Step::Done),Some(value)=>Ok(Step::Value((self.callback)(value?)?))}}}
fn pull_map<I,F,T,U>(source:I,callback:F)->Pull<MapPull<I,F>,U> where I:Iterator<Item=Outcome<T>>,F:FnMut(T)->Outcome<U>{Pull{callback:MapPull{source,callback},item:std::marker::PhantomData}}
struct FilterPull<I,F>{source:I,callback:F}
impl<I,F,T> Callback<(),Outcome<Step<T>>> for FilterPull<I,F> where I:Iterator<Item=Outcome<T>>,F:FnMut(&T)->Outcome<bool>{fn invoke(&mut self,_:())->Outcome<Step<T>>{loop{match self.source.next(){None=>return Ok(Step::Done),Some(value)=>{let value=value?;if(self.callback)(&value)?{return Ok(Step::Value(value));}}}}}}
fn pull_filter<I,F,T>(source:I,callback:F)->Pull<FilterPull<I,F>,T> where I:Iterator<Item=Outcome<T>>,F:FnMut(&T)->Outcome<bool>{Pull{callback:FilterPull{source,callback},item:std::marker::PhantomData}}
fn chain_a(values:Vec<i64>)->impl Iterator<Item=Outcome<i64>>{
 #[cfg(candidate_pull)]{pull_map(pull_filter(pull_source(values),|value|Ok(value%2==0)),|value|Ok(value*3+1))}
 #[cfg(not(candidate_pull))]{std_chain_a(values)}
}
fn chain_b(values:Vec<i64>)->impl Iterator<Item=Outcome<i64>>{
 #[cfg(candidate_pull)]{pull_filter(pull_map(pull_source(values),|value|Ok(value*3+1)),|value|Ok(value%2!=0))}
 #[cfg(not(candidate_pull))]{std_chain_b(values)}
}
fn carrier_correctness(){
 use std::{cell::RefCell,rc::Rc};
 #[derive(Debug)]struct Owner{index:usize,drops:Rc<RefCell<Vec<usize>>>}
 impl Drop for Owner{fn drop(&mut self){self.drops.borrow_mut().push(self.index);}}
 struct Capture(Rc<RefCell<usize>>);impl Drop for Capture{fn drop(&mut self){*self.0.borrow_mut()+=1;}}
 let drops=Rc::new(RefCell::new(Vec::new()));let captures=Rc::new(RefCell::new(0));let capture=Capture(captures.clone());
 let mut chain=pull_map(pull_source((0..4).map(|index|Owner{index,drops:drops.clone()}).collect()),move|value|{let _=&capture;if value.index==1{Err("retry")}else{Ok(value)}});
 let first=next(&mut chain).unwrap().unwrap();assert_eq!(first.index,0);assert_eq!(next(&mut chain).unwrap_err(),"retry");assert_eq!(*captures.borrow(),0);assert_eq!(drops.borrow().as_slice(),[1]);let third=next(&mut chain).unwrap().unwrap();assert_eq!(third.index,2);drop(chain);assert_eq!(*captures.borrow(),1);drop(first);drop(third);let mut observed=drops.borrow().clone();observed.sort_unstable();assert_eq!(observed,[0,1,2,3]);
 let mut chain=pull_filter(pull_source(vec![0,1,2,3]),|value|if *value==1{Err("filter retry")}else{Ok(*value%2==0)});assert_eq!(next(&mut chain),Ok(Some(0)));assert_eq!(next(&mut chain),Err("filter retry"));assert_eq!(next(&mut chain),Ok(Some(2)));assert_eq!(next(&mut chain),Ok(None));
}
