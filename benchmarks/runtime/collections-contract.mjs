import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {observeSliceTree} from './slice-observation.mjs';
export const collectionIDs = ['native_deque_clone'];
export function expectedCollectionValues(id, n, seed, input) {
  if (!collectionIDs.includes(id) || !Number.isSafeInteger(n) || n < 0 || n > 2048 || !Number.isSafeInteger(seed) || seed < 0 || seed > 1000000 || input !== 'collections') throw Error('Collection case');
  return Array.from({length:n}, (_, i) => [2, (i+seed)%256, 3, (i+seed)%256+1, 0, 0]);
}
export function expectedCollectionObservation(id, n, seed, input) {
  return observeSliceTree(expectedCollectionValues(id,n,seed,input));
}
export function assertCollectionSource(source, row, kind) {
  if (!collectionIDs.includes(row.id) || row.family !== 'native-collections' || !['native','typescript'].includes(kind)) throw Error('Collection family');
  if (createHash('sha256').update(source).digest('hex') !== row[kind === 'native' ? 'sourceSha256' : 'counterpartSha256']) throw Error('Collection fingerprint');
  if (row.iterations !== 2048 || row.operationsPerIteration !== 1 || row.resultEncoding !== 'array-shape-scalar-v1' || row.operationUnit !== 'One deep deque clone, independent payload mutation and removal') throw Error('Collection denominator');
  const required = ['for(let i=0;i<n;i++)','const value=(i+seed)%256;','changed.values.push(value+1);','return output;'];
  required.push(...(kind === 'native' ? ['const original=new Deque<Item>();','original.pushBack({values:[value,255]});','const copied=clone(original);','const changed=copied.popFront()!;','const retained=original.popFront()!;','output.push([retained.values.length,retained.values[0],changed.values.length,changed.values[2],original.size,copied.size]);'] : ['const original:Item[]=[];','original.push({values:[value,255]});','const copied=original.map(item=>({values:item.values.slice()}));','const changed=copied.shift()!;','const retained=original.shift()!;','output.push([retained.values.length,retained.values[0],changed.values.length,changed.values[2],original.length,copied.length]);']));
  for (const snippet of required) if (!containsCodeSnippet(source,snippet)) throw Error('Collection scenario: '+snippet);
}
