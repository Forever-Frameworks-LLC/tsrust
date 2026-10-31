// Independent standard Rust repeat, preserving receiver/count trace order.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Probe{trace:i64,calls:i64}
fn mark(probe:&mut Probe,index:i64,tag:i64)->i64{probe.trace=probe.trace.checked_mul(131).unwrap().checked_add(probe.calls).unwrap().checked_add(index%65521).unwrap().checked_add(65521).unwrap().checked_add(tag).unwrap()%65521;probe.calls=probe.calls.checked_add(1).unwrap();index}
fn markers(probe:&Probe)->Array<String>{let mut row=vec![String::Borrowed("trace")];for (index,initial) in [probe.trace,probe.calls].into_iter().enumerate(){if index==1{row.push(String::Borrowed("calls"));}let mut value=initial;for bit in (0..16).rev(){let factor=1_i64<<bit;if value>=factor{row.push(String::Borrowed("1"));value=value.checked_sub(factor).unwrap();}else{row.push(String::Borrowed("0"));}}}Array(row)}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let pool=[String::Owned(value2.to_string()),String::Borrowed("é😀\u{0000}é")];let counts:[i64;4]=[0, 1, 2, 3];
    let mut probe=Probe{trace:value1%65521,calls:0};let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(1).unwrap()).unwrap());
    for i in 0..value0{let coordinate=i.checked_add(value1).unwrap();let at=usize::try_from(mark(&mut probe,coordinate%2,1)).unwrap();
        let count=counts[usize::try_from(coordinate%4).unwrap()];let count=usize::try_from(mark(&mut probe,count,2)).unwrap();
        output.push(Array(vec![String::Owned(pool[at].repeat(count))]));
    }output.push(markers(&probe));output
}
fn main(){}
