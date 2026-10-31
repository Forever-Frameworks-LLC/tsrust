// Independent standard Rust construction and one-level iterator flattening.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn item(seed:i64,ordinal:i64)->bool{ordinal.checked_add(seed).unwrap()%3!=0}
fn piece(value:bool,index:usize,len:usize)->Vec<bool>{if index%3==0{return Vec::new();}vec![value,!value]}
fn function0(value0: i64, value1: i64) -> Array<NumericArray<bool>> {let sizes=[0_i64,1,4,5,17];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let count=sizes[usize::try_from(i.checked_add(value1).unwrap()%5).unwrap()];let mut source=Vec::new();for ordinal in 0_i64..count{source.push(item(value1,ordinal));}let len=source.len();let flattened:Vec<_>=source.into_iter().enumerate().flat_map(|(index,value)|piece(value,index,len)).collect();output.push(NumericArray(flattened));}Array(output)}
fn main(){}
