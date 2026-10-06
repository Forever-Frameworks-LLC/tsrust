import {observeSliceTree} from './slice-observation.mjs';
export const numberParseIntIds = ['number_parse_int_radix'];
// Independent of Number.parseInt: digit values are accumulated arithmetically.
function accumulate(digits,radix) { let value=0; for(const digit of digits){const code=digit.charCodeAt(0);value=value*radix+(code<58?code-48:code-87);} return value; }
export function expectedNumberParseIntValues(id,n,seed) {
 if (!numberParseIntIds.includes(id)) throw Error('Unknown Number.parseInt workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000) throw Error('Number.parseInt case bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const base=(i*13+seed)%97,decimal=base*7+i,digits=''+decimal;
  const zero=decimal===0;
  values.push([decimal,accumulate(digits,16),accumulate(digits,16),zero?-0:-accumulate(digits,36),NaN]);
 }
 return values;
}
export function expectedNumberParseIntObservation(id,n,seed) { return observeSliceTree(expectedNumberParseIntValues(id,n,seed),{numbers:true}); }
