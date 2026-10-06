import {observeSliceTree} from './slice-observation.mjs';
export const declarationListsIds=['declaration_lists'];
// Independent of declarations and loops. The converging loop starts 6 apart and
// closes by 2 per step, so it adds 6, 4 and 2 in 3 steps for every input.
// The second loop runs twice and its update list appends the first word both times.
export function expectedDeclarationListsValues(id,n,seed,input){
 if(!declarationListsIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Declaration list case');
 const result=[];for(let i=0;i<n;i++){const base=(i*13+seed)%97;result.push(['12','3',input+input,''+base,''+(2*(base%5)+6)]);}return result;
}
export function expectedDeclarationListsObservation(id,n,seed,input){return observeSliceTree(expectedDeclarationListsValues(id,n,seed,input));}
