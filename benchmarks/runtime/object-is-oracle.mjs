import {observeSliceTree} from './slice-observation.mjs';
export const objectIsIds = ['object_is_values'];
// Independent of Object.is: each answer follows from the scenario arithmetic.
export function expectedObjectIsValues(id,n,seed,input) {
 if (!objectIsIds.includes(id)) throw Error('Unknown Object.is workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string') throw Error('Object.is case bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const mode=(i+seed)%3,base=(i*13+seed)%97;
  // left and right differ by mode/4. Equal operands give +0 and +0, then NaN and NaN.
  // Unequal operands give opposite signs, then 1 and 1.
  // base/4 equals base only for zero.
  values.push([mode===0,mode===0,true,mode===1||input.length===0,mode===0,base===0]);
 }
 return values;
}
export function expectedObjectIsObservation(id,n,seed,input) { return observeSliceTree(expectedObjectIsValues(id,n,seed,input)); }
