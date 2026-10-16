type TsrSharedCache<K,V> = std::sync::Arc<std::sync::Mutex<TsrLocalCache<K,V>>>;
type TsrLocalCache<K,V> = tsr_cache::timed::Timed<K,V,TsrCacheKeys,fn()->std::time::Instant>;
fn tsr_cache_new<K:TsrSetKey,V>(capacity:i64)->Result<TsrLocalCache<K,V>,()>{
 let capacity=usize::try_from(capacity).ok().and_then(std::num::NonZeroUsize::new).ok_or(())?;
 Ok(tsr_cache::timed::Timed::new(tsr_cache::Lru::with_policy(capacity,TsrCacheKeys),std::time::Instant::now))
}
fn tsr_cache_set<K:TsrSetKey,V>(cache:&mut TsrLocalCache<K,V>,key:K,value:V,ttl:Option<i64>)->Result<Option<(K,V)>,(&'static str,K,V)>{
 let ttl=match ttl{None=>None,Some(ms) if ms>0=>Some(std::time::Duration::from_millis(ms as u64)),Some(_)=>return Err(("ttl",key,value))};
 cache.insert(key,value,ttl).map_err(|error|(match error.kind{tsr_cache::timed::DeadlineError::ZeroDuration=>"ttl",tsr_cache::timed::DeadlineError::Overflow=>"deadline"},error.key,error.value))
}
