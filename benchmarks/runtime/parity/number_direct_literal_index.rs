// Independent fixed Rust literal evaluation with observable side-effect order.
struct Probe{trace:i64,calls:i64}
fn mark(probe:&mut Probe,value:f64,tag:i64)->f64{probe.calls=probe.calls.checked_add(1).unwrap();probe.trace=probe.trace.checked_mul(131).unwrap().checked_add(tag).unwrap()%1000000007;value}
fn mark_index(probe:&mut Probe,index:i64)->usize{mark(probe,0.0,9);usize::try_from(index).unwrap()}
fn function0(value0: i64, value1: i64) -> Vec<f64>{let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(2).unwrap()).unwrap());let mut probe=Probe{trace:value1,calls:0};for i in 0..value0{let values=[mark(&mut probe,value1 as f64+i as f64*0.25,1),mark(&mut probe,-0.0,2),mark(&mut probe,f64::NAN,3),mark(&mut probe,f64::INFINITY,4),mark(&mut probe,f64::NEG_INFINITY,5),mark(&mut probe,f64::from_bits(1),6),mark(&mut probe,f64::MAX,7),mark(&mut probe,-2.5,8)];let at=mark_index(&mut probe,i.checked_mul(3).unwrap().checked_add(value1).unwrap()%8);output.push(values[at]);}output.push(probe.trace as f64);output.push(probe.calls as f64);output}
fn main(){}
