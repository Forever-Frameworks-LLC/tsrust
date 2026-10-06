import {observeSliceTree} from './slice-observation.mjs';
export const removalFamilies=['int','number','boolean','string','nested','record','null','undefined'];
export const arrayRemovalIds=['pop','shift'].flatMap(method=>removalFamilies.map(family=>'array_'+method+'_'+family));
export function validateArrayRemovalCase(id,n,seed,input){
 if(!arrayRemovalIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Bounded dense removal case');
 const external=id.endsWith('_string')||id.endsWith('_record');if(!external){if(input!==undefined)throw Error('No external input');return undefined;}if(input===undefined)input='dynamic text';if(typeof input!=='string')throw Error('String input');let bytes=0;for(const c of input){const p=c.codePointAt(0);if(p>=0xd800&&p<=0xdfff)throw Error('Lone surrogate excluded');bytes+=p<128?1:p<2048?2:p<65536?3:4;}if(bytes>256)throw Error('Max256 UTF8');return input;
}
function digits(value){if(value===0)return '0';let result='';while(value>0){result='0123456789'[value%10]+result;value=Math.floor(value/10);}return result;}
function quarter(seed){const whole=Math.floor(seed/4),remainder=seed%4;return digits(whole)+['','.25','.5','.75'][remainder];}
function contents(family,seed,input){
 if(family==='int')return [digits(seed),digits(seed+1),digits(seed+2)];
 if(family==='number')return [quarter(seed),'0','-2.5'];
 if(family==='boolean')return [seed%2===0?'true':'false','false','true'];
 if(family==='string')return [input,input+'!','é😀e\u0301\u0000'];
 if(family==='nested')return ['2:'+digits(seed)+',1','0:','2:'+digits(seed+2)+',3'];
 if(family==='record')return [digits(seed)+':'+input,digits(seed+1)+':'+input+'!',digits(seed+2)+':é😀e\u0301\u0000'];
 return [family,family,family];
}
// Compute index/length evolution directly. Neither tested removal method is called.
export function expectedArrayRemovalValues(id,n,seed,input){
 input=validateArrayRemovalCase(id,n,seed,input);const [,method,family]=id.split('_'),pool=contents(family,seed,input),kind=family==='int'||family==='number'?'number':['nested','record','null'].includes(family)?'object':family,out=[];
 for(let i=0;i<n;i++){let length=(i+seed)%4,start=0;for(let operation=0;operation<2;operation++){
  if(length===0)out.push('undefined','undefined');else{const index=method==='pop'?start+length-1:start;out.push(kind,pool[index]);length--;if(method==='shift')start++;}
  out.push(digits(length),length>0?pool[start]:'<absent>',length>1?pool[start+1]:'<absent>');
 }}return out;
}
export function expectedArrayRemovalObservation(id,n,seed,input){return observeSliceTree(expectedArrayRemovalValues(id,n,seed,input));}
