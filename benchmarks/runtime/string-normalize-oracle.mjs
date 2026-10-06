import {observeSliceTree} from './slice-observation.mjs';
export const stringNormalizeIds=['string_normalize_forms'];
// Hand-written normal forms. No normalize operation appears in this oracle.
const forms=[['é','é','é','é'],['Åﬁ①','Åﬁ①','Åfi1','Åfi1'],['각','각','각','각'],['à̕','à̕','à̕','à̕'],['é😀','é😀','é😀','é😀']];
export function expectedStringNormalizeValues(id,n,seed) {
 if(!stringNormalizeIds.includes(id))throw Error('Unknown normalization workload');
 if(!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Normalization bounds');
 const values=[];
 for(let i=0;i<n;i++){const count=((i*13+seed)%97)%3+1,cell=forms[i%5].map(text=>text.repeat(count));values.push([...cell,cell[0],cell[0],cell[i%4]]);}
 return values;
}
export function expectedStringNormalizeObservation(id,n,seed){return observeSliceTree(expectedStringNormalizeValues(id,n,seed));}
