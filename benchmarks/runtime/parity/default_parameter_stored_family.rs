// Independent standard Rust optional arguments, owned rest packs and closures.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Trace{calls:i64,defaults:i64}
fn count_call(trace:&mut Trace){trace.calls=trace.calls.checked_add(1).unwrap();}
fn count_default(trace:&mut Trace){trace.defaults=trace.defaults.checked_add(1).unwrap();}
fn arguments(base:i64,mode:i64)->Vec<i64>{(0..[0_i64,1,5][usize::try_from(mode).unwrap()]).map(|at|base.checked_add(at).unwrap()).collect()}
fn text(input:&str,base:i64,kind:&str)->std::string::String{input.repeat(usize::try_from(base%3+1).unwrap())+":"+kind+&base.to_string()}
fn pack<T>(values:Vec<T>)->Vec<T>{values}
struct Holder{run:fn(&mut Trace,Option<i64>)->i64}
fn default_value(trace:&mut Trace,value:i64)->i64{count_default(trace);value}
fn first(trace:&mut Trace,value:Option<i64>)->i64{let value=value.unwrap_or_else(||default_value(trace,10));count_call(trace);value.checked_add(1).unwrap()}
fn second(trace:&mut Trace,value:Option<i64>)->i64{let value=value.unwrap_or_else(||default_value(trace,100));count_call(trace);value.checked_add(2).unwrap()}
fn forward(trace:&mut Trace,holder:&Holder,value:Option<i64>)->i64{(holder.run)(trace,value)}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {let _=value2;let mut trace=Trace{calls:0,defaults:0};let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%4;let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let before_calls=trace.calls;let before_defaults=trace.defaults;let holder=Holder{run:if i.checked_add(value1).unwrap()%2==0{first}else{second}};let observed=forward(&mut trace,&holder,if mode<2{None}else{Some(base)});output.push(NumericArray(vec![observed]));output.push(NumericArray(vec![trace.calls.checked_sub(before_calls).unwrap(),trace.defaults.checked_sub(before_defaults).unwrap()]));}Array(output)}
fn main(){}
