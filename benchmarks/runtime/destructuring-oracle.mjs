import {observeSliceTree} from './slice-observation.mjs';
export const destructuringIds = ['destructuring_patterns'];
// Independent of destructuring: every cell is written as arithmetic and text
// joins over the scenario inputs, with no pattern syntax.
export function expectedDestructuringValues(id,n,seed,input) {
 if (!destructuringIds.includes(id)) throw Error('Unknown destructuring workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string') throw Error('Destructuring case bounds');
 const values=[];
 for(let i=0;i<n;i++) { const base=(i*13+seed)%97; values.push([''+(base+1+base+i%7),input+'!'+input,'ab',''+(base+base+1)]); }
 return values;
}
export function expectedDestructuringObservation(id,n,seed,input) { return observeSliceTree(expectedDestructuringValues(id,n,seed,input)); }
