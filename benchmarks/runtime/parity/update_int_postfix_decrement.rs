// Independent Rust update: retain expression result and updated target.
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn update(mut value:i64)->(i64,i64){let previous=value;value=value.checked_sub(1).expect("integer decrement overflow");(previous,value)}
fn function0(value0: i64, value1: i64) -> NumericArray<i64> {
    let operands:[i64;8]=[2, 3, 8, 16, 64, 129, 256, 1024];let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());
    for i in 0..value0{let index=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();let(observed,value)=update(operands[index]);output.extend([observed,value]);}NumericArray(output)
}
fn main(){}
