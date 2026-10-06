import {observeSliceTree} from './slice-observation.mjs';import {observeNumber} from './number-observation.mjs';import {validateArrayRemovalCase} from './array-removal-oracle.mjs';
export const unshiftFamilies=['int','number','boolean','string','nested','record','null','undefined'],unshiftIds=unshiftFamilies.map(f=>'array_unshift_'+f);
export function validateUnshiftCase(id,n,seed,input){if(!unshiftIds.includes(id))throw Error('unshift ID');return validateArrayRemovalCase(id.replace('array_unshift_','array_pop_'),n,seed,input);}
function digits(v){if(v===0)return'0';let result='';while(v>0){result='0123456789'[v%10]+result;v=Math.floor(v/10);}return result;}
export function expectedUnshiftValues(id,n,seed,input){input=validateUnshiftCase(id,n,seed,input);const f=id.slice(14),out=[];
 function old(j){if(f==='int')return digits(seed+j);if(f==='number')return [-0,seed/4,NaN,Infinity][j%4];if(f==='boolean')return (seed+j)%2===0?'true':'false';if(f==='string')return input+':'+digits(j);if(f==='nested')return '2:'+digits(seed+j)+','+digits(j);if(f==='record')return digits(seed+j)+':'+input+':'+digits(j);return f;}
 function front(j){if(f==='int')return digits(seed+101+j);if(f==='number')return [-0,seed/4,NaN][j];if(f==='boolean')return ['false','true','true'][j];if(f==='string')return [input+':A',input+':B','é😀e\u0301\u0000:C'][j];if(f==='nested')return ['2:'+digits(seed)+',1','0:','2:'+digits(seed+2)+',3'][j];if(f==='record')return [digits(seed+101)+':'+input+':A',digits(seed+102)+':'+input+':B',digits(seed+103)+':é😀e\u0301\u0000:C'][j];return f;}
 // Independent new index partition; no tested unshift or array insertion operation.
 for(let i=0;i<n;i++){const count=[0,1,4,5,17][(i+seed)%5],arity=(i*7+seed)%3,args=[0,1,3][arity],length=count+args;out.push(f==='number'?length:digits(length),f==='number'?length:digits(length));for(let j=0;j<length;j++)out.push(j<args?front(j):old(j-args));}return out;
}
export function expectedUnshiftObservation(id,n,seed,input){const v=expectedUnshiftValues(id,n,seed,input);return id==='array_unshift_number'?observeNumber(v):observeSliceTree(v);}
