import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export const ownedIteratorIDs=['iterator_map_filter_reduce','iterator_take_array','iterator_flat_array','iterator_flat_iterator'];
export function ownedIteratorChecksum(id,n,seed){
 if(!ownedIteratorIDs.includes(id))throw Error(`Unknown iterator workload ${id}`);
 let result=0;
 for(let i=0;i<n;i++){
  let observed=7,count=0;
  for(let j=0;j<64;j++){
   const value=(j*17+seed+i)%997;
   if(id==='iterator_flat_array'||id==='iterator_flat_iterator'){
    observed=(observed*131+value)%1000000007;observed=(observed*131+value+1)%1000000007;
   }else if(value%2===0){
    if(id==='iterator_take_array'&&count===17)break;
    observed=(observed*131+value*3+1)%1000000007;count++;
   }
  }result=(result*131+observed)%1000000007;
 }return result;
}
export function assertOwnedIteratorSource(source,w,engine){
 assert(ownedIteratorIDs.includes(w.id));assert(['native','typescript'].includes(engine));assert.equal(w.family,'owned-iterators');assert.equal(w.iterations,2000);assert.equal(w.operationsPerIteration,64);assert.equal(w.result,'int');assert.deepEqual(w.parameters,['int','int']);assert.equal(createHash('sha256').update(source).digest('hex'),w[engine==='native'?'sourceSha256':'counterpartSha256']);
 for(const witness of ['j<64','(j*17+seed+i)%997','(result*131+observed)%1000000007'])if(!containsCodeSnippet(source,witness))throw Error(`Iterator source guard missing ${witness}`);
 if(engine==='native'){
  for(const witness of ['Iterator.from(values)',w.id==='iterator_take_array'?'.take(17).toArray()':w.id.startsWith('iterator_flat_')?'.flatMap(' : '.filter(value=>value%2===0).map(value=>value*3+1)'])if(!containsCodeSnippet(source,witness))throw Error(`Iterator native guard missing ${witness}`);
 }else if(!containsCodeSnippet(source,'function* source(values:number[])'))throw Error('Matched generator source missing');
}

export const ownedIteratorReferenceOperations={
 iterator_map_filter_reduce:['.filter(|value|value%2==0).map(|value|value*3+1).fold(7,|sum,value|(sum*131+value)%1000000007)'],
 iterator_take_array:['.filter(|value|value%2==0).map(|value|value*3+1).take(17).collect()','fn observe(values:Vec<i64>)->i64{values.into_iter().fold(7,|sum,value|(sum*131+value)%1000000007)}'],
 iterator_flat_array:['.flat_map(|value|vec![value,value+1]).fold(7,|sum,value|(sum*131+value)%1000000007)'],
 iterator_flat_iterator:['.flat_map(|value|vec![value,value+1].into_iter()).fold(7,|sum,value|(sum*131+value)%1000000007)'],
};
export function assertOwnedIteratorReference(source,row){assert(ownedIteratorIDs.includes(row.id));assert.equal(row.family,'owned-iterators');assert.equal(createHash('sha256').update(source).digest('hex'),row.rustReferenceSha256);for(const witness of ['fn function0(value0:i64,value1:i64)->i64','let n=value0;let seed=value1','for i in 0..n','for j in 0..64','values.push((j*17+seed+i)%997)','result=(result*131+observed)%1000000007',...ownedIteratorReferenceOperations[row.id]])assert(containsCodeSnippet(source,witness),row.id+': '+witness);assert.match(source,/result\s*}\s*fn observe/);}
