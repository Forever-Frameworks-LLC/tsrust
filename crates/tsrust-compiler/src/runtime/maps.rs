//! Ordered Map storage reuses collection key hashing without duplicating keys.
pub(crate) const MAP_INLINE_FIRST: &str = r#"
const TSR_MAP_INLINE_FIRST:bool=true;
type TsrMapFirst<P>=Option<P>;
"#;
pub(crate) const MAP_INLINE_RUNTIME: &str = r#"
trait TsrMapFirstSlot<P>:Default {
    const INLINE:bool;
    fn as_ref(&self)->Option<&P>;
    fn as_mut(&mut self)->Option<&mut P>;
    fn take(&mut self)->Option<P>;
    fn replace(&mut self,value:P)->Option<P>;
}
impl<P> TsrMapFirstSlot<P> for Option<P>{
    const INLINE:bool=true;
    fn as_ref(&self)->Option<&P>{Option::as_ref(self)}
    fn as_mut(&mut self)->Option<&mut P>{Option::as_mut(self)}
    fn take(&mut self)->Option<P>{Option::take(self)}
    fn replace(&mut self,value:P)->Option<P>{Option::replace(self,value)}
}
struct TsrMapNoFirst<P>(std::marker::PhantomData<P>);
impl<P> Default for TsrMapNoFirst<P>{fn default()->Self{Self(std::marker::PhantomData)}}
impl<P> Clone for TsrMapNoFirst<P>{fn clone(&self)->Self{Self::default()}}
impl<P> TsrMapFirstSlot<P> for TsrMapNoFirst<P>{
    const INLINE:bool=false;
    fn as_ref(&self)->Option<&P>{None}
    fn as_mut(&mut self)->Option<&mut P>{None}
    fn take(&mut self)->Option<P>{None}
    fn replace(&mut self,_:P)->Option<P>{unreachable!("inline Map storage disabled")}
}
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
struct TsrMap<K:TsrSetKey,V,P:TsrMapEntry<K,V>=(K,V),F:TsrMapFirstSlot<P>=TsrMapFirst<P>> {
    first:F,
    slots:Vec<Option<P>>,
    buckets:TsrSetBuckets,
    length:usize,
    marker:std::marker::PhantomData<(K,V)>,
}
impl<K:TsrSetKey,V,P:TsrMapEntry<K,V>,F:TsrMapFirstSlot<P>> TsrMap<K,V,P,F> {
    fn tsr_owned_items(mut self)->impl Iterator<Item=P>{self.first.take().into_iter().chain(self.slots.into_iter().flatten())}

