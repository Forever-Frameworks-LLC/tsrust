import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {observeSliceTree} from './slice-observation.mjs';
export const collectionIDs = ['native_deque_clone','native_heap','native_ordered_set','native_ordered_map'];
export function expectedCollectionValues(id, n, seed, input) {
  if (!collectionIDs.includes(id) || !Number.isSafeInteger(n) || n < 0 || n > 2048 || !Number.isSafeInteger(seed) || seed < 0 || seed > 1000000 || input !== 'collections') throw Error('Collection case');
  if(id==='native_ordered_map') return Array.from({length:n},(_,i)=>{
    const first=(i+seed)%256;const wrap=Math.max(0,first+9-256);let weighted=0;
    for(let position=0;position<9;position++){const key=position<wrap?position:first+position-wrap;weighted+=key*(position+1);}
    const sum=3*(9*first+36-256*wrap)+9+999-(first*3+1);
    return [9,wrap?0:first,wrap?255:first+8,weighted,sum,((first+4)%256)*3+1,1,8,9];
  });
  if(id==='native_ordered_set') return Array.from({length:n},(_,i)=>{
    const first=(i+seed)%256;const wrap=Math.max(0,first+9-256);let weighted=0;
    for(let position=0;position<9;position++){const value=position<wrap?position:first+position-wrap;weighted+=value*(position+1);}
    return [9,wrap?0:first,wrap?255:first+8,weighted,1,1,8,9];
  });
  if(id==='native_heap') return Array.from({length:n},(_,i)=>{const first=(i+seed)%256;const wrap=Math.max(0,first+9-256);return [9,wrap>0?255:first+8,9*first+36-256*wrap,0,9*first+36-256*wrap,9*first+36-256*wrap,9];});
  return Array.from({length:n}, (_, i) => [2, (i+seed)%256, 3, (i+seed)%256+1, 0, 0, (i+seed)%256]);
}
export function expectedCollectionObservation(id, n, seed, input) {
  return observeSliceTree(expectedCollectionValues(id,n,seed,input));
}
export function assertCollectionSource(source, row, kind) {
  if (!collectionIDs.includes(row.id) || row.family !== 'native-collections' || !['native','typescript'].includes(kind)) throw Error('Collection family');
  if (createHash('sha256').update(source).digest('hex') !== row[kind === 'native' ? 'sourceSha256' : 'counterpartSha256']) throw Error('Collection fingerprint');
  if(row.id==='native_ordered_map') {
    if(row.iterations!==2048 || row.operationsPerIteration!==1 || row.resultEncoding!=='array-shape-scalar-v1' || row.operationUnit!=='Nine key/value inserts, replacement, ordered traversal, clone, lookup and deletion')throw Error('Collection denominator');
    const required=['for(let i=0;i<n;i++)','for(let j=0;j<9;j++){const key=(i+seed+j)%256;values.set(key,key*3+1);}','values.set((i+seed)%256,999);','const key=entry[0];if(count===0){first=key;}count++;weighted+=key*count;sum+=entry[1];last=key;','const key=(i+seed+4)%256;','const selected=values.get(key)!;','const removed=values.delete(key)?1:0;','output.push([count,first,last,weighted,sum,selected,removed,values.size,copied.size]);','return output;'];
    required.push(...(kind==='native'?['const values=new OrderedMap<int,int>();','for(const entry of values)','const copied=clone(values);']:['const values=new Map<number,number>();','for(const entry of Array.from(values).sort((a,b)=>a[0]-b[0]))','const copied=new Map(values);']));
    for(const snippet of required)if(!containsCodeSnippet(source,snippet))throw Error('Collection scenario: '+snippet);
    return;
  }
  if(row.id==='native_ordered_set') {
    if(row.iterations!==2048 || row.operationsPerIteration!==1 || row.resultEncoding!=='array-shape-scalar-v1' || row.operationUnit!=='Nine unique inserts, duplicate insertion, ordered traversal, clone, lookup and deletion')throw Error('Collection denominator');
    const required=['for(let i=0;i<n;i++)','for(let j=0;j<9;j++){values.add((i+seed+j)%256);}','values.add((i+seed)%256);','if(count===0){first=value;}count++;weighted+=value*count;last=value;','const key=(i+seed+4)%256;','const found=values.has(key)?1:0;','const removed=values.delete(key)?1:0;','output.push([count,first,last,weighted,found,removed,values.size,copied.size]);','return output;'];
    required.push(...(kind==='native'?['const values=new OrderedSet<int>();','for(const value of values)','const copied=clone(values);']:['const values=new Set<number>();','for(const value of Array.from(values).sort((a,b)=>a-b))','const copied=new Set(values);']));
    for(const snippet of required)if(!containsCodeSnippet(source,snippet))throw Error('Collection scenario: '+snippet);
    return;
  }
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
