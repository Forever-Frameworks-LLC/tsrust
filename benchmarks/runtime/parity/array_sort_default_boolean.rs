// Independent standard Rust stable sorting and owned Vec copies.
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn compare_number(left:&f64,right:&f64)->std::cmp::Ordering{match(left.is_nan(),right.is_nan()){(true,true)=>std::cmp::Ordering::Equal,(true,false)=>std::cmp::Ordering::Greater,(false,true)=>std::cmp::Ordering::Less,(false,false)=>left.partial_cmp(right).unwrap()}}
fn function0(value0: i64, value1: i64) -> Array<NumericArray<bool>> {let sizes=[0_i64,1,4,5,17];let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(1).unwrap()).unwrap());
 for i in 0..value0{let count=sizes[usize::try_from(i.checked_add(value1).unwrap()%5).unwrap()];let mut values=Vec::new();for j in 0..count{values.push(j%3!=0);}values.sort();output.push(NumericArray(values));}Array(output)}
fn main(){}
