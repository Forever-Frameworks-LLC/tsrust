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
fn argument(trace:&mut Trace,value:i64)->i64{count_call(trace);value}
fn with_prefix(trace:&mut Trace,prefix:Option<i64>,values:Vec<i64>)->Vec<i64>{let prefix=prefix.unwrap_or_else(||{count_default(trace);10});let mut result=vec![prefix];result.extend(values);result}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {let _=value2;let mut trace=Trace{calls:0,defaults:0};let mut arguments_trace=Trace{calls:0,defaults:0};let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let before=arguments_trace.calls;let defaults=trace.defaults;let values=arguments(base,mode).into_iter().map(|value|argument(&mut arguments_trace,value)).collect();let observed=with_prefix(&mut trace,if mode<2{None}else{Some(base)},values);output.push(NumericArray(observed));output.push(NumericArray(vec![arguments_trace.calls.checked_sub(before).unwrap(),trace.defaults.checked_sub(defaults).unwrap()]));}Array(output)}
fn main(){}
