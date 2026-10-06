import {observeSliceTree} from './slice-observation.mjs';
export const callableExpressionsIds=['callable_expressions'];
// Direct arithmetic supplies the reference; no closures or computed calls.
export function expectedCallableExpressionsValues(id,n,seed,input){
 if(!callableExpressionsIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Callable expressions case');
 const result=[];for(let i=0;i<n;i++){const base=(i*13+seed)%97;
 result.push([String(base+3),String(base+5),String(base*2),String(base+4),String(base+2),'7',String(base+5),'1,2',input+base]);}return result;
}
export function expectedCallableExpressionsObservation(id,n,seed,input){return observeSliceTree(expectedCallableExpressionsValues(id,n,seed,input));}
