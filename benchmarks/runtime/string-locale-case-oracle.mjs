import {observeSliceTree} from './slice-observation.mjs';
export const stringLocaleCaseIds = ['string_locale_case_text'];
// Independent of the case methods: each word has its lowercase and uppercase
// forms written out, and repetition repeats those forms.
const words=[['ab','AB'],['éé','ÉÉ'],['ßs','SSS'],['ωω','ΩΩ']];
export function expectedStringLocaleCaseValues(id,n,seed) {
 if (!stringLocaleCaseIds.includes(id)) throw Error('Unknown string locale case workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000) throw Error('String locale case bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const count=((i*13+seed)%97)%3+1,[lower,upper]=words[i%4];
  let low='',up='';
  for(let copy=0;copy<count;copy++){low+=lower;up+=upper;}
  values.push([low,up]);
 }
 return values;
}
export function expectedStringLocaleCaseObservation(id,n,seed) { return observeSliceTree(expectedStringLocaleCaseValues(id,n,seed)); }
