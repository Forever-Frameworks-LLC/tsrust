// Independent standard Rust array construction, ownership and iteration.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
// ponytail: this tiny insertion-ordered corpus uses linear Vec membership;
// use an ordered-collection crate if a large-cardinality corpus is introduced.
fn ordered_unique(values:impl IntoIterator<Item=i64>)->Vec<i64>{let mut keys=Vec::new();for value in values{if !keys.contains(&value){keys.push(value);}}keys}
fn ordered_set(values:&mut Vec<(std::string::String,i64)>,key:std::string::String,value:i64){if let Some((_,stored))=values.iter_mut().find(|(stored,_)|stored==&key){*stored=value;}else{values.push((key,value));}}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let base=i.checked_add(value1).unwrap();let mut keys=ordered_unique([base.checked_add(2).unwrap(),base,base.checked_add(2).unwrap()]);let copied=keys.clone();keys.clear();let mapped=ordered_unique([base.checked_add(2).unwrap(),base]).into_iter().enumerate().map(|(index,value)|value.checked_mul(10).unwrap().checked_add(i64::try_from(index).unwrap()).unwrap()).collect::<Vec<_>>();let mut values=Vec::new();ordered_set(&mut values,value2.to_string(),base);ordered_set(&mut values,"tail".to_owned(),base.checked_add(1).unwrap());let entries=values.clone();let labels=values.iter().enumerate().map(|(index,(key,value))|key.to_owned()+":"+&value.checked_add(i64::try_from(index).unwrap()).unwrap().to_string()).collect::<Vec<_>>();let rows=[(base,(value2.to_string(),||7_i64))];let read=rows.iter().enumerate().map(|(index,(key,(label,_action)))|label.to_owned()+":"+&key.checked_add(i64::try_from(index).unwrap()).unwrap().to_string()).collect::<Vec<_>>();output.push(Array(vec![String::Owned(copied.iter().map(i64::to_string).collect::<Vec<_>>().join(",")),String::Owned(mapped.iter().map(i64::to_string).collect::<Vec<_>>().join(",")),String::Owned(entries.len().to_string()),String::Owned(labels.join("|")),String::Owned(read.join("|"))]));}output}
fn main(){}
