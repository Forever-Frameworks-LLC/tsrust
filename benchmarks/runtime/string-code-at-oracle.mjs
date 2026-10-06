import {observeSliceTree} from './slice-observation.mjs';
export const stringCodeAtIds = ['string_code_at_positions'];
// Independent of codePointAt and charCodeAt: the text repeats four known
// characters, so the code at a position follows from a table.
const codes=[97,233,8364,122];
export function expectedStringCodeAtValues(id,n,seed) {
 if (!stringCodeAtIds.includes(id)) throw Error('Unknown string code at workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000) throw Error('String code at case bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const mode=(i+seed)%3,position=(i*13+seed)%9,length=4*(mode+1),inside=position<length;
  values.push([inside?codes[position%4]:-1,inside?codes[position%4]:NaN,-1,NaN,codes[mode]]);
 }
 return values;
}
export function expectedStringCodeAtObservation(id,n,seed) { return observeSliceTree(expectedStringCodeAtValues(id,n,seed),{numbers:true}); }
