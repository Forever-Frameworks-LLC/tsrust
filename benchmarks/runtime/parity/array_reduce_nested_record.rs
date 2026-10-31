// Independent safe Rust reductions using ordered iterator folds and owned state.
type String=std::borrow::Cow<'static,str>;
fn visit(trace:&mut i64,visits:&mut i64,index:usize,len:usize,i:i64){*trace=trace.checked_mul(131).unwrap().checked_add(i64::try_from(index).unwrap()).unwrap().checked_add(i64::try_from(len).unwrap()).unwrap().checked_add(i).unwrap()%1000000007;*visits=visits.checked_add(1).unwrap();}
// Standard owned record; nominal field names satisfy the observation ABI.
#[derive(Clone)]struct Record0{field0:i64,field1:String,field2:Vec<Vec<i64>>,field3:i64,field4:i64}
fn initial()->Record0{Record0{field0:0,field1:String::Borrowed("initial"),field2:vec![],field3:0,field4:0}}
fn function0(value0: i64, value1: i64) -> Vec<Vec<Record0>> {let base=vec![vec![Record0{field0:value1,field1:String::Borrowed("é"),field2:vec![vec![value1]],field3:0,field4:0}],vec![],vec![Record0{field0:2,field1:String::Borrowed("😀"),field2:vec![],field3:0,field4:0}],vec![Record0{field0:3,field1:String::Borrowed("x"),field2:vec![vec![1]],field3:0,field4:0}],vec![]];let mut trace=value1;let mut visits=0_i64;let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%5;let count=[0_usize,1,4,5,5][usize::try_from(mode).unwrap()];let values=&base[..count];let mut iter=values.iter().enumerate();let initial=if mode==3{iter.next().unwrap().1.clone()}else{Vec::new()};let selected=iter.fold(initial,|a,(index,v)|{visit(&mut trace,&mut visits,index,values.len(),i);v.clone()});output.push(selected);}output}
fn main(){}
