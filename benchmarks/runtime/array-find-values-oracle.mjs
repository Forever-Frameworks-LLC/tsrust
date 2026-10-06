import {observeSliceTree} from './slice-observation.mjs';
import {observeNumber} from './number-observation.mjs';
import {validateArrayRemovalCase} from './array-removal-oracle.mjs';
export const findFamilies=['int','number','boolean','string','nested','record','null','undefined'];
export const findValueIds=['find','find_last'].flatMap(method=>findFamilies.map(f=>'array_'+method+'_'+f));
export function findInfo(id){if(!findValueIds.includes(id))throw Error('find-value ID');const reverse=id.startsWith('array_find_last_');return {reverse,family:id.slice(reverse?16:11)};}
export function validateFindValueCase(id,n,seed,input){const {family}=findInfo(id);return validateArrayRemovalCase('array_pop_'+family,n,seed,input);}
function digits(v){if(v===0)return '0';let out='';while(v>0){out='0123456789'[v%10]+out;v=Math.floor(v/10);}return out;}
function pool(f,seed,input){if(f==='int')return [digits(seed),digits(seed+1),digits(seed+2)];if(f==='number')return [-0,seed/4,NaN];if(f==='boolean')return [seed%2===0?'true':'false','false','true'];if(f==='string')return [input,input+'!','é😀e\u0301\u0000'];if(f==='nested')return ['2:'+digits(seed)+',1','0:','2:'+digits(seed+2)+',3'];if(f==='record')return [digits(seed)+':'+input,digits(seed+1)+':'+input+'!',digits(seed+2)+':é😀e\u0301\u0000'];return [f,f,f];}
// Independent ordered integer-index walk; no tested Array.find/findLast calls.
export function expectedFindValues(id,n,seed,input){input=validateFindValueCase(id,n,seed,input);const {family:f,reverse}=findInfo(id),p=pool(f,seed,input),out=[];
 for(let i=0;i<n;i++){const count=(i+seed)%4,mode=(i*7+seed)%5;let selected=-1,visits=0,trace=0;
  for(let step=0;step<count;step++){const index=reverse?count-1-step:step;visits++;trace=trace*10+index+1;const match=mode===1&&index===0||mode===2&&index%2===0||mode===3&&index>=1||mode===4;if(match){selected=index;break;}}
  if(f==='number')out.push(selected<0?0:1,selected<0?NaN:p[selected],count,visits,trace,visits*count,...[0,1,2].map(j=>j<count?p[j]:NaN));
  else{const type=selected<0||f==='undefined'?'undefined':f==='int'?'number':['nested','record','null'].includes(f)?'object':f;out.push(type,selected<0?'undefined':p[selected],digits(count),digits(visits),digits(trace),digits(visits*count),...[0,1,2].map(j=>j<count?p[j]:'<absent>'));}
 }return out;
}
export function expectedFindObservation(id,n,seed,input){const values=expectedFindValues(id,n,seed,input);return findInfo(id).family==='number'?observeNumber(values):observeSliceTree(values);}
