import {observeSliceTree} from './slice-observation.mjs';
export const inOperatorIds=['in_operator'];
// Direct formulas and fixed key text; never call the tested key operations.
export function expectedInOperatorValues(id,n,seed,input){
 if(!inOperatorIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('In operator case');
 const result=[];for(let i=0;i<n;i++){const base=(i*13+seed)%97;
 result.push([String(base%2===0),String(base%5<3),'false','true','name,amount,','012',input+base,String(base)]);}
 return result;
}
export function expectedInOperatorObservation(id,n,seed,input){return observeSliceTree(expectedInOperatorValues(id,n,seed,input));}
