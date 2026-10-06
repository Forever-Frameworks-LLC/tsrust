import {observeSliceTree} from './slice-observation.mjs';
export const labeledControlIds=['labeled_control'];
// Fixed trace alternatives and direct arithmetic; no labeled control here.
export function expectedLabeledControlValues(id,n,seed,input){
 if(!labeledControlIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Labeled control case');
 const result=[],traces=['101112202122','000102202122','000102101112'];
 for(let i=0;i<n;i++){const base=(i*13+seed)%97;
 result.push([traces[base%3],'3',String(base%4+1),String(base%3+1),base%2===0?'1':'2',String(base),input+base]);}
 return result;
}
export function expectedLabeledControlObservation(id,n,seed,input){return observeSliceTree(expectedLabeledControlValues(id,n,seed,input));}
