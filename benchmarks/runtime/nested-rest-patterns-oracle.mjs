import {observeSliceTree} from './slice-observation.mjs';
export const nestedRestPatternsIds=['nested_rest_patterns'];
export function expectedNestedRestPatternsValues(id,n,seed,input){
 if(!nestedRestPatternsIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Nested rest case');
 const result=[];for(let i=0;i<n;i++){const base=(i*13+seed)%97;result.push([input,''+base,input+input+'|end','2',''+base,input,''+(base+1),input+input+':tail','3']);}return result;
}
export function expectedNestedRestPatternsObservation(id,n,seed,input){return observeSliceTree(expectedNestedRestPatternsValues(id,n,seed,input));}
