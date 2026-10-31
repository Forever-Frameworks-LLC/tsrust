// Independent safe Rust reductions using ordered iterator folds and owned state.
type String=std::borrow::Cow<'static,str>;
fn visit(trace:&mut i64,visits:&mut i64,index:usize,len:usize,i:i64){*trace=trace.checked_mul(131).unwrap().checked_add(i64::try_from(index).unwrap()).unwrap().checked_add(i64::try_from(len).unwrap()).unwrap().checked_add(i).unwrap()%1000000007;*visits=visits.checked_add(1).unwrap();}
// Standard owned record; nominal field names satisfy the observation ABI.
#[derive(Clone)]struct Record0{field0:i64,field1:String,field2:Vec<Vec<i64>>,field3:i64,field4:i64}
fn initial()->Record0{Record0{field0:0,field1:String::Borrowed("initial"),field2:vec![],field3:0,field4:0}}
fn function0(value0: i64, value1: i64) -> Record0 {let base=vec![value1,value1.checked_add(1).unwrap(),2,3,value1.checked_add(4).unwrap()];let mut trace=value1;let mut visits=0_i64;let mut last=initial();for i in 0..value0{let mode=i.checked_add(value1).unwrap()%5;let count=[0_usize,1,4,5,5][usize::try_from(mode).unwrap()];let values=&base[..count];let selected=values.iter().enumerate().rev().fold(initial(),|mut a,(index,v)|{visit(&mut trace,&mut visits,index,values.len(),i);a.field2.push(vec![*v,i64::try_from(index).unwrap()]);Record0{field0:a.field0.checked_mul(17).unwrap().checked_add(*v).unwrap().checked_add(i64::try_from(index).unwrap()).unwrap()%1000003,field1:String::Owned(a.field1.replacen("initial","owned",1)),field2:a.field2,field3:trace,field4:visits}});last=selected;}last}
fn main(){}
