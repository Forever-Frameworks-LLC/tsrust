import {observeSliceTree} from './slice-observation.mjs';
export const stringAtIds = ['string_at_characters'];
// Independent of at(): positions are resolved against an explicit code-point list.
export function expectedStringAtValues(id,n,seed,input) {
 if (!stringAtIds.includes(id)) throw Error('Unknown string at workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string') throw Error('String at case bounds');
 const unit=[];for(const letter of input)unit.push(letter);
 const values=[];
 for(let i=0;i<n;i++) {
  const mode=(i+seed)%3,letters=[];for(let at=0;at<mode;at++)letters.push(...unit);
  const pick=position=>{const index=position<0?letters.length+position:position;return index>=0&&index<letters.length?letters[index]:'none';};
  values.push([pick(i%7),pick(-1-mode)]);
 }
 return values;
}
export function expectedStringAtObservation(id,n,seed,input) { return observeSliceTree(expectedStringAtValues(id,n,seed,input)); }
