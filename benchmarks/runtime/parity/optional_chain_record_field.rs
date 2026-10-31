// Independent standard Rust optional owned access with retained effect counters.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Trace{bases:i64,indices:i64,args:i64}
struct Leaf{id:i64,label:std::string::String}
struct Carrier{item:Leaf}
fn make_carrier(trace:&mut Trace,value:i64,input:&str)->Option<Option<Carrier>>{trace.bases=trace.bases.checked_add(1).unwrap();match value%5{0=>None,1=>Some(None),_=>Some(Some(Carrier{item:Leaf{id:value%4,label:if value%3==0{std::string::String::new()}else{format!("{input}:{value}")}}}))}}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());let mut trace=Trace{bases:0,indices:0,args:0};
    for i in 0..value0{let value=i.checked_add(value1).unwrap();let before=(trace.bases,trace.indices,trace.args);
        let selected=make_carrier(&mut trace,value,value2.as_ref()).flatten().map(|carrier|carrier.item);
        let(db,di,da)=(trace.bases.checked_sub(before.0).unwrap(),trace.indices.checked_sub(before.1).unwrap(),trace.args.checked_sub(before.2).unwrap());
        output.push(Array(match selected{None=>Vec::new(),Some(value)=>vec![String::Owned(value.id.to_string()),String::Owned(value.label)]}));output.push(Array(vec![String::Owned(db.to_string()),String::Owned(di.to_string()),String::Owned(da.to_string())]));
    }output
}
fn main(){}
