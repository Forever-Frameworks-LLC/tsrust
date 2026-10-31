// Independent standard Rust for_each over borrowed, read-only source slices.
type String=std::borrow::Cow<'static,str>;
fn string_weight(value:&str,input:&str)->i64{let mut weight=1_i64;for (condition,extra) in [(value==input,3),(value.contains('\0'),5),(value=="e\u{301}",7),(value=="é",11),(value=="😀",13)]{if condition{weight=weight.checked_add(extra).unwrap();}}weight}
fn function0(value0: i64, value1: i64) -> f64 {let base=[vec![value1 as f64/4.0,-0.0],vec![],vec![0.25,1.5],vec![-2.75],vec![0.0]];let mut trace=value1 as f64/4.0;let mut visits=0_i64;for i in 0..value0{let mode=i.checked_add(value1).unwrap()%5;let count=[0_usize,1,4,5,5][usize::try_from(mode).unwrap()];let selected=&base[..count];selected.iter().enumerate().for_each(|(index,value)|{let weight=value.iter().fold(0.0,|weight,item|weight+*item);trace=(trace*131.0+weight+index as f64/4.0+selected.len() as f64/4.0+i as f64/4.0)%1000003.0;visits=visits.checked_add(1).unwrap();});}trace+visits as f64/4.0}
fn main(){}
