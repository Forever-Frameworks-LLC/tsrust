import {observeSliceTree} from './slice-observation.mjs';
export const forPatternInitializersIds=['for_pattern_initializers'];
// Independent of loops and destructuring: each cell is closed-form arithmetic or a text join.
// The record loop visits start, start+1 and start+2 and skips the middle value.
// The defaulted loop adds 1 and 2. The rest loop runs while its copied tail has 1 and then 2 elements.
export function expectedForPatternInitializersValues(id,n,seed,input){
 if(!forPatternInitializersIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('For pattern case');
 const result=[];for(let i=0;i<n;i++){const start=((i*13+seed)%97)%3;result.push([''+(start+start+2),'3',input+'1'+input+'2','2']);}return result;
}
export function expectedForPatternInitializersObservation(id,n,seed,input){return observeSliceTree(expectedForPatternInitializersValues(id,n,seed,input));}
