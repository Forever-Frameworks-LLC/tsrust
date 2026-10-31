// Independent standard Rust owned packs, borrowed calls and ordered evaluation.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn integers(base:i64,count:i64)->Vec<i64>{(0..count).map(|at|base.checked_add(at).unwrap()).collect()}
fn pack<T>(values:Vec<T>)->Vec<T>{values}
fn text(input:&str,value:i64)->std::string::String{input.repeat(usize::try_from(value%3+1).unwrap())+":"+&value.to_string()}
struct Trace{calls:i64,defaults:i64}
fn arguments_values(trace:&mut Trace,base:i64,count:i64)->Vec<i64>{trace.calls=trace.calls.checked_add(1).unwrap();integers(base,count)}
fn make_action(text:std::string::String)->impl FnOnce(&mut Trace,Vec<i64>)->Vec<String>{move|trace,values|{trace.calls=trace.calls.checked_add(1).unwrap();let mut result=vec![String::Owned(text)];result.extend(values.into_iter().map(|value|String::Owned(value.to_string())));result}}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());let mut calls=Trace{calls:0,defaults:0};let mut arguments=Trace{calls:0,defaults:0};for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let count=[0_i64,1,5][usize::try_from(mode).unwrap()];let argument_before=arguments.calls;let call_before=calls.calls;let action=if i.checked_add(value1).unwrap()%4!=0{Some(make_action(value2.repeat(usize::try_from(mode+1).unwrap())))}else{None};let result=if let Some(action)=action{let values=arguments_values(&mut arguments,base,count);action(&mut calls,values)}else{vec![String::Borrowed("missing")]};output.push(Array(result));output.push(Array(vec![String::Owned(arguments.calls.checked_sub(argument_before).unwrap().to_string()),String::Owned(calls.calls.checked_sub(call_before).unwrap().to_string())]));}output}
fn main(){}
