// Reuse Map/Set key semantics, including SameValueZero numeric comparisons.
struct TsrCacheKeys;
impl<K:TsrSetKey> tsr_cache::KeyPolicy<K> for TsrCacheKeys {
 fn hash(&self,key:&K)->u64{key.tsr_hash()}
 fn same(&self,left:&K,right:&K)->bool{left.tsr_same(right)}
 fn normalize(&self,key:K)->K{key.tsr_normalize()}
}
