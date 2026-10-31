// Independent Rust slice with matched signed byte coordinates.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn coordinate(position:i64,len:usize)->usize{if position<0{usize::try_from(i64::try_from(len).unwrap().saturating_add(position).max(0)).unwrap()}else{usize::try_from(position).unwrap().min(len)}}
fn slice(text:&str,start:i64,end:i64)->&str{let a=coordinate(start,text.len());let b=coordinate(end,text.len());if b<=a{return "";}text.get(a..b).expect("UTF-8 boundary required")}
fn byte_coordinate(boundaries:&[i64],bytes:i64,width:i64,ordinal:i64)->i64{if ordinal<0{let index=width.checked_add(ordinal).unwrap();if index<0{-bytes}else{boundaries[usize::try_from(index).unwrap()]-bytes}}else if ordinal>width{bytes}else{boundaries[usize::try_from(ordinal).unwrap()]}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let mut boundaries=value2.char_indices().map(|(index,_)|i64::try_from(index).unwrap()).collect::<Vec<_>>();boundaries.push(i64::try_from(value2.len()).unwrap());let width=i64::try_from(boundaries.len()-1).unwrap();let bytes=i64::try_from(value2.len()).unwrap();let range=width.checked_mul(3).unwrap().checked_add(1).unwrap();let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let ordinal_start=(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%range).checked_sub(width).unwrap();let ordinal_end=(i.checked_mul(7).unwrap().checked_add(value1).unwrap()%range).checked_sub(width).unwrap();let start=byte_coordinate(&boundaries,bytes,width,ordinal_start);let end=byte_coordinate(&boundaries,bytes,width,ordinal_end);output.push(String::Owned(slice(value2.as_ref(),start,end).to_owned()));}Array(output)
}
fn main(){}
