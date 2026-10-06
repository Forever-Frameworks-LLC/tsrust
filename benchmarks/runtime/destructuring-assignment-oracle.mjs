import {observeSliceTree} from './slice-observation.mjs';
export const destructuringAssignmentIds=['destructuring_assignment'];
// Independent of destructuring and loops: the Fibonacci cell comes from a table.
// The loop advances the pair base%4+2 times, so f0 is the Fibonacci number of that index.
const fibonacci=[0,1,1,2,3,5,8];
export function expectedDestructuringAssignmentValues(id,n,seed,input){
 if(!destructuringAssignmentIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Destructuring assignment case');
 const result=[];for(let i=0;i<n;i++){const base=(i*13+seed)%97;result.push([''+base,input,''+fibonacci[base%4+2],''+(base+2),input+'?',''+(base+base+2),input+'?!']);}return result;
}
export function expectedDestructuringAssignmentObservation(id,n,seed,input){return observeSliceTree(expectedDestructuringAssignmentValues(id,n,seed,input));}
