import {observeSliceTree} from './slice-observation.mjs';
import {validateArrayRemovalCase} from './array-removal-oracle.mjs';
export const flatFamilies=['int','number','boolean','string','nested','record','null','undefined'];
export const flatIds=['flat','flat_map'].flatMap(m=>flatFamilies.map(f=>'array_'+m+'_'+f));
export function flatInfo(id){if(!flatIds.includes(id))throw Error('flat ID');return {method:id.startsWith('array_flat_map_')?'flatMap':'flat',family:id.split('_').at(-1)};}
export function validateFlatCase(id,n,seed,input){return validateArrayRemovalCase('array_pop_'+flatInfo(id).family,n,seed,input);}
function digits(value){if(value===0)return'0';let text='';while(value>0){text='0123456789'[value%10]+text;value=Math.floor(value/10);}return text;}
// Manual index/piece construction: neither tested flattening method is used.
export function expectedFlatValues(id,n,seed,input){input=validateFlatCase(id,n,seed,input);const {method,family:f}=flatInfo(id),out=[];
 function leaf(ordinal){if(f==='int')return seed+ordinal;if(f==='number')return[-0,0,NaN,Infinity,-Infinity,seed/4,-2.5][(ordinal*3+seed)%7];if(f==='boolean')return(ordinal+seed)%3!==0;if(f==='string')return input+':'+digits(ordinal);if(f==='nested')return[seed+ordinal,ordinal];if(f==='record')return{id:seed+ordinal,label:input+':'+digits(ordinal)};return f==='null'?null:undefined;}
 function callback(v,index,count){if(index%3===0)return[];if(f==='int')return[v,v+index+count];if(f==='number')return[v,(index+count)/4];if(f==='boolean')return[v,!v];if(f==='string')return[v+':a',v+':b'];if(f==='nested')return[[...v,index,count],[]];if(f==='record')return[{id:v.id+index,label:v.label+':a'},{id:count,label:v.label+':b'}];return[f==='null'?null:undefined,f==='null'?null:undefined];}
 for(let i=0;i<n;i++){const count=[0,1,4,5,17][(i+seed)%5],result=[];for(let j=0;j<count;j++){if(method==='flat'){for(let k=0;k<(j+seed)%3;k++)result.push(leaf(j*3+k));}else{const values=callback(leaf(j),j,count);for(const value of values)result.push(value);}}
 if(f==='record')out.push(result.map(v=>digits(v.id)+':'+v.label));else if(f==='null'||f==='undefined')out.push(result.map(()=>f));else out.push(result);
 }return out;
}
export function expectedFlatObservation(id,n,seed,input){return observeSliceTree(expectedFlatValues(id,n,seed,input),{numbers:flatInfo(id).family==='number'});}