    fn new()->Self{Self{first:Default::default(),slots:Vec::new(),buckets:TsrSetBuckets::default(),length:0,marker:std::marker::PhantomData}}
    fn from_values(values:impl IntoIterator<Item=(K,V)>)->Self{
        let mut result=Self::new();for(key,value)in values{result.set(key,value);}result
    }
    fn from_mapped<U,Q:TsrMapEntry<K,U>,G:TsrMapFirstSlot<Q>>(source:TsrMap<K,U,Q,G>,mut convert:impl FnMut(U)->V)->Self{
        if F::INLINE && source.length<=1 {
            let mut result=Self::new();
            if let Some(entry)=source.tsr_owned_items().next(){
                let(key,value)=entry.into_pair();result.first.replace(P::from_pair(key,convert(value)));result.length=1;
            }
            return result;
        }
        // shortcut: indexed maps retain their bucket rebuild; revisit if their remaps profile hot.
        if source.first.as_ref().is_some() || source.slots.len()>TSR_SMALL_SLOTS{
            return Self::from_values(source.tsr_owned_items().map(|entry|{let(key,value)=entry.into_pair();(key,convert(value))}));
        }
        let slots=source.slots.into_iter().filter_map(|entry|{
            let(key,value)=entry?.into_pair();Some(Some(P::from_pair(key,convert(value))))
        }).collect();
        Self{first:Default::default(),slots,buckets:source.buckets,length:source.length,marker:std::marker::PhantomData}
    }
    /// Small maps scan their slots: no hashing and no bucket table.
    fn find<Q:TsrSetQuery<K>>(&self,query:&Q)->Option<usize>{
        if let Some(entry)=self.first.as_ref(){return query.tsr_query_same(entry.key()).then_some(0);}
        if self.slots.len()<=TSR_SMALL_SLOTS{return self.slots.iter().position(|entry|entry.as_ref().is_some_and(|entry|query.tsr_query_same(entry.key())));}
        let hash=query.tsr_query_hash();
        self.buckets.get(&hash).and_then(|indices|indices.iter().copied().find(|&index|self.slots[index].as_ref().is_some_and(|entry|query.tsr_query_same(entry.key()))))
    }
    fn set(&mut self,key:K,value:V){
        let key=key.tsr_normalize();
        if let Some(index)=self.find(&key){
            let entry=if self.first.as_ref().is_some(){self.first.as_mut()}else{self.slots[index].as_mut()};
            *entry.expect("occupied Map slot").value_mut()=value;return;
        }
        if F::INLINE && self.slots.is_empty() && self.first.as_ref().is_none(){self.first.replace(P::from_pair(key,value));self.length=1;return;}
        if self.first.as_ref().is_some(){self.slots.push(self.first.take());}
        let index=self.slots.len();
        let hash=(index>=TSR_SMALL_SLOTS).then(||key.tsr_hash());
        self.slots.push(Some(P::from_pair(key,value)));self.length+=1;
        if index==TSR_SMALL_SLOTS{
            // Leaving the small form: index every live slot once.
            for (index,entry) in self.slots.iter().enumerate(){if let Some(entry)=entry{self.buckets.entry(entry.key().tsr_hash()).or_default().push(index);}}
        }else if let Some(hash)=hash{self.buckets.entry(hash).or_default().push(index);}
    }
    fn get(&self,key:impl TsrSetQuery<K>)->Option<&V>{
        let index=self.find(&key)?;self.first.as_ref().or_else(||self.slots[index].as_ref()).map(TsrMapEntry::value)
    }
    fn get_mut(&mut self,key:impl TsrSetQuery<K>)->Option<&mut V>{
        let index=self.find(&key)?;
        let entry=if self.first.as_ref().is_some(){self.first.as_mut()}else{self.slots[index].as_mut()};
        entry.map(TsrMapEntry::value_mut)
    }
    fn into_value(mut self,key:impl TsrSetQuery<K>)->Option<V>{
        let index=self.find(&key)?;self.first.take().or_else(||self.slots[index].take()).map(|entry|entry.into_pair().1)
    }
    fn has(&self,key:impl TsrSetQuery<K>)->bool{self.find(&key).is_some()}
    fn delete(&mut self,key:impl TsrSetQuery<K>)->bool{
        let Some(index)=self.find(&key)else{return false;};
        if self.first.as_ref().is_some(){self.first.take();self.length-=1;return true;}
        if self.slots.len()>TSR_SMALL_SLOTS{
            let hash=key.tsr_query_hash();
            if let Some(indices)=self.buckets.get_mut(&hash){indices.retain(|&stored|stored!=index);}
            if self.buckets.get(&hash).is_some_and(TsrSetBucket::is_empty){self.buckets.remove(&hash);}
        }
        self.slots[index].take();self.length-=1;true
    }
    fn clear(&mut self){self.first.take();self.slots.clear();self.buckets.clear();self.length=0;}
    fn len(&self)->usize{self.length}
    fn entry_iter(&self)->impl Iterator<Item=&P>{self.first.as_ref().into_iter().chain(self.slots.iter().filter_map(Option::as_ref))}
    fn iter(&self)->impl Iterator<Item=(&K,&V)>{self.entry_iter().map(|entry|(entry.key(),entry.value()))}
}
"#;

// Keep Vec-only programs on their existing generated code and layout.
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
    buckets:TsrSetBuckets,
    length:usize,
    marker:std::marker::PhantomData<(K,V)>,
}
impl<K:TsrSetKey,V,P:TsrMapEntry<K,V>> TsrMap<K,V,P> {
    fn tsr_owned_items(self)->impl Iterator<Item=P>{self.slots.into_iter().flatten()}

