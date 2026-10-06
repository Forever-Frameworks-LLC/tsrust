import {observeSliceTree} from './slice-observation.mjs';
export const mathRandomIds = ['math_random_range'];
// Independent of Math.random: draws differ per run, so the workload retains
// only properties of the range. Every draw lies in [0, 1), a scaled draw lies
// in [0, sides), and two consecutive 53-bit draws differ.
export function expectedMathRandomValues(id,n,seed) {
 if (!mathRandomIds.includes(id)) throw Error('Unknown Math.random workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000) throw Error('Math.random case bounds');
 return Array.from({length:n},()=>[true,true,true,true]);
}
export function expectedMathRandomObservation(id,n,seed) { return observeSliceTree(expectedMathRandomValues(id,n,seed)); }
