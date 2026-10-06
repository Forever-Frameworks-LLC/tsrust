import {observeSliceTree} from './slice-observation.mjs';
export const switchStatementsIds=['switch_statements'];
// Independent of switch: table lookups and conditions. Six consecutive loop
// values hit each remainder twice: two skipped, two worth 110 and two worth 101.
const tiers=['zero','low','low','other','other','five','other'];
export function expectedSwitchStatementsValues(id,n,seed,input){
 if(!switchStatementsIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Switch case');
 const result=[];for(let i=0;i<n;i++){const base=(i*13+seed)%97;
  const text=base%2!==0?'number':input==='abc'?'short':input==='dynamic é🙂 text'?'long':'number';
  result.push([tiers[base%7],'422',text,base%3===0?'THREE':base===13?'thirteen':'none']);}
 return result;
}
export function expectedSwitchStatementsObservation(id,n,seed,input){return observeSliceTree(expectedSwitchStatementsValues(id,n,seed,input));}