    fn new()->Self{Self{slots:Vec::new(),buckets:TsrSetBuckets::default(),length:0,marker:std::marker::PhantomData}}
    fn from_values(values:impl IntoIterator<Item=(K,V)>)->Self{
        let mut result=Self::new();for(key,value)in values{result.set(key,value);}result
    }
    fn from_mapped<U,Q:TsrMapEntry<K,U>>(source:TsrMap<K,U,Q>,mut convert:impl FnMut(U)->V)->Self{
        // shortcut: indexed maps retain their bucket rebuild; revisit if their remaps profile hot.
        if source.slots.len()>TSR_SMALL_SLOTS{
            return Self::from_values(source.tsr_owned_items().map(|entry|{let(key,value)=entry.into_pair();(key,convert(value))}));
        }
        let slots=source.slots.into_iter().filter_map(|entry|{
            let(key,value)=entry?.into_pair();Some(Some(P::from_pair(key,convert(value))))
        }).collect();
        Self{slots,buckets:source.buckets,length:source.length,marker:std::marker::PhantomData}
    }
    /// Small maps scan their slots: no hashing and no bucket table.
    fn find<Q:TsrSetQuery<K>>(&self,query:&Q)->Option<usize>{
        if self.slots.len()<=TSR_SMALL_SLOTS{return self.slots.iter().position(|entry|entry.as_ref().is_some_and(|entry|query.tsr_query_same(entry.key())));}
        let hash=query.tsr_query_hash();
        self.buckets.get(&hash).and_then(|indices|indices.iter().copied().find(|&index|self.slots[index].as_ref().is_some_and(|entry|query.tsr_query_same(entry.key()))))
    }
    fn set(&mut self,key:K,value:V){
        let key=key.tsr_normalize();
        if let Some(index)=self.find(&key){*self.slots[index].as_mut().expect("occupied Map slot").value_mut()=value;return;}
        let index=self.slots.len();
        let hash=(index>=TSR_SMALL_SLOTS).then(||key.tsr_hash());
        self.slots.push(Some(P::from_pair(key,value)));self.length+=1;
        if index==TSR_SMALL_SLOTS{
            // Leaving the small form: index every live slot once.
            for (index,entry) in self.slots.iter().enumerate(){if let Some(entry)=entry{self.buckets.entry(entry.key().tsr_hash()).or_default().push(index);}}
        }else if let Some(hash)=hash{self.buckets.entry(hash).or_default().push(index);}
    }
    fn get(&self,key:impl TsrSetQuery<K>)->Option<&V>{
        let index=self.find(&key)?;self.slots[index].as_ref().map(TsrMapEntry::value)
    }
    fn get_mut(&mut self,key:impl TsrSetQuery<K>)->Option<&mut V>{
        let index=self.find(&key)?;self.slots[index].as_mut().map(TsrMapEntry::value_mut)
    }
    fn into_value(mut self,key:impl TsrSetQuery<K>)->Option<V>{
        let index=self.find(&key)?;self.slots[index].take().map(|entry|entry.into_pair().1)
    }
    fn has(&self,key:impl TsrSetQuery<K>)->bool{self.find(&key).is_some()}
    fn delete(&mut self,key:impl TsrSetQuery<K>)->bool{
        let Some(index)=self.find(&key)else{return false;};
        if self.slots.len()>TSR_SMALL_SLOTS{
            let hash=key.tsr_query_hash();
            if let Some(indices)=self.buckets.get_mut(&hash){indices.retain(|&stored|stored!=index);}
            if self.buckets.get(&hash).is_some_and(TsrSetBucket::is_empty){self.buckets.remove(&hash);}
        }
        self.slots[index].take();self.length-=1;true
    }
    fn clear(&mut self){self.slots.clear();self.buckets.clear();self.length=0;}
    fn len(&self)->usize{self.length}
    fn entry_iter(&self)->impl Iterator<Item=&P>{self.slots.iter().filter_map(Option::as_ref)}
    fn iter(&self)->impl Iterator<Item=(&K,&V)>{self.entry_iter().map(|entry|(entry.key(),entry.value()))}
}
"#;
