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
fn apply<F:Fn(&mut Trace,Vec<i64>)->Vec<i64>>(action:&F,trace:&mut Trace,mode:i64,base:i64)->Vec<i64>{action(trace,arguments(base,mode))}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {let _=value2;let mut trace=Trace{calls:0,defaults:0};let offset=value1%11;let callback=|call_trace:&mut Trace,values:Vec<i64>|{count_call(call_trace);let mut result=vec![offset];result.extend(values.into_iter().map(|value|value.checked_add(offset).unwrap()));result};let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());for i in 0..value0{let before=trace.calls;let mode=i.checked_add(value1).unwrap()%3;let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;output.push(NumericArray(apply(&callback,&mut trace,mode,base)));output.push(NumericArray(vec![trace.calls.checked_sub(before).unwrap()]));}Array(output)}
fn main(){}
