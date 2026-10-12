//! Ordered Map storage reuses collection key hashing without duplicating keys.
pub(crate) const MAP_RUNTIME: &str = r#"
trait TsrMapEntry<K,V> {
    fn from_pair(key:K,value:V)->Self;
    fn key(&self)->&K;
    fn value(&self)->&V;
    fn value_mut(&mut self)->&mut V;
    fn into_pair(self)->(K,V);
}
impl<K,V> TsrMapEntry<K,V> for (K,V) {
    fn from_pair(key:K,value:V)->Self{(key,value)}
    fn key(&self)->&K{&self.0}
    fn value(&self)->&V{&self.1}
    fn value_mut(&mut self)->&mut V{&mut self.1}
    fn into_pair(self)->(K,V){self}
}
#[derive(Clone)]
struct TsrMap<K:TsrSetKey,V,P:TsrMapEntry<K,V>=(K,V)> {
    slots:Vec<Option<P>>,
    buckets:std::collections::HashMap<u64,TsrSetBucket>,
    length:usize,
    marker:std::marker::PhantomData<(K,V)>,
}
impl<K:TsrSetKey,V,P:TsrMapEntry<K,V>> TsrMap<K,V,P> {
    fn tsr_owned_items(self)->impl Iterator<Item=P>{self.slots.into_iter().flatten()}

    fn new()->Self{Self{slots:Vec::new(),buckets:std::collections::HashMap::new(),length:0,marker:std::marker::PhantomData}}
    fn from_values(values:impl IntoIterator<Item=(K,V)>)->Self{
        let mut result=Self::new();for(key,value)in values{result.set(key,value);}result
    }
    fn query_location<Q:TsrSetQuery<K>>(&self,query:&Q,hash:u64)->Option<usize>{
        self.buckets.get(&hash).and_then(|indices|indices.iter().copied().find(|&index|self.slots[index].as_ref().is_some_and(|entry|query.tsr_query_same(entry.key()))))
    }
    fn set(&mut self,key:K,value:V){
        let key=key.tsr_normalize();let hash=key.tsr_hash();
        if let Some(index)=self.query_location(&key,hash){*self.slots[index].as_mut().expect("occupied Map slot").value_mut()=value;return;}
        let index=self.slots.len();self.slots.push(Some(P::from_pair(key,value)));
        self.buckets.entry(hash).or_default().push(index);self.length+=1;
    }
    fn get(&self,key:impl TsrSetQuery<K>)->Option<&V>{
        let index=self.query_location(&key,key.tsr_query_hash())?;self.slots[index].as_ref().map(TsrMapEntry::value)
    }
    fn get_mut(&mut self,key:impl TsrSetQuery<K>)->Option<&mut V>{
        let index=self.query_location(&key,key.tsr_query_hash())?;self.slots[index].as_mut().map(TsrMapEntry::value_mut)
    }
    fn into_value(mut self,key:impl TsrSetQuery<K>)->Option<V>{
        let index=self.query_location(&key,key.tsr_query_hash())?;self.slots[index].take().map(|entry|entry.into_pair().1)
    }
    fn has(&self,key:impl TsrSetQuery<K>)->bool{self.query_location(&key,key.tsr_query_hash()).is_some()}
    fn delete(&mut self,key:impl TsrSetQuery<K>)->bool{
        let hash=key.tsr_query_hash();let Some(index)=self.query_location(&key,hash)else{return false;};
        if let Some(indices)=self.buckets.get_mut(&hash){indices.retain(|&stored|stored!=index);}
        if self.buckets.get(&hash).is_some_and(TsrSetBucket::is_empty){self.buckets.remove(&hash);}
        self.slots[index].take();self.length-=1;true
    }
    fn clear(&mut self){self.slots.clear();self.buckets.clear();self.length=0;}
    fn len(&self)->usize{self.length}
    fn entry_iter(&self)->impl Iterator<Item=&P>{self.slots.iter().filter_map(Option::as_ref)}
    fn iter(&self)->impl Iterator<Item=(&K,&V)>{self.entry_iter().map(|entry|(entry.key(),entry.value()))}
}
"#;
