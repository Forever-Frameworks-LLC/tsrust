// Independently authored Rust evaluation-order witness with standard Vecs.
struct Probe{trace:i64}
fn mark(probe:&mut Probe,tag:i64){probe.trace=probe.trace.checked_mul(10).unwrap().checked_add(tag).unwrap()%1000003;}
fn receiver(base:&[i64],count:usize,probe:&mut Probe)->Vec<i64>{mark(probe,1);base[..count].to_vec()}
fn array_argument(base:&[i64],end:usize,probe:&mut Probe)->Vec<i64>{mark(probe,2);base[1..end].to_vec()}
fn scalar_argument(value:i64,probe:&mut Probe)->i64{mark(probe,3);value}
fn function0(value0: i64, value1: i64) -> i64 {let base=[value1,value1.checked_add(1).unwrap(),2,0,value1.checked_add(4).unwrap()];let mut probe=Probe{trace:value1};let mut trace=value1;for i in 0..value0{let mode=i.checked_add(value1).unwrap()%4;let count=[0_usize,1,4,5][usize::try_from(mode).unwrap()];let mut output=receiver(&base,count,&mut probe);let array=array_argument(&base,usize::try_from(mode%3+2).unwrap(),&mut probe);let scalar=scalar_argument(value1.checked_add(i).unwrap(),&mut probe);output.extend(array);output.push(scalar);for (index,value) in output.into_iter().enumerate(){trace=trace.checked_mul(131).unwrap().checked_add(value).unwrap().checked_add(i64::try_from(index).unwrap()).unwrap().checked_add(i).unwrap()%1000000007;}trace=trace.checked_mul(131).unwrap().checked_add(probe.trace).unwrap()%1000000007;}trace}
fn main(){}
