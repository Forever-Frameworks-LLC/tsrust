import {performance} from 'node:perf_hooks';
type Result<T>={kind:'Ok';value:T};
function unwrap<T>(result:Result<T>):T{return result.value;}
class LruCache<K,V>{
 private entries=new Map<K,V>();
 constructor(private capacity:number){if(!Number.isSafeInteger(capacity)||capacity<=0)throw RangeError('capacity');}
 set(key:K,value:V):Result<{key:K;value:V}|undefined>{
  let displaced:{key:K;value:V}|undefined;
  if(this.entries.has(key)){displaced={key,value:this.entries.get(key)!};this.entries.delete(key);}
  else if(this.entries.size===this.capacity){const oldest=this.entries.entries().next().value!;displaced={key:oldest[0],value:oldest[1]};this.entries.delete(oldest[0]);}
  this.entries.set(key,value);return {kind:'Ok',value:displaced};
 }
 get(key:K):V|undefined{if(!this.entries.has(key))return undefined;const value=this.entries.get(key)!;this.entries.delete(key);this.entries.set(key,value);return value;}
 size():number{return this.entries.size;}
 clear():void{this.entries.clear();}
}
function batch():number{let checksum=0;const cache=new LruCache<number,number>(16);for(let i=0;i<256;i+=1){unwrap(cache.set(i,i*3));const found=cache.get(i);if(found!==undefined){checksum+=found;}}checksum+=cache.size();cache.clear();return checksum;}
for(let warmup=0;warmup<8;warmup+=1){batch();}for(let sample=0;sample<31;sample+=1){const start=performance.now();const checksum=batch();const elapsed=performance.now()-start;console.log(checksum);console.log(elapsed);}
