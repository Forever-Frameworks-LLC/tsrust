// Independent standard Rust nullable transport with owned output.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn display_number(value:f64)->String{if value.is_nan(){String::Borrowed("NaN")}else if value==f64::INFINITY{String::Borrowed("Infinity")}else if value==f64::NEG_INFINITY{String::Borrowed("-Infinity")}else if value==0.0{String::Borrowed("0")}else{String::Owned(value.to_string())}}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let values=[value1 as f64,-0.0,0.5,-2.5,17.0,f64::NAN,f64::INFINITY,f64::NEG_INFINITY];
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(4).unwrap()).unwrap());
    for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();let value=if mode==0{None}else{Some(values[at])};
match value{None=>output.extend(["object","null","false","sentinel"].map(String::Borrowed)),Some(value)=>output.extend([String::Borrowed("number"),display_number(value),String::Borrowed(if value!=0.0 && !value.is_nan(){"true"}else{"false"}),String::Borrowed("payload")])}}
    Array(output)
}
fn main(){}
