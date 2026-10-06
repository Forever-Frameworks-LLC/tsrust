import {observeSliceTree} from './slice-observation.mjs';
import {validateArrayRemovalCase} from './array-removal-oracle.mjs';
export const arrayAtFamilies=['int','number','boolean','string','nested','record','null','undefined'];
export const arrayAtIds=arrayAtFamilies.map(family=>'array_at_'+family);
export function validateArrayAtCase(id,n,seed,input){if(!arrayAtIds.includes(id))throw Error('Array.at ID');return validateArrayRemovalCase(id.replace('array_at_','array_pop_'),n,seed,input);}
function digits(value){if(value===0)return '0';let result='';while(value>0){result='0123456789'[value%10]+result;value=Math.floor(value/10);}return result;}
function quarter(seed){return digits(Math.floor(seed/4))+['','.25','.5','.75'][seed%4];}
function contents(family,seed,input){
 if(family==='int')return [digits(seed),digits(seed+1),digits(seed+2)];
 if(family==='number')return [quarter(seed),'0','-2.5'];
 if(family==='boolean')return [seed%2===0?'true':'false','false','true'];
 if(family==='string')return [input,input+'!','é😀e\u0301\u0000'];
 if(family==='nested')return ['2:'+digits(seed)+',1','0:','2:'+digits(seed+2)+',3'];
 if(family==='record')return [digits(seed)+':'+input,digits(seed+1)+':'+input+'!',digits(seed+2)+':é😀e\u0301\u0000'];
 return [family,family,family];
}
// Direct relative-index arithmetic on independently specified complete text leaves.
export function expectedArrayAtValues(id,n,seed,input){
 input=validateArrayAtCase(id,n,seed,input);const family=id.slice(9),pool=contents(family,seed,input),kind=['int','number'].includes(family)?'number':['nested','record','null'].includes(family)?'object':family,out=[];
 for(let i=0;i<n;i++){const length=(i+seed)%4,index=(i*5+seed)%9-4,at=index<0?length+index:index,valid=at>=0&&at<length;
 out.push(valid?kind:'undefined',valid?pool[at]:'undefined',digits(length),...Array.from({length:3},(_,slot)=>slot<length?pool[slot]:'<absent>'));}return out;
}
export function expectedArrayAtObservation(id,n,seed,input){return observeSliceTree(expectedArrayAtValues(id,n,seed,input));}
