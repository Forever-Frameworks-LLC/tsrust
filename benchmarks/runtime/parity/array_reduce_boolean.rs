// Independent safe Rust reductions using ordered iterator folds and owned state.
type String=std::borrow::Cow<'static,str>;
fn visit(trace:&mut i64,visits:&mut i64,index:usize,len:usize,i:i64){*trace=trace.checked_mul(131).unwrap().checked_add(i64::try_from(index).unwrap()).unwrap().checked_add(i64::try_from(len).unwrap()).unwrap().checked_add(i).unwrap()%1000000007;*visits=visits.checked_add(1).unwrap();}
fn function0(value0: i64, value1: i64) -> bool {let base=vec![true,false,value1%2==0,true,false];let mut trace=value1;let mut visits=0_i64;let mut last=false;for i in 0..value0{let mode=i.checked_add(value1).unwrap()%5;let count=[0_usize,1,4,5,5][usize::try_from(mode).unwrap()];let values=&base[..count];let mut iter=values.iter().enumerate();let initial=if mode==3{iter.next().unwrap().1.clone()}else{last};let selected=iter.fold(initial,|a,(index,v)|{visit(&mut trace,&mut visits,index,values.len(),i);(a!=*v)!=(index%2==0)});last=selected;}last!=(trace%2==0)}
fn main(){}
