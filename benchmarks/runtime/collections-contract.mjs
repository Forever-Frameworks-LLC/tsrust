import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {observeSliceTree} from './slice-observation.mjs';
export const collectionIDs = ['native_deque_clone','native_heap'];
export function expectedCollectionValues(id, n, seed, input) {
  if (!collectionIDs.includes(id) || !Number.isSafeInteger(n) || n < 0 || n > 2048 || !Number.isSafeInteger(seed) || seed < 0 || seed > 1000000 || input !== 'collections') throw Error('Collection case');
  if(id==='native_heap') return Array.from({length:n},(_,i)=>{const first=(i+seed)%256;const wrap=Math.max(0,first+9-256);return [9,wrap>0?255:first+8,9*first+36-256*wrap,0,9*first+36-256*wrap,9*first+36-256*wrap,9];});
  return Array.from({length:n}, (_, i) => [2, (i+seed)%256, 3, (i+seed)%256+1, 0, 0, (i+seed)%256]);
}
export function expectedCollectionObservation(id, n, seed, input) {
  return observeSliceTree(expectedCollectionValues(id,n,seed,input));
}
export function assertCollectionSource(source, row, kind) {
  if (!collectionIDs.includes(row.id) || row.family !== 'native-collections' || !['native','typescript'].includes(kind)) throw Error('Collection family');
  if (createHash('sha256').update(source).digest('hex') !== row[kind === 'native' ? 'sourceSha256' : 'counterpartSha256']) throw Error('Collection fingerprint');
  if (row.id==='native_heap') {
    if(row.iterations!==2048 || row.operationsPerIteration!==1 || row.resultEncoding!=='array-shape-scalar-v1' || row.operationUnit!=='Nine heap inserts, borrowed traversal, peek, independent clone, call/array spread and descending removal')throw Error('Collection denominator');
    const required=['for(let i=0;i<n;i++)','for(let j=0;j<9;j++){values.push((i+seed+j)%256);}','const spread=sumItems(...values);','const flattened=[...values];','let traversed=0;','for(const value of values){traversed+=value;}','let sum=0;','return output;'];
    required.push(...(kind==='native'?['const values=new Heap<int>();','const top=values.peek()!;','const copied=clone(values);','while(copied.size>0){sum+=copied.pop()!;}','output.push([values.size,top,sum,copied.size,traversed,spread,flattened.length]);']:['const values:number[]=[];','values.sort((a,b)=>a-b);','const top=values[values.length-1];','const copied=values.slice();','while(copied.length>0){sum+=copied.pop()!;}','output.push([values.length,top,sum,copied.length,traversed,spread,flattened.length]);']));
    for(const snippet of required)if(!containsCodeSnippet(source,snippet))throw Error('Collection scenario: '+snippet);
    return;
  }
  if (row.iterations !== 2048 || row.operationsPerIteration !== 1 || row.resultEncoding !== 'array-shape-scalar-v1' || row.operationUnit !== 'One borrowed nested read, deep deque clone, independent payload mutation and removal') throw Error('Collection denominator');
  const required = ['for(let i=0;i<n;i++)','const value=(i+seed)%256;','changed.values.push(value+1);','return output;'];
  required.push(...(kind === 'native' ? ['const observed=original.front()!.values[0];','const original=new Deque<Item>();','original.pushBack({values:[value,255]});','const copied=clone(original);','const changed=copied.popFront()!;','const retained=original.popFront()!;','output.push([retained.values.length,retained.values[0],changed.values.length,changed.values[2],original.size,copied.size,observed]);'] : ['const observed=original[0].values[0];','const original:Item[]=[];','original.push({values:[value,255]});','const copied=original.map(item=>({values:item.values.slice()}));','const changed=copied.shift()!;','const retained=original.shift()!;','output.push([retained.values.length,retained.values[0],changed.values.length,changed.values[2],original.length,copied.length,observed]);']));
  for (const snippet of required) if (!containsCodeSnippet(source,snippet)) throw Error('Collection scenario: '+snippet);
}
