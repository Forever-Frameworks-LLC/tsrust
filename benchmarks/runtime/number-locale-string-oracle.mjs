import {observeSliceTree} from './slice-observation.mjs';
export const numberLocaleStringIds=['number_locale_string_decimal'];
const digits='0123456789';
function grouped(integer) {
 let text='';do{const digit=integer%10;text=digits[digit]+text;integer=(integer-digit)/10;}while(integer>0);
 let output='';for(let i=0;i<text.length;i++){if(i!==0&&(text.length-i)%3===0)output+=',';output+=text[i];}return output;
}
export function expectedNumberLocaleStringValues(id,n,seed) {
 if(!numberLocaleStringIds.includes(id))throw Error('Unknown numeric toLocaleString workload');
 if(!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Numeric toLocaleString bounds');
 const values=[],specials=['-0','-0.001','NaN','∞','-∞'];
 for(let i=0;i<n;i++) {
  const whole=((i*13+seed)%97)*1001;
  // Written rounded fractions: 0, 0.2345 and 0.9995 become 0, .235 and one whole unit.
  const positive=i%3===0?grouped(whole):i%3===1?grouped(whole)+'.235':grouped(whole+1);
  values.push([grouped(whole),positive,'-'+positive,specials[i%5],'1,000,000,000,000,000,000,000']);
 }
 return values;
}
export function expectedNumberLocaleStringObservation(id,n,seed){return observeSliceTree(expectedNumberLocaleStringValues(id,n,seed));}
