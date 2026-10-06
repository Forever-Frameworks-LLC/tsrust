import {observeSliceTree} from './slice-observation.mjs';
export const mathF16roundIds = ['math_f16round_values'];
// Independent of Math.f16round: the half-precision grid step is found by
// doubling, and the nearest grid point is chosen with ties to the even multiple.
export function nearestHalf(value) {
 if (Number.isNaN(value)||value===0||!Number.isFinite(value)) return value;
 const magnitude=Math.abs(value);
 if (magnitude>=65520) return value<0?-Infinity:Infinity;
 let step=2**-24; // the step below 2^-13, where 2^-14 is the smallest normal value
 for(let top=2**-13;magnitude>=top;top*=2) step*=2;
 const below=Math.floor(magnitude/step),rest=magnitude/step-below;
 const count=rest>0.5||(rest===0.5&&below%2===1)?below+1:below;
 return (value<0?-1:1)*count*step;
}
export function expectedMathF16roundValues(id,n,seed) {
 if (!mathF16roundIds.includes(id)) throw Error('Unknown Math.f16round workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000) throw Error('Math.f16round case bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const base=(i*13+seed)%97;
  const signed=value=>{const result=nearestHalf(value);return result===0&&value<0?-0:result;};
  values.push([signed(base/7+0.1),signed(2049+base*2),signed(-(base+1)/67108864),signed(65500+base),signed(i*0.001-1)]);
 }
 return values;
}
export function expectedMathF16roundObservation(id,n,seed) { return observeSliceTree(expectedMathF16roundValues(id,n,seed),{numbers:true}); }
