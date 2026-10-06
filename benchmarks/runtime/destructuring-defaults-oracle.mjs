import {observeSliceTree} from './slice-observation.mjs';
export const destructuringDefaultsIds=['destructuring_defaults'];
export function expectedDestructuringDefaultsValues(id,n,seed,input){
 if(!destructuringDefaultsIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Destructuring defaults case');
 const result=[];let calls=0;
 for(let i=0;i<n;i++){const base=(i*13+seed)%97,mode=(i+seed)%3;if((i+seed)%2===0)calls++;const label=mode===0?'none':input.repeat(base%3+1);const described=mode===1?'null':mode===2?'7':''+base;const amount=mode===1?'null':mode===2?'8':''+base;result.push([input,'tail',label+':'+described,amount,''+calls]);}
 return result;
}
export function expectedDestructuringDefaultsObservation(id,n,seed,input){return observeSliceTree(expectedDestructuringDefaultsValues(id,n,seed,input));}
