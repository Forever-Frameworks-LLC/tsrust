import {observeSliceTree} from './slice-observation.mjs';
export const deferredInitializationIds=['deferred_initialization'];
// Independent of declarations without initializers: every cell is a direct
// expression of the iteration's base value.
export function expectedDeferredInitializationValues(id,n,seed,input){
 if(!deferredInitializationIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Deferred initialization case');
 const result=[];for(let i=0;i<n;i++){const base=(i*13+seed)%97,step=base%5;
  result.push([base>=64?'high':base>=32?'mid':'low',''+(step*2+1),step===0?'zero':step<3?'few':'many',''+base+input,'true']);}
 return result;
}
export function expectedDeferredInitializationObservation(id,n,seed,input){return observeSliceTree(expectedDeferredInitializationValues(id,n,seed,input));}
