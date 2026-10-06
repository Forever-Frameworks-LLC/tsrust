import {observeSliceTree} from './slice-observation.mjs';import {validateArrayRemovalCase} from './array-removal-oracle.mjs';
export const nullishFamilies=['int','number','boolean','string','array','record','nullable_fallback_int','null_undefined'];
export const nullishIds=nullishFamilies.map(f=>'nullish_'+f+(f==='nullable_fallback_int'?'':'_value'));
export function nullishFamily(id){const at=nullishIds.indexOf(id);if(at<0)throw Error('Nullish ID');return nullishFamilies[at];}
export function validateNullishCase(id,n,seed,input){const f=nullishFamily(id);return validateArrayRemovalCase('array_pop_'+(f==='array'||f==='nullable_fallback_int'?'int':f==='null_undefined'?'string':f),n,seed,input);}
function digits(value){if(value===0)return'0';let out='';while(value>0){out='0123456789'[value%10]+out;value=Math.floor(value/10);}return out;}
// Independent tag selection and complete literal payloads; never uses coalescing.
export function expectedNullishValues(id,n,seed,input){input=validateNullishCase(id,n,seed,input);const f=nullishFamily(id),out=[];let calls=0;
 for(let i=0;i<n;i++){const value=seed+i,missing=f==='null_undefined'||value%5<2,delta=missing?1:0;calls+=delta;let selected;
 if(missing){if(f==='int')selected=value+100;else if(f==='number')selected=(value+100)/4;else if(f==='boolean')selected=value%2===0;else if(f==='string'||f==='null_undefined')selected=input+'|fallback:'+digits(value);else if(f==='array')selected=[value+100,value+101,value+102];else if(f==='record')selected=digits(value+100)+':'+input+'|fallback:'+digits(value);else selected=value%2===0?null:value+100;}
 else{if(f==='int'||f==='nullable_fallback_int')selected=value%4;else if(f==='number')selected=[-0,0,NaN,Infinity,-Infinity,2.5,-2.5,value/4][value%8];else if(f==='boolean')selected=value%3!==0;else if(f==='string')selected=value%3===0?'':input+':'+digits(value);else if(f==='array')selected=value%3===0?[]:[value,value+1];else if(f==='record')selected=digits(value%4)+':'+(value%3===0?'':input+':'+digits(value));}
 if(f==='array'){out.push(selected);out.push([delta]);}else if(f==='boolean')out.push([selected,delta===0,delta===1]);else if(f==='nullable_fallback_int')out.push([selected===null?'null':digits(selected),selected===null?'object':'number',digits(delta)]);else out.push([selected,['string','record','null_undefined'].includes(f)?digits(delta):delta]);
 }if(f!=='boolean')out.push([['string','record','nullable_fallback_int','null_undefined'].includes(f)?digits(calls):calls]);return out;
}
export function expectedNullishObservation(id,n,seed,input){return observeSliceTree(expectedNullishValues(id,n,seed,input),{numbers:nullishFamily(id)==='number'});}
