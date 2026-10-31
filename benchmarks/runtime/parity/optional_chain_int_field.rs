// Independent Rust optional field access; base evaluation remains observable.
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Carrier{value:i64}
struct Trace{bases:i64,indices:i64,args:i64}
fn make_carrier(trace:&mut Trace,value:i64)->Option<Option<Carrier>>{trace.bases=trace.bases.checked_add(1).unwrap();match value%5{0=>None,1=>Some(None),_=>Some(Some(Carrier{value:value%4}))}}
fn function0(value0: i64, value1: i64) -> Array<NumericArray<i64>> {
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());let mut trace=Trace{bases:0,indices:0,args:0};
    for i in 0..value0{let value=i.checked_add(value1).unwrap();let before=(trace.bases,trace.indices,trace.args);
        let selected=make_carrier(&mut trace,value).flatten().map(|carrier|carrier.value);
        let(db,di,da)=(trace.bases.checked_sub(before.0).unwrap(),trace.indices.checked_sub(before.1).unwrap(),trace.args.checked_sub(before.2).unwrap());
        output.push(match selected{None=>NumericArray(Vec::new()),Some(selected)=>NumericArray(vec![selected])});output.push(NumericArray(vec![db,di,da]));
    }Array(output)
}
fn main(){}
