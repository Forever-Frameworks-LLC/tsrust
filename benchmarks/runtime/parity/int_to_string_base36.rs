// Independent Rust signed integer formatting; radix output uses a sign
// and unsigned magnitude rather than signed two-complement formatting.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn format_integer(value:i64)->std::string::String{{let mut magnitude=value.unsigned_abs();let mut digits=Vec::with_capacity(14);loop{let digit=u32::try_from(magnitude%36).unwrap();digits.push(u8::try_from(u32::from(char::from_digit(digit,36).unwrap())).unwrap());magnitude/=36;if magnitude==0{break;}}if value<0{digits.push(b'-');}digits.reverse();std::string::String::from_utf8(digits).unwrap()}}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let values:[i64;16]=[0, 1, -1, 2, -2, 35, -35, 255, -255, 65535, -65535, 2147483647, -2147483648, 9007199254740991, -9007199254740991, 4294967296];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%16).unwrap();output.push(String::Owned(format_integer(values[at])));}Array(output)
}
fn main(){}
