import {observeSliceTree} from './slice-observation.mjs';
import {validateArrayRemovalCase} from './array-removal-oracle.mjs';
export const ofFamilies=['int','number','boolean','string','nested','record','null','undefined'];
export const ofIds=ofFamilies.map(f=>'array_of_'+f);
export function ofFamily(id){if(!ofIds.includes(id))throw Error('Array.of ID');return id.slice(9);}
export function validateOfCase(id,n,seed,input){return validateArrayRemovalCase('array_pop_'+ofFamily(id),n,seed,input);}
function digits(value){if(value===0)return'0';let out='';while(value>0){out='0123456789'[value%10]+out;value=Math.floor(value/10);}return out;}
// Independent literal/ordinal construction, never invokes Array.of.
export function expectedOfValues(id,n,seed,input){input=validateOfCase(id,n,seed,input);const family=ofFamily(id),out=[];
 for(let i=0;i<n;i++){const value=seed+i,arity=[0,1,5][value%3],row=[];for(let ordinal=0;ordinal<arity;ordinal++){
 let leaf;if(family==='int')leaf=value+ordinal;else if(family==='number')leaf=[-0,0,NaN,Infinity,-Infinity,value/4,-2.5][(value+ordinal*3)%7];else if(family==='boolean')leaf=(value+ordinal)%3!==0;else if(family==='string')leaf=input+':'+digits(ordinal);else if(family==='nested')leaf=[value+ordinal,ordinal];else if(family==='record')leaf=digits(value+ordinal)+':'+input+':'+digits(ordinal);else leaf=family;row.push(leaf);
 }out.push(row);}return out;
}
export function expectedOfObservation(id,n,seed,input){return observeSliceTree(expectedOfValues(id,n,seed,input),{numbers:ofFamily(id)==='number'});}
