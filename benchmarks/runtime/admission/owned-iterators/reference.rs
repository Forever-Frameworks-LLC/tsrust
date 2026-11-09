// Admission only: owned Rust iterator composition with recoverable error items.
type Outcome<T> = Result<T,&'static str>;
fn source<T>(values:Vec<T>)->impl Iterator<Item=Outcome<T>>{values.into_iter().map(Ok)}
fn map<I,T,U,F>(input:I,mut callback:F)->impl Iterator<Item=Outcome<U>> where I:Iterator<Item=Outcome<T>>,F:FnMut(T)->Outcome<U>{input.map(move|value|value.and_then(&mut callback))}
fn filter<I,T,F>(input:I,mut callback:F)->impl Iterator<Item=Outcome<T>> where I:Iterator<Item=Outcome<T>>,F:FnMut(&T)->Outcome<bool>{input.filter_map(move|value|match value{Err(error)=>Some(Err(error)),Ok(value)=>match callback(&value){Ok(true)=>Some(Ok(value)),Ok(false)=>None,Err(error)=>Some(Err(error))}})}
fn checked_count(value:i64)->Outcome<usize>{usize::try_from(value).map_err(|_|"count")}
fn drop_items<I,T>(input:I,mut count:usize)->impl Iterator<Item=Outcome<T>> where I:Iterator<Item=Outcome<T>>{input.filter_map(move|value|{let skipping=count>0;if skipping{count-=1;}match value{Err(error)=>Some(Err(error)),Ok(value)if skipping=>{drop(value);None},Ok(value)=>Some(Ok(value))}})}
fn flat_array<I,T,U,F>(input:I,mut callback:F)->impl Iterator<Item=Outcome<U>> where I:Iterator<Item=Outcome<T>>,F:FnMut(T)->Outcome<Vec<U>>{input.flat_map(move|value|{let(error,values)=match value.and_then(&mut callback){Ok(values)=>(None,values),Err(error)=>(Some(error),Vec::new())};error.into_iter().map(Err).chain(values.into_iter().map(Ok))})}
fn flat_iterator<I,J,T,U,F>(input:I,mut callback:F)->impl Iterator<Item=Outcome<U>> where I:Iterator<Item=Outcome<T>>,J:Iterator<Item=Outcome<U>>,F:FnMut(T)->Outcome<J>{input.flat_map(move|value|{let(error,inner)=match value.and_then(&mut callback){Ok(inner)=>(None,Some(inner)),Err(error)=>(Some(error),None)};error.into_iter().map(Err).chain(inner.into_iter().flatten())})}
fn reduce<I,T,A,F>(mut input:I,initial:A,mut callback:F)->Outcome<A> where I:Iterator<Item=Outcome<T>>,F:FnMut(A,T)->Outcome<A>{input.try_fold(initial,move|acc,value|callback(acc,value?))}
fn for_each<I,T,F>(input:I,mut callback:F)->Outcome<()> where I:Iterator<Item=Outcome<T>>,F:FnMut(T)->Outcome<()>{for value in input{callback(value?)?;}Ok(())}
fn some<I,T,F>(input:I,mut callback:F)->Outcome<bool> where I:Iterator<Item=Outcome<T>>,F:FnMut(T)->Outcome<bool>{for value in input{if callback(value?)?{return Ok(true)}}Ok(false)}
fn every<I,T,F>(input:I,mut callback:F)->Outcome<bool> where I:Iterator<Item=Outcome<T>>,F:FnMut(T)->Outcome<bool>{for value in input{if !callback(value?)?{return Ok(false)}}Ok(true)}
fn find<I,T,F>(input:I,mut callback:F)->Outcome<Option<T>> where I:Iterator<Item=Outcome<T>>,F:FnMut(&T)->Outcome<bool>{for value in input{let value=value?;if callback(&value)?{return Ok(Some(value))}}Ok(None)}
fn to_array<I,T>(input:I)->Outcome<Vec<T>> where I:Iterator<Item=Outcome<T>>{input.collect()}
fn next<I,T>(input:&mut I)->Outcome<Option<T>> where I:Iterator<Item=Outcome<T>>{input.next().transpose()}
enum Either<A,B>{Left(A),Right(B)}
impl<A,B,T> Iterator for Either<A,B> where A:Iterator<Item=T>,B:Iterator<Item=T>{type Item=T;fn next(&mut self)->Option<T>{match self{Self::Left(value)=>value.next(),Self::Right(value)=>value.next()}}}
fn input(size:usize,seed:i64)->Vec<i64>{(0..size as i64).map(|value|value+seed).collect()}
fn chain_a(values:Vec<i64>)->impl Iterator<Item=Outcome<i64>>{map(filter(source(values),|value|Ok(value%2==0)),|value|Ok(value*3+1))}
fn chain_b(values:Vec<i64>)->impl Iterator<Item=Outcome<i64>>{filter(map(source(values),|value|Ok(value*3+1)),|value|Ok(value%2!=0))}
fn joined(values:Vec<i64>,left:bool)->impl Iterator<Item=Outcome<i64>>{if left{Either::Left(chain_a(values))}else{Either::Right(chain_b(values))}}
fn boxed(values:Vec<i64>,left:bool)->Box<dyn Iterator<Item=Outcome<i64>>>{
 let base:Box<dyn Iterator<Item=Outcome<i64>>>=Box::new(source(values));
 if left{let selected:Box<dyn Iterator<Item=Outcome<i64>>>=Box::new(filter(base,|value|Ok(value%2==0)));Box::new(map(selected,|value|Ok(value*3+1)))}
 else{let mapped:Box<dyn Iterator<Item=Outcome<i64>>>=Box::new(map(base,|value|Ok(value*3+1)));Box::new(filter(mapped,|value|Ok(value%2!=0)))}
}
fn digest(values:impl IntoIterator<Item=i64>)->i64{values.into_iter().fold(7,|sum,value|(sum*131+value)%1000000007)}
fn terminal<I:Iterator<Item=Outcome<i64>>>(name:&str,mut chain:I,seed:i64)->i64{match name{
 "reduce"=>reduce(chain,7,|sum,value|Ok((sum*131+value)%1000000007)).unwrap(),
 "some"=>{let mut visited=0;let result=some(chain,|value|{visited+=1;Ok(value>=seed*3+7)}).unwrap();visited*2+i64::from(result)},
 "take-array"=>digest(to_array(chain.take(8)).unwrap()),
 "partial-drop"=>{let first=next(&mut chain).unwrap().unwrap_or(-1);drop(chain);first},
 _=>panic!("unknown terminal"),
}}
fn kernel(name:&str,size:usize,seed:i64)->i64{
 let values=input(size,seed);
 if name=="flat-array"{return digest(to_array(flat_array(source(values).take(8),|value|Ok(vec![value,value+1]))).unwrap());}
 if name=="flat-iterator"{return digest(to_array(flat_iterator(source(values).take(8),|value|Ok(source(vec![value,value+1])))).unwrap());}
 let left=std::hint::black_box(seed%2==0);
 #[cfg(candidate_boxed)]{terminal(name,boxed(values,left),seed)}
 #[cfg(candidate_enum)]{terminal(name,joined(values,left),seed)}
 #[cfg(not(any(candidate_boxed,candidate_enum)))]{if left{terminal(name,chain_a(values),seed)}else{terminal(name,chain_b(values),seed)}}
}
fn correctness(){
 use std::{cell::RefCell,rc::Rc};
 #[derive(Debug)] struct Owner{index:usize,drops:Rc<RefCell<Vec<usize>>>}
 impl Drop for Owner{fn drop(&mut self){self.drops.borrow_mut().push(self.index);}}
 struct Capture(Rc<RefCell<usize>>);impl Drop for Capture{fn drop(&mut self){*self.0.borrow_mut()+=1;}}
 let drops=Rc::new(RefCell::new(Vec::new()));let owners=|| (0..6).map(|index|Owner{index,drops:drops.clone()}).collect::<Vec<_>>();
 let check_all=||{let mut observed=drops.borrow().clone();observed.sort_unstable();assert_eq!(observed,[0,1,2,3,4,5]);drops.borrow_mut().clear();};
 let output=to_array(drop_items(filter(source(owners()),|value|Ok(value.index%2==0)),1).take(1)).unwrap();assert_eq!(output[0].index,2);assert_eq!(drops.borrow().len(),5);drop(output);check_all();
 let capture_drops=Rc::new(RefCell::new(0));let capture=Capture(capture_drops.clone());let mut chain=map(source(owners()),move|value|{let _=&capture;if value.index==1{Err("map")}else{Ok(value)}});
 let first=next(&mut chain).unwrap().unwrap();assert_eq!(first.index,0);assert_eq!(next(&mut chain).unwrap_err(),"map");assert_eq!(drops.borrow().as_slice(),[1]);assert_eq!(*capture_drops.borrow(),0);let third=next(&mut chain).unwrap().unwrap();assert_eq!(third.index,2);drop(chain);assert_eq!(*capture_drops.borrow(),1);drop(first);drop(third);check_all();
 let mut chain=drop_items(map(source(owners()),|value|if value.index==1{Err("drop error")}else{Ok(value)}),1);assert_eq!(next(&mut chain).unwrap_err(),"drop error");assert_eq!(next(&mut chain).unwrap().unwrap().index,2);drop(chain);check_all();
 let mut chain=drop_items(std::iter::once(Err("first drop event")).chain(source(owners())),1);assert_eq!(next(&mut chain).unwrap_err(),"first drop event");let first=next(&mut chain).unwrap().unwrap();assert_eq!(first.index,0);drop(chain);drop(first);check_all();
 let mut chain=std::iter::once(Err("first take event")).chain(source(owners())).take(1);assert_eq!(next(&mut chain).unwrap_err(),"first take event");assert!(next(&mut chain).unwrap().is_none());drop(chain);check_all();
 let mut chain=filter(source(owners()),|value|if value.index==1{Err("filter")}else{Ok(true)});let first=next(&mut chain).unwrap().unwrap();assert_eq!(next(&mut chain).unwrap_err(),"filter");let third=next(&mut chain).unwrap().unwrap();assert_eq!(third.index,2);drop(chain);drop(first);drop(third);check_all();
 assert_eq!(checked_count(-1),Err("count"));assert_eq!(checked_count(0),Ok(0));assert_eq!(checked_count(i64::MAX),usize::try_from(i64::MAX).map_err(|_|"count"));
 assert_eq!(some(source(owners()),|value|if value.index==2{Err("some")}else{Ok(false)}),Err("some"));check_all();
 assert_eq!(every(source(owners()),|value|if value.index==2{Err("every")}else{Ok(true)}),Err("every"));check_all();
 assert!(matches!(find(source(owners()),|value|if value.index==2{Err("find")}else{Ok(false)}),Err("find")));check_all();
 assert_eq!(reduce(source(owners()),0,|acc,value|if value.index==2{Err("reduce")}else{Ok(acc+value.index)}),Err("reduce"));check_all();
 assert!(matches!(to_array(map(source(owners()),|value|if value.index==2{Err("collect")}else{Ok(value)})),Err("collect")));check_all();
 let mut visited=Vec::new();assert_eq!(some(source(owners()),|value|{visited.push(value.index);Ok(value.index==2)}),Ok(true));assert_eq!(visited,[0,1,2]);check_all();
 let mut visited=Vec::new();assert_eq!(every(source(owners()),|value|{visited.push(value.index);Ok(value.index<2)}),Ok(false));assert_eq!(visited,[0,1,2]);check_all();
 let found=find(source(owners()),|value|Ok(value.index==2)).unwrap().unwrap();assert_eq!(found.index,2);assert_eq!(drops.borrow().len(),5);drop(found);check_all();
 assert_eq!(for_each(source(owners()),|value|if value.index==2{Err("terminal")}else{Ok(())}),Err("terminal"));check_all();
 assert_eq!(reduce(source(owners()),Vec::new(),|mut acc,value|{acc.push(value.index);Ok(acc)}).unwrap(),[0,1,2,3,4,5]);check_all();
 let mut pulls=0;let chain=map(source(owners()),|value|{pulls+=1;Ok(value)}).take(0);drop(to_array(chain).unwrap());assert_eq!(pulls,0);check_all();
 for mode in [false,true]{let data=owners();let chain=if mode{Either::Left(source(data))}else{Either::Right(map(source(data),Ok))};drop(to_array(chain).unwrap());check_all();}
 let mut order=Vec::new();let mut chain=flat_array(source(vec![0,1,2]),|value|{order.push(value);if value==1{Err("flat")}else{Ok(vec![value,value+10])}});assert_eq!(next(&mut chain),Ok(Some(0)));assert_eq!(next(&mut chain),Ok(Some(10)));assert_eq!(next(&mut chain),Err("flat"));assert_eq!(next(&mut chain),Ok(Some(2)));assert_eq!(next(&mut chain),Ok(Some(12)));assert_eq!(next(&mut chain),Ok(None));drop(chain);assert_eq!(order,[0,1,2]);
 let mut chain=flat_iterator(source(vec![0,1,2]),|value|if value==1{Err("outer")}else{Ok(vec![Ok(value),Err("inner"),Ok(value+10)].into_iter())});assert_eq!(next(&mut chain),Ok(Some(0)));assert_eq!(next(&mut chain),Err("inner"));assert_eq!(next(&mut chain),Ok(Some(10)));assert_eq!(next(&mut chain),Err("outer"));assert_eq!(next(&mut chain),Ok(Some(2)));drop(chain);
 println!("{{\"kind\":\"contracts\",\"nonCloneOwners\":true,\"partialDrop\":true,\"retryAfterError\":true,\"errorsNotSkipped\":true,\"consumingSomeEvery\":true,\"borrowedFilterFind\":true,\"ownedFlatMapForms\":true,\"finiteOwnerJoin\":true,\"takeZeroPulls\":0,\"surplusDrops\":0}}");
}
