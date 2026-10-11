trait TsrOrderedKey {
    fn tsr_compare(&self, other: &Self) -> std::cmp::Ordering;
    fn tsr_normalize(self) -> Self where Self: Sized { self }
}
impl TsrOrderedKey for i64 { fn tsr_compare(&self, other: &Self) -> std::cmp::Ordering { self.cmp(other) } }
impl TsrOrderedKey for bool { fn tsr_compare(&self, other: &Self) -> std::cmp::Ordering { self.cmp(other) } }
impl TsrOrderedKey for String { fn tsr_compare(&self, other: &Self) -> std::cmp::Ordering { { let left: &str = self.as_ref(); let right: &str = other.as_ref(); left.cmp(right) } } }
impl TsrOrderedKey for f64 {
    fn tsr_compare(&self, other: &Self) -> std::cmp::Ordering { TsrOrderedNumber::new(*self).cmp(&TsrOrderedNumber::new(*other)) }
    fn tsr_normalize(self) -> Self { TsrOrderedNumber::new(self).value() }
}
#[derive(Clone)]
struct TsrOrdered<T>(T);
impl<T:TsrOrderedKey> TsrOrdered<T> { fn new(value:T)->Self{Self(value.tsr_normalize())} }
impl<T:TsrOrderedKey> PartialEq for TsrOrdered<T>{fn eq(&self,other:&Self)->bool{self.cmp(other)==std::cmp::Ordering::Equal}}
impl<T:TsrOrderedKey> Eq for TsrOrdered<T>{}
impl<T:TsrOrderedKey> PartialOrd for TsrOrdered<T>{fn partial_cmp(&self,other:&Self)->Option<std::cmp::Ordering>{Some(self.cmp(other))}}
impl<T:TsrOrderedKey> Ord for TsrOrdered<T>{fn cmp(&self,other:&Self)->std::cmp::Ordering{self.0.tsr_compare(&other.0)}}

impl<T:TsrOrderedKey> TsrHeap<TsrOrdered<T>> {
    fn into_keys(self) -> impl Iterator<Item=T> { self.values.into_iter().map(|value|value.0) }
    fn key_iter(&self) -> impl Iterator<Item=&T> { self.values.iter().map(|value| &value.0) }
}

impl<T:TsrOrderedKey> TsrOrderedSet<TsrOrdered<T>> {
    fn key_iter(&self)->impl Iterator<Item=&T>{self.values.iter().map(|value|&value.0)}
    fn into_keys(self)->impl Iterator<Item=T>{self.values.into_iter().map(|value|value.0)}
}
impl<T:TsrOrderedKey+Copy> TsrOrderedSet<TsrOrdered<T>> {
    fn contains_copy_key(&self,key:T)->bool{self.values.contains(&TsrOrdered::new(key))}
    fn remove_copy_key(&mut self,key:T)->bool{self.values.remove(&TsrOrdered::new(key))}
}
impl std::borrow::Borrow<str> for TsrOrdered<String> {fn borrow(&self)->&str{self.0.as_ref()}}
impl TsrOrderedSet<TsrOrdered<String>> {
    fn contains_string_key(&self,key:&String)->bool{self.values.contains::<str>(key.as_ref())}
    fn remove_string_key(&mut self,key:&String)->bool{self.values.remove::<str>(key.as_ref())}
}

impl<K:TsrOrderedKey,V> TsrOrderedMap<TsrOrdered<K>,V> {
    fn from_values(values:impl IntoIterator<Item=(K,V)>)->Self {
        let mut result=Self::new();
        for (key,value) in values {result.insert(TsrOrdered::new(key),value);}
        result
    }
    fn source_iter(&self)->impl Iterator<Item=(&K,&V)>{self.values.iter().map(|(key,value)|(&key.0,value))}
    fn into_source_entries(self)->impl Iterator<Item=(K,V)>{self.values.into_iter().map(|(key,value)|(key.0,value))}
    fn tsr_owned_items(self)->impl Iterator<Item=(K,V)>{self.into_source_entries()}
}
impl<K:TsrOrderedKey+Copy,V> TsrOrderedMap<TsrOrdered<K>,V> {
    fn get_copy_key(&self,key:K)->Option<&V>{self.values.get(&TsrOrdered::new(key))}
    fn get_mut_copy_key(&mut self,key:K)->Option<&mut V>{self.values.get_mut(&TsrOrdered::new(key))}
    fn contains_copy_key(&self,key:K)->bool{self.values.contains_key(&TsrOrdered::new(key))}
    fn remove_copy_key(&mut self,key:K)->Option<V>{self.values.remove(&TsrOrdered::new(key))}
    fn into_value_copy_key(mut self,key:K)->Option<V>{self.remove_copy_key(key)}
}
impl<V> TsrOrderedMap<TsrOrdered<String>,V> {
    fn get_string_key(&self,key:&String)->Option<&V>{self.values.get::<str>(key.as_ref())}
    fn get_mut_string_key(&mut self,key:&String)->Option<&mut V>{self.values.get_mut::<str>(key.as_ref())}
    fn contains_string_key(&self,key:&String)->bool{self.values.contains_key::<str>(key.as_ref())}
    fn remove_string_key(&mut self,key:&String)->Option<V>{self.values.remove::<str>(key.as_ref())}
    fn into_value_string_key(mut self,key:&String)->Option<V>{self.remove_string_key(key)}
}
