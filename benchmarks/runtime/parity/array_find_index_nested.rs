// Independent standard Rust iterators with retained callback order and state.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn visit_predicate(trace:&mut i64,visits:&mut i64,index:usize,i:i64,len:usize){*trace=trace.checked_mul(131).unwrap().checked_add(i64::try_from(index).unwrap()).unwrap().checked_add(*visits).unwrap().checked_add(i).unwrap().checked_add(i64::try_from(len).unwrap()).unwrap()%1000000007;*visits=visits.checked_add(1).unwrap();}
fn visit_transform(trace:&mut i64,visits:&mut i64,i:i64,seed:i64){*trace=trace.checked_mul(131).unwrap().checked_add(*visits).unwrap().checked_add(i).unwrap().checked_add(seed).unwrap()%1000000007;*visits=visits.checked_add(1).unwrap();}
fn function0(value0: i64, value1: i64) -> Array<NumericArray<i64>> {let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());let mut trace=value1;let mut visits=0_i64;for i in 0..value0{let base=value1.checked_add(i).unwrap();let mode=i.checked_add(value1).unwrap()%11;let values=if mode==0{Vec::new()}else{vec![vec![base,1],vec![],vec![2,3,4],vec![base.checked_add(5).unwrap()],vec![],vec![9],vec![],vec![11,12,13]]};let (left,right,all)=match mode{2=>(0,-1,false),3=>(7,-1,false),4=>(3,-1,false),5=>(0,7,false),6=>(2,5,false),7=>(-1,-1,true),8=>(1,-1,false),9=>(6,-1,false),10=>(4,-1,false),_=>(-1,-1,false)};let answer=values.iter().enumerate().find(|&(index,value)|{visit_predicate(&mut trace,&mut visits,index,i,values.len());let hit=(!value.is_empty()) && (all||i64::try_from(index).unwrap()==left||i64::try_from(index).unwrap()==right);hit}).map(|(index,_)|i64::try_from(index).unwrap().checked_add(1).unwrap()).unwrap_or(0);output.push(NumericArray(vec![answer,trace,visits]));}Array(output)}
fn main(){}
