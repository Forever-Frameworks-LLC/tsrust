import {observeSliceTree} from './slice-observation.mjs';
export const unionArrayIds = ['union_array_values'];
// Independent of the array methods: each cell follows from the four known
// elements (an int, a repeated text, null, and a text or int tail).
export function expectedUnionArrayValues(id,n,seed,input) {
 if (!unionArrayIds.includes(id)) throw Error('Unknown union array workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string') throw Error('Union array case bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const base=(i*13+seed)%97,mode=(i+seed)%3,textTail=mode===0;
  let repeated=''; for(let copy=0;copy<mode;copy++) repeated+=input;
  const last=textTail?'tail':''+(base+1);
  values.push([''+(textTail?base:base+base+1),repeated+(textTail?'tail':''),base+'|'+repeated+'||'+last,'true',textTail?'-1':'3',last,textTail?'2':'1']);
 }
 return values;
}
export function expectedUnionArrayObservation(id,n,seed,input) { return observeSliceTree(expectedUnionArrayValues(id,n,seed,input)); }
