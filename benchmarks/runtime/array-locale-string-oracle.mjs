import {observeSliceTree} from './slice-observation.mjs';
export const arrayLocaleStringIds=['array_locale_string_values'];
const digits='0123456789';
function grouped(integer) {
 let text='';do{const digit=integer%10;text=digits[digit]+text;integer=(integer-digit)/10;}while(integer>0);
 let output='';for(let i=0;i<text.length;i++){if(i!==0&&(text.length-i)%3===0)output+=',';output+=text[i];}return output;
}
export function expectedArrayLocaleStringValues(id,n,seed,input) {
 if(!arrayLocaleStringIds.includes(id))throw Error('Unknown array toLocaleString workload');
 if(!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Numeric toLocaleString bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const base=(i*13+seed)%97,whole=base*1001;
  values.push([grouped(whole)+','+(whole===0?'':'-')+grouped(whole),grouped(1000+base)+'.5,-0,1.235',input+',é😀,','true,false',',',',','[object Object],[object Object]','1,000.5,-0,,'+grouped(2000+base)+'.235']);
 }

 return values;
}
export function expectedArrayLocaleStringObservation(id,n,seed,input){return observeSliceTree(expectedArrayLocaleStringValues(id,n,seed,input));}
