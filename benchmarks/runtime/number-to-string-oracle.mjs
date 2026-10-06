import {observeSliceTree} from './slice-observation.mjs';
export const numberToStringIds = ['number_to_string_radix'];
const digits='0123456789abcdefghijklmnopqrstuvwxyz';
// Independent of Number.prototype.toString: the whole part is converted by
// repeated division and the exact eighth fraction by exact scaled integers.
function whole(value,radix) { if(value===0)return '0'; let text=''; while(value>0){text=digits[value%radix]+text;value=(value-value%radix)/radix;} return text; }
function fraction(eighths,radix) {
 if(eighths===0)return '';
 let text='.',numerator=eighths;
 while(numerator!==0){numerator*=radix;const digit=(numerator-numerator%8)/8;text+=digits[digit];numerator%=8;}
 return text;
}
export function expectedNumberToStringValues(id,n,seed) {
 if (!numberToStringIds.includes(id)) throw Error('Unknown number toString workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000) throw Error('Number toString case bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const base=(i*13+seed)%97,eighths=i%8,zero=base===0&&eighths===0;
  const text=radix=>whole(base,radix)+fraction(eighths,radix);
  values.push([text(10),text(2),text(16),text(36),(zero?'':'-')+text(8)]);
 }
 return values;
}
export function expectedNumberToStringObservation(id,n,seed) { return observeSliceTree(expectedNumberToStringValues(id,n,seed)); }
