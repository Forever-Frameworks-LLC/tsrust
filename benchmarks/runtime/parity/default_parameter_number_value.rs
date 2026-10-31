// Independent optional Rust argument and lazy fallback with retained counters.
type String=std::borrow::Cow<'static,str>;
struct Trace{calls:i64,defaults:i64}
fn apply(trace:&mut Trace,base:i64,value:Option<f64>)->f64{let value=value.unwrap_or_else(||{trace.defaults=trace.defaults.checked_add(1).unwrap();base.checked_add(100).unwrap() as f64/4.0});trace.calls=trace.calls.checked_add(1).unwrap();value}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Vec<f64>>{let _=value2;let values=[0.0,-0.0,f64::from_bits(1),-2.5,0.5,f64::INFINITY,f64::NEG_INFINITY,f64::NAN];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());let mut trace=Trace{calls:0,defaults:0};for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let coordinate=i.checked_mul(13).unwrap().checked_add(value1).unwrap();let base=coordinate%97;let at=usize::try_from(coordinate%8).unwrap();let before_calls=trace.calls;let before_defaults=trace.defaults;let observed=apply(&mut trace,base,if mode==2{Some(values[at])}else{None});output.push(vec![observed,trace.calls.checked_sub(before_calls).unwrap() as f64,trace.defaults.checked_sub(before_defaults).unwrap() as f64]);}output}
fn main(){}
