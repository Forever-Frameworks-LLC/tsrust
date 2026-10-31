// Independent standard Rust optional owned access with retained effect counters.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Trace{bases:i64,indices:i64,args:i64}
fn make_carrier(trace:&mut Trace,value:i64)->Option<Option<Vec<i64>>>{trace.bases=trace.bases.checked_add(1).unwrap();match value%5{0=>None,1=>Some(None),_=>Some(Some((0..value%4+1).map(|j|value.checked_add(j).unwrap()).collect()))}}
fn function0(value0: i64, value1: i64) -> Array<NumericArray<i64>> {
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());let mut trace=Trace{bases:0,indices:0,args:0};
    for i in 0..value0{let value=i.checked_add(value1).unwrap();let before=(trace.bases,trace.indices,trace.args);
        let selected=make_carrier(&mut trace,value).flatten().map(|items|{trace.indices=trace.indices.checked_add(1).unwrap();let at=usize::try_from(value%(value%4+1)).unwrap();items[at]});
        let(db,di,da)=(trace.bases.checked_sub(before.0).unwrap(),trace.indices.checked_sub(before.1).unwrap(),trace.args.checked_sub(before.2).unwrap());
        output.push(NumericArray(match selected{None=>Vec::new(),Some(value)=>vec![value]}));output.push(NumericArray(vec![db,di,da]));
    }Array(output)
}
fn main(){}
